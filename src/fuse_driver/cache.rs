use crate::debug;
use crate::sql_translation_layer::TranslationLayer;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

pub struct WriteCache {
	cache_thread_tx: mpsc::Sender<CacheThreadMessage>,
	cache_thread_rx: mpsc::Receiver<()>
}

pub struct WriteCommand {
	pub inode: u64,
	pub offset: u64,
	pub data: Vec<u8>
}

pub enum CacheThreadMessage {
	Flush,
	Write(WriteCommand)
}

struct CacheThread {
	tl: Arc<Mutex<TranslationLayer>>,
	cache: Vec<u8>,
	cache_ptr: usize,
	cache_inode_offset: u64,
	last_inode: u64,
	tx: mpsc::Sender<()>,
	rx: mpsc::Receiver<CacheThreadMessage>
}

impl CacheThread {
	fn write(&mut self, write: WriteCommand) {
		if write.inode != self.last_inode
			|| write.offset < self.cache_inode_offset
			|| write.offset >= self.cache_inode_offset + self.cache.len() as u64 {

			self.flush();
			self.last_inode = write.inode;
			self.cache_inode_offset = write.offset;
		}

		let mut written = 0usize;

		while written < write.data.len() {
			let avail_in_cache = self.cache.len() - self.cache_ptr;
			let to_write = write.data.len() - written;
			let will_write = usize::min(avail_in_cache, to_write);

			self.cache[self.cache_ptr..self.cache_ptr + will_write].copy_from_slice(&write.data[written..written + will_write]);

			self.cache_ptr += will_write;
			if self.cache_ptr >= self.cache.len() { self.flush(); }

			written += will_write;
		}
	}

	fn flush(&mut self) {
		if self.cache_ptr == 0 { return; }

		debug!("CACHE: flushing inode {}, offset {}, {} bytes", self.last_inode, self.cache_inode_offset, self.cache_ptr);

		let _ = self.tl.lock().unwrap().unsafe_write(self.last_inode, self.cache_inode_offset, &self.cache[..self.cache_ptr]); // TODO - error handling

		self.cache_inode_offset += self.cache_ptr as u64;
		self.cache_ptr = 0;
	}
	
	fn run_loop(&mut self) {
		let msg = match self.rx.recv_timeout(Duration::from_millis(100)) {
			Ok(val) => val,
			Err(_) => {
				self.flush();
				return
			}
		};

		match msg {
			CacheThreadMessage::Write(write) => self.write(write),
			CacheThreadMessage::Flush => self.flush()
		}

		self.tx.send(()).unwrap();
	}

	pub fn run(tl: Arc<Mutex<TranslationLayer>>, size: usize, tx: mpsc::Sender<()>, rx: mpsc::Receiver<CacheThreadMessage>) -> ! {
		let mut new = Self {
			tl,
			cache: vec![0u8; size],
			cache_ptr: 0usize,
			cache_inode_offset: 0u64,
			last_inode: u64::MAX,
			tx,
			rx
		};

		loop {
			new.run_loop();
		}
	}
}

impl WriteCache {
	pub fn new(tl: Arc<Mutex<TranslationLayer>>, size: usize) -> Self {
		let (tx, rxsub) = mpsc::channel();
		let (txsub, rx) = mpsc::channel();

		std::thread::spawn(move || {
			CacheThread::run(tl, size, txsub, rxsub);
		});

		Self {
			cache_thread_tx: tx,
			cache_thread_rx: rx
		}
	}
	
	pub fn send_msg(&mut self, msg: CacheThreadMessage) {
		self.cache_thread_tx.send(msg).unwrap();
	}

	pub fn wait_for_thread(&mut self) {
		self.cache_thread_rx.recv().unwrap();
	}

	pub fn flush(&mut self) {
		self.send_msg(CacheThreadMessage::Flush);
		self.wait_for_thread();
	}

	pub fn write(&mut self, inode: u64, offset: u64, data: Vec<u8>) {
		self.send_msg(CacheThreadMessage::Write(WriteCommand {
			inode,
			offset,
			data
		}));
		self.wait_for_thread();
	}
}

