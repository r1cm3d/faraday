//! Boolean status cell renderer for table views.
use ratatui::{
    style::{Color, Style},
    text::{Span, Spans},
};

#[allow(dead_code)]
pub fn bool_cell(label: &str, active: Option<bool>) -> Spans<'static> {
    let (text, color) = match active {
        Some(true) => (format!("{}:ON ", label), Color::Green),
        Some(false) => (format!("{}:OFF", label), Color::Gray),
        None => (format!("{}:-- ", label), Color::DarkGray),
    };
    Spans::from(vec![Span::styled(text, Style::default().fg(color))])
}
