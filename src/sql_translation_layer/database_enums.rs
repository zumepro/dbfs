pub enum FileType {
	RegularFile,
	Directory,
	SymbolicLink,
	Unknown,
}
impl From<&String> for FileType {
	fn from(value: &String) -> Self {
		match value.as_str() {
			"-" => Self::RegularFile,
			"d" => Self::Directory,
			"l" => Self::SymbolicLink,
			_ => Self::Unknown,
		}
	}
}
