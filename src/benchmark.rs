use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::gpio::{Gpio0, Input, PinDriver};
use esp_idf_svc::hal::task::notification::Notification;
use ratatui::prelude::{Backend, Terminal};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget, Wrap},
};
use std::marker::PhantomData;
use std::thread;
use std::time::Duration;
use compact_str::format_compact;
use ratatui::layout::Margin;
use crate::catpuccin::CATPPUCCIN;
use crate::lorem::LOREM_IPSUM;
use crate::fps::FpsWidget;
use crate::header::render_header;

#[derive(Debug)]
pub struct Benchmark<B: Backend> {
    frame_count: u32,
    style_mode: usize,
    spans_cache: [Vec<Span<'static>>; 4],
    fps_widget: FpsWidget,
    _marker: PhantomData<B>,
}

impl<B: Backend> Benchmark<B> {
    pub fn new() -> Self {
        let spans_cache = [
            Self::generate_spans_for_mode(0),
            Self::generate_spans_for_mode(1),
            Self::generate_spans_for_mode(2),
            Self::generate_spans_for_mode(3),
        ];
        
        Self {
            frame_count: 0,
            style_mode: 0,
            spans_cache,
            fps_widget: FpsWidget::new().with_label(true).with_style(CATPPUCCIN.green),
            _marker: PhantomData,
        }
    }

    fn generate_spans_for_mode(mode: usize) -> Vec<Span<'static>> {
        let text_len = 2048;
        lorem_ipsum(text_len, 0)
            .map(|word| Self::style_word_for_mode(word, mode))
            .collect()
    }

    fn style_word_for_mode(word: &str, mode: usize) -> Span<'static> {
        let colors = colors();

        let color = match mode {
            0 => CATPPUCCIN.text,
            1 => if word.starts_with('e') { CATPPUCCIN.green } else { CATPPUCCIN.text },
            2 => {
                let hash: usize = word.chars().take(1).map(|c| c as usize / 10).sum();
                colors[hash % colors.len()]
            },
            _ => {
                let hash: usize = word.chars().map(|c| c as usize).sum();
                colors[hash % colors.len()]
            }
        };
        
        Span::styled(word.to_string(), Style::default().fg(color))
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
                self.style_mode = (self.style_mode + 1) % 4;
                if self.style_mode == 0 {
                    return Ok(());
                } else {
                    thread::sleep(Duration::from_millis(200));
                    button.enable_interrupt().unwrap();
                }
            }
            self.frame_count += 1;
            self.fps_widget.fps.tick();
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))
                .unwrap();
        }
    }
}

impl<B: Backend> Default for Benchmark<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Backend> Widget for &Benchmark<B> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Percentage(100),
            Constraint::Length(3),
        ]).split(area);

        self.render_header(layout[0], buf);
        self.render_benchmark(layout[1], buf);
        self.render_footer(layout[2], buf);
    }
}

impl<B: Backend> Benchmark<B> {
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let title = format_compact!("Text Stress Test [{}]", self.style_mode + 1);
        render_header(area, buf, &title, CATPPUCCIN.green);
    }

    fn render_benchmark(&self, area: Rect, buf: &mut Buffer) {
        let cached_spans = &self.spans_cache[self.style_mode];
        let offset = (4 * self.frame_count as usize) % cached_spans.len().saturating_sub(50);
        
        let visible_spans = cached_spans.iter()
            .skip(offset)
            .take(350) // Limit spans for small screen
            .cloned();

        let paragraph = Paragraph::new(Line::from_iter(visible_spans))
            .wrap(Wrap { trim: true });
        
        paragraph.render(area, buf);
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::layout::Margin;
        
        let fps_area = area.inner(Margin::new(8, 0));
        self.fps_widget.render(fps_area, buf);
    }
}

fn lorem_ipsum(len: usize, word_offset: usize) -> impl Iterator<Item = &'static str> {
    let mut acc = 0;
    
    LOREM_IPSUM
        .split(" ")
        .cycle()
        .skip(word_offset)
        .flat_map(|w| [w, " "].into_iter())
        .take_while(move |w| {
            let is_within_screen = acc <= len;
            acc += w.len();
            is_within_screen
        })
}

fn colors() -> [Color; 16] {
    [
        CATPPUCCIN.text,
        CATPPUCCIN.red,
        CATPPUCCIN.green,
        CATPPUCCIN.yellow,
        CATPPUCCIN.blue,
        CATPPUCCIN.mauve,
        CATPPUCCIN.teal,
        CATPPUCCIN.peach,
        CATPPUCCIN.maroon,
        CATPPUCCIN.pink,
        CATPPUCCIN.lavender,
        CATPPUCCIN.sky,
        CATPPUCCIN.sapphire,
        CATPPUCCIN.flamingo,
        CATPPUCCIN.rosewater,
        CATPPUCCIN.subtext1,
    ]
}