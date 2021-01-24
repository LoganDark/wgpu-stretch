use glob::{glob, PatternError, GlobError};
use shaderc::{ShaderKind, Compiler};

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("glob error")]
	PatternError(#[from] PatternError),
	#[error("glob error")]
	GlobError(#[from] GlobError),
	#[error("I/O error")]
	IoError(#[from] std::io::Error),
	#[error("shaderc error")]
	Shaderc(#[from] shaderc::Error)
}

fn main() -> Result<(), Error> {
	let paths: Result<Vec<_>, _> = glob("./src/shaders/*.frag")?
		.chain(glob("./src/shaders/*.vert")?)
		.chain(glob("./src/shaders/*.comp")?)
		.collect();
	let paths = paths?;

	let mut compiler = Compiler::new().expect("Couldn't create SPIR-V compiler");

	for path in paths {
		println!("cargo:rerun-if-changed={}", path.to_str().unwrap());

		let compiled = compiler.compile_into_spirv(
			&String::from_utf8(std::fs::read(&path)?).unwrap(),
			match path.extension().unwrap().to_str().unwrap() {
				"vert" => ShaderKind::Vertex,
				"frag" => ShaderKind::Fragment,
				_ => unreachable!()
			},
			path.file_name().unwrap().to_str().unwrap(),
			"main",
			None
		)?;

		std::fs::write(
			format!("{}.spv", path.to_str().unwrap()),
			compiled.as_binary_u8()
		)?;
	}

	Ok(())
}
