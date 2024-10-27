use crate::sql_translation_layer::{BLOCK_SIZE, MAX_NAME_LEN};
use crate::sql_translation_layer::driver_objects;
use crate::sql_translation_layer::TranslationLayer;
use crate::debug;

use fuser;
use libc::{EINVAL, ENOENT, ENOTEMPTY};

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
	fn into(self) -> u16 {
		(self.owner as u16) << 6 | (self.group as u16) << 3 | self.other as u16
	}
}

impl Into<driver_objects::Permissions> for u16 {
	fn into(self) -> driver_objects::Permissions {
	    driver_objects::Permissions {
			owner: ((self >> 6) & 7) as u8,
			group: ((self >> 3) & 7) as u8,
			other: (self & 7) as u8
		}
	}
}

impl Into<fuser::FileAttr> for driver_objects::FileAttr {
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
			rdev: 0,
			blksize: 4096,
			flags: 0,
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

		if size == 0 {
			debug!(" -> OK, no read operation necessary");
			reply.data(&[]);
			return
		}

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

	fn readlink(&mut self, _req: &fuser::Request<'_>, inode: u64, reply: fuser::ReplyData) {
	    debug!("readlink: inode {}", inode);
		let size: u32 = match self.tl.filesize(inode) {
			Ok(size) => {
				debug!(" -> size {}", size.bytes);
				size.bytes as u32
			},
			Err(err) => {
				debug!(" -> Err while determining link size: {:?}", err);
				reply.error(ENOENT);
				return
			}
		};

		self.read(_req, inode, 0, 0, size, 0, None, reply);
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

	fn statfs(&mut self, _req: &fuser::Request<'_>, inode: u64, reply: fuser::ReplyStatfs) {
	    debug!("statfs: inode {}", inode);

		let stat = match self.tl.statfs() {
			Ok(val) => {
				debug!(" -> OK {:?}", val);
				val
			},
			Err(err) => {
				debug!(" -> Err {:?}", err);
				reply.error(ENOENT);
				return
			}
		};

		reply.statfs(
			stat.free_blocks  + stat.used_blocks,
			stat.free_blocks,
			stat.free_blocks,
			stat.used_inodes,
			stat.free_blocks,
			BLOCK_SIZE,
			MAX_NAME_LEN,
			BLOCK_SIZE
		);
	}

	fn mkdir(
		&mut self,
		req: &fuser::Request<'_>,
		parent_inode: u64,
		name: &OsStr,
		mode: u32,
		_umask: u32,
		reply: fuser::ReplyEntry,
	) {
		debug!("mkdir: parent inode {}, name {:?}, mode {:o}", parent_inode, name, mode);

		let time = std::time::SystemTime::now();
		let attr = driver_objects::FileSetAttr {
			uid: req.uid(),
			gid: req.gid(),
			atime: time,
			mtime: time,
			ctime: time,
			perm: (mode as u16).into()
		};

		match self.tl.mknod(parent_inode, name, driver_objects::FileType::Directory, attr) {
			Ok(attr) => {
				debug!(" -> OK {:?}", attr);
				reply.entry(&TTL, &attr.into(), 0);
			},
			Err(err) => {
				debug!(" -> Err {:?}", err);
				reply.error(ENOENT);
			}
		}
	}

	fn rmdir(&mut self, _req: &fuser::Request<'_>, parent_inode: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
	    debug!("rmdir: parent inode {}, name {:?}", parent_inode, name);

		let inode = match self.tl.lookup_id(name, parent_inode) {
			Ok(inode) => inode,
			Err(err) => {
				debug!(" -> Err while performing lookup: {:?}", err);
				reply.error(ENOENT);
				return
			}
		};

		let children = match self.tl.count_children(inode) {
			Ok(val) => val,
			Err(err) => {
				debug!(" -> Err while counting children: {:?}", err);
				reply.error(ENOENT);
				return
			}
		};

		if children > 2 {
			debug!(" -> Err, directory not empty");
			reply.error(ENOTEMPTY);
			return
		}

		self.unlink(_req, parent_inode, name, reply);
	}

	fn unlink(&mut self, _req: &fuser::Request<'_>, parent_inode: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
	    debug!("unlink: parent inode {}, name {:?}", parent_inode, name);

		if let Err(err) = self.tl.unlink(parent_inode, name) {
			debug!(" -> Err {:?}", err);
			reply.error(ENOENT);
			return
		}

		debug!(" -> OK");
		reply.ok();
	}

	fn link(
		&mut self,
		_req: &fuser::Request<'_>,
		inode: u64,
		new_parent_inode: u64,
		new_name: &OsStr,
		reply: fuser::ReplyEntry,
	) {
	    debug!("link: inode {}, new parent inode {}, new name {:?}", inode, new_parent_inode, new_name);

		if let Err(err) = self.tl.link(new_parent_inode, new_name, inode) {
			debug!(" -> Err while creating link: {:?}", err);
			reply.error(ENOENT);
			return
		}

		let attr = match self.tl.getattr(inode) {
			Ok(attr) => attr,
			Err(err) => {
				debug!(" -> Err while fetching attributes: {:?}", err);
				reply.error(ENOENT);
				return
			}
		};

		debug!(" -> OK");
		reply.entry(&TTL, &attr.into(), 0);
	}

	fn symlink(
		&mut self,
		req: &fuser::Request<'_>,
		parent_inode: u64,
		link_name: &OsStr,
		target: &std::path::Path,
		reply: fuser::ReplyEntry,
	) {
		debug!("symlink: parent inode {}, name {:?}, target {:?}", parent_inode, link_name, target);

		let target = match target.to_str() {
			Some(target) => target.as_bytes(),
			None => {
				debug!(" -> Err parsing path");
				reply.error(EINVAL);
				return
			}
		};

		let time = std::time::SystemTime::now();
		let attr = driver_objects::FileSetAttr {
			uid: req.uid(),
			gid: req.gid(),
			atime: time,
			ctime: time,
			mtime: time,
			perm: driver_objects::Permissions { owner: 7, group: 7, other: 7 }
		};

		let attr = match self.tl.mknod(parent_inode, link_name, driver_objects::FileType::Symlink, attr) {
			Ok(attr) => attr,
			Err(err) => {
				debug!(" -> Err while creating node: {:?}", err);
				reply.error(ENOENT);
				return
			}
		};

		if let Err(err) = self.tl.write(attr.ino.into(), 0, target) {
			debug!(" -> Err while writing symlink data: {:?}", err);
			reply.error(ENOENT);
			return
		}

		match self.tl.getattr(attr.ino.into()) {
			Ok(attr) => {
				debug!(" -> OK: {:?}", &attr);
				reply.entry(&TTL, &attr.into(), 0);
			},
			Err(err) => {
				debug!(" -> Err while fetching updated attributes: {:?}", err);
				reply.error(ENOENT);
			}
		}
	}

	// TODO - setattr
	// TODO - mknod
	// TODO - rename
	// TODO - create
	// TODO - write
	// TODO - open (?)
}

pub fn run_forever(tl: TranslationLayer, mountpoint: &str) -> ! {
	let options = vec![fuser::MountOption::RW, fuser::MountOption::FSName("dbfs".to_string())];
	let driver = DbfsDriver::new(tl);
	fuser::mount2(driver, mountpoint, &options).unwrap();
	panic!("FUSE driver crashed");
}

