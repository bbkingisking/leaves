use crate::models::{Poem, Version};
use std::collections::HashMap;
use ratatui::widgets::ListState;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
	Viewing,
	Menu,
	AuthorList,
	LanguageList,
	TitleList,
	FilteredList,
	Search,
	VersionSelect,
}

pub struct App {
	pub poems: Vec<Poem>,
	pub current_poem: usize,
	pub current_version: String,
	pub mode: AppMode,
	pub previous_mode: Option<AppMode>,
	pub scroll_position: u16,
	pub viewport_height: Option<u16>,
	pub author_counts: HashMap<String, usize>,
	pub author_list_state: ListState,
	pub language_counts: HashMap<String, usize>,
	pub language_list_state: ListState,
	pub title_list_state: ListState,
	pub filtered_list_state: ListState,
	pub menu_state: ListState,
	pub filtered_poems: Option<Vec<usize>>,
	pub search_query: String,
	pub search_list_state: ListState,
	pub search_results: Vec<usize>,
	pub version_list_state: ListState,
}

impl App {
	pub fn new(poems: Vec<Poem>) -> Self {
		let author_counts = poems.iter().map(|p| p.canonical.author.clone()).fold(HashMap::new(), |mut map, author| {
			*map.entry(author).or_insert(0) += 1;
			map
		});
		let language_counts = poems.iter().flat_map(|p| {
			std::iter::once(p.canonical.language.clone()).chain(p.other_versions.values().map(|v| v.language.clone()))
		}).fold(HashMap::new(), |mut map, lang| {
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
			search_query: String::new(),
			search_list_state: {
				let mut state = ListState::default();
				state.select(Some(0));
				state
			},
			search_results: Vec::new(),
			version_list_state: {
				let mut state = ListState::default();
				state.select(Some(0));
				state
			},
		}
	}
	pub fn get_current_version(&self) -> &Version {
		let poem = &self.poems[self.current_poem];
		if self.current_version == "canonical" {
			&poem.canonical
		} else {
			poem.other_versions.get(&self.current_version).unwrap_or(&poem.canonical)
		}
	}
	pub fn get_sorted_titles(&self) -> Vec<(usize, String)> {
		let mut titles: Vec<_> = self.poems.iter().enumerate().map(|(i, p)| (i, p.canonical.title.clone())).collect();
		titles.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
		titles
	}
	pub fn next_title(&mut self) {
		let titles = self.get_sorted_titles();
		let i = match self.title_list_state.selected() {
			Some(i) => (i + 1) % titles.len(),
			None => 0,
		};
		self.title_list_state.select(Some(i));
	}
	pub fn previous_title(&mut self) {
		let titles = self.get_sorted_titles();
		let i = match self.title_list_state.selected() {
			Some(i) => if i == 0 { titles.len() - 1 } else { i - 1 },
			None => 0,
		};
		self.title_list_state.select(Some(i));
	}
	pub fn select_current_title(&mut self) {
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
	pub fn next_author(&mut self) {
		let authors = self.get_sorted_authors();
		let i = match self.author_list_state.selected() {
			Some(i) => (i + 1) % authors.len(),
			None => 0,
		};
		self.author_list_state.select(Some(i));
	}
	pub fn previous_author(&mut self) {
		let authors = self.get_sorted_authors();
		let i = match self.author_list_state.selected() {
			Some(i) => if i == 0 { authors.len() - 1 } else { i - 1 },
			None => 0,
		};
		self.author_list_state.select(Some(i));
	}
	pub fn get_sorted_authors(&self) -> Vec<String> {
		let mut authors: Vec<_> = self.author_counts.keys().cloned().collect();
		authors.sort();
		authors
	}
	pub fn select_current_author(&mut self) {
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
	pub fn get_sorted_languages(&self) -> Vec<String> {
		let mut languages: Vec<_> = self.language_counts.keys().cloned().collect();
		languages.sort_by_key(|lang| std::cmp::Reverse(self.language_counts[lang]));
		languages
	}
	pub fn next_language(&mut self) {
		let languages = self.get_sorted_languages();
		let i = match self.language_list_state.selected() {
			Some(i) => (i + 1) % languages.len(),
			None => 0,
		};
		self.language_list_state.select(Some(i));
	}
	pub fn previous_language(&mut self) {
		let languages = self.get_sorted_languages();
		let i = match self.language_list_state.selected() {
			Some(i) => if i == 0 { languages.len() - 1 } else { i - 1 },
			None => 0,
		};
		self.language_list_state.select(Some(i));
	}
	pub fn select_current_language(&mut self) {
		if let Some(index) = self.language_list_state.selected() {
			let languages = self.get_sorted_languages();
			if let Some(language) = languages.get(index) {
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
				self.filtered_poems = Some(filtered_with_versions.iter().map(|(i, _)| *i).collect());
				self.filtered_list_state.select(Some(0));
				self.previous_mode = Some(AppMode::LanguageList);
				self.mode = AppMode::FilteredList;
			}
		}
	}
	pub fn get_version_in_language(&self, poem_idx: usize, language: &str) -> (&Version, bool) {
		let poem = &self.poems[poem_idx];
		if poem.canonical.language == language {
			return (&poem.canonical, true);
		}
		for version in poem.other_versions.values() {
			if version.language == language {
				return (version, true);
			}
		}
		(&poem.canonical, false)
	}
	pub fn get_filtered_list_title(&self) -> String {
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
			Some(AppMode::TitleList) => return "Search Results".to_string(),
			_ => {}
		}
		"Filtered Poems".to_string()
	}
	pub fn scroll_up(&mut self, delta: u16) {
		self.scroll_position = self.scroll_position.saturating_sub(delta);
	}
	pub fn scroll_down(&mut self, delta: u16, max_scroll: u16) {
		self.scroll_position = (self.scroll_position.saturating_add(delta)).min(max_scroll);
	}
	pub fn next_poem(&mut self) {
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
	pub fn previous_poem(&mut self) {
		match &self.filtered_poems {
			Some(indices) => {
				let current_pos = indices.iter().position(|&i| i == self.current_poem).unwrap_or(0);
				self.current_poem = if current_pos == 0 { indices[indices.len() - 1] } else { indices[current_pos - 1] };
			}
			None => {
				self.current_poem = if self.current_poem == 0 { self.poems.len() - 1 } else { self.current_poem - 1 };
			}
		}
	}
	pub fn next_filtered(&mut self) {
		if let Some(indices) = &self.filtered_poems {
			let i = match self.filtered_list_state.selected() {
				Some(i) => (i + 1) % indices.len(),
				None => 0,
			};
			self.filtered_list_state.select(Some(i));
		}
	}
	pub fn previous_filtered(&mut self) {
		if let Some(indices) = &self.filtered_poems {
			let i = match self.filtered_list_state.selected() {
				Some(i) => if i == 0 { indices.len() - 1 } else { i - 1 },
				None => 0,
			};
			self.filtered_list_state.select(Some(i));
		}
	}
	pub fn select_current_filtered(&mut self) {
		if let Some(index) = self.filtered_list_state.selected() {
			if let Some(indices) = &self.filtered_poems {
				if let Some(&poem_index) = indices.get(index) {
					self.current_poem = poem_index;
					if let Some(AppMode::LanguageList) = self.previous_mode {
						if let Some(selected_lang_idx) = self.language_list_state.selected() {
							let languages = self.get_sorted_languages();
							if let Some(language) = languages.get(selected_lang_idx) {
								let poem = &self.poems[poem_index];
								if poem.canonical.language == *language {
									self.current_version = "canonical".to_string();
								} else {
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
						self.current_version = "canonical".to_string();
					}
					self.mode = AppMode::Viewing;
				}
			}
		}
	}
	pub fn set_mode(&mut self, new_mode: AppMode) {
		self.mode = new_mode;
		self.scroll_position = 0;
	}
	pub fn update_search_results(&mut self) {
		let query = self.search_query.to_lowercase();
		if query.is_empty() {
			self.search_results.clear();
			self.search_list_state.select(None);
		} else {
			self.search_results = self.poems.iter().enumerate().filter_map(|(i, poem)| {
				if poem.canonical.title.to_lowercase().contains(&query) || poem.canonical.author.to_lowercase().contains(&query) {
					Some(i)
				} else {
					None
				}
			}).collect();
			if self.search_results.is_empty() {
				self.search_list_state.select(None);
			} else if self.search_list_state.selected().is_none() {
				self.search_list_state.select(Some(0));
			}
		}
	}
}
