use eframe::egui;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

/// Toast notification helper for MAMEUIx.
pub struct NotificationManager {
    toasts: Toasts,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            toasts: Toasts::new()
                .anchor(egui::Align2::RIGHT_TOP, (16.0, 16.0))
                .direction(egui::Direction::BottomUp),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        self.toasts.show(ctx);
    }

    pub fn info(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.push(ToastKind::Info, title, message, 3.0);
    }

    pub fn success(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.push(ToastKind::Success, title, message, 3.5);
    }

    pub fn warning(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.push(ToastKind::Warning, title, message, 4.0);
    }

    pub fn error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.push(ToastKind::Error, title, message, 5.0);
    }

    fn push(
        &mut self,
        kind: ToastKind,
        title: impl Into<String>,
        message: impl Into<String>,
        seconds: f64,
    ) {
        let text = format!("{}\n{}", title.into(), message.into());
        let options = ToastOptions::default()
            .duration_in_seconds(seconds)
            .show_progress(true);
        self.toasts.add(Toast {
            kind,
            text: text.into(),
            options,
            ..Default::default()
        });
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}
