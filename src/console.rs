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
    fn color(&self) -> egui::Color32 {
        match self {
            LogLevel::Trace => egui::Color32::GRAY,
            LogLevel::Debug => egui::Color32::LIGHT_BLUE,
            LogLevel::Info => egui::Color32::WHITE,
            LogLevel::Warn => egui::Color32::YELLOW,
            LogLevel::Error => egui::Color32::RED,
        }
    }

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

    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        let Some(mut console) = world.get_resource_mut::<ConsoleState>() else {
            ui.label("ConsoleState resource not found");
            return;
        };

        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                console.clear();
            }
            ui.checkbox(&mut console.auto_scroll, "Auto-scroll");
            ui.separator();
            ui.checkbox(&mut console.show_info, "Info");
            ui.checkbox(&mut console.show_warn, "Warn");
            ui.checkbox(&mut console.show_error, "Error");
            ui.checkbox(&mut console.show_debug, "Debug");
        });

        ui.separator();

        // Filter
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut console.filter_text);
        });

        ui.separator();

        // Log entries
        let filter = console.filter_text.clone();
        let auto_scroll = console.auto_scroll;
        let show_info = console.show_info;
        let show_warn = console.show_warn;
        let show_error = console.show_error;
        let show_debug = console.show_debug;
        let show_trace = console.show_trace;

        let filtered: Vec<_> = console
            .logs
            .iter()
            .filter(|log| {
                let level_ok = match log.level {
                    LogLevel::Trace => show_trace,
                    LogLevel::Debug => show_debug,
                    LogLevel::Info => show_info,
                    LogLevel::Warn => show_warn,
                    LogLevel::Error => show_error,
                };
                let text_ok = filter.is_empty()
                    || log.message.contains(&filter)
                    || log.target.contains(&filter);
                level_ok && text_ok
            })
            .cloned()
            .collect();

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(auto_scroll)
            .show(ui, |ui| {
                for entry in &filtered {
                    ui.horizontal(|ui| {
                        ui.colored_label(entry.level.color(), entry.level.label());
                        ui.label(&entry.message);
                    });
                }
            });
    }

    fn closable(&self) -> bool {
        true
    }
}
