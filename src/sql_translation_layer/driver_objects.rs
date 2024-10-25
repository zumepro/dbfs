//! Objects for use with the FUSE driver
//!
//! These objects contain **only values extracted from the database** in the appropriate format.


use std::ffi::OsString;
use std::time::SystemTime;

use super::database_enums;


/// Database supported `FileType`s
#[derive(Clone, Debug, PartialEq)]
pub enum FileType {
	File,
	Directory,
	Symlink,
}
impl TryFrom<database_enums::FileType> for FileType {
	type Error = super::Error;

	fn try_from(value: database_enums::FileType) -> Result<Self, Self::Error> {
		Ok(match value {
			database_enums::FileType::RegularFile => Self::File,
			database_enums::FileType::SymbolicLink => Self::Symlink,
			database_enums::FileType::Directory => Self::Directory,
			database_enums::FileType::Unknown => Err(super::Error::RuntimeError("unknown filetype"))?,
		})
	}
}


/// Directory entry structure, returned by the driver when processing a `readdir` request.
pub struct DirectoryEntry {
	pub inode: u64,
	pub ftype: FileType,
	pub name: OsString
}


/// FS object permissions for owner (user) and group
///
/// # Permission values
///
/// | Flag | Permission | Value |
/// | ---- | ---------- | ----- |
/// | `x`  | Execute    | **1** |
/// | `w`  | Write      | **2** |
/// | `r`  | Read       | **4** |
///
/// # Values in fields
/// The values in fields in this struct are the sum of all permissions for the user or group.
#[derive(Debug, PartialEq)]
pub struct Permissions {
	pub owner: u8,
	pub group: u8,
	pub other: u8,
}


/// Database supported `FileAttr`ibute
#[derive(Debug, PartialEq)]
pub struct FileAttr {
	/// Inode id
	pub ino: u32,
	/// Owner user id
	pub uid: u32,
	/// Group id
	pub gid: u32,
	/// Number of hardlinks
	pub hardlinks: u32,
	/// Filesize in bytes
	pub bytes: u64,
	/// Filesize in blocks (4096) - rounded up
	pub blocks: u64,
	/// Time of last access
	pub atime: SystemTime,
	/// Time of last modification
	pub mtime: SystemTime,
	/// Time of creation
	pub ctime: SystemTime,
	/// Kind of file (see [`FileType`] for more info)
	pub kind: FileType,
	/// User + Group permissions (see [`Permissions`] for more info)
	pub perm: Permissions,
}
