# Leaves - terminal poetry reader

A TUI (Terminal User Interface) application for reading and managing multilingual poetry collections. Just learning `rust` and `ratatui`.

https://github.com/user-attachments/assets/4618727b-7302-44ae-917e-afab3d403ea3

## Features

- [x] Browse poems by author, language, or title
- [x] Support for multiple versions (e.g. translations, different spellings, &c.) of the same poem
- [x] Support for right-to-left (RTL) and vertical layouts without Unicode control characters (terminal emulators don't seem to like them)
- [x] Smooth scrolling for long poems

## To do

- [ ] Change version toggle to a pop-up menu with a list
- [ ] Search the base directory for `.poem` files recursively 
- [ ] Add a "browse by folder tree structure" option 
- [ ] Allow exporting to pdf, md, and custom XHTML
- [ ] Decide how to render epigraphs
- [ ] Add some language-specific parsing (e.g. half-nonbreaking spaces `U+202F` instead of whitespaces before punctuation in French)
- [ ] Record nicer demos

## Usage

Very much intended for personal use only but if you want to try it out/build upon it, there are a few considerations.

- It reads poems from a custom YAML schema that I haven't fully documented (yet), but the poems in `example_poems` can get you started. Alternatively, here is an incomplete definition of the `POEM` schema:

```yaml
---
canonical:
  title: "Original Title"
  author: "Author Name"
  language: "Original Language"
  rtl: false      # Optional
  vertical: false # Optional
  text: |
   line 1
   line 2
   …

other_version: # Optional
  title: "Translated Title"
  author: "Translator Name"
  language: "Target Language"
  rtl: false      # Optional
  vertical: false # Optional
  text: |
   line 1
   line 2
   …
```

- It is currently hardcoded to look for files  with `.poem` extension in `~/Documents/poetry/`. Change in the source file if needed. 

### Controls

- Navigation:
  - `←/→` - Previous/next poem
  - `↑/↓` - Scroll poem/navigate lists
  - `Enter` - Select item
  - `Backspace` - Return to previous view
- Views:
  - `m` - Open menu
  - `s` - Switch between versions/translations
  - `q` - Quit

## Dependencies

- ratatui
- crossterm
- serde
- serde_yaml
