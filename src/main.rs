use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, SetTitle, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
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
    #[serde(skip)]
    filename: String,
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
    TitleList,
    FilteredList,
}

struct App {
    poems: Vec<Poem>,
    current_poem: usize,
    current_version: String,
    mode: AppMode,
    previous_mode: Option<AppMode>,
    scroll_position: u16,
    viewport_height: Option<u16>,  // Add this field
    author_counts: HashMap<String, usize>,
    author_list_state: ListState,

    language_counts: HashMap<String, usize>,
    language_list_state: ListState,

    title_list_state: ListState,

    filtered_list_state: ListState,

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

        let mut title_list_state = ListState::default();
        title_list_state.select(Some(0));

        let mut filtered_list_state = ListState::default();
        filtered_list_state.select(Some(0));

        Self {
            poems,
            current_poem: 0,
            current_version: "canonical".to_string(),
            mode: AppMode::Menu,
            previous_mode: None,
            scroll_position: 0, 
            viewport_height: None,
            author_counts,
            author_list_state: list_state,
            language_counts,
            language_list_state,
            menu_state,
            title_list_state,
            filtered_list_state,
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

    fn get_sorted_titles(&self) -> Vec<(usize, String)> {
        let mut titles: Vec<_> = self.poems.iter()
            .enumerate()
            .map(|(i, p)| (i, p.canonical.title.clone()))
            .collect();
        titles.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
        titles
    }

    fn next_title(&mut self) {
        let titles = self.get_sorted_titles();
        let i = match self.title_list_state.selected() {
            Some(i) => (i + 1) % titles.len(),
            None => 0,
        };
        self.title_list_state.select(Some(i));
    }

    fn previous_title(&mut self) {
        let titles = self.get_sorted_titles();
        let i = match self.title_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    titles.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.title_list_state.select(Some(i));
    }

    fn select_current_title(&mut self) {
        if let Some(index) = self.title_list_state.selected() {
            let titles = self.get_sorted_titles();
            if let Some((poem_index, _)) = titles.get(index) {
                self.current_poem = *poem_index;
                self.current_version = "canonical".to_string();
                self.filtered_poems = Some(vec![*poem_index]);
                self.previous_mode = Some(AppMode::TitleList);
                self.mode = AppMode::Viewing;
            }
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
                self.filtered_list_state.select(Some(0));
                self.previous_mode = Some(AppMode::AuthorList);
                self.mode = AppMode::FilteredList;
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
                // Store both index and version key for poems in the selected language
                let mut filtered_with_versions = Vec::new();
                for (i, poem) in self.poems.iter().enumerate() {
                    if poem.canonical.language == *language {
                        filtered_with_versions.push((i, "canonical".to_string()));
                    }
                    for (version_key, version) in &poem.other_versions {
                        if version.language == *language {
                            filtered_with_versions.push((i, version_key.clone()));
                        }
                    }
                }
                
                if !filtered_with_versions.is_empty() {
                    self.current_poem = filtered_with_versions[0].0;
                    self.current_version = filtered_with_versions[0].1.clone();
                }
                // Store just the indices in filtered_poems for the list view
                self.filtered_poems = Some(filtered_with_versions.iter().map(|(i, _)| *i).collect());
                self.filtered_list_state.select(Some(0));
                self.previous_mode = Some(AppMode::LanguageList);
                self.mode = AppMode::FilteredList;
            }
        }
    }

    fn get_version_in_language(&self, poem_idx: usize, language: &str) -> (&Version, bool) {
        let poem = &self.poems[poem_idx];
        
        // First check canonical version
        if poem.canonical.language == language {
            return (&poem.canonical, true);
        }
        
        // Then check other versions
        for version in poem.other_versions.values() {
            if version.language == language {
                return (version, true);
            }
        }
        
        // Fallback to canonical if no version found in target language
        (&poem.canonical, false)
    }

    fn get_filtered_list_title(&self) -> String {
        match self.previous_mode {
            Some(AppMode::AuthorList) => {
                if let Some(index) = self.author_list_state.selected() {
                    let authors = self.get_sorted_authors();
                    if let Some(author) = authors.get(index) {
                        return format!("Poems by {}", author);
                    }
                }
            },
            Some(AppMode::LanguageList) => {
                if let Some(index) = self.language_list_state.selected() {
                    let languages = self.get_sorted_languages();
                    if let Some(language) = languages.get(index) {
                        return format!("Poems in {}", language);
                    }
                }
            },
            Some(AppMode::TitleList) => {
                return "Search Results".to_string();
            },
            _ => {}
        }
        "Filtered Poems".to_string()
    }

    fn scroll_up(&mut self) {
        if self.scroll_position > 0 {
            self.scroll_position = self.scroll_position.saturating_sub(1);
        }
    }

