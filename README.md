# mdview

A fast, feature-rich terminal Markdown viewer built with Rust.

mdview renders Markdown files directly in your terminal with syntax highlighting, mouse support, and live file reloading.

## Features

- **Syntax highlighting** for fenced code blocks (powered by [syntect](https://github.com/trishume/syntect))
- **Full Markdown support**: headings, bold, italic, strikethrough, blockquotes, lists, task lists, tables, footnotes, math, horizontal rules, and YAML front matter
- **Interactive navigation**: vim-style keybindings, mouse scrolling, and link opening
- **Search** with match highlighting and navigation
- **Built-in file picker** for browsing and opening Markdown files
- **Live reload**: automatically re-renders when the file changes on disk
- **Automatic dark/light theme** detection
- **Piped input**: read Markdown from stdin

## Installation

### From crates.io

```
cargo install mdview
```

### From source

```
git clone https://github.com/dandan2611/MDView.git
cd md-tui
cargo install --path .
```

### Pre-built binaries

Download binaries for Linux, macOS, and Windows from the [Releases](https://github.com/dandan2611/MDView/releases) page.

## Usage

```
# View a file
mdview README.md

# Open the file picker in the current directory
mdview

# Read from stdin
cat README.md | mdview
```

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` | Scroll down |
| `k` / `Up` | Scroll up |
| `d` / `PageDown` | Scroll down half page |
| `u` / `PageUp` | Scroll up half page |
| `g` / `Home` | Go to top |
| `G` / `End` | Go to bottom |

### Search

| Key | Action |
|-----|--------|
| `/` | Open search |
| `n` | Next match |
| `N` | Previous match |
| `Enter` | Confirm search |
| `Esc` | Cancel search |

### Links

| Key | Action |
|-----|--------|
| `Tab` | Focus next link |
| `Shift+Tab` | Focus previous link |
| `Enter` | Open focused link in browser |

### Other

| Key | Action |
|-----|--------|
| `w` | Toggle table text wrapping |
| `q` / `Esc` | Quit (or return to file picker) |

Mouse scrolling and click-to-open links are also supported.

## Markdown Support

| Feature | Syntax |
|---------|--------|
| Headings | `# H1` through `###### H6` |
| Emphasis | `**bold**`, `*italic*`, `~~strikethrough~~` |
| Code | `` `inline` `` and fenced blocks with language |
| Blockquotes | `> text` (nested supported) |
| Lists | `- unordered`, `1. ordered` (nested supported) |
| Task lists | `- [x] done`, `- [ ] todo` |
| Tables | Pipe tables with alignment |
| Links | `[text](url)` |
| Images | `![alt](url)` |
| Footnotes | `[^1]` references and definitions |
| Math | `$inline$` and `$$display$$` |
| Horizontal rules | `---` |
| YAML front matter | `---` delimited metadata (hidden) |
| Smart punctuation | Curly quotes, em-dashes, ellipsis |

## Theming

mdview automatically detects your terminal's dark or light background using the `COLORFGBG` environment variable and adjusts colors accordingly. It defaults to dark mode if detection fails.

Code blocks use the **base16-ocean** color scheme (dark or light variant).

## License

[GPL-3.0](LICENSE)
