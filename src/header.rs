use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Span,
    widgets::Widget,
};

/// Renders a header with consistent styling and positioning
pub fn render_header(area: Rect, buf: &mut Buffer, title: &str, color: ratatui::style::Color) {
    let mut area = area;
    area.x += 9;
    area.width -= 9;
    Span::from(title).style(Style::default().fg(color))
        .render(area, buf);
}