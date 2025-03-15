mod models;
mod app;
mod ui;

use crossterm::{
	event::{self, Event, KeyCode, KeyModifiers},
	terminal::{disable_raw_mode, enable_raw_mode, SetTitle, EnterAlternateScreen, LeaveAlternateScreen},
	execute,
};
use ratatui::{
	Terminal,
	widgets::{Block, Borders, Paragraph, List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState},
	layout::{Constraint, Direction, Layout},
	style::{Style, Color},
	text::{Line, Span},
};
use textwrap;
use std::{io, path::PathBuf};
use app::App;
use models::load_poems;
use rand::Rng;

fn main() -> Result<(), io::Error> {
	enable_raw_mode()?;
	execute!(io::stdout(), EnterAlternateScreen)?;
	let mut stdout = io::stdout();
	execute!(stdout, SetTitle("leaves"))?;
	let backend = ratatui::backend::CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;
	let poems = load_poems()?;
	let mut app = App::new(poems);
	loop {
		terminal.draw(|f| {
			let chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(1), Constraint::Length(1)].as_ref()).split(f.size());
			if let app::AppMode::Viewing = app.mode {
				app.viewport_height = Some(chunks[0].height.saturating_sub(2));
			}
			let status_bar = match app.mode {
				app::AppMode::Viewing => {
					let mut items = vec![
						if app.filtered_poems.is_none() && app.previous_mode.is_none() {
							("m/backspace", "menu")
						} else {
							("m", "menu")
						},
						("←/→", "navigate poems")
					];
					let text = ui::render_poem_text(app.get_current_version());
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
					ui::render_status_bar(items)
				},
				app::AppMode::Menu => ui::render_status_bar(vec![
					("q", "quit"),
					("↑/↓", "select"),
					("enter", "choose")
				]),
				app::AppMode::AuthorList | app::AppMode::LanguageList | app::AppMode::TitleList | app::AppMode::FilteredList => ui::render_status_bar(vec![
					("↑/↓", "select"),
					("enter", "choose"),
					("backspace", "back")
				]),
			};
			match app.mode {
				app::AppMode::Viewing => {
					let version = app.get_current_version();
					let mut poem_text = String::new();
					if let Some(epigraph) = &version.epigraph {
						poem_text.push_str(epigraph);
						poem_text.push('\n');
					}
					poem_text.push_str(&ui::render_poem_text(version));
					let alignment = if version.rtl.unwrap_or(false) {
						ratatui::layout::Alignment::Right
					} else {
						ratatui::layout::Alignment::Left
					};
					// Use the overall chunk height to compute an approximate viewport height
					let viewport_height = chunks[0].height.saturating_sub(2) as usize;
					let total_lines = poem_text.lines().count();
					let max_scroll = total_lines.saturating_sub(viewport_height) as u16;
					let scroll_offset = app.scroll_position.min(max_scroll);
					let title = Line::from(vec![
						Span::raw(" "),
						Span::styled(&version.author, Style::default().fg(Color::Yellow)),
						Span::raw(" - "),
						Span::styled(&version.title, Style::default().fg(Color::Yellow))
					]);
					let poem_block = Block::default().title(title).borders(Borders::ALL);
					let inner_area = poem_block.inner(chunks[0]);
					let content_chunks = Layout::default()
						.direction(Direction::Horizontal)
						.constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
						.split(inner_area);
					let actual_viewport_height = content_chunks[0].height as usize;
					let max_width = content_chunks[0].width as usize;
					let options = textwrap::Options::new(max_width)
						.subsequent_indent("  ");
					let wrapped_text: String = poem_text.lines()
						.map(|line| {
							if line.trim().is_empty() {
								String::new()
							} else {
								textwrap::fill(line, options.clone())
							}
						})
						.collect::<Vec<_>>()
						.join("\n");
					let poem_para = Paragraph::new(wrapped_text)
						.style(Style::default().fg(Color::White))
						.alignment(alignment)
						.scroll((scroll_offset, 0));
					f.render_widget(poem_block.clone(), chunks[0]);
					f.render_widget(poem_para, content_chunks[0]);
					if total_lines > actual_viewport_height {
						let content_length = total_lines.saturating_sub(actual_viewport_height).saturating_add(1);
						let mut scrollbar_state = ScrollbarState::new(content_length)
							.position(app.scroll_position as usize)
							.viewport_content_length(actual_viewport_height);
						let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
							.begin_symbol(Some("▲"))
							.end_symbol(Some("▼"))
							.thumb_symbol("▐")
							.track_symbol(Some("│"));
						f.render_stateful_widget(scrollbar, content_chunks[1], &mut scrollbar_state);
					}
				},
				app::AppMode::Menu => {
					let items = vec![
						ListItem::new(format!("Browse by author ({})", app.author_counts.len())),
						ListItem::new(format!("Browse by language ({})", app.language_counts.len())),
						ListItem::new(format!("Browse by title ({})", app.poems.len())),
						ListItem::new("Random poem"),
					];
					let menu = List::new(items).block(Block::default().title(Span::styled("Menu", Style::default().fg(Color::Yellow))).borders(Borders::ALL)).style(Style::default().fg(Color::White)).highlight_style(Style::default().fg(Color::Black).bg(Color::White));
					f.render_stateful_widget(menu, chunks[0], &mut app.menu_state);
				},
				app::AppMode::TitleList => {
					let titles = app.get_sorted_titles();
					let items: Vec<ListItem> = titles.iter().map(|(_, title)| ListItem::new(title.clone())).collect();
					let title_list = List::new(items).block(Block::default().title(Span::styled("Titles", Style::default().fg(Color::Yellow))).borders(Borders::ALL)).style(Style::default().fg(Color::White)).highlight_style(Style::default().fg(Color::Black).bg(Color::White));
					f.render_stateful_widget(title_list, chunks[0], &mut app.title_list_state);
				},
				app::AppMode::AuthorList => {
					let authors = app.get_sorted_authors();
					let items: Vec<ListItem> = authors.iter().map(|author| ListItem::new(format!("{} ({})", author, app.author_counts[author]))).collect();
					let author_list = List::new(items).block(Block::default().title(Span::styled("Authors", Style::default().fg(Color::Yellow))).borders(Borders::ALL)).style(Style::default().fg(Color::White)).highlight_style(Style::default().fg(Color::Black).bg(Color::White));
					f.render_stateful_widget(author_list, chunks[0], &mut app.author_list_state);
				},
				app::AppMode::LanguageList => {
					let languages = app.get_sorted_languages();
					let items: Vec<ListItem> = languages.iter().map(|lang| ListItem::new(format!("{} ({})", lang, app.language_counts[lang]))).collect();
					let language_list = List::new(items).block(Block::default().title(Span::styled("Languages", Style::default().fg(Color::Yellow))).borders(Borders::ALL)).style(Style::default().fg(Color::White)).highlight_style(Style::default().fg(Color::Black).bg(Color::White));
					f.render_stateful_widget(language_list, chunks[0], &mut app.language_list_state);
				},
				app::AppMode::FilteredList => {
					if let Some(indices) = &app.filtered_poems {
						let items: Vec<ListItem> = indices.iter().map(|&idx| {
							let poem = &app.poems[idx];
							let display_text = match app.previous_mode {
								Some(app::AppMode::AuthorList) => poem.canonical.title.clone(),
								Some(app::AppMode::LanguageList) => {
									if let Some(lang_index) = app.language_list_state.selected() {
										let languages = app.get_sorted_languages();
										if let Some(language) = languages.get(lang_index) {
											let (version, _found) = app.get_version_in_language(idx, language);
											format!("{} - {}", version.author, version.title)
										} else {
											format!("{} - {}", poem.canonical.author, poem.canonical.title)
										}
									} else {
										format!("{} - {}", poem.canonical.author, poem.canonical.title)
									}
								},
								_ => format!("{} - {}", poem.canonical.author, poem.canonical.title),
							};
							ListItem::new(display_text)
						}).collect();
						let filtered_list = List::new(items).block(Block::default().title(Span::styled(app.get_filtered_list_title(), Style::default().fg(Color::Yellow))).borders(Borders::ALL)).style(Style::default().fg(Color::White)).highlight_style(Style::default().fg(Color::Black).bg(Color::White));
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
						app::AppMode::Viewing => {
							if app.filtered_poems.is_some() {
								app.mode = app::AppMode::FilteredList;
							} else {
								app.mode = app::AppMode::Menu;
							}
						},
						app::AppMode::FilteredList => {
							app.mode = app.previous_mode.clone().unwrap_or(app::AppMode::Menu);
						},
						app::AppMode::AuthorList | app::AppMode::LanguageList | app::AppMode::TitleList => {
							app.set_mode(app::AppMode::Menu)
						},
						_ => {}
					}
				},
				KeyCode::Char('m') => {
					app.mode = app::AppMode::Menu;
				},
				KeyCode::Char('s') => match app.mode {
					app::AppMode::Viewing => app.toggle_version(),
					_ => {}
				},
				KeyCode::Right => match app.mode {
					app::AppMode::Viewing => app.next_poem(),
					_ => {}
				},
				KeyCode::Left => match app.mode {
					app::AppMode::Viewing => app.previous_poem(),
					_ => {}
				},
				KeyCode::Down => match app.mode {
					app::AppMode::Viewing => {
						let text = ui::render_poem_text(app.get_current_version());
						let lines = text.lines().count();
						if let Some(viewport_height) = app.viewport_height {
							let max_scroll = lines.saturating_sub(viewport_height as usize) as u16;
							app.scroll_down(1, max_scroll);
						}
					},
					app::AppMode::AuthorList => app.next_author(),
					app::AppMode::LanguageList => app.next_language(),
					app::AppMode::TitleList => app.next_title(),
					app::AppMode::FilteredList => app.next_filtered(),
					app::AppMode::Menu => {
						if let Some(i) = app.menu_state.selected() {
							let new_index = (i + 1) % 4;
							app.menu_state.select(Some(new_index));
						}
					}
				},
				KeyCode::Up => match app.mode {
					app::AppMode::Viewing => {
						app.scroll_up(1);
					},
					app::AppMode::AuthorList => app.previous_author(),
					app::AppMode::LanguageList => app.previous_language(),
					app::AppMode::TitleList => app.previous_title(),
					app::AppMode::FilteredList => app.previous_filtered(),
					app::AppMode::Menu => {
						if let Some(i) = app.menu_state.selected() {
							let new_index = if i == 0 { 0 } else { i - 1 };
							app.menu_state.select(Some(new_index));
						}
					}
				},
				KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
					match app.mode {
						app::AppMode::Viewing => {
							let home = std::env::var("HOME").expect("HOME not set");
							let poem_path = PathBuf::from(home).join("Documents").join("poetry").join(&app.poems[app.current_poem].filename);
							#[cfg(target_os = "macos")]
							std::process::Command::new("open").arg(&poem_path).spawn().expect("Failed to open file");
							#[cfg(target_os = "linux")]
							std::process::Command::new("xdg-open").arg(&poem_path).spawn().expect("Failed to open file");
							#[cfg(target_os = "windows")]
							std::process::Command::new("cmd").args(["/C", "start", poem_path.to_str().unwrap()]).spawn().expect("Failed to open file");
						},
						_ => {}
					}
				},
				KeyCode::Enter => match app.mode {
					app::AppMode::AuthorList => app.select_current_author(),
					app::AppMode::LanguageList => app.select_current_language(),
					app::AppMode::TitleList => app.select_current_title(),
					app::AppMode::FilteredList => app.select_current_filtered(),
					app::AppMode::Menu => {
						match app.menu_state.selected() {
							Some(0) => app.mode = app::AppMode::AuthorList,
							Some(1) => app.mode = app::AppMode::LanguageList,
							Some(2) => app.mode = app::AppMode::TitleList,
							Some(3) => {
								let mut rng = rand::thread_rng();
								app.current_poem = rng.gen_range(0..app.poems.len());
								app.current_version = "canonical".to_string();
								app.filtered_poems = None;
								app.mode = app::AppMode::Viewing;
							},
							_ => {}
						}
					},
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