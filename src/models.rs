use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io, fs, path::PathBuf};

// Legacy format for backward compatibility
#[derive(Debug, Serialize, Deserialize)]
struct LegacyPoem {
	canonical: Version,
	#[serde(flatten)]
	other_versions: HashMap<String, Version>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Poem {
	#[serde(flatten)]
	pub versions: HashMap<String, Version>,
	#[serde(skip)]
	pub filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
	pub title: Option<String>,
	pub author: Option<String>,
	pub language: Option<String>,
	pub epigraph: Option<String>,
	pub text: String,
	pub rtl: Option<bool>,
	pub vertical: Option<bool>,
}

impl Poem {
	pub fn canonical(&self) -> Option<&Version> {
		self.versions.get("canonical")
	}

	pub fn has_canonical(&self) -> bool {
		self.versions.contains_key("canonical")
	}
}

pub fn load_poems() -> io::Result<Vec<Poem>> {
	let home = std::env::var("HOME").expect("HOME environment variable not set");
	let poems_dir = PathBuf::from(home).join("literature").join("poetry");
	let mut poems = Vec::new();
	for entry in fs::read_dir(poems_dir)? {
		let entry = entry?;
		if entry.path().extension().and_then(|s| s.to_str()) == Some("poem") {
			let content = fs::read_to_string(entry.path())?;

			// Try to parse as new format first
			if let Ok(poem) = serde_yaml::from_str::<Poem>(&content) {
				let mut poem = poem;
				if !poem.has_canonical() {
					continue; // Skip poems without canonical version as required by schema
				}
				poem.filename = entry.path().file_name().unwrap_or_default().to_string_lossy().into();
				poems.push(poem);
			}
			// Fall back to legacy format
			else if let Ok(legacy_poem) = serde_yaml::from_str::<LegacyPoem>(&content) {
				let mut versions = HashMap::new();
				versions.insert("canonical".to_string(), legacy_poem.canonical);

				// Add other versions from the legacy format
				for (key, version) in legacy_poem.other_versions {
					versions.insert(key, version);
				}

				let poem = Poem {
					versions,
					filename: entry.path().file_name().unwrap_or_default().to_string_lossy().into(),
				};

				poems.push(poem);
			}
		}
	}
	Ok(poems)
}
