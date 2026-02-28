# bevy_workbench

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/bevy_workbench.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/bevy_workbench.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" /> <img src="https://img.shields.io/badge/Bevy-232326?style=for-the-badge&logo=bevy&logoColor=white" />

> Current Status: 🚧 Early Development (Initial version in progress)

**bevy_workbench** — A mid-level editor scaffold for Bevy, between bevy-inspector-egui and Unity/Godot editors.

| English | Simplified Chinese            |
|---------|-------------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`bevy_workbench` is an egui-based editor scaffold designed for Bevy game projects.  
It provides a structured editor layout with panels, theming, and extensibility — without imposing heavy scene management or asset pipeline opinions.

With `bevy_workbench`, you can quickly set up a development editor for your Bevy game with inspector, console, game view, and menu bar panels.  
In the future, it will support tiling layouts via `egui_tiles`, custom panel registration, and deeper Bevy ECS integration.

## Features

* **Rerun-inspired dark theme** — Professionally designed dark UI theme ported from Rerun's color system
* **Menu bar** — Top menu bar with File/Edit/View menus and Play/Stop controls
* **Panel system** — Inspector, Console, and Game View panels out of the box
* **Editor modes** — Edit / Play / Pause mode switching with keyboard shortcuts
* **Undo system** — Basic undo/redo stack infrastructure
* **Theme API** — `apply_theme_to_ctx()` for applying the dark theme to any egui context
* (Planned) **egui_tiles integration** — Flexible tiling layout for panels
* (Planned) **Custom panel registration** — Bring your own panels via trait implementation

## How to Use

1. **Add to your `Cargo.toml`**:
   ```toml
   [dependencies]
   bevy_workbench = { git = "https://github.com/Bli-AIk/bevy_workbench.git" }
   ```

2. **Basic setup**:
   ```rust
   use bevy::prelude::*;
   use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass};

   fn main() {
       App::new()
           .add_plugins(DefaultPlugins)
           .insert_resource(ClearColor(Color::BLACK))
           .add_plugins(EguiPlugin::default())
           .add_systems(Startup, |mut commands: Commands| {
               commands.spawn(Camera2d);
           })
           .add_systems(
               EguiPrimaryContextPass,
               (apply_theme, editor_ui).chain(),
           )
           .run();
   }

   fn apply_theme(mut contexts: EguiContexts, mut done: Local<bool>) {
       if *done { return; }
       let Ok(ctx) = contexts.ctx_mut() else { return };
       bevy_workbench::theme::apply_theme_to_ctx(ctx, None);
       *done = true;
   }

   fn editor_ui(mut contexts: EguiContexts) {
       let Ok(ctx) = contexts.ctx_mut() else { return };
       egui::SidePanel::left("inspector").show(ctx, |ui| {
           ui.heading("Inspector");
       });
       egui::CentralPanel::default().show(ctx, |ui| {
           ui.heading("Game View");
       });
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

| Crate                                                              | Version | Description                              |
|--------------------------------------------------------------------|---------|------------------------------------------|
| [bevy](https://crates.io/crates/bevy)                              | 0.18    | Game engine framework                    |
| [bevy_egui](https://crates.io/crates/bevy_egui)                    | 0.39    | egui integration for Bevy                |
| [bevy-inspector-egui](https://crates.io/crates/bevy-inspector-egui) | 0.36    | ECS inspector widgets                   |
| [egui](https://crates.io/crates/egui)                              | 0.33    | Immediate mode GUI library               |

## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

* Submit an **Issue** or **Pull Request**.
* Share ideas and discuss design or architecture.

## License

This project is licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
