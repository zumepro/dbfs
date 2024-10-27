mod database_objects;
mod database_enums;
pub mod driver_objects;
mod commands;


use crate::db_connector::chrono;


const CONN_LOCK_FAILED: &'static str = "could not lock onto the database connection (this could be a synchronization error)";
const DBI64_TO_DRU32_CONVERSION_ERROR_MESSAGE: &'static str = "could not convert database's i64 to u32 for the driver";


/// Filesystem block size.
pub const BLOCK_SIZE: u32 = 4096;

/// Maximum allowed file name length. Taken from the `dbfs.sql` init script.
pub const MAX_NAME_LEN: u32 = 255;


use database_objects::{FileHardlinks, FileSize, Inode, DirectoryChildrenDirectory};
use std::sync::Mutex;
use crate::db_connector::{DbConnector, DbConnectorError};


pub struct TranslationLayer (Mutex<DbConnector>);


#[derive(Debug)]
pub enum Error {
	DbConnectorError(DbConnectorError),
	RuntimeError(&'static str),
	ClientError(&'static str),
	Unimplemented,
}
impl From<DbConnectorError> for Error {
	fn from(value: DbConnectorError) -> Self {
		Self::DbConnectorError(value)
	}
}
impl std::fmt::Display for Error {
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "sql_translation_error: {} (consider running fsck)", match self {
			Self::DbConnectorError(val) => val.to_string(),
			Self::RuntimeError(val) => val.to_string(),
			Self::ClientError(val) => val.to_string(),
			Self::Unimplemented => "method isn't implemented yet".to_string(),
		})
	}
}


impl TranslationLayer {
	/// Create a [`TranslationLayer`] object and use the defaults from [`crate::settings`] to
	/// login to the database
	pub fn new() -> Result<Self, Error> {
		Ok(Self (
			Mutex::new(DbConnector::default()?)
		))
	}
	

	/// Computes the size of a file
	///
	/// # Inputs
	/// `inode: u64` is the id of the inode of the target file
	///
	/// # Warnings
	/// This function DOES NOT check whether the inode actually is a regular file or symlink.
	pub fn filesize(&mut self, inode: u64) -> Result<driver_objects::FileSize, Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		let Ok(size) = conn.query(commands::SQL_GET_FILE_SIZE, Some(&vec![inode.into()])) else {
			return Ok(FileSize { bytes: 0, blocks: 0 }.into())
		};
		let size = match size.get(0) {
			Some(val) => *val,
			None => FileSize { bytes: 0, blocks: 0 }
		};

