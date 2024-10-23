mod database_objects;
pub mod driver_objects;


use fuser;
use std::ffi::OsString;
use crate::db_connector::DbConnectorError;


pub struct TranslationLayer {}


pub enum Error {
	DbConnectorError(DbConnectorError),
	Unimplemented,
}
impl std::fmt::Display for Error {
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "sql_translation_error: {}", match self {
			Self::DbConnectorError(val) => val.to_string(),
			Self::Unimplemented => "method isn't implemented yet".to_string(),
		})
	}
}


impl TranslationLayer {
	pub fn new() -> Self {
		Self {}
	}

	pub fn getattr(&mut self, _inode: u64) -> Result<fuser::FileAttr, Error> {
		Err(Error::Unimplemented)
	}

	pub fn lookup(&mut self, _name: &std::ffi::OsStr, _parent_inode: u64) -> Result<fuser::FileAttr, Error> {
		Err(Error::Unimplemented)
	}

	pub fn readdir(&mut self, _inode: u64, _offset: u64) -> Result<Option<(u64, fuser::FileType, OsString)>, Error> {
		Err(Error::Unimplemented)
	}

	pub fn read(&mut self, _inode: u64, _offset: u64, _buffer: &mut [u8]) -> Result<(), Error> {
		Err(Error::Unimplemented)
	}
}

