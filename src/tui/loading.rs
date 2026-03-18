use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge};

use super::Theme;

pub struct LoadingView {
    pub message: String,
    pub progress: f64, // 0.0 to 1.0, use -1.0 for indeterminate
    tick: usize,
}

impl LoadingView {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            progress: -1.0,
            tick: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick += 1;
    }

    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    pub fn set_message(&mut self, message: &str) {
        self.message = message.to_string();
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let spinner_chars = ["|", "/", "-", "\\"];
        let spinner = spinner_chars[self.tick % spinner_chars.len()];

        if self.progress < 0.0 {
            // Indeterminate spinner
            let label = format!("{} {}", spinner, self.message);
            let line = Line::from(vec![
                Span::styled(format!(" {} ", spinner), theme.loading),
                Span::styled(&self.message, theme.loading),
            ]);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border);
            let paragraph = ratatui::widgets::Paragraph::new(line)
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
        } else {
            // Determinate progress bar
            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(format!(" {} ", self.message))
                        .borders(Borders::ALL)
                        .border_style(theme.border),
                )
                .gauge_style(theme.loading)
                .ratio(self.progress)
                .label(format!("{:.0}%", self.progress * 100.0));
            frame.render_widget(gauge, area);
        }
    }
}
