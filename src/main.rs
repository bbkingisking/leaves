use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    text::{Line, Span},
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::{io, fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
struct Poem {
    canonical: Version,
    modern_spelling: Option<Version>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    title: String,
    author: String,
    language: String,
    text: String,
    rtl: Option<bool>,
    vertical: Option<bool>,
}

fn load_poems() -> io::Result<Vec<Poem>> {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    let poems_dir = PathBuf::from(home).join("Documents").join("poetry");
    
    let mut poems = Vec::new();
    for entry in fs::read_dir(poems_dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("poem") {
            let content = fs::read_to_string(entry.path())?;
            if let Ok(poem) = serde_yaml::from_str(&content) {
                poems.push(poem);
            }
        }
    }
    Ok(poems)
}

fn render_poem_text(version: &Version) -> String {
    if version.vertical.unwrap_or(false) {
        let lines: Vec<Vec<char>> = version.text
            .lines()
            .map(|line| line.chars().collect())
            .collect();
        
        if lines.is_empty() { return String::new() }
        
        let height = lines.len();
        let width = lines[0].len();
        
        (0..width)
            .map(|x| {
                (0..height).rev()
                    .map(|y| lines.get(y).and_then(|line| line.get(x)).unwrap_or(&' '))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        version.text.clone()
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let poems = load_poems()?;
    let mut current_poem = 0;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0)].as_ref())
                .split(f.size());

            let poem = &poems[current_poem];
            let version = poem.modern_spelling.as_ref().unwrap_or(&poem.canonical);
            let text = render_poem_text(version);

            let title = Line::from(vec![
                Span::styled(&version.author, Style::default().fg(Color::Yellow)),
                Span::raw(" - "),
                Span::raw(&version.title)
            ]);
            let poem_block = Block::default()
                .title(title)
                .borders(Borders::ALL);

            let alignment = if version.rtl.unwrap_or(false) {
                ratatui::layout::Alignment::Right
            } else {
                ratatui::layout::Alignment::Left
            };

            let poem_para = Paragraph::new(text)
                .block(poem_block)
                .style(Style::default().fg(Color::White))
                .alignment(alignment);

            f.render_widget(poem_para, chunks[0]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Right => {
                    current_poem = (current_poem + 1) % poems.len();
                }
                KeyCode::Left => {
                    current_poem = if current_poem == 0 {
                        poems.len() - 1
                    } else {
                        current_poem - 1
                    };
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}