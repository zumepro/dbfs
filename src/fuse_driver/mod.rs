use crate::sql_translation_layer::driver_objects;
use crate::sql_translation_layer::{TranslationLayer, driver_objects::FileAttr};
use crate::debug;

use fuser;
use libc::ENOENT;

use std::ffi::OsStr;
use std::time::Duration;

struct DbfsDriver {
	tl: TranslationLayer,
	last_readdir_inode: u64,
	last_readdir: Vec<driver_objects::DirectoryEntry>
}

impl DbfsDriver {
	fn new(tl: TranslationLayer) -> Self {
		Self {
			tl,
			last_readdir_inode: u64::MAX,
			last_readdir: Vec::new()
		}
	}
}


impl Into<fuser::FileType> for driver_objects::FileType {
	fn into(self) -> fuser::FileType {
		match self {
			Self::File => fuser::FileType::RegularFile,
			Self::Directory => fuser::FileType::Directory,
			Self::Symlink => fuser::FileType::Symlink,
		}
	}
}

impl Into<u16> for driver_objects::Permissions {
	/// **REVIEW NEEDED**
	fn into(self) -> u16 {
		(self.owner as u16) << 6 | (self.group as u16) << 3 | self.other as u16
	}
}

impl Into<fuser::FileAttr> for FileAttr {
	fn into(self) -> fuser::FileAttr {
		fuser::FileAttr {
			ino: self.ino.into(),
			size: self.bytes,
			blocks: self.blocks,
			atime: self.atime,
			mtime: self.mtime,
			ctime: self.ctime,
			crtime: self.ctime,
			kind: self.kind.into(),
			perm: self.perm.into(),
			nlink: self.hardlinks.into(),
			uid: self.uid,
			gid: self.gid,
			rdev: 0, // REVIEW NEEDED
			blksize: 4096, // REVIEW NEEDED
			flags: 0, // REVIEW NEEDED
		}
	}
}


const TTL: Duration = Duration::from_secs(1);

impl fuser::Filesystem for DbfsDriver {
	fn lookup(&mut self, _req: &fuser::Request, parent_inode: u64, name: &OsStr, reply: fuser::ReplyEntry) {
		debug!("lookup: inode {}, name {:?}", parent_inode, name);
		match self.tl.lookup(name, parent_inode) {
			Ok(attr) => {
				debug!(" -> OK: {:?}", &attr);
				reply.entry(&TTL, &attr.into(), 0);
			},
			Err(err) => {
				debug!(" -> Err {:?}", err);
				reply.error(ENOENT);
			}
		}
	}

	fn getattr(&mut self, _req: &fuser::Request, inode: u64, reply: fuser::ReplyAttr) {
		debug!("getattr: inode {}", inode);
		match self.tl.getattr(inode) {
			Ok(attr) => {
				debug!(" -> OK: {:?}", &attr);
				reply.attr(&TTL, &attr.into());
			},
			Err(err) => {
				debug!(" -> Err {:?}", err);
				reply.error(ENOENT);
			}
		}
	}

	fn read(
		&mut self,
		_req: &fuser::Request,
		inode: u64,
		_fh: u64,
		offset: i64,
		size: u32,
		_flags: i32,
		_lock: Option<u64>,
		reply: fuser::ReplyData,
	) {
		debug!("read: inode {}, offset {}, size {}", inode, offset, size);
		let mut buf = vec![0u8; size as usize];
		match self.tl.read(inode, offset as u64, &mut buf) {
			Ok(()) => {
				debug!(" -> OK");
				reply.data(&buf);
			},
			Err(err) => {
				debug!(" -> Err {:?}", err);
				reply.error(ENOENT);
			}
		}
	}

	fn readdir(
		&mut self,
		_req: &fuser::Request,
		inode: u64,
		_fh: u64,
		offset: i64,
		mut reply: fuser::ReplyDirectory,
	) {
		debug!("readdir: inode {}, offset {}", inode, offset);
		if inode != self.last_readdir_inode {
			debug!(" -> cache miss, fetching from DB");
			self.last_readdir_inode = inode;
			self.last_readdir = match self.tl.readdir(inode) {
				Ok(val) => {
					debug!(" -> OK (inode {} has {} entries)", inode, val.len());
					val
				},
				Err(err) => {
					debug!(" -> Err {:?}", err);
					reply.error(ENOENT);
					return
				}
			}
		}

		let mut i = offset;

		loop {
			let entry = match self.last_readdir.get(i as usize) {
				Some(val) => val,
				None => break
			};

			debug!(" -> sending #{} (inode {}, name {:?}, type {:?})", i, entry.inode, entry.name, entry.ftype);
			i += 1;
			if reply.add(entry.inode, i, entry.ftype.clone().into(), &entry.name) {
				break
			}
		}

		debug!(" -> OK");

		reply.ok();
	}
}

pub fn run_forever(tl: TranslationLayer, mountpoint: &str) -> ! {
	let options = vec![fuser::MountOption::RO, fuser::MountOption::FSName("hello".to_string())];
	let driver = DbfsDriver::new(tl);
	fuser::mount2(driver, mountpoint, &options).unwrap();
	panic!("FUSE driver crashed");
}

