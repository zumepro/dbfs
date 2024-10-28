pub enum FileType {
	RegularFile,
	Directory,
	SymbolicLink,
	NamedPipe,
	Socket,
	Unknown,
}
impl From<&String> for FileType {
	fn from(value: &String) -> Self {
		match value.as_str() {
			"-" => Self::RegularFile,
			"d" => Self::Directory,
			"l" => Self::SymbolicLink,
			"p" => Self::NamedPipe,
			"s" => Self::Socket,
 			_ => Self::Unknown,
		}
	}
}
