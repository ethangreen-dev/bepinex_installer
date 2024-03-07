use std::str::FromStr;
use std::path::PathBuf;
use std::fmt::Display;

use serde::{Serialize, Deserialize};
use serde_with::{SerializeDisplay, DeserializeFromStr};

#[derive(Serialize, Deserialize, Debug)]
pub enum FileAction {
	Create,
	Remove,
	Modify,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrackedFile {
	pub action: FileAction,
	pub path: PathBuf,
	pub context: Option<String>,
}

#[derive(SerializeDisplay, DeserializeFromStr)]
pub struct PackageReference {
	pub namespace: String,
	pub name: String,
	pub version: Version,
}

#[derive(thiserror::Error, Debug)]
pub enum PackageReferenceParseError {
	#[error("Expected {expected} sections, got {got}.")]
	NumSections { expected: usize, got: usize },

	#[error("Failed to parse version: {0}.")]
	VersionParseFail(#[from] VersionParseError)
}

impl FromStr for PackageReference {
    type Err = PackageReferenceParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [version, name, namespace]: [&str; 3] = s
			.rsplitn(3, '-')
			.collect::<Vec<_>>()
			.try_into()
			.map_err(|v: Vec<&str>| PackageReferenceParseError::NumSections { 
				expected: 3, 
				got: v.len() - 1,
			})?;

		Ok(PackageReference {
			namespace: namespace.to_string(),
			name: name.to_string(),
			version: version.parse()?,
		})
    }
}

impl Display for PackageReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.namespace, self.name, self.version)
    }
}

#[derive(SerializeDisplay, DeserializeFromStr)]
pub struct Version {
	pub major: u32,
	pub minor: u32,
	pub patch: u32,
}

#[derive(thiserror::Error, Debug)]
pub enum VersionParseError {
	#[error("Faile dot parse an integer because: {0}.")]
	IntParse(#[from] std::num::ParseIntError),
	#[error("Expected 2 dots in version string, got {0}.")]
	DotCount(usize),
}

impl FromStr for Version {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [major, minor, patch]: [u32; 3] = s
			.splitn(3, '.')
			.map(|n| n.parse())
			.collect::<Result<Vec<_>, _>>()?
			.try_into()
			.map_err(|v: Vec<u32>| VersionParseError::DotCount(v.len() - 1))?;

		Ok(Version {
			major,
			minor,
			patch
		})
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

