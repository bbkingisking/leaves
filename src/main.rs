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
    #[serde(flatten)]
    other_versions: HashMap<String, Version>,
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

#[derive(Debug, Clone)]
enum AppMode {
    Viewing,
    Menu,
    AuthorList,
    LanguageList,
}

struct App {
    poems: Vec<Poem>,
    current_poem: usize,
    current_version: String,
    mode: AppMode,
    previous_mode: Option<AppMode>,

    author_counts: HashMap<String, usize>,
    author_list_state: ListState,

    language_counts: HashMap<String, usize>,
    language_list_state: ListState,

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

        let language_counts = poems.iter()
            .flat_map(|p| {
                std::iter::once(p.canonical.language.clone())
                    .chain(p.other_versions.values()
                        .map(|v| v.language.clone()))
            })
            .fold(HashMap::new(), |mut map, lang| {
                *map.entry(lang).or_insert(0) += 1;
                map
            });

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        
        let mut language_list_state = ListState::default();
        language_list_state.select(Some(0));

        Self {
            poems,
            current_poem: 0,
            current_version: "canonical".to_string(),  // Default version
            mode: AppMode::Menu,
            previous_mode: None,
            author_counts,
            author_list_state: list_state,
            language_counts,
            language_list_state,
            menu_state: menu_state,
            filtered_poems: None,
        }
    }

    fn toggle_version(&mut self) {
        let poem = &self.poems[self.current_poem];
        let versions: Vec<String> = std::iter::once("canonical".to_string())
            .chain(poem.other_versions.keys().cloned())
            .collect();
        
        if versions.len() > 1 {
            let current_idx = versions.iter()
                .position(|v| v == &self.current_version)
                .unwrap_or(0);
            let next_idx = (current_idx + 1) % versions.len();
            self.current_version = versions[next_idx].clone();
        }
    }

    fn get_current_version(&self) -> &Version {
        let poem = &self.poems[self.current_poem];
        if self.current_version == "canonical" {
            &poem.canonical
        } else {
            poem.other_versions.get(&self.current_version)
                .unwrap_or(&poem.canonical)
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
                self.previous_mode = Some(AppMode::AuthorList);
                self.mode = AppMode::Viewing;
            }
        }
    }

    fn get_sorted_languages(&self) -> Vec<String> {
        let mut languages: Vec<_> = self.language_counts.keys().cloned().collect();
        languages.sort();
        languages
    }

    fn next_language(&mut self) {
        let languages = self.get_sorted_languages();
        let i = match self.language_list_state.selected() {
            Some(i) => (i + 1) % languages.len(),
            None => 0,
        };
        self.language_list_state.select(Some(i));
    }

    fn previous_language(&mut self) {
        let languages = self.get_sorted_languages();
        let i = match self.language_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    languages.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.language_list_state.select(Some(i));
    }

    fn select_current_language(&mut self) {
       if let Some(index) = self.language_list_state.selected() {
           let languages = self.get_sorted_languages();
           if let Some(language) = languages.get(index) {
               let mut filtered = Vec::new();
               for (i, poem) in self.poems.iter().enumerate() {
                   if poem.canonical.language == *language {
                       filtered.push((i, "canonical"));
                   }
                   for (version_key, version) in &poem.other_versions {
                       if version.language == *language {
                           filtered.push((i, version_key));
                       }
                   }
               }
               
               self.filtered_poems = Some(filtered.iter().map(|(i, _)| *i).collect());
               if !filtered.is_empty() {
                   self.current_poem = filtered[0].0;
                   self.current_version = filtered[0].1.to_string();
               }
               self.previous_mode = Some(AppMode::LanguageList);
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
        self.mode = new_mode;
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
            .filter(|line| !line.trim().is_empty())  // Skip empty lines
            .map(|line| line.trim().chars().collect())  // Trim whitespace
            .collect();
        
        if lines.is_empty() { return String::new() }
        
        let max_width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
        let padded_lines: Vec<Vec<char>> = lines.into_iter()
            .map(|mut line| {
                while line.len() < max_width {
                    line.push(' ');
                }
                line
            })
            .collect();
        
        let height = padded_lines.len();
        
        (0..max_width)
            .map(|x| {
                (0..height).rev()
                    .map(|y| padded_lines.get(y).and_then(|line| line.get(x)).unwrap_or(&' '))
                    .collect::<String>()
                    .trim_end()  // Remove trailing spaces
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        version.text.clone()
    }
}

fn render_status_bar(items: Vec<(&str, &str)>) -> Paragraph<'static> {
    let spans: Vec<Span<'static>> = items.into_iter()
        .flat_map(|(key, desc)| vec![
            Span::styled(key.to_string(), Style::default().fg(Color::Yellow)),
            Span::raw(": ".to_string()),
            Span::raw(desc.to_string()),
            Span::raw(" | ".to_string()),
        ])
        .collect();

   // Remove trailing separator
   let mut spans = spans;
   if !spans.is_empty() {
       spans.pop();
   }

   Paragraph::new(Line::from(spans))
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

           let status_bar = match app.mode {
               AppMode::Viewing => {
                   let mut items = vec![
                       ("m", "menu"),
                       ("←/→", "navigate poems")
                   ];
                   if app.filtered_poems.is_some() {
                       items.push(("backspace", "back"));
                   }
                   if !app.poems[app.current_poem].other_versions.is_empty() {
                       items.push(("s", "switch version"));
                   }
                   render_status_bar(items)
               },
               AppMode::Menu => render_status_bar(vec![
                   ("q", "quit"),
                   ("↑/↓", "select"),
                   ("enter", "choose")
               ]),
               AppMode::AuthorList | AppMode::LanguageList => render_status_bar(vec![
                   ("↑/↓", "select"),
                   ("enter", "choose"),
                   ("backspace", "back")
               ]),
           };

            match app.mode {
                AppMode::Viewing => {
                    let version = app.get_current_version();
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
                        ListItem::new(format!("Browse by author ({})", app.author_counts.len())),
                        ListItem::new(format!("Browse by language ({})", app.language_counts.len())),
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
                AppMode::LanguageList => {
                    let languages = app.get_sorted_languages();
                    let items: Vec<ListItem> = languages.iter()
                        .map(|lang| {
                            ListItem::new(format!("{} ({})", lang, app.language_counts[lang]))
                        })
                        .collect();

                    let language_list = List::new(items)
                        .block(Block::default()
                            .title(Span::styled("Languages", Style::default().fg(Color::Yellow)))
                            .borders(Borders::ALL))
                        .style(Style::default().fg(Color::White))
                        .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

                    f.render_stateful_widget(language_list, chunks[0], &mut app.language_list_state);
                }
            }
        f.render_widget(status_bar, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => { 
                    break
                },
                KeyCode::Backspace => {
                    match app.mode {
                    AppMode::AuthorList | AppMode::LanguageList => app.set_mode(AppMode::Menu),
                    AppMode::Viewing => {
                        if app.filtered_poems.is_some() {
                            app.set_mode(match app.previous_mode.as_ref().unwrap_or(&AppMode::Menu) {
                                AppMode::AuthorList => AppMode::AuthorList,
                                AppMode::LanguageList => AppMode::LanguageList,
                                _ => AppMode::Menu,
                            });
                        }
                    }
                    _ => {}
                    }
                },                
                KeyCode::Char('m') => {
                    app.mode = AppMode::Menu;
                },
                KeyCode::Char('s') => match app.mode {
                    AppMode::Viewing => app.toggle_version(),
                    _ => {}
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
                    AppMode::LanguageList => app.next_language(),
                    AppMode::Menu => {
                        if let Some(i) = app.menu_state.selected() {
                            let new_index = (i + 1) % 2; // Update when adding more menu items
                            app.menu_state.select(Some(new_index));
                        }
                    }
                    _ => {}
                },
                KeyCode::Up => match app.mode {
                    AppMode::AuthorList => app.previous_author(),
                    AppMode::LanguageList => app.previous_language(),
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
                    AppMode::LanguageList => app.select_current_language(),
                    AppMode::Menu => {
                        match app.menu_state.selected() {
                            Some(0) => app.mode = AppMode::AuthorList,
                            Some(1) => app.mode = AppMode::LanguageList,
                            _ => {}
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