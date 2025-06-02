# Leaves - terminal poetry reader

A TUI (Terminal User Interface) application for reading poetry. Just learning `rust` and `ratatui`.

https://github.com/user-attachments/assets/904d594a-8489-4d8d-98bc-cf3f6d377762

## Features

- [x] Browse poems by author, language, or title
- [x] Support for multiple versions (e.g. translations, different spellings, &c.) of the same poem
- [x] Support for right-to-left (RTL) and vertical layouts without Unicode control characters (terminal emulators don't seem to like them)
- [x] Smooth scrolling for long poems
- [x] Add filtering/search

## To do

- [x] Add vim keybindings for common interactions
- [x] Change version toggle to a pop-up menu with a list
- [ ] Search the base directory for `.poem` files recursively
- [ ] Decide how to render epigraphs
- [ ] Record nicer demos

## Build

Just clone and `cargo build`.

## Usage

Mainly for my personal use but if you want to try it out/build upon it, there are a few considerations.

- It reads poems from a custom YAML schema which can be found [here](https://github.com/bbkingisking/poem-schema). A poetry collection that already conforms to the schema can be found [here](https://github.com/bbkingisking/poetry). These are intended to be examples that you can follow to format your own poetry collections.

- It is currently hardcoded to look for files with a `.poem` extension in `~/literature/poetry/`. Change the directory in `models::load_poems` before building if needed.

### Controls

- Navigation:
  - `←/→` - Previous/next poem
  - `↑/↓` - Scroll poem/navigate lists
  - `Enter` - Select item
  - `Backspace` - Return to previous view
- Views:
  - `m` - Main menu
  - `s` - Switch between versions/translations
  - `q` - Quit

## Dependencies

- ratatui
- crossterm
- serde
- serde_yaml