		Ok(size.into())
	}


	/// Count the number of references to an inode
	///
	/// # Inputs
	/// `inode: u64` is the id of the inode whose references will be counted
	///
	/// # Warnings
	/// This does not check whether the inode is a regular file or a symlink.
	pub fn count_hardlinks(&mut self, inode: u64) -> Result<u32, Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		let hardlinks: Vec<FileHardlinks> = conn.query(commands::SQL_COUNT_HARDLINKS, Some(&vec![inode.into()]))?;
		let hardlinks = hardlinks.get(0).ok_or(Error::RuntimeError("could not count hardlinks"))?.hardlinks;
		Ok(hardlinks.try_into().map_err(|_| Error::RuntimeError(DBI64_TO_DRU32_CONVERSION_ERROR_MESSAGE))?)
	}


	/// Count the number of subdirectories contained inside a directory
	///
	/// # Inputs
	/// `inode: u64` is the id of the directory's inode whose children will be counted
	///
	/// # Warnings
	/// This does not check whether the inode is a directory.
	pub fn count_subdirs(&mut self, inode: u64) -> Result<u32, Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		let subdirs: Vec<DirectoryChildrenDirectory> = conn.query(commands::SQL_COUNT_CHILDREN_OF_TYPE_DIRECTORY, Some(&vec![inode.into()]))?;
		let subdirs = subdirs.get(0).ok_or(Error::RuntimeError("could not count subdirectories"))?.children_dirs;
		Ok((subdirs + 2).try_into().map_err(|_| Error::RuntimeError(DBI64_TO_DRU32_CONVERSION_ERROR_MESSAGE))?)
	}


	/// Get attributes for file
	///
	/// # Inputs
	/// `_inode: u64` is the id of the inode of the target file
	///
	/// # Warnings
	/// This is a relatively expensive operation, so use as sparingly as possible.
	pub fn getattr(&mut self, _inode: u64) -> Result<driver_objects::FileAttr, Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		let inode: Vec<Inode> = conn.query(commands::SQL_GET_INODE, Some(&vec![_inode.into()]))?;
		drop(conn);

		let Some(inode) = inode.get(0) else {
			return Err(Error::RuntimeError("no inode found with given id"));
		};

		let file_type: database_enums::FileType = (&inode.file_type).into();

		let hardlinks: u32 = match file_type {
			database_enums::FileType::RegularFile | database_enums::FileType::SymbolicLink => self.count_hardlinks(_inode)?,
			database_enums::FileType::Directory => self.count_subdirs(_inode)?,
			database_enums::FileType::Unknown => 0,
		};

		let file_type: driver_objects::FileType = driver_objects::FileType::try_from(file_type)?;

		let file_size = match file_type {
			driver_objects::FileType::File | driver_objects::FileType::Symlink => {
				self.filesize(_inode)?
			},
			driver_objects::FileType::Directory => driver_objects::FileSize { bytes: 0, blocks: 0 },
		};

		Ok(driver_objects::FileAttr {
			ino: inode.id,
			uid: inode.owner,
			gid: inode.group,
			hardlinks: hardlinks.try_into().map_err(|_| Error::RuntimeError(DBI64_TO_DRU32_CONVERSION_ERROR_MESSAGE))?,
			bytes: file_size.bytes.try_into().map_err(|_| Error::RuntimeError(DBI64_TO_DRU32_CONVERSION_ERROR_MESSAGE))?,
			blocks: file_size.blocks.try_into().map_err(|_| Error::RuntimeError(DBI64_TO_DRU32_CONVERSION_ERROR_MESSAGE))?,
			atime: inode.accessed_at.into(),
			mtime: inode.modified_at.into(),
			ctime: inode.created_at.into(),
			kind: file_type,
			perm: driver_objects::Permissions {
				special: inode.special_bits,
				owner: inode.user_perm,
				group: inode.group_perm,
				other: inode.other_perm
			}
		})
	}


	/// Determines the corresponding inode ID to a parent inode and file name pair
	///
	/// # Inputs
	/// `name: &OsStr` is the name of the file
	/// `parent_inode: u64` is the inode ID of the file's parent
	pub fn lookup_id(&mut self, name: &std::ffi::OsStr, parent_inode: u64) -> Result<u64, Error> {
		let path = name.to_str().ok_or(Error::RuntimeError("could not parse path"))?.to_string();

		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		let inode: Vec<database_objects::InodeLookup> = conn.query(commands::SQL_LOOKUP_INODE_ID, Some(&vec![path.into(), parent_inode.into()]))?;
		let inode: &database_objects::InodeLookup = inode.get(0).ok_or(Error::RuntimeError("could not read inode ID"))?;

		Ok(inode.inode_id.into())
	}


	/// Get attributes for file
	///
	/// # Inputs
	/// `name: &OsStr` is the name of the file
	/// `parent_inode: u64` is the inode ID of the file's parent
	///
	/// # Warnings
	/// As this function internally calls getattr, it is also
	/// a relatively expensive operation, so use as sparingly as possible.
	pub fn lookup(&mut self, name: &std::ffi::OsStr, parent_inode: u64) -> Result<driver_objects::FileAttr, Error> {
		let inode = self.lookup_id(name, parent_inode)?;
		self.getattr(inode)
	}


	/// List a directory by inode id
	///
	/// # Inputs
	/// `inode: u64` is the id of the inode of the desired directory
	///
	/// # Warning
	///
	/// This function DOES NOT check if the given `inode` id belongs to a directory (or a
	/// different filetype). Nor does it check whether the parent is a directory.
	pub fn readdir(&mut self, inode: u64) -> Result<Vec<driver_objects::DirectoryEntry>, Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;

		let listing: Vec<database_objects::DirectoryEntry> = conn.query(commands::SQL_LIST_DIRECTORY, Some(&vec![inode.into()]))?;
		let parent: u32 = DbConnector::query::<database_objects::DirectoryParent>(&mut conn, commands::SQL_GET_DIRECTORY_PARENT, Some(&vec![inode.into()]))?.get(0).ok_or(Error::RuntimeError("could not find the parent file on readdir"))?.parent_inode_id;

		let mut entries = vec![
			driver_objects::DirectoryEntry {
				inode,
				ftype: driver_objects::FileType::Directory,
				name: ".".into()
			},
			driver_objects::DirectoryEntry {
				inode: parent.into(),
				ftype: driver_objects::FileType::Directory,
				name: "..".into()
			}
		];
		entries.append(&mut listing.iter().map(|val| Ok(driver_objects::DirectoryEntry::try_from(val)?)).collect::<Result<Vec<driver_objects::DirectoryEntry>, Error>>()?);

		Ok(entries)
	}


	/// Count the children of a directory
	///
	/// # Inputs
	/// `inode: u64` is the id of the inode of the desired directory
	///
	/// # Warning
	///
	/// This function DOES NOT check if the given `inode` id belongs to a directory (or a
	/// different filetype).
	pub fn count_children(&mut self, inode: u64) -> Result<u64, Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;

		let count: Vec<database_objects::ChildrenCount> = conn.query(commands::SQL_COUNT_DIRECTORY_CHILDREN, Some(&vec![inode.into()]))?;
		let count: &database_objects::ChildrenCount = count.get(0).ok_or(Error::RuntimeError("could not determine children count"))?;

		Ok(count.count as u64)
	}


	/// Read inode contents
	///
	/// If the destination buffer is too large, an error will be returned.
	///
	/// # Inputs
	/// `inode: u64` is the id of the inode which will be read
	/// `offset: u64` is the offset in the inode's data
	/// `buffer: &mut [u8]` is the destination buffer
	///
	/// # Outputs
	/// Read size or error
	///
	/// Besides regular errors this function can return [`Error::ClientError`]`("pointer out of
	/// range")`
	pub fn read(&mut self, inode: u64, offset: u64, buffer: &mut [u8]) -> Result<usize, Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;

		let max_bytes = buffer.len();
		let max_blocks = (max_bytes.div_ceil(4096) + 1_usize) as u64;
		let offset_blocks = offset / 4096;
		let offset = offset as usize;

		let blocks: Vec<database_objects::Block> = conn.query(commands::SQL_READ_FILE, Some(&vec![inode.into(), max_blocks.into(), offset_blocks.into()]))?;
		if blocks.len() == 0 {
			match max_bytes {
				0 => { return Ok(0); }
				_ => { return Err(Error::ClientError("read failed (pointer or size invalid)")); },
			}
		}
		let bytes: Vec<u8> = blocks.iter().flat_map(|inner| inner.data.iter()).skip(offset - (offset_blocks * 4096) as usize).take(max_bytes).map(|val| val.clone()).collect();
		let read = bytes.len();
		buffer[..bytes.len()].copy_from_slice(&bytes);

		Ok(read)
	}


	/// Fetch filesystem statistics
	///
	/// # Warning
	/// As the backend is an SQL database, the contents of the `free_blocks` and `free_inodes`
	/// fields may be completely made up, as the driver assumes that the SQL backend provides
	/// unlimited resources.
	pub fn statfs(&mut self) -> Result<driver_objects::FilesystemStat, Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		let stat: Vec<database_objects::FilesystemStat> = conn.query(commands::SQL_GET_FS_STAT, None)?;
		let stat: &database_objects::FilesystemStat = stat.get(0).ok_or(Error::RuntimeError("could not determine fs stat"))?;
		
		Ok(driver_objects::FilesystemStat {
			free_blocks: 420,
			used_blocks: stat.used_blocks as u64,
			free_inodes: 69,
			used_inodes: stat.used_inodes as u64
		})
	}


	/// Write inode contents
	///
	/// If the source buffer is larger than the current inode contents,
	/// it will be automatically resized.
	///
	/// # Inputs
	/// `inode: u64` is the id of the inode which will be written to
	/// `offset: u64` is the offset in the inode's data
	/// `buffer: &[u8]` is the source buffer
	pub fn write(&mut self, _inode: u64, _offset: u64, _buffer: &[u8]) -> Result<(), Error> {
		Err(Error::Unimplemented)
	}


	/// Create a new inode/file pair with no blocks
	///
	/// # Inputs
	/// `parent_inode: u64` specifies the parent inode where the file should be created
	/// `name: &OsStr` is the name of the file to be created
	/// `kind: FileType` sets the inode type
	/// `attr: FileSetAttr` sets the remaining inode attributes
	pub fn mknod(&mut self, _parent_inode: u64, _name: &std::ffi::OsStr, _kind: driver_objects::FileType, _attr: driver_objects::FileSetAttr) -> Result<driver_objects::FileAttr, Error> {
		Err(Error::Unimplemented)
	}


	/// Creates a new file reference to an existing inode
	///
	/// # Inputs
	/// `parent_inode: u64` specifies the parent inode where the file should be created
	/// `name: &OsStr` is the name of the file to be created
	/// `dest_inode: u64` sets the inode to which the new file will be poiting to
	pub fn link(&mut self, parent_inode: u64, name: &std::ffi::OsStr, dest_inode: u64) -> Result<(), Error> {
		let path = name.to_str().ok_or(Error::RuntimeError("could not parse path"))?.to_string();

		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		let affected = conn.command(commands::SQL_CREATE_FILE, Some(&vec![
			parent_inode.into(),
			path.into(),
			dest_inode.into()
		]))?;
		drop(conn);
		match affected {
			1 => Ok(()),
			_ => Err(Error::RuntimeError("no changes made"))
		}
	}


	/// Truncates or expands an inode by deleting or adding blocks
	///
	/// # Inputs
	/// `inode: u64` specifies the inode
	/// `new_size: u64` specifies the new size the file should have
	pub fn resize(&mut self, _inode: u64, _new_size: u64) -> Result<(), Error> {
		Err(Error::Unimplemented)
	}


	/// Sets inode attributes
	///
	/// # Inputs
	/// `inode: u64` specifies the inode
	/// `attr: FileSetAttr` sets the inode attributes
	pub fn setattr(&mut self, inode: u64, attr: driver_objects::FileSetAttr) -> Result<driver_objects::FileAttr, Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		let affected = conn.command(commands::SQL_UPDATE_INODE, Some(&vec![
			attr.uid.into(),
			attr.gid.into(),
			Into::<chrono::DateTime<chrono::Utc>>::into(attr.atime).into(),
			Into::<chrono::DateTime<chrono::Utc>>::into(attr.mtime).into(),
			Into::<chrono::DateTime<chrono::Utc>>::into(attr.ctime).into(),
			attr.perm.special.into(),
			attr.perm.owner.into(),
			attr.perm.group.into(),
			attr.perm.other.into(),
			inode.into()
		]))?;
		drop(conn);
		if affected != 1 {
			return Err(Error::RuntimeError("no changes made"));
		}
		
		self.getattr(inode)
	}


	/// Removes a reference to an inode
	///
	/// If the inode has zero references or it is a directory, it will also be deleted.
	///
	/// # Inputs
	/// `parent_inode: u64` specifies the file's parent inode
	/// `name: &OsStr` is the name of the file to be deleted
	pub fn unlink(&mut self, parent_inode: u64, name: &std::ffi::OsStr) -> Result<(), Error> {
		let inode = self.lookup_id(name, parent_inode)?;
		let path = name.to_str().ok_or(Error::RuntimeError("could not parse path"))?.to_string();

		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		let affected = conn.command(commands::SQL_DELETE_FILE, Some(&vec![path.into(), parent_inode.into()]))?;
		drop(conn);
		if affected != 1 {
			return Err(Error::RuntimeError("no changes made"));
		}

		let attr = self.getattr(inode)?;
		let delete_inode = match (attr.hardlinks, attr.kind) {
			(0, driver_objects::FileType::File | driver_objects::FileType::Symlink) => true,
			(_, driver_objects::FileType::Directory) => true,
			_ => false
		};
		if !delete_inode {
			return Ok(());
		}

		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		let affected = conn.command(commands::SQL_DELETE_INODE, Some(&vec![inode.into()]))?;
		match affected {
			1 => Ok(()),
			_ => Err(Error::RuntimeError("could not delete inode"))
		}
	}


	/// Renames/moves a file or directory
	///
	/// # Inputs
	/// `src_parent_inode: u64` specifies the file's former parent inode
	/// `src_name: &OsStr` is the name of the file to be moved
	/// `dest_parent_inode: u64` specifies the file's new parent inode
	/// `dest_name: &OsStr` is the file's new name
	pub fn rename(&mut self, src_parent_inode: u64, src_name: &std::ffi::OsStr, dest_parent_inode: u64, dest_name: &std::ffi::OsStr) -> Result<(), Error> {
		let src_path = src_name.to_str().ok_or(Error::RuntimeError("could not parse path"))?.to_string();
		let dest_path = dest_name.to_str().ok_or(Error::RuntimeError("could not parse path"))?.to_string();

		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		let affected = conn.command(commands::SQL_RENAME_FILE, Some(&vec![dest_parent_inode.into(), dest_path.into(), src_parent_inode.into(), src_path.into()]))?;

		match affected {
			1 => Ok(()),
			_ => Err(Error::RuntimeError("no changes made"))
		}
	}
}




