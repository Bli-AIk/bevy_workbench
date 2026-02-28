//! Console panel: collects and displays tracing logs.

use bevy::prelude::*;
use std::sync::{Arc, Mutex, mpsc};

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

    fn icon(&self) -> &str {
        match self {
            LogLevel::Trace | LogLevel::Debug => "🔍",
            LogLevel::Info => "ℹ",
            LogLevel::Warn => "⚠",
            LogLevel::Error => "❌",
        }
    }
}

/// Thread-safe sender for log entries (used by the tracing layer).
pub type LogSender = mpsc::Sender<LogEntry>;
/// Receiver for log entries (drained each frame by ConsoleState).
pub type LogReceiver = mpsc::Receiver<LogEntry>;

/// Create a log channel for capturing tracing output.
pub fn log_channel() -> (LogSender, LogReceiver) {
    mpsc::channel()
}

/// Resource holding console log state.
#[derive(Resource)]
pub struct ConsoleState {
    pub logs: Vec<LogEntry>,
    pub auto_scroll: bool,
    pub show_info: bool,
    pub show_warn: bool,
    pub show_error: bool,
    pub filter_text: String,
    /// Whether to auto-clear logs when entering Play mode.
    pub auto_clear_on_play: bool,
    /// Receiver end of the log channel (drained each frame).
    receiver: Option<Arc<Mutex<LogReceiver>>>,
    /// Counts by level for badge display.
    info_count: usize,
    warn_count: usize,
    error_count: usize,
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self {
            logs: Vec::new(),
            auto_scroll: true,
            show_info: true,
            show_warn: true,
            show_error: true,
            filter_text: String::new(),
            auto_clear_on_play: false,
            receiver: None,
            info_count: 0,
            warn_count: 0,
            error_count: 0,
        }
    }
}

impl ConsoleState {
    /// Create with a log receiver for tracing integration.
    pub fn with_receiver(receiver: LogReceiver) -> Self {
        Self {
            receiver: Some(Arc::new(Mutex::new(receiver))),
            ..Default::default()
        }
    }

    /// Push a new log entry.
    pub fn push(&mut self, level: LogLevel, target: &str, message: String) {
        match level {
            LogLevel::Info => self.info_count += 1,
            LogLevel::Warn => self.warn_count += 1,
            LogLevel::Error => self.error_count += 1,
            _ => {}
        }
        self.logs.push(LogEntry {
            level,
            message,
            target: target.to_string(),
        });
    }

    /// Drain any pending log entries from the channel.
    pub fn drain_channel(&mut self) {
        let Some(receiver) = &self.receiver else {
            return;
        };
        let Ok(rx) = receiver.lock() else { return };
        while let Ok(entry) = rx.try_recv() {
            match entry.level {
                LogLevel::Info => self.info_count += 1,
                LogLevel::Warn => self.warn_count += 1,
                LogLevel::Error => self.error_count += 1,
                _ => {}
            }
            self.logs.push(entry);
        }
    }

    /// Clear all logs.
    pub fn clear(&mut self) {
        self.logs.clear();
        self.info_count = 0;
        self.warn_count = 0;
        self.error_count = 0;
    }
}

/// System that drains the log channel each frame.
pub fn console_drain_system(mut state: ResMut<ConsoleState>) {
    state.drain_channel();
}

/// System that auto-clears console when entering Play mode.
pub fn console_auto_clear_system(mut state: ResMut<ConsoleState>) {
    if state.auto_clear_on_play {
        state.clear();
    }
}

/// Built-in console panel.
#[derive(Default)]
pub struct ConsolePanel;

impl WorkbenchPanel for ConsolePanel {
    fn id(&self) -> &str {
        "workbench_console"
    }

    fn title(&self) -> String {
        "Console".to_string()
    }

    fn ui(&mut self, _ui: &mut egui::Ui) {}

    fn ui_world(&mut self, ui: &mut egui::Ui, world: &mut World) {
        let mut console = world.remove_resource::<ConsoleState>().unwrap_or_default();

        // Toolbar row
        ui.horizontal(|ui| {
            // Clear button
            if ui.button("🗑 Clear").clicked() {
                console.clear();
            }

            ui.separator();

            // Level filter toggles with counts
            let info_label = format!("ℹ {} ({})", "Info", console.info_count);
            let warn_label = format!("⚠ {} ({})", "Warn", console.warn_count);
            let error_label = format!("❌ {} ({})", "Error", console.error_count);

            toggle_button(ui, &info_label, &mut console.show_info);
            toggle_button(ui, &warn_label, &mut console.show_warn);
            toggle_button(ui, &error_label, &mut console.show_error);

            ui.separator();

            // Auto-clear toggle
            ui.checkbox(&mut console.auto_clear_on_play, "Auto-clear on Play");

            ui.separator();

            // Search field
            ui.label("🔍");
            ui.add(
                egui::TextEdit::singleline(&mut console.filter_text)
                    .desired_width(150.0)
                    .hint_text("Filter..."),
            );
        });

        ui.separator();

        // Log area
        let filter_lower = console.filter_text.to_lowercase();
        let scroll = egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(console.auto_scroll);

        scroll.show(ui, |ui| {
            for entry in &console.logs {
                // Level filter
                let show = match entry.level {
                    LogLevel::Trace | LogLevel::Debug => false,
                    LogLevel::Info => console.show_info,
                    LogLevel::Warn => console.show_warn,
                    LogLevel::Error => console.show_error,
                };
                if !show {
                    continue;
                }

                // Text filter
                if !filter_lower.is_empty()
                    && !entry.message.to_lowercase().contains(&filter_lower)
                    && !entry.target.to_lowercase().contains(&filter_lower)
                {
                    continue;
                }

                ui.horizontal(|ui| {
                    let color = entry.level.color();
                    ui.colored_label(color, entry.level.icon());
                    ui.colored_label(egui::Color32::DARK_GRAY, format!("[{}]", entry.target));
                    ui.colored_label(color, &entry.message);
                });
            }
        });

        world.insert_resource(console);
    }

    fn needs_world(&self) -> bool {
        true
    }

    fn closable(&self) -> bool {
        true
    }
}

/// Helper to draw a toggle button that changes appearance based on state.
fn toggle_button(ui: &mut egui::Ui, label: &str, value: &mut bool) {
    let text = if *value {
        egui::RichText::new(label)
    } else {
        egui::RichText::new(label).weak().strikethrough()
    };
    if ui.button(text).clicked() {
        *value = !*value;
    }
}
