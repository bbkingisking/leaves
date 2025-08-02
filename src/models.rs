use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io, fs, path::{Path, PathBuf}};

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

fn collect_poems(dir: &Path, poems: &mut Vec<Poem>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                        collect_poems(&path, poems)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("poem") {
                        let content = fs::read_to_string(&path)?;
                        if let Ok(poem) = serde_yaml::from_str::<Poem>(&content) {
                                let mut poem = poem;
                                poem.filename = path.file_name().unwrap_or_default().to_string_lossy().into();
                                poems.push(poem);
                        }
                }
        }
        Ok(())
}

pub fn load_poems() -> io::Result<Vec<Poem>> {
        let home = std::env::var("HOME").expect("HOME environment variable not set");
        let poems_dir = PathBuf::from(home).join("literature").join("poetry");
        let mut poems = Vec::new();
        collect_poems(&poems_dir, &mut poems)?;
        Ok(poems)
}

#[cfg(test)]
mod tests {
        use super::*;
        use std::{fs, env};
        use rand::Rng;

        #[test]
        fn loads_poems_recursively() {
                let mut rng = rand::thread_rng();
                let base = std::env::temp_dir().join(format!("leaves_test_{}", rng.gen::<u64>()));
                let poetry_root = base.join("literature").join("poetry");
                let nested_dir = poetry_root.join("nested");
                fs::create_dir_all(&nested_dir).unwrap();
                let poem_yaml = "canonical:\n  title: 't'\n  author: 'a'\n  language: 'l'\n  text: 'x'\n";
                fs::write(poetry_root.join("one.poem"), poem_yaml).unwrap();
                fs::write(nested_dir.join("two.poem"), poem_yaml).unwrap();
                let original_home = env::var("HOME").unwrap();
                env::set_var("HOME", &base);
                let poems = load_poems();
                env::set_var("HOME", original_home);
                assert_eq!(poems.unwrap().len(), 2);
        }
}
