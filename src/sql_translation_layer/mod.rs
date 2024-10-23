mod objects;

use fuser;

use std::ffi::OsString;

pub struct TranslationLayer {}

pub enum Error {
	Unimplemented,
	OutOfEntries
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

	pub fn readdir(&mut self, _inode: u64, _offset: u64) -> Result<(u64, fuser::FileType, OsString), Error> {
		Err(Error::OutOfEntries)
	}

	pub fn read(&mut self, _inode: u64, _offset: u64, _buffer: &mut [u8]) -> Result<(), Error> {
		Err(Error::Unimplemented)
	}
}

