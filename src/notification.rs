use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::config;

#[derive(Default)]
pub enum NotificationType {
    Success,
    ErrorLow,
    ErrorMid,
    ErrorHigh,
    #[default]
    Message,
}

#[derive(Default)]
pub struct NotificationWidget {
    notif_text: String,
    notif_type: NotificationType,
}

impl NotificationType {
    pub fn get_text(&self) -> String {
        match self {
            Self::Success => String::from("Success"),
            Self::Message => String::from("Message"),
            Self::ErrorHigh | Self::ErrorMid | Self::ErrorLow => String::from("Error"),
        }
    }

    pub fn get_style(&self) -> Style {
        match self {
            Self::Message => Style::default()
                .fg(config::THEME.foreground)
                .bg(config::THEME.background),
            Self::Success => Style::default()
                .fg(config::THEME.success)
                .bg(config::THEME.background),
            Self::ErrorLow | NotificationType::ErrorMid | NotificationType::ErrorHigh => {
                Style::default()
                    .fg(config::THEME.alert)
                    .bg(config::THEME.background)
            }
        }
    }
}

impl NotificationWidget {
    pub fn text(mut self, text: &str) -> Self {
        self.notif_text = String::from(text);
        self
    }

    pub fn notif_type(mut self, level: NotificationType) -> Self {
        self.notif_type = level;
        self
    }

    pub fn generate_body(&self) -> Vec<Spans<'_>> {
        vec![
            Spans::from(vec![Span::raw(self.notif_text.clone())]),
            Spans::from(vec![Span::raw("[press space to dismiss]")]),
        ]
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>) {
        let message = Paragraph::new(self.generate_body())
            .block(
                Block::default()
                    .title(self.notif_type.get_text())
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL),
            )
            .style(self.notif_type.get_style())
            .alignment(tui::layout::Alignment::Center)
            .wrap(Wrap { trim: false });

        let render_area = center_rect(20, 12, f.size());

        f.render_widget(Clear, render_area);
        f.render_widget(message, render_area);
    }
}

fn center_rect(pct_x: u16, pct_y: u16, area: Rect) -> Rect {
    let layout = Layout::default()
        .direction(tui::layout::Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - pct_x) / 2),
                Constraint::Percentage(pct_x),
                Constraint::Percentage((100 - pct_x) / 2),
            ]
            .as_ref(),
        )
        .split(area);

    Layout::default()
        .direction(tui::layout::Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - pct_y) / 2),
                Constraint::Percentage(pct_y),
                Constraint::Percentage((100 - pct_y) / 2),
            ]
            .as_ref(),
        )
        .split(layout[1])[1]
}
