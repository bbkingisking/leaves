use std::collections::HashMap;

pub fn get_language_name(code: &str) -> Option<&'static str> {
	let lang_map: HashMap<&str, &str> = [
		("bg", "Български"), // Bulgarian
		("en", "English"),   // English
		("fa", "فارسی"),     // Persian
		("fr", "Français"),  // French
		("ja", "日本語"),     // Japanese
		("lzh", "文言"),      // Literary Chinese
		("ru", "Русский"),   // Russian
		("zh-Hans", "简体中文"), // Simplified Chinese
	].iter().cloned().collect();

	lang_map.get(code).copied()
}