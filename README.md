
---

# findr

A fast, terminal-based fuzzy file finder written in Rust.

---

## Features

* Recursively collects files from a given directory (with optional hidden file toggle)
* Fuzzy search powered by [nucleo](https://github.com/dbrgn/nucleo)
* Interactive terminal UI with search input, results list, and file preview
* Syntax-highlighted file previews using [syntect](https://github.com/trishume/syntect)
* Smooth keyboard navigation with arrow keys, Tab to switch focus, Enter to open files, and Esc to quit
* Parallel file collection with `rayon` for fast startup

---

## Usage

Build the project with Cargo:

```bash
cargo build --release
```

Run `findr` with an optional path argument (defaults to current directory):

```bash
./target/release/findr [path]
```

### Controls

* **Typing**: Enter your fuzzy search query
* **Backspace**: Remove last character in query
* **Tab**: Toggle focus between search bar and results list
* **Up/Down arrows**: Navigate the results list
* **Enter**: Open selected file in `$EDITOR`
* **Esc**: Exit the application

---

## Configuration

* By default, hidden files are included (`toggle_hidden = true` in file collection).
* The editor used to open files respects your `$EDITOR` environment variable.
* Syntax highlighting uses the `base16-ocean.dark` theme.
* **Config management is still WIP.**

---

## Dependencies

* [clap](https://crates.io/crates/clap) for command-line argument parsing
* [ignore](https://crates.io/crates/ignore) and [rayon](https://crates.io/crates/rayon) for efficient file walking and parallelism
* [crossterm](https://crates.io/crates/crossterm) and [ratatui](https://crates.io/crates/ratatui) for terminal UI rendering
* [nucleo](https://crates.io/crates/nucleo) for fuzzy matching
* [syntect](https://crates.io/crates/syntect) for syntax highlighting

---

## How it works

1. **File Collection**
   The app walks the directory tree starting from the given path, optionally including hidden files, collecting all file paths in parallel.

2. **Fuzzy Matching**
   The fuzzy matcher scores file names against the current search query and sorts them by relevance.

3. **UI Rendering**
   The terminal UI is split into three parts: search bar (top), results list (left), and syntax-highlighted file preview (right).

4. **Interaction**
   Keyboard input is handled to update the search query, move selection, open files, and switch focus.

---

## Example

```bash
./findr ~/projects
```

Start typing to filter files, navigate the results, and press Enter to open a file in your editor.

---

## License

MIT License

---
