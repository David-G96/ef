// use derive_setters::Setters;
// use lipsum::lipsum;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

#[derive(Debug, Default)]
pub struct Popup<'a> {
   title: Line<'a>,
    content: Text<'a>,
    border_style: Style,
    title_style: Style,
    style: Style,
}

impl<'a> Popup<'a> {
    pub fn new(title: Line<'a>, content: Text<'a>, border_style: Style, title_style: Style, style: Style) -> Self {
        Self { title, content, border_style, title_style, style }
    }
}

impl Widget for Popup<'_> {
 fn render(self, area: Rect, buf: &mut Buffer) {
        // ensure that all cells under the popup are cleared to avoid leaking content
        Clear.render(area, buf);
        let block = Block::new()
            .title(self.title)
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_style(self.border_style);
        Paragraph::new(self.content)
            .wrap(Wrap { trim: true })
            .style(self.style)
            .block(block)
            .render(area, buf);
    }
}
