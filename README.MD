# Leaves - terminal poetry reader

A TUI (Terminal User Interface) application for reading poetry. Just learning `rust` and `ratatui`.

https://github.com/user-attachments/assets/4618727b-7302-44ae-917e-afab3d403ea3

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

- It reads poems from a custom YAML schema that I haven't fully documented (yet), but the poems in `example_poems` can get you started. Here is an incomplete definition of the `poem` schema:

```yaml
---
canonical:
  title: poem title
  author: author name
  language: lang # ISO 639-3
  rtl: false      # Optional, defaults to false if missing
  vertical: false # Optional, defaults to false if missing 
  text: |
   line 1
   line 2
   …

other_version: # Optional, same possible sub-keys as above
```

Example minimal poem (one version, only required fields): 

```yaml
---
canonical:
  title: I'm Nobody! Who are You?
  author: Emily Dickinson
  language: eng
  text: |
    I'm nobody! Who are you?
    Are you nobody, too?
    Then there's a pair of us — don't tell!
    They'd banish us, you know.

    How dreary to be somebody!
    How public, like a frog
    To tell your name the livelong day
    To an admiring bog!
```

Example poem with four different versions and different layouts:

```yaml
---
canonical:
  title: Scene of Snow
  author: Kim Sat Gat
  language: eng
  text: |
    Snow flakes flying down
    Like March butterflies;
    Stepped on,
    Making sounds of June frogs

    The host insists on
    Staying for snow and coldness;
    Wishing his guest to stay drunken, 
    Another drink, he offers.
original:
  title: 雪景
  author: 金炳淵
  language: lzh
  rtl: true
  vertical: true
  text: |
    飛來片片三月蝶
    踏去聲聲六月蛙
    寒將不去多言雪
    醉或以留更進盃
korean:
  title: 설경
  author: 김삿갓
  language: kor
  text: |
    펄펄 날리니 춘삼월 나비같고
    뽀드득 밟는 소리 유월 개구리같네
    추워서 못 가신다고 눈을 자꾸 핑계 대며
    취중에 행여 머무를까 다시 술잔을 내미네
hanja_eum:
  title: 설경 (한자음)
  author: 김삿갓
  language: kor
  text: |
    비래편편삼월접
    답거성성유월와
    한장불거다언설
    취혹이류갱진배
```

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
