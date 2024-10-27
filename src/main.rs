mod settings;
mod cmd_args;
pub mod db_connector;
pub mod sql_translation_layer;
pub mod fuse_driver;


#[macro_export]
macro_rules! debug {
	($($e:expr),+) => {
		#[cfg(debug_assertions)]
		{
			println!($($e),+)
		}
		#[cfg(not(debug_assertions))]
		{
			($($e),+)
		}
	}
}


fn main() {
	let args = cmd_args::parse();

	#[cfg(feature = "debug")]
	debug!("connecting to db...");
	let tl = match sql_translation_layer::TranslationLayer::new() {
		Ok(val) => val,
		Err(err) => {
			eprintln!("{}", err);
			return;
		}
	};

	#[cfg(feature = "debug")]
	debug!("starting FUSE driver");
	fuse_driver::run_forever(tl, &args.mountpoint);
}


#[cfg(not(feature = "integration_testing"))]
#[cfg(test)]
mod test {
	#[test]
	fn test_test() {
		assert_eq!(1, 1);
	}
}