#[cfg(feature = "integration_testing")]
#[cfg(test)]
mod test {
	use std::ffi::OsString;
	use sqlx::types::chrono::{DateTime, Local};
	use super::*;


	#[test]
	fn getattr_dir_01() {
		let mut sql = TranslationLayer::new().unwrap();
		let attr = sql.getattr(1).unwrap();
		assert_eq!(attr, driver_objects::FileAttr {
			ino: 1,
			uid: 1,
			gid: 1,
			hardlinks: 3,
			bytes: 0,
			blocks: 0,
			atime: "2024-10-24 17:52:52+0000".parse::<DateTime<Local>>().unwrap().into(),
			mtime: "2024-10-24 17:53:10+0000".parse::<DateTime<Local>>().unwrap().into(),
			ctime: "2024-10-24 17:52:52+0000".parse::<DateTime<Local>>().unwrap().into(),
			kind: driver_objects::FileType::Directory,
			perm: driver_objects::Permissions { special: 0, owner: 7, group: 5, other: 5 },
		});
	}

	#[test]
	fn getattr_dir_02() {
		let mut sql = TranslationLayer::new().unwrap();
		let attr = sql.getattr(4).unwrap();
		assert_eq!(attr, driver_objects::FileAttr {
			ino: 4,
			uid: 2,
			gid: 2,
			hardlinks: 2,
			bytes: 0,
			blocks: 0,
			atime: "2024-10-26 16:59:30+0000".parse::<DateTime<Local>>().unwrap().into(),
			mtime: "2024-10-26 16:59:30+0000".parse::<DateTime<Local>>().unwrap().into(),
			ctime: "2024-10-26 16:59:30+0000".parse::<DateTime<Local>>().unwrap().into(),
			kind: driver_objects::FileType::Directory,
			perm: driver_objects::Permissions { special: 0, owner: 7, group: 5, other: 5 },
		});
	}
	
