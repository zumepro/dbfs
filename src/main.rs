mod settings;
mod cmd_args;
pub mod db_connector;
pub mod sql_translation_layer;
pub mod fuse_driver;


fn main() {
	let args = cmd_args::parse();
	let tl = sql_translation_layer::TranslationLayer::new();
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
