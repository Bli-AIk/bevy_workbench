# bevy_workbench

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/bevy_workbench.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/bevy_workbench.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" /> <img src="https://img.shields.io/badge/Bevy-232326?style=for-the-badge&logo=bevy&logoColor=white" />

> Current Status: 🚧 Early Development (Initial version in progress)

**bevy_workbench** — A mid-level editor scaffold for Bevy, between bevy-inspector-egui and Unity/Godot editors.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`bevy_workbench` is an egui-based editor scaffold designed for Bevy game projects.  
It provides a structured editor layout with panels, theming, and extensibility — without imposing heavy scene management
or asset pipeline opinions.

With `bevy_workbench`, you can quickly set up a development editor for your Bevy game with inspector, console, game
view, and menu bar panels.

## Features

* **egui_tiles dock layout** — Drag, rearrange, split, and close/reopen panels freely
* **Rerun-inspired dark theme** — Dark UI theme ported from Rerun's color system, plus Catppuccin and egui presets
* **Menu bar** — File/Edit/View menus with Play/Pause/Stop toolbar
* **Inspector** — Entity hierarchy and component editor powered by bevy-inspector-egui, with undo support via reflection
  snapshots
* **Console** — Tracing log bridge with severity filtering
* **Game View** — Render-to-texture viewport with focus isolation and correct coordinate mapping
* **Editor modes** — Edit / Play / Pause mode switching with GameClock and GameSchedule
* **Undo/Redo** — Layout changes, inspector edits, with undo history panel
* **Custom panel registration** — Implement `WorkbenchPanel` trait and call `app.register_panel()`
* **Configurable keybindings** — Click to re-record, add alternative bindings
* **i18n** — English / 中文 built-in, extensible with custom Fluent FTL sources
* **Theme system** — Per-mode themes with brightness control, multiple presets
* **Layout persistence** — Save/load dock layouts as JSON
* **Custom font support** — System locale detection with configurable font path
* **Settings panel** — UI scale, theme, locale, and font configuration

## How to Use

1. **Add to your `Cargo.toml`**:
   ```toml
   [dependencies]
   bevy_workbench = "0.1"
   ```

2. **Basic setup**:
   ```rust
   use bevy::prelude::*;
   use bevy_workbench::prelude::*;
   use bevy_workbench::console::console_log_layer;

   fn main() {
       App::new()
           .add_plugins(
               DefaultPlugins.set(bevy::log::LogPlugin {
                   custom_layer: console_log_layer,
                   ..default()
               }),
           )
           .insert_resource(ClearColor(Color::BLACK))
           .add_plugins(WorkbenchPlugin::default())
           .add_plugins(GameViewPlugin)
           .add_systems(Startup, |mut commands: Commands| {
               commands.spawn(Camera2d);
           })
           .run();
   }
   ```

3. **Run the example**:
   ```bash
   cargo run -p bevy_workbench --example minimal
   ```

## How to Build

### Prerequisites

* Rust 1.85 or later
* Bevy 0.18 compatible system dependencies:
  ```bash
  # Linux (Ubuntu/Debian)
  sudo apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev \
      libwayland-dev libxkbcommon-dev
  ```

### Build Steps

1. **Clone the repository**:
   ```bash
   git clone https://github.com/Bli-AIk/bevy_workbench.git
   cd bevy_workbench
   ```

2. **Build the project**:
   ```bash
   cargo build
   ```

3. **Run tests**:
   ```bash
   cargo test
   ```

4. **Run examples**:
   ```bash
   cargo run --example minimal
   ```

## Dependencies

This project uses the following crates:

| Crate                                                               | Version | Description                |
|---------------------------------------------------------------------|---------|----------------------------|
| [bevy](https://crates.io/crates/bevy)                               | 0.18    | Game engine framework      |
| [bevy_egui](https://crates.io/crates/bevy_egui)                     | 0.39    | egui integration for Bevy  |
| [bevy-inspector-egui](https://crates.io/crates/bevy-inspector-egui) | 0.36    | ECS inspector widgets      |
| [egui](https://crates.io/crates/egui)                               | 0.33    | Immediate mode GUI library |
| [egui_tiles](https://crates.io/crates/egui_tiles)                   | 0.14    | Tiling dock layout         |
| [catppuccin-egui](https://crates.io/crates/catppuccin-egui)         | 5.7     | Catppuccin theme presets   |

## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

* Submit an **Issue** or **Pull Request**.
* Share ideas and discuss design or architecture.

## License

This project is licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)
  or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
