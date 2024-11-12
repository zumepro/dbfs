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

fn mount(args: cmd_args::ArgMount) {
	debug!("connecting to db...");
	let tl = match sql_translation_layer::TranslationLayer::new() {
		Ok(val) => val,
		Err(err) => {
			eprintln!("{}", err);
			return;
		}
	};

	debug!("starting FUSE driver");
	fuse_driver::run_forever(tl, &args.mountpoint, args.allow_root, args.allow_other);
}

fn main() {
	let args = cmd_args::parse();

	match args.command {
		cmd_args::ArgCommand::Mount(args) => mount(args)
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
