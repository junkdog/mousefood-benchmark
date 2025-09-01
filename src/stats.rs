use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::gpio::{Gpio0, Input, PinDriver};
use esp_idf_svc::hal::task::notification::Notification;
use esp_idf_svc::sys::{heap_caps_get_free_size, heap_caps_get_total_size, MALLOC_CAP_8BIT};
use ratatui::prelude::{Backend, Color, Terminal};
use ratatui::widgets::BorderType;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Gauge, Padding, Paragraph, Widget},
};
use std::marker::PhantomData;
use compact_str::format_compact;
use ratatui::layout::{Margin, Size};
use crate::catpuccin::CATPPUCCIN;
use crate::fps::FpsWidget;
use crate::header::render_header;

#[derive(Debug)]
pub struct Stats<B: Backend> {
    fps_widget: FpsWidget,
    _marker: PhantomData<B>,
}

impl<B: Backend> Stats<B> {
    pub fn new() -> Self {
        Self {
            fps_widget: FpsWidget::new().with_label(true).with_style(CATPPUCCIN.green),
            _marker: PhantomData,
        }
    }

    pub fn run(
        mut self,
        terminal: &mut Terminal<B>,
        notification: &mut Notification,
        button: &mut PinDriver<Gpio0, Input>,
    ) -> std::io::Result<()> {
        button.enable_interrupt().unwrap();
        loop {
            if notification.wait(delay::NON_BLOCK).is_some() {
                return Ok(());
            }
            self.fps_widget.fps.tick();
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))
                .unwrap();
        }
    }
}

impl<B: Backend> Default for Stats<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Backend> Widget for &Stats<B> {
    #[allow(clippy::similar_names)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ]).split(area);

        self.render_header(layout[0], buf);
        self.render_content(layout[1], buf);
        self.render_footer(layout[2], buf);
    }
}

impl<B: Backend> Stats<B> {
    fn get_memory_info(&self) -> (usize, usize) {
        unsafe {
            let free = heap_caps_get_free_size(MALLOC_CAP_8BIT) as usize;
            let total = heap_caps_get_total_size(MALLOC_CAP_8BIT) as usize;
            (free, total)
        }
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        render_header(area, buf, "Mousefood Benchmark", CATPPUCCIN.mauve);
    }

    fn render_content(&self, area: Rect, buf: &mut Buffer) {
        let inner = area.inner(Margin::new(6, 1));
        
        let terminal_size = format!("Terminal: {}x{}", area.width, area.height);
        let (free_memory, total_memory) = self.get_memory_info();
        let used_memory = total_memory - free_memory;
        let memory_usage = format!("Memory: {}KB used / {}KB total", used_memory / 1024, total_memory / 1024);
        let free_memory_str = format!("Free: {}KB", free_memory / 1024);
        
        let content = vec![
            Line::from(vec![
                Span::styled("System Status", Style::default().fg(CATPPUCCIN.yellow).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("• ", Style::default().fg(CATPPUCCIN.green)),
                Span::styled("Terminal: ", Style::default().fg(CATPPUCCIN.blue).bold()),
                Span::styled(format_compact!("{}x{}", area.width, area.height), Style::default().fg(CATPPUCCIN.text)),
            ]),
            Line::from(vec![
                Span::styled("• ", Style::default().fg(CATPPUCCIN.green)),
                Span::styled("Memory: ", Style::default().fg(CATPPUCCIN.blue).bold()),
                Span::styled(format_compact!("{}KB used / {}KB total", used_memory / 1024, total_memory / 1024), Style::default().fg(CATPPUCCIN.text)),
            ]),
            Line::from(vec![
                Span::styled("• ", Style::default().fg(CATPPUCCIN.green)),
                Span::styled("Free: ", Style::default().fg(CATPPUCCIN.blue).bold()),
                Span::styled(format_compact!("{}KB", free_memory / 1024), Style::default().fg(CATPPUCCIN.text)),
            ]),
            Line::from(vec![
                Span::styled("• ", Style::default().fg(CATPPUCCIN.green)),
                Span::styled("Status: ", Style::default().fg(CATPPUCCIN.blue).bold()),
                Span::styled("Ready", Style::default().fg(CATPPUCCIN.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Benchmark Controls", Style::default().fg(CATPPUCCIN.yellow).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("• ", Style::default().fg(CATPPUCCIN.blue)),
                Span::styled("Press S1 to start benchmark", Style::default().fg(CATPPUCCIN.subtext1)),
            ]),
            Line::from(vec![
                Span::styled("• ", Style::default().fg(CATPPUCCIN.blue)),
                Span::styled("Press and hold to exit", Style::default().fg(CATPPUCCIN.subtext1)),
            ]),
        ];

        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(CATPPUCCIN.text));
        
        paragraph.render(inner, buf);
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let fps_area = area.inner(Margin::new(8, 0));
        self.fps_widget.render(fps_area, buf);
    }
}