    fn scroll_down(&mut self, max_scroll: u16) {
        if self.scroll_position < max_scroll {
            self.scroll_position = self.scroll_position.saturating_add(1);
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

    fn next_filtered(&mut self) {
        if let Some(indices) = &self.filtered_poems {
            let i = match self.filtered_list_state.selected() {
                Some(i) => (i + 1) % indices.len(),
                None => 0,
            };
            self.filtered_list_state.select(Some(i));
        }
    }

    fn previous_filtered(&mut self) {
        if let Some(indices) = &self.filtered_poems {
            let i = match self.filtered_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        indices.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.filtered_list_state.select(Some(i));
        }
    }

    fn select_current_filtered(&mut self) {
        if let Some(index) = self.filtered_list_state.selected() {
            if let Some(indices) = &self.filtered_poems {
                if let Some(&poem_index) = indices.get(index) {
                    self.current_poem = poem_index;
                    // When coming from language list, we need to find the correct version
                    if let Some(AppMode::LanguageList) = self.previous_mode {
                        if let Some(selected_lang_idx) = self.language_list_state.selected() {
                            let languages = self.get_sorted_languages();
                            if let Some(language) = languages.get(selected_lang_idx) {
                                // First check if canonical version matches
                                let poem = &self.poems[poem_index];
                                if poem.canonical.language == *language {
                                    self.current_version = "canonical".to_string();
                                } else {
                                    // Otherwise find the matching version
                                    for (version_key, version) in &poem.other_versions {
                                        if version.language == *language {
                                            self.current_version = version_key.clone();
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // For other modes (like author list), default to canonical
                        self.current_version = "canonical".to_string();
                    }
                    self.mode = AppMode::Viewing;
                }
            }
        }
    }

    fn set_mode(&mut self, new_mode: AppMode) {
        self.mode = new_mode;
        self.scroll_position = 0; 
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
            if let Ok(poem) = serde_yaml::from_str::<Poem>(&content) {
                let mut poem = poem;
                poem.filename = entry.path().file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into();
                poems.push(poem);
            }
        }
    }
    Ok(poems)
}

fn render_poem_text(version: &Version) -> String {
    if !version.vertical.unwrap_or(false) {
        return version.text.clone();
    }

    // Get max width first
    let width = version.text
        .lines()
        .map(|line| line.trim().chars().count())
        .max()
        .unwrap_or(0);

    // Pad all lines to the same width using full-width spaces
    let lines: Vec<Vec<char>> = version.text
        .lines()
        .map(|line| {
            let mut chars: Vec<char> = line.trim().chars().collect();
            while chars.len() < width {
                chars.push('　'); // Full-width space
            }
            chars
        })
        .collect();

    if lines.is_empty() {
        return String::new();
    }

    // Get dimensions
    let height = lines.len();

    // Create columns right-to-left
    let mut result = Vec::new();
    for x in 0..width {
        let mut column = Vec::new();
        for y in 0..height {
            let ch = lines
                .get(height - 1 - y) // Reverse y-axis for top-to-bottom
                .and_then(|line| line.get(x)) // Keep x-axis as is for right-to-left
                .copied()
                .unwrap_or('　'); // Full-width space for missing characters
            column.push(ch);
        }

        // Add the column to the result
        result.push(column.into_iter().collect::<String>());
    }

    result.join("\n")
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
    execute!(io::stdout(), EnterAlternateScreen)?;

    let mut stdout = io::stdout();
    
    // Set the terminal title
    execute!(stdout, SetTitle("leaves"))?;

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
                    Constraint::Length(1),
                ].as_ref())
                .split(f.size());

            if let AppMode::Viewing = app.mode {
                app.viewport_height = Some(chunks[0].height.saturating_sub(2));
            }

           let status_bar = match app.mode {
                AppMode::Viewing => {
                    let mut items = vec![
                        ("m", "menu"),
                        ("←/→", "navigate poems")
                    ];
                    
                    let text = render_poem_text(app.get_current_version());
                    let lines = text.lines().count();
                    let viewport_height = chunks[0].height as usize - 2;
                    
                    if lines > viewport_height {
                        items.push(("↑/↓", "scroll"));
                    }
                    
                    if app.filtered_poems.is_some() {
                        items.push(("backspace", "back to list"));
                    }
                    if !app.poems[app.current_poem].other_versions.is_empty() {
                        items.push(("s", "switch version"));
                    }
                    items.push(("ctrl+e", "edit"));
                    render_status_bar(items)
                },
               AppMode::Menu => render_status_bar(vec![
                   ("q", "quit"),
                   ("↑/↓", "select"),
                   ("enter", "choose")
               ]),
               AppMode::AuthorList | AppMode::LanguageList | AppMode::TitleList | AppMode::FilteredList => render_status_bar(vec![
                   ("↑/↓", "select"),
                   ("enter", "choose"),
                   ("backspace", "back")
               ]),
           };

            match app.mode {
                AppMode::Viewing => {
                    let version = app.get_current_version();
                    let text = render_poem_text(version);
                    let lines: Vec<&str> = text.lines().collect();
                    
                    let viewport_height = app.viewport_height.unwrap_or(chunks[0].height.saturating_sub(2)) as usize;
                    let has_scroll = lines.len() > viewport_height;
                    
                    let title = Line::from(vec![
                        Span::styled(&version.author, Style::default().fg(Color::Yellow)),
                        Span::raw(" - "),
                        Span::styled(&version.title, Style::default().fg(Color::Yellow)),
                    ]);
                    
                    let poem_block = Block::default()
                        .title(title)
                        .borders(Borders::ALL);

                    // Create inner area for content and scrollbar
                    let inner_area = poem_block.inner(chunks[0]);
                    let inner_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Min(1),
                            Constraint::Length(1),
                        ].as_ref())
                        .split(inner_area);

                    // Create scroll indicator if needed
                    let scroll_indicator = if has_scroll {
                        let total_lines = lines.len() as f64;
                        let visible_lines = viewport_height as f64;
                        let scroll_pos = app.scroll_position as f64;
                        
                        let scroll_height = ((visible_lines / total_lines) * (viewport_height - 2) as f64).max(1.0) as usize;
                        let max_scroll = total_lines - visible_lines;
                        let scroll_pos = if max_scroll > 0.0 {
                            ((scroll_pos / max_scroll) * (viewport_height - scroll_height - 2) as f64) + 1.0
                        } else {
                            1.0
                        } as usize;
                        
                        let mut indicator = vec!["│"; viewport_height];
                        indicator[0] = "▲";
                        indicator[viewport_height - 1] = "▼";
                        for i in scroll_pos..scroll_pos + scroll_height {
                            if i > 0 && i < viewport_height - 1 {
                                indicator[i] = "▐";
                            }
                        }
                        indicator.join("\n")
                    } else {
                        String::new()
                    };
                    let visible_text = lines
                        .iter()
                        .skip(app.scroll_position as usize)
                        .take(viewport_height)
                        .copied()
                        .collect::<Vec<&str>>()
                        .join("\n");

                    let alignment = if version.rtl.unwrap_or(false) {
                        ratatui::layout::Alignment::Right
                    } else {
                        ratatui::layout::Alignment::Left
                    };

                    // Render block first
                    f.render_widget(poem_block, chunks[0]);

                    // Then render content and scrollbar inside it
                    let poem_para = Paragraph::new(visible_text)
                        .style(Style::default().fg(Color::White))
                        .alignment(alignment);
                    f.render_widget(poem_para, inner_chunks[0]);

                    if has_scroll {
                        let scrollbar = Paragraph::new(scroll_indicator)
                            .style(Style::default().fg(Color::DarkGray));
                        f.render_widget(scrollbar, inner_chunks[1]);
                    }
                },
                AppMode::Menu => {
                    let items = vec![
                        ListItem::new(format!("Browse by author ({})", app.author_counts.len())),
                        ListItem::new(format!("Browse by language ({})", app.language_counts.len())),
                        ListItem::new(format!("Browse by title ({})", app.poems.len())),
                    ];
                    let menu = List::new(items)
                        .block(Block::default()
                            .title(Span::styled("Menu", Style::default().fg(Color::Yellow)))
                            .borders(Borders::ALL))
                        .style(Style::default().fg(Color::White))
                        .highlight_style(Style::default().fg(Color::Black).bg(Color::White));
                    f.render_stateful_widget(menu, chunks[0], &mut app.menu_state);
                },
                AppMode::TitleList => {
                    let titles = app.get_sorted_titles();
                    let items: Vec<ListItem> = titles.iter()
                        .map(|(_, title)| ListItem::new(title.clone()))
                        .collect();

                    let title_list = List::new(items)
                        .block(Block::default()
                            .title(Span::styled("Titles", Style::default().fg(Color::Yellow)))
                            .borders(Borders::ALL))
                        .style(Style::default().fg(Color::White))
                        .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

                    f.render_stateful_widget(title_list, chunks[0], &mut app.title_list_state);
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
                },
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
                AppMode::FilteredList => {
                    if let Some(indices) = &app.filtered_poems {
                        let items: Vec<ListItem> = indices.iter()
                            .map(|&idx| {
                                let poem = &app.poems[idx];
                                let display_text = match app.previous_mode {
                                    Some(AppMode::AuthorList) => {
                                        // When coming from author list, show only titles
                                        poem.canonical.title.clone()
                                    },
                                    Some(AppMode::LanguageList) => {
                                        if let Some(lang_index) = app.language_list_state.selected() {
                                            let languages = app.get_sorted_languages();
                                            if let Some(language) = languages.get(lang_index) {
                                                // Get the version in the target language
                                                let (version, _found) = app.get_version_in_language(idx, language);
                                                format!("{} - {}", version.author, version.title)
                                            } else {
                                                format!("{} - {}", poem.canonical.author, poem.canonical.title)
                                            }
                                        } else {
                                            format!("{} - {}", poem.canonical.author, poem.canonical.title)
                                        }
                                    },
                                    _ => {
                                        format!("{} - {}", poem.canonical.author, poem.canonical.title)
                                    }
                                };
                                ListItem::new(display_text)
                            })
                            .collect();

                        let filtered_list = List::new(items)
                            .block(Block::default()
                                .title(Span::styled(
                                    app.get_filtered_list_title(),
                                    Style::default().fg(Color::Yellow)
                                ))
                                .borders(Borders::ALL))
                            .style(Style::default().fg(Color::White))
                            .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

                        f.render_stateful_widget(filtered_list, chunks[0], &mut app.filtered_list_state);
                    }
                }
            }
            f.render_widget(status_bar, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Backspace => {
                    match app.mode {
                        AppMode::Viewing => {
                            if app.filtered_poems.is_some() {
                                app.mode = AppMode::FilteredList;
                            }
                        },
                        AppMode::FilteredList => {
                            app.mode = app.previous_mode.clone().unwrap_or(AppMode::Menu);
                        },
                        AppMode::AuthorList | AppMode::LanguageList | AppMode::TitleList => {
                            app.set_mode(AppMode::Menu)
                        },
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
                KeyCode::Right => match app.mode {
                    AppMode::Viewing => app.next_poem(),
                    _ => {}
                },
                KeyCode::Left => match app.mode {
                    AppMode::Viewing => app.previous_poem(),
                    _ => {}
                },
                KeyCode::Down => match app.mode {
                    AppMode::Viewing => {
                        let text = render_poem_text(app.get_current_version());
                        let lines = text.lines().count();
                        if let Some(viewport_height) = app.viewport_height {
                            let max_scroll = lines.saturating_sub(viewport_height as usize) as u16;
                            app.scroll_down(max_scroll);
                        }
                    },
                    AppMode::AuthorList => app.next_author(),
                    AppMode::LanguageList => app.next_language(),
                    AppMode::TitleList => app.next_title(),
                    AppMode::FilteredList => app.next_filtered(),
                    AppMode::Menu => {
                        if let Some(i) = app.menu_state.selected() {
                            let new_index = (i + 1) % 3; // Now 3 menu items
                            app.menu_state.select(Some(new_index));
                        }
                    }
                },
                KeyCode::Up => match app.mode {
                    AppMode::Viewing => {
                        app.scroll_up();
                    },
                    AppMode::AuthorList => app.previous_author(),
                    AppMode::LanguageList => app.previous_language(),
                    AppMode::TitleList => app.previous_title(),
                    AppMode::FilteredList => app.previous_filtered(),
                    AppMode::Menu => {
                        if let Some(i) = app.menu_state.selected() {
                            let new_index = if i == 0 { 0 } else { i - 1 };
                            app.menu_state.select(Some(new_index));
                        }
                    }
                },
                KeyCode::Char('e') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    match app.mode {
                        AppMode::Viewing => {
                            let home = std::env::var("HOME").expect("HOME not set");
                            let poem_path = PathBuf::from(home)
                                .join("Documents")
                                .join("poetry")
                                .join(&app.poems[app.current_poem].filename);
                            
                            #[cfg(target_os = "macos")]
                            std::process::Command::new("open")
                                .arg(&poem_path)
                                .spawn()
                                .expect("Failed to open file");

                            #[cfg(target_os = "linux")]
                            std::process::Command::new("xdg-open")
                                .arg(&poem_path)
                                .spawn()
                                .expect("Failed to open file");

                            #[cfg(target_os = "windows")] 
                            std::process::Command::new("cmd")
                                .args(["/C", "start", poem_path.to_str().unwrap()])
                                .spawn()
                                .expect("Failed to open file");
                        },
                        _ => {}
                    }
                },
                KeyCode::Enter => match app.mode {
                    AppMode::AuthorList => app.select_current_author(),
                    AppMode::LanguageList => app.select_current_language(),
                    AppMode::TitleList => app.select_current_title(),
                    AppMode::FilteredList => app.select_current_filtered(),
                    AppMode::Menu => {
                        match app.menu_state.selected() {
                            Some(0) => app.mode = AppMode::AuthorList,
                            Some(1) => app.mode = AppMode::LanguageList,
                            Some(2) => app.mode = AppMode::TitleList,
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
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}