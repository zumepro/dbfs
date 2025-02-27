//! Objects for use with the FUSE driver
//!
//! These objects contain **only values extracted from the database** in the appropriate format.


use std::ffi::OsString;
use std::time::SystemTime;

use super::{database_enums, database_objects};


/// Database supported `FileType`s
#[derive(Clone, Debug, PartialEq)]
pub enum FileType {
	File,
	Directory,
	Symlink,
	NamedPipe,
	Socket
}
impl TryFrom<database_enums::FileType> for FileType {
	type Error = super::Error;

	fn try_from(value: database_enums::FileType) -> Result<Self, Self::Error> {
		Ok(match value {
			database_enums::FileType::RegularFile => Self::File,
			database_enums::FileType::SymbolicLink => Self::Symlink,
			database_enums::FileType::Directory => Self::Directory,
			database_enums::FileType::NamedPipe => Self::NamedPipe,
			database_enums::FileType::Socket => Self::Socket,
			database_enums::FileType::Unknown => Err(super::Error::RuntimeError("unknown filetype"))?,
		})
	}
}
impl Into<String> for FileType {
	fn into(self) -> String {
	    match self {
			FileType::File => "-".to_string(),
			FileType::Directory => "d".to_string(),
			FileType::Symlink => "l".to_string(),
			FileType::NamedPipe => "p".to_string(),
			FileType::Socket => "s".to_string()
		}
	}
}


/// Directory entry structure, returned by the driver when processing a `readdir` request.
#[derive(Debug, PartialEq)]
pub struct DirectoryEntry {
	pub inode: u64,
	pub ftype: FileType,
	pub name: OsString
}
impl TryFrom<&database_objects::DirectoryEntry> for DirectoryEntry {
	type Error = super::Error;

	fn try_from(value: &database_objects::DirectoryEntry) -> Result<Self, Self::Error> {
		Ok(Self {
			inode: value.inode_id.into(),
			ftype: <&String as Into<database_enums::FileType>>::into(&value.file_type).try_into()?,
			name: value.name.clone().into(),
		})
	}
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
	pub special: u8,
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


/// Like `FileAttr`, but used for setting attributes
///
/// Omits `ino`, `hardlinks`, `bytes`, `blocks` and `kind`, as those fields
/// are not actually modifiable by simply writing to the inode.
#[derive(Debug, PartialEq)]
pub struct FileSetAttr {
	/// Owner user id
	pub uid: u32,
	/// Group id
	pub gid: u32,
	/// Time of last access
	pub atime: SystemTime,
	/// Time of last modification
	pub mtime: SystemTime,
	/// Time of creation
	pub ctime: SystemTime,
	/// User + Group permissions (see [`Permissions`] for more info)
	pub perm: Permissions,
}


/// File size structure
#[derive(Debug, PartialEq)]
pub struct FileSize {
	/// File size in bytes
    pub bytes: u64,
	/// File size in blocks
    pub blocks: u64,
}
impl Into<FileSize> for database_objects::FileSize {
	fn into(self) -> FileSize {
	    FileSize {
			bytes: self.bytes,
			blocks: self.blocks
		}
	}
}


/// Filesystem statistics structure
#[derive(Debug, PartialEq)]
pub struct FilesystemStat {
	pub used_blocks: u64,
	pub used_inodes: u64
}