	#[test]
	fn getattr_smaller_file() {
		let mut sql = TranslationLayer::new().unwrap();
		let attr = sql.getattr(2).unwrap();
		assert_eq!(attr, driver_objects::FileAttr {
			ino: 2,
			uid: 2,
			gid: 2,
			hardlinks: 1,
			bytes: 14,
			blocks: 1,
			atime: "2024-10-24 17:54:00+0000".parse::<DateTime<Local>>().unwrap().into(),
			mtime: "2024-10-24 17:54:00+0000".parse::<DateTime<Local>>().unwrap().into(),
			ctime: "2024-10-24 17:54:00+0000".parse::<DateTime<Local>>().unwrap().into(),
			kind: driver_objects::FileType::File,
			perm: driver_objects::Permissions { special: 0, owner: 6, group: 4, other: 4 },
		});
	}

	#[test]
	fn getattr_larger_file() {
		let mut sql = TranslationLayer::new().unwrap();
		let attr = sql.getattr(3).unwrap();
		assert_eq!(attr, driver_objects::FileAttr {
			ino: 3,
			uid: 2,
			gid: 2,
			hardlinks: 2,
			bytes: 4096 * 3 + 5,
			blocks: 4,
			atime: "2024-10-24 17:56:34+0000".parse::<DateTime<Local>>().unwrap().into(),
			mtime: "2024-10-24 17:57:14+0000".parse::<DateTime<Local>>().unwrap().into(),
			ctime: "2024-10-24 17:56:34+0000".parse::<DateTime<Local>>().unwrap().into(),
			kind: driver_objects::FileType::File,
			perm: driver_objects::Permissions { special: 0, owner: 6, group: 4, other: 4 },
		});
	}

