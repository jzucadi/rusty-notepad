# Notepadr

<img width="903" height="663" alt="Screenshot 2025-12-09 at 10 00 23 PM" src="https://github.com/user-attachments/assets/a073a56f-de26-47c3-9e34-d1d1a9b01a66" />

A simple, lightweight text editor built with Rust and [egui](https://github.com/emilk/egui). Features the beautiful [Catppuccin Mocha](https://github.com/catppuccin/catppuccin) color scheme.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

- Clean, distraction-free text editing
- Catppuccin Mocha dark theme
- Native file dialogs for Open/Save
- Unsaved changes protection
- Cross-platform (Windows, macOS, Linux)

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New file |
| `Ctrl+O` | Open file |
| `Ctrl+S` | Save |
| `Ctrl+Shift+S` | Save As |

> On macOS, use `Cmd` instead of `Ctrl`

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/jzucadi/rusty-notepad.git
cd rusty-notepad

# Build and run
cargo run --release
```

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/yourusername/rusty-notepad/releases) page.

## Building

### Requirements

- Rust 1.70+ (install via [rustup](https://rustup.rs/))

### Build Commands

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

The compiled binary will be in `target/release/rusty-notepad` (or `rusty-notepad.exe` on Windows).

## Dependencies

- [eframe](https://crates.io/crates/eframe) - egui framework for native apps
- [rfd](https://crates.io/crates/rfd) - Native file dialogs

## License

MIT License - see [LICENSE](LICENSE) for details.
