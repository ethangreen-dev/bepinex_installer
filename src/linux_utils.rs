use std::fs::File;
use std::io::Read;
use std::path::Path;

const MAGIC: [u8; 4] = [0x7F, 0x45, 0x4C, 0x46];

pub enum Arch {
	X86,
	X86_64,
}

impl Arch {
	pub fn from_file(binary: &Path) -> Option<Self> {
		if !binary.is_file() {
			return None;
		}

		// Read the first 4 bytes of the file.
		let mut file = File::open(binary).ok()?;
		let mut buffer = [0_u8; 0x14];

		file.read_exact(&mut buffer).ok()?;

		if buffer[0..4] != MAGIC {
			return None;
		}

		let mut arch_bytes = [0_u8; 2];
		arch_bytes.copy_from_slice(&buffer[0x12..0x14]);

		let arch = match buffer[4] {
			0x1 => u16::from_le_bytes(arch_bytes),
			0x2 => u16::from_be_bytes(arch_bytes),
			_ => panic!(""),
		};

		match arch {
			0x3 => Some(Arch::X86),
			0x3E => Some(Arch::X86_64),
			_ => None
 		}
	}
}
