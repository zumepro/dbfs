use argp;

#[derive(argp::FromArgs)]
#[argp(description = "dbfs - FUSE driver for the revolutionary dbfs")]
pub struct CmdArgs {
	#[argp(positional)]
    #[argp(description = "Path to the mountpoint.")]
	pub mountpoint: String
}

pub fn parse() -> CmdArgs {
	let args: CmdArgs = argp::parse_args_or_exit(argp::DEFAULT);
	args
}