	#[test]
	fn listdir_root() {
		let mut sql = TranslationLayer::new().unwrap();
		let listing = sql.readdir(1).unwrap();
		assert_eq!(listing, vec![
			driver_objects::DirectoryEntry { inode: 1, ftype: driver_objects::FileType::Directory, name: ".".into() },
			driver_objects::DirectoryEntry { inode: 1, ftype: driver_objects::FileType::Directory, name: "..".into() },
			driver_objects::DirectoryEntry { inode: 2, ftype: driver_objects::FileType::File, name: "test.txt".into() },
			driver_objects::DirectoryEntry { inode: 3, ftype: driver_objects::FileType::File, name: "hardlink_to_test.bin".into() },
			driver_objects::DirectoryEntry { inode: 3, ftype: driver_objects::FileType::File, name: "test.bin".into() },
			driver_objects::DirectoryEntry { inode: 4, ftype: driver_objects::FileType::Directory, name: "more_testing".into() },
			driver_objects::DirectoryEntry { inode: 8, ftype: driver_objects::FileType::Symlink, name: "symlink_to_test.txt".into() }
		]);
	}

	#[test]
	fn lookup_01() {
		let mut sql = TranslationLayer::new().unwrap();
		let entry = sql.lookup(&OsString::from("test.txt"), 1).unwrap();
		assert_eq!(entry, driver_objects::FileAttr  {
			ino: 2,
			uid: 2,
			gid: 2,
			hardlinks: 1,
			bytes: 14,
			blocks: 1,
			atime: "2024-10-24 17:54:00+0000".parse::<DateTime<Local>>().unwrap().into(),
			mtime: "2024-10-24 17:54:00+0000".parse::<DateTime<Local>>().unwrap().into(),
			ctime: "2024-10-24 17:54:00+0000".parse::<DateTime<Local>>().unwrap().into(),
			kind: driver_objects::FileType::File,
			perm: driver_objects::Permissions { special: 0, owner: 6, group: 4, other: 4 },
		});
	}

