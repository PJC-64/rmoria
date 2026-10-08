# Rmoria

**Rmoria** is a modern, memory-safe Rust port of the classic 1980s roguelike **Umoria**.

It implements true 1:1 mechanical parity with Classic Umoria, including the authentic 198x66 map scale, dynamic room generation, speed engine, canonical bestiary, and all 419 original treasure items, shops, and magic mechanics.

## Features

- **1:1 Mechanical Parity**: Every stat, monster behavior, spell, and item drop strictly follows the canonical Umoria algorithms.
- **Dual Interface Modes**: Play in the traditional terminal (`cargo run`), or use the **Native Desktop GUI**.
- **macOS GUI Version**: A fully packaged standalone macOS application with a graphical interface powered by Tauri and Svelte.

## Download for macOS

You can download the packaged macOS `.dmg` app from the [Releases](https://github.com/PJC-64/rmoria/releases) page.

1. Download `rmoria_1.0.0.dmg`.
2. Open the DMG and drag **Rmoria.app** to your Applications folder.

## Building from Source

To compile and play the standard terminal version:

```bash
cargo build --release
./target/release/rmoria
```

To build the Tauri GUI version:

```bash
cd gui
npm install
cd ..
npx @tauri-apps/cli build
```

## Contributing

Rmoria is completely open-source. Feel free to submit issues or pull requests.
