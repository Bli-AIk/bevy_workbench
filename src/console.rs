//! Console panel: collects and displays tracing logs.

use bevy::prelude::*;

use crate::dock::WorkbenchPanel;

/// A single log entry.
#[derive(Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub target: String,
}

/// Log severity level.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    #[allow(dead_code)]
    fn color(&self) -> egui::Color32 {
        match self {
            LogLevel::Trace => egui::Color32::GRAY,
            LogLevel::Debug => egui::Color32::LIGHT_BLUE,
            LogLevel::Info => egui::Color32::WHITE,
            LogLevel::Warn => egui::Color32::YELLOW,
            LogLevel::Error => egui::Color32::RED,
        }
    }

    #[allow(dead_code)]
    fn label(&self) -> &str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// Resource holding console log state.
#[derive(Resource)]
pub struct ConsoleState {
    pub logs: Vec<LogEntry>,
    pub auto_scroll: bool,
    pub show_trace: bool,
    pub show_debug: bool,
    pub show_info: bool,
    pub show_warn: bool,
    pub show_error: bool,
    pub filter_text: String,
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self {
            logs: Vec::new(),
            auto_scroll: true,
            show_trace: false,
            show_debug: false,
            show_info: true,
            show_warn: true,
            show_error: true,
            filter_text: String::new(),
        }
    }
}

impl ConsoleState {
    /// Push a new log entry.
    pub fn push(&mut self, level: LogLevel, target: &str, message: String) {
        self.logs.push(LogEntry {
            level,
            message,
            target: target.to_string(),
        });
    }

    /// Clear all logs.
    pub fn clear(&mut self) {
        self.logs.clear();
    }
}

/// Built-in console panel.
pub struct ConsolePanel;

impl WorkbenchPanel for ConsolePanel {
    fn id(&self) -> &str {
        "workbench_console"
    }

    fn title(&self) -> String {
        "Console".to_string()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.label("Console output\n(Log collection — coming soon)");
        });
    }

    fn closable(&self) -> bool {
        true
    }
}
