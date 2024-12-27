use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, List, ListItem, ListState},
    text::{Line, Span},
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::{io, fs, path::PathBuf, collections::HashMap};

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

#[derive(Debug)]
enum AppMode {
    Viewing,
    Menu,
    AuthorList,
}

struct App {
    poems: Vec<Poem>,
    current_poem: usize,
    mode: AppMode,
    previous_mode: Option<AppMode>,
    author_counts: HashMap<String, usize>,
    author_list_state: ListState,
    menu_state: ListState,
    filtered_poems: Option<Vec<usize>>,
}

impl App {
    fn new(poems: Vec<Poem>) -> Self {
        let author_counts = poems.iter()
            .map(|p| p.canonical.author.clone())
            .fold(HashMap::new(), |mut map, author| {
                *map.entry(author).or_insert(0) += 1;
                map
            });
        
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        
        Self {
            poems,
            current_poem: 0,
            mode: AppMode::Menu,
            previous_mode: None,
            author_counts,
            author_list_state: list_state,
            menu_state: menu_state,
            filtered_poems: None,
        }
    }

    fn next_author(&mut self) {
        let authors = self.get_sorted_authors();
        let i = match self.author_list_state.selected() {
            Some(i) => (i + 1) % authors.len(),
            None => 0,
        };
        self.author_list_state.select(Some(i));
    }

    fn previous_author(&mut self) {
        let authors = self.get_sorted_authors();
        let i = match self.author_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    authors.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.author_list_state.select(Some(i));
    }

    fn get_sorted_authors(&self) -> Vec<String> {
        let mut authors: Vec<_> = self.author_counts.keys().cloned().collect();
        authors.sort();
        authors
    }

    fn select_current_author(&mut self) {
        if let Some(index) = self.author_list_state.selected() {
            let authors = self.get_sorted_authors();
            if let Some(author) = authors.get(index) {
                self.filtered_poems = Some(
                    self.poems.iter()
                        .enumerate()
                        .filter(|(_, poem)| &poem.canonical.author == author)
                        .map(|(i, _)| i)
                        .collect()
                );
                if let Some(ref indices) = self.filtered_poems {
                    if !indices.is_empty() {
                        self.current_poem = indices[0];
                    }
                }
                self.mode = AppMode::Viewing;
            }
        }
    }

    fn next_poem(&mut self) {
        match &self.filtered_poems {
            Some(indices) => {
                let current_pos = indices.iter().position(|&i| i == self.current_poem).unwrap_or(0);
                self.current_poem = indices[(current_pos + 1) % indices.len()];
            }
            None => {
                self.current_poem = (self.current_poem + 1) % self.poems.len();
            }
        }
    }

    fn previous_poem(&mut self) {
        match &self.filtered_poems {
            Some(indices) => {
                let current_pos = indices.iter().position(|&i| i == self.current_poem).unwrap_or(0);
                self.current_poem = if current_pos == 0 {
                    indices[indices.len() - 1]
                } else {
                    indices[current_pos - 1]
                };
            }
            None => {
                self.current_poem = if self.current_poem == 0 {
                    self.poems.len() - 1
                } else {
                    self.current_poem - 1
                };
            }
        }
    }

    fn set_mode(&mut self, new_mode: AppMode) {
        self.previous_mode = Some(std::mem::replace(&mut self.mode, new_mode));
    }
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

fn render_status_bar() -> Paragraph<'static> {
    Paragraph::new(Line::from(vec![
        Span::styled("m", Style::default().fg(Color::Yellow)),
        Span::raw(": menu | "),
        Span::styled("←/→", Style::default().fg(Color::Yellow)),
        Span::raw(": navigate poems | "),
        Span::styled("backspace", Style::default().fg(Color::Yellow)),
        Span::raw(": back | "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(": quit | "),
        Span::styled("s", Style::default().fg(Color::Yellow)),
        Span::raw(": switch version"),
    ]))
    .alignment(ratatui::layout::Alignment::Left)
    .block(Block::default())
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let poems = load_poems()?;
    let mut app = App::new(poems);

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(1),  // Height for status bar
                ].as_ref())
                .split(f.size());

            match app.mode {
                AppMode::Viewing => {
                    let poem = &app.poems[app.current_poem];
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
                },
                AppMode::Menu => {
                    let items = vec![
                        ListItem::new("Browse by Author")
                    ];
                    let menu = List::new(items)
                        .block(Block::default()
                            .title(Span::styled("Menu", Style::default().fg(Color::Yellow)))
                            .borders(Borders::ALL))
                        .style(Style::default().fg(Color::White))
                        .highlight_style(Style::default().fg(Color::Black).bg(Color::White));
                    f.render_stateful_widget(menu, chunks[0], &mut app.menu_state);
                },
                AppMode::AuthorList => {
                    let authors = app.get_sorted_authors();
                    let items: Vec<ListItem> = authors.iter()
                        .map(|author| {
                            ListItem::new(format!("{} ({})", author, app.author_counts[author]))
                        })
                        .collect();

                    let author_list = List::new(items)
                        .block(Block::default()
                            .title(Span::styled("Authors", Style::default().fg(Color::Yellow)))
                            .borders(Borders::ALL))
                        .style(Style::default().fg(Color::White))
                        .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

                    f.render_stateful_widget(author_list, chunks[0], &mut app.author_list_state);
                }
            }
            f.render_widget(render_status_bar(), chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    match app.mode {
                        AppMode::Viewing => break,
                        _ => {
                            app.mode = AppMode::Viewing;
                            app.filtered_poems = None;
                        }
                    }
                },
                KeyCode::Backspace => {
                    match app.mode {
                        AppMode::AuthorList => app.set_mode(AppMode::Menu),
                        AppMode::Viewing => {
                            if app.filtered_poems.is_some() {
                                app.set_mode(AppMode::AuthorList);
                            }
                        }
                        _ => {}
                    }
                },                
                KeyCode::Char('m') => {
                    app.mode = AppMode::Menu;
                },
                KeyCode::Char('1') => {
                    if matches!(app.mode, AppMode::Menu) {
                        app.mode = AppMode::AuthorList;
                    }
                },
                KeyCode::Right => match app.mode {
                    AppMode::Viewing => app.next_poem(),
                    _ => {}
                },
                KeyCode::Left => match app.mode {
                    AppMode::Viewing => app.previous_poem(),
                    _ => {}
                },
                KeyCode::Down => match app.mode {
                    AppMode::AuthorList => app.next_author(),
                    AppMode::Menu => {
                        if let Some(i) = app.menu_state.selected() {
                            let new_index = (i + 1) % 1; // Update when adding more menu items
                            app.menu_state.select(Some(new_index));
                        }
                    }
                    _ => {}
                },
                KeyCode::Up => match app.mode {
                    AppMode::AuthorList => app.previous_author(),
                    AppMode::Menu => {
                        if let Some(i) = app.menu_state.selected() {
                            let new_index = if i == 0 { 0 } else { i - 1 };
                            app.menu_state.select(Some(new_index));
                        }
                    }
                    _ => {}
                },
                KeyCode::Enter => match app.mode {
                    AppMode::AuthorList => app.select_current_author(),
                    AppMode::Menu => {
                        if let Some(0) = app.menu_state.selected() {
                            app.mode = AppMode::AuthorList;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}