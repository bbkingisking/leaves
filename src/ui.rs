use crossterm::terminal;
use crate::models::Version;
use unicode_bidi::BidiInfo;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    widgets::{Paragraph},
    text::{Line, Span},
    style::{Style, Color},
    prelude::*,
};

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
	// Case 1: No vertical or RTL formatting enabled.
	// Simply parse the markdown and return the result.
	if !version.vertical.unwrap_or(false) && !version.rtl.unwrap_or(false) {
		return parse_markdown(&version.text);
	}

	// Case 2: RTL formatting only (vertical is false).
	// Parse the markdown, then reverse each line for proper RTL display.
	if !version.vertical.unwrap_or(false) && version.rtl.unwrap_or(false) {
		let text = parse_markdown(&version.text);
		return process_rtl_text(&text);
	}

	// Case 3: Vertical formatting is enabled.
	// First, get the terminal size; default to 80x24 if unavailable.
	let (_cols, rows) = terminal::size().unwrap_or((80, 24));
	// Reserve a few rows (e.g., for UI elements) and set the viewport height.
	let viewport_height = rows.saturating_sub(3) as usize;

	// Split the original text into individual lines.
	let lines: Vec<&str> = version.text.lines().collect();
	// Determine the maximum number of characters in any line (after trimming).
	let max_line_length = lines.iter().map(|l| l.trim().chars().count()).max().unwrap_or(0);

	// If the longest line fits within the viewport height,
	// render without wrapping by building a character matrix.
	if max_line_length <= viewport_height {
		let width = max_line_length;
		// Create a matrix of characters where each row represents a line.
		// Shorter lines are padded with a full-width space ('　') to ensure equal length.
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
		// Render the poem vertically by reading the matrix column-wise in reverse row order.
		return (0..width)
			.map(|x| (0..height).rev().map(|y| matrix[y][x]).collect::<String>())
			.collect::<Vec<String>>()
			.join("\n");
	} else {
		// Otherwise, one or more lines are too long and need wrapping.
		// Process each original line individually, splitting it into segments that fit the viewport height.
		let mut groups: Vec<Vec<Vec<char>>> = Vec::new();
		for line in lines {
			// Trim the line and convert it into a vector of characters.
			let chars: Vec<char> = line.trim().chars().collect();
			let mut segments: Vec<Vec<char>> = Vec::new();
			let mut start = 0;
			// Split the line into segments of at most viewport_height characters.
			while start < chars.len() {
				let end = (start + viewport_height).min(chars.len());
				let mut seg: Vec<char> = chars[start..end].to_vec();
				// If the segment is shorter than viewport_height, pad it with a full-width space.
				while seg.len() < viewport_height {
					seg.push('　');
				}
				segments.push(seg);
				start += viewport_height;
			}
			// For RTL text, reverse the order of segments to preserve the correct reading order.
			if version.rtl.unwrap_or(false) {
				segments.reverse();
			}
			groups.push(segments);
		}

		// Combine all segments from every line into a single vector of columns.
		// For RTL texts, reverse the overall order of the groups.
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

		// Build the final output by reading the characters row by row across all columns.
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

fn process_rtl_text(text: &str) -> String {
	text
		.lines()
		.map(|line| {
			let bidi_info = BidiInfo::new(line, None);
			let para = bidi_info.paragraphs.first().unwrap();
			bidi_info.reorder_line(para, para.range.clone()).into_owned()
		})
		.collect::<Vec<_>>()
		.join("\n")
}

pub fn render_vertical_rtl_title(author: &str, title: &str) -> String {
	// Create vertical text that reads from top to bottom for RTL mode
	// Format: "Author - Title" but displayed vertically
	let full_text = format!("{}|{}", author, title);

	// Convert to vertical text (each character on its own line)
	// Don't skip any characters, including spaces
	full_text.chars().map(|c| c.to_string()).collect::<Vec<String>>().join("\n")
}

pub fn popup_area(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
	let popup_layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Percentage((100 - height_percent) / 2),
			Constraint::Percentage(height_percent),
			Constraint::Percentage((100 - height_percent) / 2),
		])
		.split(area);

	Layout::default()
		.direction(Direction::Horizontal)
		.constraints([
			Constraint::Percentage((100 - width_percent) / 2),
			Constraint::Percentage(width_percent),
			Constraint::Percentage((100 - width_percent) / 2),
		])
		.split(popup_layout[1])[1]
}