	#[test]
	fn lookup_02() {
		let mut sql = TranslationLayer::new().unwrap();
		let entry = sql.lookup(&OsString::from("test.bin"), 1).unwrap();
		assert_eq!(entry, driver_objects::FileAttr  {
			ino: 3,
			uid: 2,
			gid: 2,
			hardlinks: 2,
			bytes: 4096 * 3 + 5,
			blocks: 4,
			atime: "2024-10-24 17:56:34+0000".parse::<DateTime<Local>>().unwrap().into(),
			mtime: "2024-10-24 17:57:14+0000".parse::<DateTime<Local>>().unwrap().into(),
			ctime: "2024-10-24 17:56:34+0000".parse::<DateTime<Local>>().unwrap().into(),
			kind: driver_objects::FileType::File,
			perm: driver_objects::Permissions { special: 0, owner: 6, group: 4, other: 4 },
		});
	}

	#[test]
	fn hardlink_01() {
		let mut sql = TranslationLayer::new().unwrap();
		let entry_1 = sql.lookup(&OsString::from("test.bin"), 1).unwrap();
		let entry_2 = sql.lookup(&OsString::from("hardlink_to_test.bin"), 1).unwrap();
		assert_eq!(entry_1, entry_2);
	}

	#[test]
	fn read_file_01() {
		let mut sql = TranslationLayer::new().unwrap();
		let buffer: &mut [u8] = &mut [0; 14];
		let read = sql.read(2, 0, buffer).unwrap();
		assert_eq!(buffer, "Hello, world!\n".as_bytes());
		assert_eq!(read, 14);
	}

	#[test]
	fn read_file_02() {
		let mut sql = TranslationLayer::new().unwrap();
		let buffer: &mut [u8] = &mut [0; 4];
		let read = sql.read(2, 0, buffer).unwrap();
		assert_eq!(buffer, "Hell".as_bytes());
		assert_eq!(read, 4);
	}

	#[test]
	fn read_file_03() {
		let mut sql = TranslationLayer::new().unwrap();
		let buffer: &mut [u8] = &mut [0; 14];
		let read = sql.read(2, 4096, buffer);
		println!("{:?}", read);
		assert!(match read { Ok(_) => false, Err(Error::ClientError(val)) => { assert_eq!(val.to_string(), "read failed (pointer or size invalid)"); true }, Err(_) => false });
	}

	#[test]
	fn read_file_04() {
		let mut sql = TranslationLayer::new().unwrap();
		let buffer: &mut [u8] = &mut [0; 4097];
		let read = sql.read(3, 4096 * 2, buffer).unwrap();
		let target: &mut [u8] = &mut [0; 4097];
		target[target.len() - 1] = 'a' as u8;
		assert_eq!(read, 4097);
		assert_eq!(buffer, target);
	}
}
