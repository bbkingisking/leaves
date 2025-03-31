use std::collections::HashMap;

pub fn get_language_name(code: &str) -> Option<&'static str> {
	let lang_map: HashMap<&str, &str> = [
		("bul", "Български"),    // Bulgarian
		("eng", "English"),      // English
		("fas", "فارسی"),        // Persian
		("fra", "Français"),     // French
		("jpn", "日本語"),        // Japanese
		("ojp", "上代日本語"), // Classical Japanese
		("lzh", "文言"),          // Literary Chinese
		("rus", "Русский"),      // Russian
		("zho-Hans", "简体中文"), // Written Mandarin in Simplified Chinese
		("zho-Hant", "繁體中文"), // Written Mandarin in Traditional Chinese (e.g., Taiwanese poetry)
		("yue-Hant", "粵語"),      // Written Cantonese (Traditional)
		("mn", "Монгол"),        // Mongolian (Default Traditional script)
		("mn-Latn", "Mongolian (Latin)"), // Mongolian in Latin script
	].iter().cloned().collect();

	lang_map.get(code).copied()
}
