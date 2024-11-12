mod settings;
mod cmd_args;
pub mod db_connector;
pub mod sql_translation_layer;
pub mod fuse_driver;

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug {
	($($e:expr),+) => {
		println!($($e),+)
	}
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug {
	($($e:expr),+) => {
		($($e),+)
	}
}

fn create_driver() -> Option<fuse_driver::DbfsDriver> {
	debug!("connecting to db...");
	let tl = match sql_translation_layer::TranslationLayer::new() {
		Ok(val) => val,
		Err(err) => {
			eprintln!("{}", err);
			return None;
		}
	};

	debug!("starting FUSE driver");
	Some(fuse_driver::DbfsDriver::new(tl))
}

fn mount(args: cmd_args::ArgMount) {
	if let Some(driver) = create_driver() {
		driver.run_forever(&args.mountpoint, args.allow_root, args.allow_other);
	}
}

fn format(_args: cmd_args::ArgFormat) {
	if let Some(mut driver) = create_driver() {
		debug!("erasing fs...");
		debug!("{:?}", driver.format());
	}
}

fn main() {
	let args = cmd_args::parse();

	match args.command {
		cmd_args::ArgCommand::Mount(args) => mount(args),
		cmd_args::ArgCommand::Format(args) => format(args)
	}
}


#[cfg(not(feature = "integration_testing"))]
#[cfg(test)]
mod test {
	#[test]
	fn test_test() {
		assert_eq!(1, 1);
	}
}
