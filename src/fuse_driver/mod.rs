use crate::sql_translation_layer::{TranslationLayer, Error};

use fuser;
use libc::ENOENT;

use std::ffi::OsStr;
use std::time::Duration;

struct DbfsDriver {
	tl: TranslationLayer
}

impl DbfsDriver {
	fn new(tl: TranslationLayer) -> Self {
		Self { tl }
	}
}

const TTL: Duration = Duration::from_secs(1);

impl fuser::Filesystem for DbfsDriver {
	fn lookup(&mut self, _req: &fuser::Request, parent_inode: u64, name: &OsStr, reply: fuser::ReplyEntry) {
		match self.tl.lookup(name, parent_inode) {
			Ok(attr) => reply.entry(&TTL, &attr, 0),
			Err(_) => reply.error(ENOENT)
		}
	}

	fn getattr(&mut self, _req: &fuser::Request, inode: u64, reply: fuser::ReplyAttr) {
		match self.tl.getattr(inode) {
			Ok(attr) => reply.attr(&TTL, &attr),
			Err(_) => reply.error(ENOENT)
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
		let mut buf = vec![0u8; size as usize];
		match self.tl.read(inode, offset as u64, &mut buf) {
			Ok(()) => reply.data(&buf),
			Err(_) => reply.error(ENOENT)
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
		let mut i = offset;
		
		loop {
			let entry = match self.tl.readdir(inode, offset as u64) {
				Ok(entry) => entry,
				Err(err) => match err {
					Error::OutOfEntries => break,
					_ => {
						reply.error(ENOENT);
						return
					}
				}
			};

			i += 1;
			if reply.add(entry.0, i, entry.1, entry.2) {
				break
			}
		}

		reply.ok();
	}
}

pub fn run_forever(tl: TranslationLayer, mountpoint: &str) -> ! {
	let options = vec![fuser::MountOption::RO, fuser::MountOption::FSName("hello".to_string()), fuser::MountOption::AutoUnmount, fuser::MountOption::AllowRoot];
	let driver = DbfsDriver::new(tl);
	fuser::mount2(driver, mountpoint, &options).unwrap();
	panic!("FUSE driver crashed");
}

