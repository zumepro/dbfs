mod database_objects;
mod database_enums;
pub mod driver_objects;
mod commands;


use crate::{db_connector::chrono, settings};


const CONN_LOCK_FAILED: &'static str = "could not lock onto the database connection (this could be a synchronization error)";
const DBI64_TO_DRU32_CONVERSION_ERROR_MESSAGE: &'static str = "could not convert database's i64 to u32 for the driver";
const DRU64_TO_DBU32_CONVERSION_ERROR_MESSAGE: &'static str = "could not convert driver's u64 to u32 for the database";
const OOB_WRITE: &'static str = "write is possibly out of bounds";


/// Maximum allowed file name length. Taken from the `dbfs.sql` init script.
pub const MAX_NAME_LEN: u32 = 255;


use database_objects::{DirectoryChildrenDirectory, FileHardlinks, FileSize, FileSizeAndHead, Inode};
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


/// Will try to clone a part of one slice into a part of other slice
/// If anything is outside of range - it will cancel the operation
/// If the two slices have different sizes - it will panic.
macro_rules! try_slice_from_slice {
	($source:expr, $source_range:expr, $target:expr, $target_range:expr) => {
		$target.get_mut($target_range).map(|left| -> Option<_> { Some(left.copy_from_slice(&$source.get($source_range)?)) });
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
			database_enums::FileType::RegularFile
				| database_enums::FileType::SymbolicLink
				| database_enums::FileType::Socket
				| database_enums::FileType::NamedPipe => self.count_hardlinks(_inode)?,
			database_enums::FileType::Directory => self.count_subdirs(_inode)?,
			database_enums::FileType::Unknown => 0,
		};

		let file_type: driver_objects::FileType = driver_objects::FileType::try_from(file_type)?;

		let file_size = match file_type {
			driver_objects::FileType::File | driver_objects::FileType::Symlink => {
				self.filesize(_inode)?
			},
			driver_objects::FileType::Socket
				| driver_objects::FileType::NamedPipe
				| driver_objects::FileType::Directory => driver_objects::FileSize { bytes: 0, blocks: 0 },
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


	/// Count the children of a directory (+2 for "." and "..")
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

		Ok((count.children + 2) as u64)
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
		let max_blocks = (max_bytes.div_ceil(settings::FILE_BLOCK_SIZE_USIZE) + 1_usize) as u64;
		let offset_blocks = offset / settings::FILE_BLOCK_SIZE;
		let offset = offset as usize;

		let blocks: Vec<database_objects::BlockData> = conn.query(commands::SQL_READ_FILE, Some(&vec![inode.into(), max_blocks.into(), offset_blocks.into()]))?;
		if blocks.len() == 0 {
			match max_bytes {
				0 => { return Ok(0); }
				_ => { return Err(Error::ClientError("read failed (pointer or size invalid)")); },
			}
		}
		let bytes: Vec<u8> = blocks.iter().flat_map(|inner| inner.data.iter()).skip(offset - (offset_blocks * settings::FILE_BLOCK_SIZE) as usize).take(max_bytes).map(|val| val.clone()).collect();
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
			used_blocks: stat.used_blocks as u64,
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
	pub fn write(&mut self, inode: u64, offset: u64, buffer: &[u8]) -> Result<(), Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;

		let length = buffer.len();
		if length == 0 { return Ok(()); }
		let start_block = (offset / settings::FILE_BLOCK_SIZE) as usize;
		let start = offset as usize - start_block * settings::FILE_BLOCK_SIZE_USIZE;
		let end = offset as usize + length - 1;
		let end_block = (end + 1).div_ceil(settings::FILE_BLOCK_SIZE_USIZE);
		let block_count = end_block - start_block;

		let size = match conn.query(commands::SQL_GET_SIZE_AND_HEAD, Some(&vec![inode.into()])) {
			Ok(val) => match val.get(0) {
				Some(val) => *val,
				None => FileSizeAndHead { bytes: 0, blocks: 0, last_block_id: 0 }
			},
			Err(_) => FileSizeAndHead { bytes: 0, blocks: 0, last_block_id: 0 }
		};
		let db_filesize = size.bytes.try_into().map_err(|_| Error::RuntimeError(DBI64_TO_DRU32_CONVERSION_ERROR_MESSAGE))?;
		let db_bc: u32 = size.blocks.try_into().map_err(|_| Error::RuntimeError(DBI64_TO_DRU32_CONVERSION_ERROR_MESSAGE))?;
		// If not large enough, make it bigger
		if end > db_filesize {
			conn.command(commands::SQL_RESIZE_LAST_BLOCK, Some(&vec![settings::FILE_BLOCK_SIZE.into(), settings::FILE_BLOCK_SIZE.into(), inode.into()]))?;
		}
		if end_block as u32 > db_bc {
			conn.command(commands::dynamic_queries::sql_pad_file(inode.try_into().map_err(|_| Error::RuntimeError(DRU64_TO_DBU32_CONVERSION_ERROR_MESSAGE))?, size.last_block_id.into(), end_block as u32 - db_bc).as_str(), None)?;
		}

		let mut blocks: Vec<database_objects::Block> = conn.query(commands::SQL_GET_FULL_BLOCKS, Some(&vec![inode.into(), (block_count as u64).into(), (start_block as u64).into()]))?;
		if blocks.len() != block_count { return Err(Error::ClientError(OOB_WRITE)); }

		let query = commands::dynamic_queries::sql_write(&blocks);

		for (current_block, current_inblock_pos, byte) in buffer.iter().enumerate().map(|(idx, val)| {
			let current_block = (idx + start) / settings::FILE_BLOCK_SIZE_USIZE;
			let current_inblock_pos = idx + start - current_block * settings::FILE_BLOCK_SIZE_USIZE;
			(current_block, current_inblock_pos, val)
		}) {
			blocks[current_block].data[current_inblock_pos] = *byte;
		}

		if blocks.len() != block_count { return Err(Error::ClientError(OOB_WRITE)); }

		let _command_status = conn.command(query.as_str(), Some(&blocks.iter().map(|val| val.data.clone().into()).collect()))?;

		//if blocks.len() as u64 != command_status.rows_affected { return Err(Error::ClientError(OOB_WRITE)); }

		Ok(())
	}


	/// Write inode contents
	///
	/// If the source buffer is larger than the current inode contents,
	/// it will be automatically resized.
	///
	/// # Note
	/// Larger writes can benefit more from this function
	///
	/// # Warning
	/// This function makes multiple presumptions about the file contents:
	/// - The `block_id`s are consistent (from 1 to n incrementally)
	/// - Each block (except the last one) has size of exactly [`settings::FILE_BLOCK_SIZE`] octets
	///
	/// # Inputs
	/// `inode: u64` is the id of the inode which will be written to
	/// `offset: u64` is the offset in the inode's data
	/// `buffer: &[u8]` is the source buffer
	pub fn unsafe_write(&mut self, inode: u64, offset: u64, buffer: &[u8]) -> Result<(), Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;

		let buffer_len = buffer.len() as u64;
		if buffer.len() == 0 { return Ok(()); }

		// Some pointer calculations
		let start_block = offset / settings::FILE_BLOCK_SIZE;
		let end_block = (offset + buffer_len - 1) / settings::FILE_BLOCK_SIZE;
		let start_idx = offset - start_block * settings::FILE_BLOCK_SIZE;
		let end_idx = offset + buffer_len - end_block * settings::FILE_BLOCK_SIZE - 1;


		// Fetch stuff from the DB (like the current block count)
		// and init the buffer with paddings
		let mut to_write: Vec<u8>;
		let blocks = if start_idx == 0 && end_idx == settings::FILE_BLOCK_SIZE - 1 {
			let result = conn.query(commands::SQL_GET_SIZE_ONLY, Some(&vec![inode.into()]))?;
			let result_item: &database_objects::FileWriteInfoSizeOnly = result.get(0).ok_or(Error::RuntimeError("could not get filesize"))?;

			to_write = vec![0; buffer_len as usize];
			result_item.blocks
		} else if start_idx == 0 {
			let result = conn.query(commands::SQL_GET_SIZE_AND_SINGLE_BLOCK_DATA, Some(&vec![inode.into(), end_block.into()]))?;
			let result_item: &database_objects::FileWriteInfoSingleBlock = result.get(0).ok_or(Error::RuntimeError("could not get filesize and block"))?;

			let padding_end = result_item.block_data.len() as u64;
			let padding_end = if end_idx >= padding_end { 0 } else { padding_end - end_idx - 1 };
			to_write = vec![0; (buffer_len + padding_end) as usize];
			try_slice_from_slice!(&result_item.block_data, end_idx as usize + 1.., to_write, buffer_len as usize..);
			result_item.blocks
		} else if end_idx == settings::FILE_BLOCK_SIZE - 1 {
			let result = conn.query(commands::SQL_GET_SIZE_AND_SINGLE_BLOCK_DATA, Some(&vec![inode.into(), start_block.into()]))?;
			let result_item: &database_objects::FileWriteInfoSingleBlock = result.get(0).ok_or(Error::RuntimeError("could not get filesize and block"))?;
			
			let padding_start = std::cmp::min(result_item.block_data.len() as u64, start_idx);
			to_write = vec![0; (start_idx + buffer_len) as usize];
			try_slice_from_slice!(&result_item.block_data, 0..padding_start as usize, to_write, 0..padding_start as usize);
			result_item.blocks
		} else if start_block == end_block {
			let result = conn.query(commands::SQL_GET_SIZE_AND_SINGLE_BLOCK_DATA, Some(&vec![inode.into(), start_block.into()]))?;
			let result_item: &database_objects::FileWriteInfoSingleBlock = result.get(0).ok_or(Error::RuntimeError("could not get filesize and block"))?;

			let padding_start = std::cmp::min(result_item.block_data.len() as u64, start_idx);
			let padding_end = result_item.block_data.len() as u64;
			let padding_end = if end_idx >= padding_end { 0 } else { padding_end - end_idx - 1 };
			to_write = vec![0; (start_idx + buffer_len + padding_end) as usize];
			try_slice_from_slice!(&result_item.block_data, 0..padding_start as usize, to_write, 0..padding_start as usize);
			try_slice_from_slice!(&result_item.block_data, end_idx as usize + 1.., to_write, buffer_len as usize..);
			result_item.blocks
		} else {
			let result = conn.query(commands::SQL_GET_SIZE_AND_BLOCK_DATA, Some(&vec![inode.into(), start_block.into(), end_block.into()]))?;
			let result_item: &database_objects::FileWriteInfo = result.get(0).ok_or(Error::RuntimeError("could not get filesize and block"))?;

			let padding_start = std::cmp::min(result_item.start_block_data.len() as u64, start_idx);
			let padding_end = result_item.end_block_data.len() as u64;
			let padding_end = if end_idx >= padding_end { 0 } else { padding_end - end_idx - 1 };
			to_write = vec![0; (start_idx + buffer_len + padding_end) as usize];
			to_write[0..padding_start as usize].copy_from_slice(&result_item.start_block_data[0..padding_start as usize]);
			to_write[buffer_len as usize..].copy_from_slice(&result_item.end_block_data[end_idx as usize + 1..]);
			result_item.blocks
		};
		let blocks: u64 = blocks.try_into().map_err(|_| Error::RuntimeError(DBI64_TO_DRU32_CONVERSION_ERROR_MESSAGE))?;

		if blocks < start_block + 1 {
			conn.command(commands::SQL_PAD_LAST_BLOCK, Some(&vec![settings::FILE_BLOCK_SIZE.into(), inode.into()]))?;
		}
		if blocks < start_block { 
			// Oh no... we need to pad the file up to the insertion point
			let command = commands::dynamic_queries::sql_pad_until(inode, blocks + 1, start_block + 1);
			conn.command(command.as_str(), None)?;
		}

		// Copy buffer
		to_write[start_idx as usize..=((end_block - start_block) as usize * settings::FILE_BLOCK_SIZE_USIZE + end_idx as usize)].copy_from_slice(buffer);

		// Convert data to a useful format
		let mut data: Vec<_> = Vec::new();
		let mut ptr = 0;
		while ptr < to_write.len() {
			data.push(Vec::from(&to_write[ptr..std::cmp::min(ptr + settings::FILE_BLOCK_SIZE_USIZE, to_write.len())]).into());
			ptr += settings::FILE_BLOCK_SIZE_USIZE;
		}

		// Generate the insert query
		let command = commands::dynamic_queries::sql_unsafe_write(inode, start_block + 1, end_block + 1);
		// Now let's INSERT ... good luck
		conn.command(command.as_str(), Some(&data))?;

		Ok(())
	}


	/// Create a new inode/file pair with no blocks
	///
	/// # Inputs
	/// `parent_inode: u64` specifies the parent inode where the file should be created
	/// `name: &OsStr` is the name of the file to be created
	/// `kind: FileType` sets the inode type
	/// `attr: FileSetAttr` sets the remaining inode attributes
	pub fn mknod(&mut self, parent_inode: u64, name: &std::ffi::OsStr, kind: driver_objects::FileType, attr: driver_objects::FileSetAttr) -> Result<driver_objects::FileAttr, Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		let inode = conn.command(commands::SQL_CREATE_INODE, Some(&vec![
			attr.uid.into(),
			attr.gid.into(),
			Into::<String>::into(kind).into(),
			attr.perm.special.into(),
			attr.perm.owner.into(),
			attr.perm.group.into(),
			attr.perm.other.into()
		]))?.last_insert_id;
		drop(conn);

		self.link(parent_inode, name, inode)?;
		self.getattr(inode)
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
		let status = conn.command(commands::SQL_CREATE_FILE, Some(&vec![
			parent_inode.into(),
			path.into(),
			dest_inode.into()
		]))?;
		drop(conn);
		match status.rows_affected {
			1 => Ok(()),
			_ => Err(Error::RuntimeError("no changes made"))
		}
	}


	/// Truncates or expands an inode by deleting or adding blocks
	///
	/// # Inputs
	/// `inode: u64` specifies the inode
	/// `new_size: u64` specifies the new size the file should have
	pub fn resize(&mut self, inode: u64, new_size: u64) -> Result<(), Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		if new_size == 0 {
			conn.command(commands::SQL_DROP_BLOCKS, Some(&vec![inode.into()]))?;
			return Ok(());
		}

		// Get the current file head
		let file_head: Vec<database_objects::FileHead> = conn.query(commands::SQL_GET_FILE_HEAD, Some(&vec![inode.into()]))?;
		let file_head = file_head.get(0).ok_or(Error::RuntimeError("could not get filesize"))?;

		// Pad with null blocks if necessary
		let block_count: u64 = file_head.bc.try_into().map_err(|_| Error::RuntimeError(DBI64_TO_DRU32_CONVERSION_ERROR_MESSAGE))?;
		let new_block_count = new_size.div_ceil(settings::FILE_BLOCK_SIZE);
		let strip_blocks_count = if block_count < new_block_count {
			conn.command(commands::SQL_RESIZE_LAST_BLOCK, Some(&vec![settings::FILE_BLOCK_SIZE.into(), settings::FILE_BLOCK_SIZE.into(), inode.into()]))?;
			conn.command(commands::dynamic_queries::sql_pad_file(
				inode.try_into().map_err(|_| Error::RuntimeError(DRU64_TO_DBU32_CONVERSION_ERROR_MESSAGE))?,
				file_head.last_block_id,
				(new_block_count - block_count).try_into().map_err(|_| Error::RuntimeError(DRU64_TO_DBU32_CONVERSION_ERROR_MESSAGE))?
			).as_str(), None)?;
			0
		} else {
			block_count - new_block_count
		};

		// Trim the file to the desired byte size
		let new_last_block_size = new_size - (new_block_count - 1) * settings::FILE_BLOCK_SIZE;
		if strip_blocks_count != 0 {
			conn.command(commands::SQL_TRIM_BLOCKS, Some(&vec![inode.into(), strip_blocks_count.into()]))?;
		}
		conn.command(commands::SQL_RESIZE_LAST_BLOCK, Some(&vec![new_last_block_size.into(), new_last_block_size.into(), inode.into()]))?;

		Ok(())
	}


	/// Sets inode attributes
	///
	/// # Inputs
	/// `inode: u64` specifies the inode
	/// `attr: FileSetAttr` sets the inode attributes
	pub fn setattr(&mut self, inode: u64, attr: driver_objects::FileSetAttr) -> Result<driver_objects::FileAttr, Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError(CONN_LOCK_FAILED))?;
		let status = conn.command(commands::SQL_UPDATE_INODE, Some(&vec![
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
		if status.rows_affected != 1 {
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
		let status = conn.command(commands::SQL_DELETE_FILE, Some(&vec![path.into(), parent_inode.into()]))?;
		drop(conn);
		if status.rows_affected != 1 {
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
		let status = conn.command(commands::SQL_DELETE_INODE, Some(&vec![inode.into()]))?;
		match status.rows_affected {
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
		let status = conn.command(commands::SQL_RENAME_FILE, Some(&vec![dest_parent_inode.into(), dest_path.into(), src_parent_inode.into(), src_path.into()]))?;

		match status.rows_affected {
			1 => Ok(()),
			_ => Err(Error::RuntimeError("no changes made"))
		}
	}
}




#[cfg(feature = "integration_testing")]
#[cfg(test)]
mod test {
	use std::ffi::OsString;
	use serial_test::serial;
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
	#[serial]
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
	#[serial]
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
	#[serial]
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
	#[serial]
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

	#[test]
	#[serial]
	fn write_to_file_01() {
		let mut sql = TranslationLayer::new().unwrap();
		let original: &[u8] = &"Hello, world!\n".as_bytes();
		let buffer: &[u8] = &"Wasup".as_bytes();
		let _write = sql.write(2, 0, buffer).unwrap();
		let changed_value: &mut [u8] = &mut [0; 14];
		let read = sql.read(2, 0, changed_value).unwrap();
		let _write = sql.write(2, 0, original).unwrap();
		assert_eq!(read, 14);
		assert_eq!(changed_value, "Wasup, world!\n".as_bytes());
	}

	#[test]
	#[serial]
	fn write_to_file_02() {
		let mut sql = TranslationLayer::new().unwrap();
		let original: &mut[u8] = &mut[0; 4096 * 3 + 4];
		original[4096*3..].copy_from_slice("aaaa".as_bytes());
		let buffer: &[u8] = &"bbbb".as_bytes();
		let _write = sql.write(3, 4096 * 3 - 1, buffer).unwrap();
		let changed_value: &mut [u8] = &mut [0; 7];
		let read = sql.read(3, 4096 * 3 - 1, changed_value).unwrap();
		let _write = sql.write(3, 0, original).unwrap();
		assert_eq!(changed_value, "bbbba\n\0".as_bytes());
		assert_eq!(read, 6);
	}

	#[test]
	#[serial]
	fn resize_01() {
		let mut sql = TranslationLayer::new().unwrap();
		sql.resize(3, 4096 * 3 + 3).unwrap();
		let original: &mut[u8] = &mut[0; 4096 * 3 + 5];
		let read_bytes = sql.read(3, 0, original).unwrap();
		println!("{:?}", original);
		assert_eq!(read_bytes, 4096 * 3 + 3);

		sql.resize(3, 4096 * 3 + 5).unwrap();
		sql.write(3, 4096 * 3 + 3, &['a' as u8, '\n' as u8]).unwrap();
		let read_bytes = sql.read(3, 0, original).unwrap();
		assert_eq!(read_bytes, 4096 * 3 + 5);
	}

	#[test]
	#[serial]
	fn resize_02() {
		let mut sql = TranslationLayer::new().unwrap();
		sql.resize(3, 4096 + 1028).unwrap();
		let original: &mut[u8] = &mut[0; 4096 * 3 + 5];
		let read_bytes = sql.read(3, 0, original).unwrap();
		println!("{:?}", original);
		assert_eq!(read_bytes, 4096 + 1028);

		sql.resize(3, 4096 * 3 + 5).unwrap();
		sql.write(3, 4096 * 3, &['a' as u8, 'a' as u8, 'a' as u8,'a' as u8, '\n' as u8]).unwrap();
		let read_bytes = sql.read(3, 0, original).unwrap();
		assert_eq!(read_bytes, 4096 * 3 + 5);
	}

	#[test]
	#[serial]
	fn write_01() {
		// END PAD
		let mut sql = TranslationLayer::new().unwrap();
		sql.unsafe_write(3, 4096, &[1_u8; 1024]).unwrap();
		let read = &mut [0_u8; 4096];
		let read_bytes = sql.read(3, 4096, read).unwrap();
		sql.unsafe_write(3, 4096, &[0_u8; 1024]).unwrap();
		let mut target = Vec::from([1_u8; 1024]);
		target.extend_from_slice(&[0_u8; 4096 - 1024]);
		assert_eq!(read, target.as_slice());
		assert_eq!(read_bytes, 4096);
	}

	#[test]
	#[serial]
	fn write_02() {
		// START PAD
		let mut sql = TranslationLayer::new().unwrap();
		sql.unsafe_write(3, 4096*2-1024, &[1_u8; 1024]).unwrap();
		let read = &mut [0_u8; 4096];
		let read_bytes = sql.read(3, 4096, read).unwrap();
		sql.unsafe_write(3, 4096*2-1024, &[0_u8; 1024]).unwrap();
		let mut target = Vec::from([0_u8; 4096-1024]);
		target.extend_from_slice(&[1_u8; 1024]);
		assert_eq!(read, target.as_slice());
		assert_eq!(read_bytes, 4096);
	}

	#[test]
	#[serial]
	fn write_03() {
		// ALIGNED
		let mut sql = TranslationLayer::new().unwrap();
		sql.unsafe_write(3, 0, &[1_u8; 2*4096]).unwrap();
		let read = &mut [0_u8; 4096*2];
		let read_bytes = sql.read(3, 0, read).unwrap();
		sql.unsafe_write(3, 0, &[0_u8; 2*4096]).unwrap();
		let target = Vec::from([1_u8; 4096*2]);
		assert_eq!(read, target.as_slice());
		assert_eq!(read_bytes, 4096*2);
	}

	#[test]
	#[serial]
	fn write_04() {
		// END PAD
		let mut sql = TranslationLayer::new().unwrap();
		sql.unsafe_write(3, 0, &[1_u8; 2*4096-1]).unwrap();
		let read = &mut [0_u8; 4096*2];
		let read_bytes = sql.read(3, 0, read).unwrap();
		sql.unsafe_write(3, 0, &[0_u8; 2*4096-1]).unwrap();
		let mut target = Vec::from([1_u8; 4096*2-1]);
		target.extend_from_slice(&[0_u8; 1]);
		assert_eq!(read, target.as_slice());
		assert_eq!(read_bytes, 4096*2);
	}

	#[test]
	#[serial]
	fn disjoint_write_01() {
		let mut sql = TranslationLayer::new().unwrap();
		sql.unsafe_write(3, 4096 * 4, &[2_u8; 4096]).unwrap();
		let read = &mut [0_u8; 4096*2];
		let read_bytes = sql.read(3, 4096 * 3, read).unwrap();
		sql.resize(3, 4096 * 3 + 5).unwrap();
		let mut target: Vec<u8> = Vec::from(['a' as u8, 'a' as u8, 'a' as u8, 'a' as u8, '\n' as u8]);
		target.extend_from_slice(&[0_u8; 4096 - 5]);
		target.extend_from_slice(&[2_u8; 4096]);
		assert_eq!(read, target.as_slice());
		assert_eq!(read_bytes, 4096*2);
	}

	#[test]
	#[serial]
	fn disjoint_write_02() {
		let mut sql = TranslationLayer::new().unwrap();
		sql.unsafe_write(3, 4096 * 5, &[2_u8; 4096]).unwrap();
		let read = &mut [0_u8; 4096*3];
		let read_bytes = sql.read(3, 4096 * 3, read).unwrap();
		sql.resize(3, 4096 * 3 + 5).unwrap();
		let mut target: Vec<u8> = Vec::from(['a' as u8, 'a' as u8, 'a' as u8, 'a' as u8, '\n' as u8]);
		target.extend_from_slice(&[0_u8; 4096 - 5 + 4096]);
		target.extend_from_slice(&[2_u8; 4096]);
		assert_eq!(read, target.as_slice());
		assert_eq!(read_bytes, 4096*3);
	}

	#[test]
	#[serial]
	fn disjoint_write_03() {
		let mut sql = TranslationLayer::new().unwrap();
		sql.unsafe_write(3, 4096 * 9, &[2_u8; 4096]).unwrap();
		let read = &mut [0_u8; 4096*8];
		let read_bytes = sql.read(3, 4096 * 3, read).unwrap();
		sql.resize(3, 4096 * 3 + 5).unwrap();
		let mut target: Vec<u8> = Vec::from(['a' as u8, 'a' as u8, 'a' as u8, 'a' as u8, '\n' as u8]);
		target.extend_from_slice(&[0_u8; 4096 - 5 + 4096 * 5]);
		target.extend_from_slice(&[2_u8; 4096]);
		target.extend_from_slice(&[0_u8; 4096]);
		assert_eq!(read, target.as_slice());
		assert_eq!(read_bytes, 4096*7);
	}
}
