use ratatui::{
	widgets::Paragraph,
	layout::Alignment,
	text::{Line, Span},
	style::{Style, Color},
};
use crossterm::terminal;
use crate::models::Version;

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
		return text
			.lines()
			.map(|line| line.chars().rev().collect::<String>())
			.collect::<Vec<_>>()
			.join("\n");
	}
	let (_cols, rows) = terminal::size().unwrap_or((80, 24));
	let viewport_height = rows.saturating_sub(3) as usize;
	let lines: Vec<&str> = version.text.lines().collect();
	let max_line_length = lines.iter().map(|l| l.trim().chars().count()).max().unwrap_or(0);
	if max_line_length <= viewport_height {
		let width = max_line_length;
		let matrix: Vec<Vec<char>> = lines
			.iter()
			.map(|line| {
				let mut v: Vec<char> = line.trim().chars().collect();
				while v.len() < width {
					v.push('　');
				}
				v
			})
			.collect();
		let height = matrix.len();
		return (0..width)
			.map(|x| (0..height).rev().map(|y| matrix[y][x]).collect::<String>())
			.collect::<Vec<String>>()
			.join("\n");
	} else {
		let mut groups: Vec<Vec<Vec<char>>> = Vec::new();
		for line in lines {
			let chars: Vec<char> = line.trim().chars().collect();
			let mut segments: Vec<Vec<char>> = Vec::new();
			let mut start = 0;
			while start < chars.len() {
				let end = (start + viewport_height).min(chars.len());
				let mut seg: Vec<char> = chars[start..end].to_vec();
				while seg.len() < viewport_height {
					seg.push('　');
				}
				segments.push(seg);
				start += viewport_height;
			}
			if version.rtl.unwrap_or(false) {
				segments.reverse();
			}
			groups.push(segments);
		}
		let mut all_columns: Vec<Vec<char>> = Vec::new();
		if version.rtl.unwrap_or(false) {
			groups.reverse();
			for group in groups {
				for seg in group {
					all_columns.push(seg);
				}
			}
		} else {
			for group in groups {
				for seg in group {
					all_columns.push(seg);
				}
			}
		}
		let mut output_lines: Vec<String> = Vec::with_capacity(viewport_height);
		for row in 0..viewport_height {
			let mut line = String::new();
			for col in &all_columns {
				line.push(col[row]);
			}
			output_lines.push(line);
		}
		return output_lines.join("\n");
	}
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
