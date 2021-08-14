use filedescriptor::{FileDescriptor, StdioDescriptor};

pub fn pipe() -> Result<(), Box<dyn std::error::Error>> {
	#[cfg(target_os = "windows")]
	let sink = std::fs::File::create("nul")?;

	#[cfg(target_os = "linux")]
	let sink = std::fs::File::create("/dev/null")?;

	FileDescriptor::redirect_stdio(&sink, StdioDescriptor::Stdout)?;

	Ok(())
}
