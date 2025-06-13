# Findr

A simple terminal user interface (TUI) built with Rust using [ratatui](https://github.com/tui-rs-revival/ratatui) and [crossterm](https://crates.io/crates/crossterm).

---

## Overview

This project demonstrates a minimal TUI setup in Rust. It opens a terminal interface with a bordered box displaying a “Hello, world!” message.

The goal is to build a file search tool with fuzzy matching and a TUI interface, but this is just the starting point.

---

## Usage

1. **Clone the repository:**

   ```bash
   git clone https://github.com/yourusername/yourproject.git
   cd yourproject
   ```

2. **Build and run:**

   ```bash
   cargo run
   ```

3. **Controls:**

   - Press `q` or `Esc` to exit the program.

---

## Project Structure

- `src/main.rs`: Entry point that starts the TUI application.
- `src/ui.rs`: Contains the UI setup and main event loop for the TUI.

---

## Dependencies

- [`ratatui`](https://crates.io/crates/ratatui) for building terminal UI.
- [`crossterm`](https://crates.io/crates/crossterm) for terminal control and input handling.

---

## Next Steps

- Implement file listing and navigation.
- Add fuzzy search functionality.
- Support opening files/directories from the UI.

---

## License

[MIT](LICENSE) or your preferred license.
