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

/// Takes the bits masked by 0xF000.
impl TryInto<driver_objects::FileType> for u32 {
	type Error = ();

	fn try_into(self) -> Result<driver_objects::FileType, Self::Error> {
	    match (self >> 12) & 0xF {
			0x4 => Ok(driver_objects::FileType::Directory),
			0x8 => Ok(driver_objects::FileType::File),
			0xA => Ok(driver_objects::FileType::Symlink),
			_ => Err(())
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
		(self.special as u16) << 9 | (self.owner as u16) << 6 | (self.group as u16) << 3 | self.other as u16
	}
}

impl Into<driver_objects::Permissions> for u16 {
	fn into(self) -> driver_objects::Permissions {
		driver_objects::Permissions {
			special: ((self >> 9) & 7) as u8,
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
		debug!("lookup: inode {}, name {:?}", &parent_inode, &name);
		match self.tl.lookup(name, parent_inode) {
			Ok(attr) => {
				debug!(" -> OK: {:?}", &attr);
				reply.entry(&TTL, &attr.into(), 0);
			},
			Err(err) => {
				debug!(" -> Err {:?}", &err);
				reply.error(ENOENT);
			}
		}
	}

	fn getattr(&mut self, _req: &fuser::Request, inode: u64, reply: fuser::ReplyAttr) {
		debug!("getattr: inode {}", &inode);
		match self.tl.getattr(inode) {
			Ok(attr) => {
				debug!(" -> OK: {:?}", &attr);
				reply.attr(&TTL, &attr.into());
			},
			Err(err) => {
				debug!(" -> Err {:?}", &err);
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
		debug!("read: inode {}, offset {}, size {}", &inode, &offset, &size);

		if size == 0 {
			debug!(" -> OK, no read operation necessary");
			reply.data(&[]);
			return
		}

		let mut buf = vec![0u8; size as usize];
		match self.tl.read(inode, offset as u64, &mut buf) {
			Ok(read_bytes) => {
				debug!(" -> OK (read {})", read_bytes);
				reply.data(&buf);
			},
			Err(err) => {
				debug!(" -> Err {:?}", &err);
				reply.error(ENOENT);
			}
		}
	}

	fn readlink(&mut self, _req: &fuser::Request<'_>, inode: u64, reply: fuser::ReplyData) {
		debug!("readlink: inode {}", &inode);
		let size: u32 = match self.tl.filesize(inode) {
			Ok(size) => {
				debug!(" -> size {}", &size.bytes);
				size.bytes as u32
			},
			Err(err) => {
				debug!(" -> Err while determining link size: {:?}", &err);
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
		debug!("readdir: inode {}, offset {}", &inode, &offset);
		if inode != self.last_readdir_inode {
			debug!(" -> cache miss, fetching from DB");
			self.last_readdir_inode = inode;
			self.last_readdir = match self.tl.readdir(inode) {
				Ok(val) => {
					debug!(" -> OK (inode {} has {} entries)", &inode, &val.len());
					val
				},
				Err(err) => {
					debug!(" -> Err {:?}", &err);
					reply.error(ENOENT);
					return
				}
			}
		}

		let mut i = offset;

		loop {
			let entry = match self.last_readdir.get(i as usize) {
				Some(val) => val,
				None => {
					// Invalidate the cache
					self.last_readdir_inode = u64::MAX;
					break
				}
			};

			debug!(" -> sending #{} (inode {}, name {:?}, type {:?})", &i, &entry.inode, &entry.name, &entry.ftype);
			i += 1;
			if reply.add(entry.inode, i, entry.ftype.clone().into(), &entry.name) {
				break
			}
		}

		debug!(" -> OK");
		reply.ok();
	}

	fn statfs(&mut self, _req: &fuser::Request<'_>, inode: u64, reply: fuser::ReplyStatfs) {
		debug!("statfs: inode {}", &inode);

		let stat = match self.tl.statfs() {
			Ok(val) => {
				debug!(" -> OK {:?}", &val);
				val
			},
			Err(err) => {
				debug!(" -> Err {:?}", &err);
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
		debug!("mkdir: parent inode {}, name {:?}, mode {:o}", &parent_inode, &name, &mode);

		match mode.try_into() {
			Ok(driver_objects::FileType::Directory) => {},
			kind @ _ => {
				debug!(" -> Err - invalid mode, not directory: {:?}", &kind);
				reply.error(EINVAL);
				return
			}
		};

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
				debug!(" -> OK {:?}", &attr);
				reply.entry(&TTL, &attr.into(), 0);
			},
			Err(err) => {
				debug!(" -> Err {:?}", &err);
				reply.error(ENOENT);
			}
		}
	}

	fn rmdir(&mut self, _req: &fuser::Request<'_>, parent_inode: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
		debug!("rmdir: parent inode {}, name {:?}", &parent_inode, &name);

		let inode = match self.tl.lookup_id(name, parent_inode) {
			Ok(inode) => inode,
			Err(err) => {
				debug!(" -> Err while performing lookup: {:?}", &err);
				reply.error(ENOENT);
				return
			}
		};

		let children = match self.tl.count_children(inode) {
			Ok(val) => val,
			Err(err) => {
				debug!(" -> Err while counting children: {:?}", &err);
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
		debug!("unlink: parent inode {}, name {:?}", &parent_inode, &name);

		if let Err(err) = self.tl.unlink(parent_inode, name) {
			debug!(" -> Err {:?}", &err);
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
		debug!("link: inode {}, new parent inode {}, new name {:?}", &inode, &new_parent_inode, &new_name);

		if let Err(err) = self.tl.link(new_parent_inode, new_name, inode) {
			debug!(" -> Err while creating link: {:?}", &err);
			reply.error(ENOENT);
			return
		}

		let attr = match self.tl.getattr(inode) {
			Ok(attr) => attr,
			Err(err) => {
				debug!(" -> Err while fetching attributes: {:?}", &err);
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
		debug!("symlink: parent inode {}, name {:?}, target {:?}", &parent_inode, &link_name, &target);

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
			perm: driver_objects::Permissions { special: 0, owner: 7, group: 7, other: 7 }
		};

		let attr = match self.tl.mknod(parent_inode, link_name, driver_objects::FileType::Symlink, attr) {
			Ok(attr) => attr,
			Err(err) => {
				debug!(" -> Err while creating node: {:?}", &err);
				reply.error(ENOENT);
				return
			}
		};

		if let Err(err) = self.tl.write(attr.ino.into(), 0, target) {
			debug!(" -> Err while writing symlink data: {:?}", &err);
			reply.error(ENOENT);
			return
		}

		match self.tl.getattr(attr.ino.into()) {
			Ok(attr) => {
				debug!(" -> OK: {:?}", &attr);
				reply.entry(&TTL, &attr.into(), 0);
			},
			Err(err) => {
				debug!(" -> Err while fetching updated attributes: {:?}", &err);
				reply.error(ENOENT);
			}
		}
	}

	fn setattr(
		&mut self,
		_req: &fuser::Request<'_>,
		inode: u64,
		mode: Option<u32>,
		uid: Option<u32>,
		gid: Option<u32>,
		size: Option<u64>,
		atime: Option<fuser::TimeOrNow>,
		mtime: Option<fuser::TimeOrNow>,
		ctime: Option<std::time::SystemTime>,
		_fh: Option<u64>,
		_crtime: Option<std::time::SystemTime>,
		_chgtime: Option<std::time::SystemTime>,
		_bkuptime: Option<std::time::SystemTime>,
		_flags: Option<u32>,
		reply: fuser::ReplyAttr,
	) {
		debug!("setattr: inode {}", inode);

		let oldattr = match self.tl.getattr(inode) {
			Ok(attr) => attr,
			Err(err) => {
				debug!(" -> Err while fetching old attributes: {:?}", &err);
				reply.error(ENOENT);
				return
			}
		};
		debug!(" -> old attr: {:?}", &oldattr);

		if let Some(size) = size {
			debug!(" -> truncating from {} to {} bytes", &oldattr.bytes, &size);
			if let Err(err) = self.tl.resize(inode, size) {
				debug!(" -> Err while truncating: {:?}", &err);
				reply.error(ENOENT);
				return
			}
		}

		let time = std::time::SystemTime::now();
		let setattr = driver_objects::FileSetAttr {
			uid: uid.unwrap_or_else(|| oldattr.uid),
			gid: gid.unwrap_or_else(|| oldattr.gid),
			atime: match atime {
				Some(fuser::TimeOrNow::SpecificTime(val)) => val,
				Some(fuser::TimeOrNow::Now) => time,
				None => oldattr.atime
			},
			mtime: match mtime {
				Some(fuser::TimeOrNow::SpecificTime(val)) => val,
				Some(fuser::TimeOrNow::Now) => time,
				None => oldattr.mtime
			},
			ctime: ctime.unwrap_or_else(|| oldattr.ctime),
			perm: match mode {
				Some(mode) => {
					match (mode.try_into(), oldattr.kind) {
						(Ok(driver_objects::FileType::Directory), driver_objects::FileType::Directory) => {},
						(Ok(driver_objects::FileType::File), driver_objects::FileType::File) => {},
						(Ok(driver_objects::FileType::Symlink), driver_objects::FileType::Symlink) => {},
						modes @ _ => {
							debug!(" -> Err - attempted to change mode from {:?} to {:?}", &modes.0, &modes.1);
							reply.error(EINVAL);
							return
						}
					}
					(mode as u16).into()
				},
				None => oldattr.perm
			}
		};

		debug!(" -> setting attr to: {:?}", &setattr);
		let newattr = match self.tl.setattr(inode, setattr) {
			Ok(attr) => attr,
			Err(err) => {
				debug!(" -> Err while setting attributes: {:?}", &err);
				reply.error(ENOENT);
				return
			}
		};

		debug!(" -> OK: {:?}", &newattr);
		reply.attr(&TTL, &newattr.into());
	}

	fn mknod(
		&mut self,
		req: &fuser::Request<'_>,
		parent_inode: u64,
		name: &OsStr,
		mode: u32,
		_umask: u32,
		_rdev: u32,
		reply: fuser::ReplyEntry,
	) {
		debug!("mknod: parent inode {}, name {:?}, mode {:o}", &parent_inode, &name, &mode);

		match mode.try_into() {
			Ok(driver_objects::FileType::File) => {},
			kind @ _ => {
				debug!(" -> Err - invalid mode, not regular file: {:?}", &kind);
				reply.error(EINVAL);
				return
			}
		};

		let time = std::time::SystemTime::now();
		let attr = driver_objects::FileSetAttr {
			uid: req.uid(),
			gid: req.gid(),
			atime: time,
			mtime: time,
			ctime: time,
			perm: (mode as u16).into()
		};

		match self.tl.mknod(parent_inode, name, driver_objects::FileType::File, attr) {
			Ok(attr) => {
				debug!(" -> OK {:?}", &attr);
				reply.entry(&TTL, &attr.into(), 0);
			},
			Err(err) => {
				debug!(" -> Err {:?}", &err);
				reply.error(ENOENT);
			}
		}
	}

	fn rename(
		&mut self,
		_req: &fuser::Request<'_>,
		parent_inode: u64,
		name: &OsStr,
		new_parent_inode: u64,
		new_name: &OsStr,
		_flags: u32,
		reply: fuser::ReplyEmpty,
	) {
		debug!("rename: parent inode {}, name {:?} to parent inode {}, name {:?}", &parent_inode, &name, &new_parent_inode, &new_name);

		if let Err(err) = self.tl.rename(parent_inode, name, new_parent_inode, new_name) {
			debug!(" -> Err {:?}", err);
			reply.error(ENOENT);
			return
		}

		debug!(" -> OK");
		reply.ok();
	}

	fn write(
		&mut self,
		_req: &fuser::Request<'_>,
		inode: u64,
		_fh: u64,
		offset: i64,
		data: &[u8],
		_write_flags: u32,
		_flags: i32,
		_lock_owner: Option<u64>,
		reply: fuser::ReplyWrite,
	) {
		debug!("write: inode {}, offset {}, data len {}", &inode, &offset, &data.len());

		if let Err(err) = self.tl.write(inode, offset as u64, data) {
			debug!(" -> Err {:?}", err);
			reply.error(ENOENT);
			return
		}

		debug!(" -> OK");
		reply.written(data.len() as u32);
	}

	// TODO - create (?)
	// TODO - open (?)
}

pub fn run_forever(tl: TranslationLayer, mountpoint: &str) -> ! {
	let options = vec![fuser::MountOption::RW, fuser::MountOption::FSName("dbfs".to_string()), fuser::MountOption::DefaultPermissions];
	let driver = DbfsDriver::new(tl);
	fuser::mount2(driver, mountpoint, &options).unwrap();
	panic!("FUSE driver crashed");
}

