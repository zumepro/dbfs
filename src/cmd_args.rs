use argp;

#[derive(argp::FromArgs)]
#[argp(description = "dbfs - FUSE driver for dbfs")]
pub struct CmdArgs {
	#[argp(subcommand)]
	pub command: ArgCommand
}

#[derive(argp::FromArgs)]
#[argp(subcommand)]
pub enum ArgCommand {
	Mount(ArgMount),
	Format(ArgFormat),
	Import(ArgImport)
}

#[derive(argp::FromArgs)]
#[argp(description = "Mount the filesystem using FUSE.")]
#[argp(subcommand, name = "mount")]
pub struct ArgMount {
	#[argp(switch)]
	#[argp(description = "Allows root access.")]
	pub allow_root: bool,

	#[argp(switch)]
	#[argp(description = "Allows other non-root user access.")]
	pub allow_other: bool,

	#[argp(positional)]
    #[argp(description = "Path to the mountpoint.")]
	pub mountpoint: String
}

#[derive(argp::FromArgs)]
#[argp(description = "Formats the filesystem.")]
#[argp(subcommand, name = "format")]
pub struct ArgFormat {}

#[derive(argp::FromArgs)]
#[argp(description = "Imports another filesystem (or just a directory) into dbfs.")]
#[argp(subcommand, name = "import")]
pub struct ArgImport {
	#[argp(positional)]
    #[argp(description = "Path to the source filesystem.")]
	pub source: String
}

pub fn parse() -> CmdArgs {
	let args: CmdArgs = argp::parse_args_or_exit(argp::DEFAULT);
	args
}

