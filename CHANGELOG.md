# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.2](https://github.com/Bli-AIk/bevy_workbench/compare/v0.3.1...v0.3.2) - 2026-03-09

### Miscellaneous Tasks

- *(lint)* improve #[expect] reason detection in tokei scripts

### Refactor

- *(dock)* simplify panel lookup with conditional let

## [0.3.1](https://github.com/Bli-AIk/bevy_workbench/compare/v0.3.0...v0.3.1) - 2026-03-07

### Added

- *(dock)* add window menu hidden panel support

### Documentation

- *(bevy_workbench)* update readme version references to 0.3

## [0.3.0](https://github.com/Bli-AIk/bevy_workbench/compare/v0.2.0...v0.3.0) - 2026-03-07

### Added

- *(workbench)* add extensible settings panel with custom sections
- *(dock)* add show_in_window_menu method to WorkbenchPanel trait

### Documentation

- *(bevy_workbench)* update readme version references to 0.2

### Refactor

- *(menu_bar)* extract UI functions to reduce nesting
- *(bevy_workbench)* conditionally compile native-only features
- *(bevy_workbench)* conditionally compile native-only features

## [0.2.0](https://github.com/Bli-AIk/bevy_workbench/compare/v0.1.0...v0.2.0) - 2026-03-07

### Added

- *(control_link)* implement drag-and-drop UI for entity linking
- *(ci)* add tokei lint checks to crate workflows
- *(i18n)* add t_args method for localized strings with arguments
- *(game_view)* add toolbar toggle support for external plugins
- *(game_view)* add support for external game camera hijacking
- add configurable toolbar, game view zoom, and menu bar extensions ([#4](https://github.com/Bli-AIk/bevy_workbench/pull/4))

### Fixed

- *(inspector)* wrap inspector UI in catch_unwind to handle reflection panics

### Miscellaneous Tasks

- add clippy configuration
- *(crates)* add readme and repository fields to Cargo.toml files
- *(ci)* exclude passing tests from nextest run
- *(ci)* replace cargo test with cargo-nextest

### Refactor

- extract UI functions to reduce nesting depth
- *(dock)* sort center panes with game_view last

## [0.1.0](https://github.com/Bli-AIk/bevy_workbench/compare/v0.0.0...v0.1.0) - 2026-02-28

### Miscellaneous Tasks

- *(bevy_workbench)* bump version to 0.1.0
