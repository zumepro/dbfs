mod database_objects;
mod database_enums;
pub mod driver_objects;
mod commands;


const DBI64_TO_DRU32_CONVERSION_ERROR_MESSAGE: &'static str = "could not convert database's u64 to i32 for the driver";


use database_objects::{FileHardlinks, FileSize, Inode};
use driver_objects::DirectoryEntry;
use std::sync::Mutex;
use crate::db_connector::{DbConnector, DbConnectorError};


pub struct TranslationLayer (Mutex<DbConnector>);


#[derive(Debug)]
pub enum Error {
	DbConnectorError(DbConnectorError),
	RuntimeError(&'static str),
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
	

	/// Get attributes for file
	///
	/// # Inputs
	/// `_inode: u64` is the id of the inode of the target file
	///
	/// # Warnings
	/// This is a relatively expensive operation, so use as sparingly as possible.
	pub fn getattr(&mut self, _inode: u64) -> Result<driver_objects::FileAttr, Error> {
		let mut conn = self.0.lock().map_err(|_| Error::RuntimeError("mutex lock error"))?;
		let inode: Vec<Inode> = conn.query(commands::SQL_GET_INODE, Some(&vec![_inode.into()]))?;
		let Some(inode) = inode.get(0) else {
			return Err(Error::RuntimeError("no inode found with given id"));
		};

		let hardlinks: Vec<FileHardlinks> = conn.query(commands::SQL_COUNT_HARDLINKS, Some(&vec![_inode.into()]))?;
		let hardlinks: i64 = hardlinks.get(0).ok_or(Error::RuntimeError("could not count hardlinks"))?.hardlinks;
		if hardlinks == 0 { return Err(Error::RuntimeError("found an orphaned inode")); }

		let file_type: database_enums::FileType = (&inode.file_type).into();
		let file_type: driver_objects::FileType = driver_objects::FileType::try_from(file_type)?;

		let file_size: FileSize = match file_type {
			driver_objects::FileType::File | driver_objects::FileType::Symlink => match conn.query(commands::SQL_GET_FILE_SIZE, Some(&vec![_inode.into()]))?.get(0) {
				Some(val) => *val,
				None => FileSize { bytes: 0, blocks: 0 },
			},
			driver_objects::FileType::Directory => FileSize { bytes: 0, blocks: 0 },
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
			perm: driver_objects::Permissions { owner: inode.user_perm, group: inode.group_perm, other: inode.other_perm }
		})
	}


	pub fn lookup(&mut self, _name: &std::ffi::OsStr, _parent_inode: u64) -> Result<driver_objects::FileAttr, Error> {
		Err(Error::Unimplemented)
	}


	pub fn readdir(&mut self, _inode: u64) -> Result<Vec<DirectoryEntry>, Error> {
		Err(Error::Unimplemented)
	}


	pub fn read(&mut self, _inode: u64, _offset: u64, _buffer: &mut [u8]) -> Result<(), Error> {
		Err(Error::Unimplemented)
	}
}




#[cfg(feature = "integration_testing")]
#[cfg(test)]
mod test {
	use sqlx::types::chrono::{DateTime, Local};
	use super::*;


	#[test]
	fn getattr_dir() {
		let mut sql = TranslationLayer::new().unwrap();
		let attr = sql.getattr(1).unwrap();
		assert_eq!(attr, driver_objects::FileAttr {
			ino: 1,
			uid: 1,
			gid: 1,
			hardlinks: 1,
			bytes: 0,
			blocks: 0,
			atime: "2024-10-24 17:52:52+0000".parse::<DateTime<Local>>().unwrap().into(),
			mtime: "2024-10-24 17:53:10+0000".parse::<DateTime<Local>>().unwrap().into(),
			ctime: "2024-10-24 17:52:52+0000".parse::<DateTime<Local>>().unwrap().into(),
			kind: driver_objects::FileType::Directory,
			perm: driver_objects::Permissions { owner: 7, group: 5, other: 5 },
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
			perm: driver_objects::Permissions { owner: 6, group: 4, other: 4 },
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
			hardlinks: 1,
			bytes: 4096 * 3 + 5,
			blocks: 4,
			atime: "2024-10-24 17:56:34+0000".parse::<DateTime<Local>>().unwrap().into(),
			mtime: "2024-10-24 17:57:14+0000".parse::<DateTime<Local>>().unwrap().into(),
			ctime: "2024-10-24 17:56:34+0000".parse::<DateTime<Local>>().unwrap().into(),
			kind: driver_objects::FileType::File,
			perm: driver_objects::Permissions { owner: 6, group: 4, other: 4 },
		});
	}
}
