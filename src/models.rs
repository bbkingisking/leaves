use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io, fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Poem {
	pub canonical: Version,
	#[serde(flatten)]
	pub other_versions: HashMap<String, Version>,
	#[serde(skip)]
	pub filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
	pub title: String,
	pub author: String,
	pub language: String,
	pub epigraph: Option<String>,
	pub text: String,
	pub rtl: Option<bool>,
	pub vertical: Option<bool>,
}

pub fn load_poems() -> io::Result<Vec<Poem>> {
	let home = std::env::var("HOME").expect("HOME environment variable not set");
	let poems_dir = PathBuf::from(home).join("Documents").join("poetry");
	let mut poems = Vec::new();
	for entry in fs::read_dir(poems_dir)? {
		let entry = entry?;
		if entry.path().extension().and_then(|s| s.to_str()) == Some("poem") {
			let content = fs::read_to_string(entry.path())?;
			if let Ok(poem) = serde_yaml::from_str::<Poem>(&content) {
				let mut poem = poem;
				poem.filename = entry.path().file_name().unwrap_or_default().to_string_lossy().into();
				poems.push(poem);
			}
		}
	}
	Ok(poems)
}
