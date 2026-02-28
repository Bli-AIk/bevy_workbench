# bevy_workbench

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/bevy_workbench.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/bevy_workbench.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" /> <img src="https://img.shields.io/badge/Bevy-232326?style=for-the-badge&logo=bevy&logoColor=white" />

> 当前状态：🚧 早期开发（初始版本开发中）

**bevy_workbench** — 面向 Bevy 的中级编辑器脚手架，定位介于 bevy-inspector-egui 和 Unity/Godot 编辑器之间。

| English                | 简体中文 |
|------------------------|----------|
| [English](./readme.md) | 简体中文 |

## 简介

`bevy_workbench` 是一个基于 egui 的编辑器脚手架，专为 Bevy 游戏项目设计。  
它提供结构化的编辑器布局、面板系统和主题定制能力，同时不强制施加重量级的场景管理或资产管线。

使用 `bevy_workbench`，你可以快速为 Bevy 游戏搭建开发编辑器，包含检查器、控制台、游戏视图和菜单栏面板。  
未来将支持通过 `egui_tiles` 实现灵活的瓦片布局、自定义面板注册，以及更深层次的 Bevy ECS 集成。

## 功能特性

* **Rerun 风格暗色主题** — 专业设计的暗色 UI 主题，移植自 Rerun 的配色体系
* **菜单栏** — 顶部菜单栏，包含文件/编辑/视图菜单和播放/停止控制
* **面板系统** — 内置检查器、控制台和游戏视图面板
* **编辑器模式** — 编辑 / 播放 / 暂停模式切换，支持键盘快捷键
* **撤销系统** — 基础撤销/重做栈基础设施
* **主题 API** — `apply_theme_to_ctx()` 可将暗色主题应用到任意 egui 上下文
* （计划中）**egui_tiles 集成** — 灵活的面板瓦片布局
* （计划中）**自定义面板注册** — 通过 trait 实现自定义面板

## 使用方法

1. **添加到 `Cargo.toml`**：
   ```toml
   [dependencies]
   bevy_workbench = { git = "https://github.com/Bli-AIk/bevy_workbench.git" }
   ```

2. **基本用法**：
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

3. **运行示例**：
   ```bash
   cargo run -p bevy_workbench --example minimal
   ```

## 构建方法

### 前置要求

* Rust 1.85 或更高版本
* Bevy 0.18 兼容的系统依赖：
  ```bash
  # Linux (Ubuntu/Debian)
  sudo apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev \
      libwayland-dev libxkbcommon-dev
  ```

### 构建步骤

1. **克隆仓库**：
   ```bash
   git clone https://github.com/Bli-AIk/bevy_workbench.git
   cd bevy_workbench
   ```

2. **构建项目**：
   ```bash
   cargo build
   ```

3. **运行测试**：
   ```bash
   cargo test
   ```

4. **运行示例**：
   ```bash
   cargo run --example minimal
   ```

## 依赖

本项目使用以下 crate：

| Crate                                                              | 版本    | 描述                    |
|--------------------------------------------------------------------|---------|-------------------------|
| [bevy](https://crates.io/crates/bevy)                              | 0.18    | 游戏引擎框架            |
| [bevy_egui](https://crates.io/crates/bevy_egui)                    | 0.39    | Bevy 的 egui 集成       |
| [bevy-inspector-egui](https://crates.io/crates/bevy-inspector-egui) | 0.36    | ECS 检查器小部件        |
| [egui](https://crates.io/crates/egui)                              | 0.33    | 即时模式 GUI 库         |

## 贡献

欢迎贡献！
无论是修复 bug、添加功能还是改进文档：

* 提交 **Issue** 或 **Pull Request**。
* 分享想法，讨论设计或架构。

## 许可证

本项目使用以下任一许可证：

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) 或 [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) 或 [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

由你选择。
