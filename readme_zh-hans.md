# bevy_workbench

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/bevy_workbench.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/bevy_workbench.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" /> <img src="https://img.shields.io/badge/Bevy-232326?style=for-the-badge&logo=bevy&logoColor=white" />

> 当前状态：🚧 早期开发（初始版本开发中）

**bevy_workbench** — 面向 Bevy 的中级编辑器脚手架，定位介于 bevy-inspector-egui 和 Unity/Godot 编辑器之间。

| English                | 简体中文 |
|------------------------|------|
| [English](./readme.md) | 简体中文 |

## 简介

`bevy_workbench` 是一个基于 egui 的编辑器脚手架，专为 Bevy 游戏项目设计。  
它提供结构化的编辑器布局、面板系统和主题定制能力，同时不强制施加重量级的场景管理或资产管线。

使用 `bevy_workbench`，你可以快速为 Bevy 游戏搭建开发编辑器，包含检查器、控制台、游戏视图和菜单栏面板。

## 功能特性

* **egui_tiles 停靠布局** — 自由拖拽、重排、分割、关闭和重新打开面板
* **Rerun 风格暗色主题** — 移植自 Rerun 配色体系的暗色 UI 主题，另有 Catppuccin 和 egui 预设
* **菜单栏** — 文件/编辑/视图菜单，附带播放/暂停/停止工具栏
* **检查器** — 基于 bevy-inspector-egui 的实体层级与组件编辑器，支持基于反射快照的撤销
* **控制台** — tracing 日志桥接，支持严重级别过滤
* **游戏视图** — 渲染到纹理视口，具备焦点隔离和正确的坐标映射
* **编辑器模式** — 编辑 / 播放 / 暂停模式切换，配合 GameClock 和 GameSchedule
* **撤销/重做** — 布局变更、检查器编辑，附带撤销历史面板
* **自定义面板注册** — 实现 `WorkbenchPanel` trait 并调用 `app.register_panel()`
* **可配置快捷键** — 点击重新录制，支持添加替代绑定
* **国际化** — 内置英文 / 中文，可通过自定义 Fluent FTL 源扩展
* **主题系统** — 按模式配置主题与亮度，多种预设可选
* **布局持久化** — 以 JSON 格式保存/加载停靠布局
* **自定义字体** — 系统区域检测，可配置字体路径
* **设置面板** — UI 缩放、主题、语言和字体配置

## 使用方法

1. **添加到 `Cargo.toml`**：
   ```toml
   [dependencies]
   bevy_workbench = "0.2"
   ```

2. **基本用法**：
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

| Crate                                                               | 版本   | 描述              |
|---------------------------------------------------------------------|------|-----------------|
| [bevy](https://crates.io/crates/bevy)                               | 0.18 | 游戏引擎框架          |
| [bevy_egui](https://crates.io/crates/bevy_egui)                     | 0.39 | Bevy 的 egui 集成  |
| [bevy-inspector-egui](https://crates.io/crates/bevy-inspector-egui) | 0.36 | ECS 检查器小部件      |
| [egui](https://crates.io/crates/egui)                               | 0.33 | 即时模式 GUI 库      |
| [egui_tiles](https://crates.io/crates/egui_tiles)                   | 0.14 | 停靠瓦片布局          |
| [catppuccin-egui](https://crates.io/crates/catppuccin-egui)         | 5.7  | Catppuccin 主题预设 |

## 贡献

欢迎贡献！
无论是修复 bug、添加功能还是改进文档：

* 提交 **Issue** 或 **Pull Request**。
* 分享想法，讨论设计或架构。

## 许可证

本项目使用以下任一许可证：

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)
  或 [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) 或 [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

由你选择。
