//! Objects for use with the FUSE driver
//!
//! These objects contain **only values extracted from the database** in the appropriate format.


use std::time::SystemTime;


/// Database supported `FileType`s
#[derive(Debug)]
pub enum FileType {
	File,
	Directory,
	Symlink,
}


/// FS object permissions for owner (user) and group
///
/// # Permission values
///
/// - `x` (execute) - **1**
/// - `w` (write)   - **2**
/// - `r` (read)    - **4**
///
/// # Values in fields
/// The values in fields in this struct are the sum of all permissions for the user or group.
#[derive(Debug)]
pub struct Permissions {
	pub owner: u8,
	pub group: u8,
}


/// Database supported `FileAttr`ibute
#[derive(Debug)]
pub struct FileAttr {
	/// Inode id
	pub ino: u32,
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
