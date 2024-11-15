mod cache;

use crate::settings;
use crate::sql_translation_layer::MAX_NAME_LEN;
use crate::sql_translation_layer::driver_objects;
use crate::sql_translation_layer::TranslationLayer;
use crate::sql_translation_layer::Error;
use crate::debug;

use fuser;
use libc::EINTR;
use libc::EIO;
use libc::{EINVAL, ENOENT, ENOTEMPTY};

use std::ffi::OsStr;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::MetadataExt;
use std::io::Read;
use std::time::Duration;
use std::sync::{Arc, Mutex};

const TTL: Duration = Duration::from_secs(1);

/// Takes the bits masked by 0xF000.
impl TryInto<driver_objects::FileType> for u32 {
	type Error = ();

	fn try_into(self) -> Result<driver_objects::FileType, Self::Error> {
	    match (self >> 12) & 0xF {
			0x1 => Ok(driver_objects::FileType::NamedPipe),
			0x4 => Ok(driver_objects::FileType::Directory),
			0x8 => Ok(driver_objects::FileType::File),
			0xA => Ok(driver_objects::FileType::Symlink),
			0xC => Ok(driver_objects::FileType::Socket),
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
			Self::NamedPipe => fuser::FileType::NamedPipe,
			Self::Socket => fuser::FileType::Socket,
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

impl Into<i32> for Error {
	fn into(self) -> i32 {
	    match self {
			Self::DbConnectorError(_) => EIO,
			Self::DbLockError => EIO,
			Self::NotFoundError(_) => ENOENT,
			Self::ClientError(_) => EINVAL,
			Self::Unimplemented => EINTR,
			Self::RuntimeError(_) => EIO
		}
	}
}

fn format_mode_block(mode: u32, weird_execute_char: Option<char>) -> String {
	let r = match mode & 4 {
		0 => '-',
		_ => 'r'
	};
	let w = match mode & 2 {
		0 => '-',
		_ => 'w'
	};
	let x = match weird_execute_char {
		Some(val) => val,
		None => match mode & 1 {
			0 => '-',
			_ => 'x'
		}
	};

	format!("{}{}{}", r, w, x)
}

fn format_metadata(metadata: &std::fs::Metadata) -> String {
	let prefix = match (metadata.is_file(), metadata.is_dir(), metadata.is_symlink()) {
		(true, false, false) => '-',
		(false, true, false) => 'd',
		(false, false, true) => 'l',
		_ => '?'
	};

	let mode = metadata.mode();

	let user = format_mode_block(mode >> 6, match mode & (1 << 11) {
		0 => None,
		_ => Some('s')
	});
	let group = format_mode_block(mode >> 3, match mode & (1 << 10) {
		0 => None,
		_ => Some('s')
	});
	let others = format_mode_block(mode, match mode & (1 << 9) {
		0 => None,
		_ => Some('t')
	});

	format!("{}{}{}{}, user {}, group {}", prefix, user, group, others, metadata.uid(), metadata.gid())
}

#[derive(Debug)]
struct HardLink {
	src_inode: u64,
	dbfs_inode: u64
}

fn import_recurse(tl: &mut TranslationLayer, path: &std::path::PathBuf, parent_inode: u64, links: &mut Vec<HardLink>) -> Result<(), Error> {
	// TODO - hardlinks
	// TODO - xattr (error if any)

	if parent_inode == 0 && !path.is_dir() {
		return Err(Error::RuntimeError("source root is not a directory"))
	}

	let metadata = match path.symlink_metadata() {
		Ok(val) => val,
		Err(val) => {
			debug!("fs premature error on {:?}: {:?}", path, val);
			return Err(Error::RuntimeError("filesystem error"))
		}
	};

	println!("{} {:?}", format_metadata(&metadata), &path.as_os_str());

	let attr = driver_objects::FileSetAttr {
		uid: metadata.uid(),
		gid: metadata.gid(),
		atime: metadata.accessed().unwrap(),
		mtime: metadata.modified().unwrap(),
		ctime: metadata.created().unwrap(),
		perm: (metadata.mode() as u16).into()
	};

	let ftype = metadata.file_type();

	if ftype.is_dir() {
		let parent_inode = if parent_inode != 0 {
			let name = path.components().last().unwrap().as_os_str();
			tl.mknod(parent_inode, name, driver_objects::FileType::Directory, attr)?.ino as u64
		} else {
			tl.setattr(1, attr)?; // Root
			1u64
		};

		let entries = match std::fs::read_dir(&path) {
			Ok(val) => val,
			Err(_) => return Err(Error::RuntimeError("could not iterate over path"))
		};

		for entry in entries {
			let Ok(entry) = entry else { continue };
			let path = entry.path();

			import_recurse(tl, &path, parent_inode, links)?;
		}

		return Ok(())
	}

	if ftype.is_symlink() {
		if metadata.nlink() > 1 {
			panic!("Symlinked hardlink detected!");
		}

		let name = path.components().last().unwrap().as_os_str();
		let link = match path.read_link() {
			Ok(val) => val,
			Err(_) => return Err(Error::RuntimeError("could not read symlink"))
		};
		let link: String = link.as_os_str().to_string_lossy().into();
		debug!(" -> {}", link);
		
		let ino = tl.mknod(parent_inode, name, driver_objects::FileType::Symlink, attr)?.ino as u64;
		tl.unsafe_write(ino, 0, link.as_bytes())?;

		return Ok(())
	}

	if ftype.is_file() {
		let name = path.components().last().unwrap().as_os_str();
		
		if metadata.nlink() > 1 {
			for link in &*links {
				if link.src_inode != metadata.ino() { continue; }

				tl.link(parent_inode, name, link.dbfs_inode)?;
				return Ok(())
			}
		}

		let ino = tl.mknod(parent_inode, name, driver_objects::FileType::File, attr)?.ino as u64;

		if metadata.nlink() > 1 {
			links.push(HardLink { src_inode: metadata.ino(), dbfs_inode: ino });
		}

		let mut infile = std::fs::File::open(&path).unwrap();
		let mut buf = vec![0u8; 1048576];
		let mut offset = 0usize;

		loop {
			let read = match infile.read(&mut buf) {
				Ok(val) => val,
				Err(val) => {
					debug!("fs read error at offset {}: {:?}", offset, val);
					return Err(Error::RuntimeError("fs read error"))
				}
			};
			if read <= 0 { break }
			debug!(" -> writing {} bytes at offset {}", &read, &offset);
			tl.unsafe_write(ino, offset as u64, &buf[..read])?;
			offset += read;
		}

		return Ok(())
	}

	if ftype.is_fifo() {
		let name = path.components().last().unwrap().as_os_str();
		
		tl.mknod(parent_inode, name, driver_objects::FileType::NamedPipe, attr)?;

		return Ok(())
	}

	if ftype.is_socket() {
		let name = path.components().last().unwrap().as_os_str();
		
		tl.mknod(parent_inode, name, driver_objects::FileType::Socket, attr)?;

		return Ok(())
	}

	Err(Error::RuntimeError("invalid file"))
}

pub struct DbfsDriver {
	tl: Arc<Mutex<TranslationLayer>>,
	last_readdir_inode: u64,
	last_readdir: Vec<driver_objects::DirectoryEntry>,
	cache: cache::WriteCache
}

impl DbfsDriver {
	pub fn new(tl: TranslationLayer) -> Self {
		let tl = Arc::new(Mutex::new(tl));

		Self {
			tl: tl.clone(),
			last_readdir_inode: u64::MAX,
			last_readdir: Vec::new(),
			cache: cache::WriteCache::new(tl.clone(), 1 << 20)
		}
	}

	pub fn run_forever(self, mountpoint: &str, root: bool, others: bool) -> ! {
		let mut options = vec![fuser::MountOption::RW, fuser::MountOption::FSName("dbfs".to_string()), fuser::MountOption::DefaultPermissions];
		if root { options.push(fuser::MountOption::AllowRoot); }
		if others { options.push(fuser::MountOption::AllowOther); }
		fuser::mount2(self, mountpoint, &options).unwrap();
		panic!("FUSE driver crashed");
	}

	pub fn format(&mut self) -> Result<(), Error> {
		let mut tl = self.tl.lock().unwrap();
		tl.format()
	}

	pub fn import(&mut self, path: &std::path::Path) -> Result<(), Error> {
		let mut tl = self.tl.lock().unwrap();
		let path = std::path::PathBuf::from(path);
		let mut links: Vec<HardLink> = Vec::new();
		import_recurse(&mut tl, &path, 0, &mut links)?;
		println!("done, detected {} inodes with multiple links", links.len());
		Ok(())
	}
}

impl fuser::Filesystem for DbfsDriver {
	fn lookup(&mut self, _req: &fuser::Request, parent_inode: u64, name: &OsStr, reply: fuser::ReplyEntry) {
		debug!("lookup: inode {}, name {:?}", &parent_inode, &name);
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

		match tl.lookup(name, parent_inode) {
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
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

		match tl.getattr(inode) {
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
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

		if size == 0 {
			debug!(" -> OK, no read operation necessary");
			reply.data(&[]);
			return
		}

		let mut buf = vec![0u8; size as usize];
		match tl.read(inode, offset as u64, &mut buf) {
			Ok(read_bytes) => {
				debug!(" -> OK (read {})", read_bytes);
				reply.data(&buf[..read_bytes]);
			},
			Err(err) => {
				debug!(" -> Err {:?}", &err);
				reply.error(ENOENT);
			}
		}
	}

	fn readlink(&mut self, _req: &fuser::Request<'_>, inode: u64, reply: fuser::ReplyData) {
		debug!("readlink: inode {}", &inode);
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

		let size: u32 = match tl.filesize(inode) {
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
		drop(tl);

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
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

		if inode != self.last_readdir_inode {
			debug!(" -> cache miss, fetching from DB");
			self.last_readdir_inode = inode;
			self.last_readdir = match tl.readdir(inode) {
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
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

		let stat = match tl.statfs() {
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

		let total_blocks = 1 << 48;

		reply.statfs(
			total_blocks,
			total_blocks - stat.used_blocks,
			total_blocks - stat.used_blocks,
			stat.used_inodes,
			total_blocks - stat.used_blocks,
			settings::FILE_BLOCK_SIZE_32,
			MAX_NAME_LEN,
			settings::FILE_BLOCK_SIZE_32
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
		debug!("mkdir: parent inode {}, name {:?}, mode {:o}, user {}, group {}", &parent_inode, &name, &mode, req.uid(), req.gid());
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

		let time = std::time::SystemTime::now();
		let attr = driver_objects::FileSetAttr {
			uid: req.uid(),
			gid: req.gid(),
			atime: time,
			mtime: time,
			ctime: time,
			perm: (mode as u16).into()
		};

		match tl.mknod(parent_inode, name, driver_objects::FileType::Directory, attr) {
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
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

		let inode = match tl.lookup_id(name, parent_inode) {
			Ok(inode) => inode,
			Err(err) => {
				debug!(" -> Err while performing lookup: {:?}", &err);
				reply.error(ENOENT);
				return
			}
		};

		let children = match tl.count_children(inode) {
			Ok(val) => val,
			Err(err) => {
				debug!(" -> Err while counting children: {:?}", &err);
				reply.error(ENOENT);
				return
			}
		};
		drop(tl);

		if children > 2 {
			debug!(" -> Err, directory not empty");
			reply.error(ENOTEMPTY);
			return
		}

		debug!(" -> OK, safe number of children to delete {}, passing request to unlink", children);
		self.unlink(_req, parent_inode, name, reply);
	}

	fn unlink(&mut self, _req: &fuser::Request<'_>, parent_inode: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
		debug!("unlink: parent inode {}, name {:?}", &parent_inode, &name);
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

		if let Err(err) = tl.unlink(parent_inode, name) {
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
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

		if let Err(err) = tl.link(new_parent_inode, new_name, inode) {
			debug!(" -> Err while creating link: {:?}", &err);
			reply.error(ENOENT);
			return
		}

		let attr = match tl.getattr(inode) {
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
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

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

		let attr = match tl.mknod(parent_inode, link_name, driver_objects::FileType::Symlink, attr) {
			Ok(attr) => attr,
			Err(err) => {
				debug!(" -> Err while creating node: {:?}", &err);
				reply.error(ENOENT);
				return
			}
		};

		if let Err(err) = tl.unsafe_write(attr.ino.into(), 0, target) {
			debug!(" -> Err while writing symlink data: {:?}", &err);
			reply.error(ENOENT);
			return
		}

		match tl.getattr(attr.ino.into()) {
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
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

		let oldattr = match tl.getattr(inode) {
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
			if let Err(err) = tl.resize(inode, size) {
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
		let newattr = match tl.setattr(inode, setattr) {
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
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

		let kind = match mode.try_into() {
			Ok(kind @ driver_objects::FileType::File) => kind,
			Ok(kind @ driver_objects::FileType::Socket) => kind,
			Ok(kind @ driver_objects::FileType::NamedPipe) => kind,
			kind @ _ => {
				debug!(" -> Err - invalid mode, not regular file, socket or pipe: {:?}", &kind);
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

		match tl.mknod(parent_inode, name, kind, attr) {
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
		self.cache.flush();
		let mut tl = self.tl.lock().unwrap();

		if let Ok(_) = tl.lookup_id(new_name, new_parent_inode) {
			debug!(" -> destination exists, deleting the existing file in the destination");
			if let Err(err) = tl.unlink(new_parent_inode, new_name) {
				debug!(" -> Err while deleting: {:?}", err);
				reply.error(ENOENT);
				return
			}
			debug!(" -> OK");
		}

		if let Err(err) = tl.rename(parent_inode, name, new_parent_inode, new_name) {
			debug!(" -> Err while renaming: {:?}", err);
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

		self.cache.write(inode, offset as u64, data.to_vec());
		// if let Err(err) = self.tl.lock().unwrap().unsafe_write(inode, offset as u64, data) {
		// 	debug!(" -> Err {:?}", err);
		// 	reply.error(ENOENT);
		// 	return
		// }

		debug!(" -> OK");
		reply.written(data.len() as u32);
	}
}

