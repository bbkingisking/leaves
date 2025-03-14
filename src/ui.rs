use ratatui::{
	widgets::Paragraph,
	layout::Alignment,
	text::{Line, Span},
	style::{Style, Color},
};
use crossterm::terminal;
use crate::models::Version;

pub fn wrap_text(text: &str, max_width: usize) -> String {
	let paragraphs: Vec<&str> = text.split('\n').collect();
	let mut wrapped = Vec::new();
	for paragraph in paragraphs {
		let words: Vec<&str> = paragraph.split_whitespace().collect();
		let mut lines = Vec::new();
		let mut current_line = String::from("  ");
		for word in words {
			if current_line.len() + word.len() + 1 <= max_width {
				if current_line.len() > 2 {
					current_line.push(' ');
				}
				current_line.push_str(word);
			} else {
				lines.push(current_line);
				current_line = String::from("  ");
				current_line.push_str(word);
			}
		}
		if !current_line.is_empty() {
			lines.push(current_line);
		}
		wrapped.push(lines.join("\n"));
	}
	wrapped.join("\n")
}

pub fn parse_markdown(text: &str) -> String {
	let mut result = String::new();
	let mut in_bold = false;
	let mut in_italic = false;
	let mut chars = text.chars().peekable();
	while let Some(c) = chars.next() {
		match c {
			'#' if chars.peek() == Some(&'#') => {
				chars.next();
				let mut title = String::new();
				while let Some(&next) = chars.peek() {
					if next == '\n' { break; }
					title.push(chars.next().unwrap());
				}
				result.push_str(&format!("  ——— **{}** ——— ", title.trim()));
			},
			'*' => {
				if chars.peek() == Some(&'*') {
					chars.next();
					in_bold = !in_bold;
					result.push_str("**");
				} else {
					in_italic = !in_italic;
					result.push('_');
				}
			},
			_ => result.push(c)
		}
	}
	result
}

pub fn render_poem_text(version: &Version) -> String {
	if !version.vertical.unwrap_or(false) && !version.rtl.unwrap_or(false) {
		return parse_markdown(&version.text);
	}
	if !version.vertical.unwrap_or(false) && version.rtl.unwrap_or(false) {
		let text = parse_markdown(&version.text);
		return text.lines().map(|line| line.chars().rev().collect::<String>()).collect::<Vec<_>>().join("\n");
	}
	if version.vertical.unwrap_or(false) {
		if version.rtl.unwrap_or(false) {
			let (cols, rows) = terminal::size().unwrap_or((80, 24));
			let viewport_height = rows.saturating_sub(3) as usize;
			let text = version.text.replace("\n", "");
			let total_chars = text.chars().count();
			let num_columns = (total_chars + viewport_height - 1) / viewport_height;
			let mut columns: Vec<Vec<char>> = Vec::with_capacity(num_columns);
			let mut char_iter = text.chars();
			for _ in 0..num_columns {
				let mut col = Vec::with_capacity(viewport_height);
				for _ in 0..viewport_height {
					if let Some(c) = char_iter.next() {
						col.push(c);
					} else {
						col.push(' ');
					}
				}
				columns.push(col);
			}
			columns.reverse();
			let mut output_lines: Vec<String> = Vec::with_capacity(viewport_height);
			for row in 0..viewport_height {
				let mut line = String::new();
				for col in &columns {
					line.push(col[row]);
				}
				output_lines.push(line);
			}
			return output_lines.join("\n");
		} else {
			let width = version.text.lines().map(|line| line.trim().chars().count()).max().unwrap_or(0);
			let lines: Vec<Vec<char>> = version.text.lines().map(|line| {
				let mut chars: Vec<char> = line.trim().chars().collect();
				while chars.len() < width {
					chars.push('　');
				}
				chars
			}).collect();
			if lines.is_empty() {
				return String::new();
			}
			let height = lines.len();
			return (0..width).map(|x| {
				(0..height).rev().map(|y| lines[y][x]).collect::<String>()
			}).collect::<Vec<String>>().join("\n");
		}
	}
	String::new()
}

pub fn render_status_bar(items: Vec<(&str, &str)>) -> Paragraph<'static> {
	let spans: Vec<Span<'static>> = items.into_iter().flat_map(|(key, desc)| vec![
		Span::styled(key.to_string(), Style::default().fg(Color::Yellow)),
		Span::raw(": ".to_string()),
		Span::raw(desc.to_string()),
		Span::raw(" | ".to_string()),
	]).collect();
	let mut spans = spans;
	if !spans.is_empty() {
		spans.pop();
	}
	Paragraph::new(Line::from(spans)).alignment(Alignment::Left)
}
