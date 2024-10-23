mod settings;
mod cmd_args;
mod db_connector;
mod sql_translation_layer;
mod fuse_driver;


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
