use crate::catpuccin::CATPPUCCIN;
use crate::fps::FpsWidget;
use crate::header::render_header;
use compact_str::format_compact;
use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::gpio::{Gpio0, Input, PinDriver};
use esp_idf_svc::hal::task::notification::Notification;
use esp_idf_svc::sys::{heap_caps_get_free_size, heap_caps_get_total_size, MALLOC_CAP_8BIT};
use ratatui::prelude::{Backend, Color, Terminal};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};
use ratatui::layout::Margin;
use std::marker::PhantomData;
use tachyonfx::Motion::{LeftToRight, RightToLeft, UpToDown};
use tachyonfx::{fx, CellFilter, ColorSpace, Duration, EffectManager, Interpolation, Motion, ToRgbComponents};
use crate::worm_buffer::WormBuffer;

pub struct Nonsense<B: Backend> {
    fps_widget: FpsWidget,
    effects: EffectManager<()>,
    content: Paragraph<'static>,
    worm_buffer: WormBuffer,
    _marker: PhantomData<B>,
}

impl<B: Backend> Nonsense<B> {
    pub fn new() -> Self {
        let area = Rect::new(0, 3, 53, 15);

        let (free_memory, total_memory) = Self::get_memory_info();
        let used_memory = total_memory - free_memory;

        let content = vec![
            Line::from(vec![
                Span::styled("Transcendental Metrics", Style::default().fg(CATPPUCCIN.yellow)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("• ", Style::default().fg(CATPPUCCIN.green)),
                Span::styled("Wobble Factor: ", Style::default().fg(CATPPUCCIN.blue)),
                Span::styled(format_compact!("{:.2}% discombobulated", (used_memory as f32 / total_memory as f32) * 142.7), Style::default().fg(CATPPUCCIN.text)),
            ]),
            Line::from(vec![
                Span::styled("• ", Style::default().fg(CATPPUCCIN.green)),
                Span::styled("Fluxion Rate: ", Style::default().fg(CATPPUCCIN.blue)),
                Span::styled(format_compact!("{} jiggawatts", area.width * area.height / 17), Style::default().fg(CATPPUCCIN.text)),
            ]),
            Line::from(vec![
                Span::styled("• ", Style::default().fg(CATPPUCCIN.green)),
                Span::styled("Quantum Entanglement: ", Style::default().fg(CATPPUCCIN.blue)),
                Span::styled("Highly probable", Style::default().fg(CATPPUCCIN.text)),
            ]),
            Line::from(vec![
                Span::styled("• ", Style::default().fg(CATPPUCCIN.green)),
                Span::styled("Whimsy Level: ", Style::default().fg(CATPPUCCIN.blue)),
                Span::styled(format_compact!("{}% nonsensical", (free_memory / 1024) % 100), Style::default().fg(CATPPUCCIN.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Pseudoscientific Readings", Style::default().fg(CATPPUCCIN.yellow)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("• ", Style::default().fg(CATPPUCCIN.blue)),
                Span::styled("Blorping intensifies at ", Style::default().fg(CATPPUCCIN.subtext1)),
                Span::styled(format_compact!("{}Hz", area.height % 60 + 20), Style::default().fg(CATPPUCCIN.peach)),
            ]),
            Line::from(vec![
                Span::styled("• ", Style::default().fg(CATPPUCCIN.blue)),
                Span::styled("Squiggly lines detected: ", Style::default().fg(CATPPUCCIN.subtext1)),
                Span::styled(format_compact!("{}", (used_memory / 512) % 1000), Style::default().fg(CATPPUCCIN.peach)),
            ]),
            Line::from(vec![
                Span::styled("• ", Style::default().fg(CATPPUCCIN.blue)),
                Span::styled("Thingamajig status: ", Style::default().fg(CATPPUCCIN.subtext1)),
                Span::styled("Optimally befuddled", Style::default().fg(CATPPUCCIN.peach)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("🌀 ", Style::default().fg(CATPPUCCIN.lavender)),
                Span::styled("The universe is ", Style::default().fg(CATPPUCCIN.subtext1)),
                Span::styled("probably ", Style::default().fg(CATPPUCCIN.yellow)),
                Span::styled("fine", Style::default().fg(CATPPUCCIN.green)),
            ]),
        ];

        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(CATPPUCCIN.text));

        let mut this = Self {
            fps_widget: FpsWidget::new().with_label(true).with_style(CATPPUCCIN.green),
            effects: EffectManager::default(),
            content: paragraph,
            worm_buffer: WormBuffer::new(),
            _marker: PhantomData,
        };

        let black_sleep = || fx::prolong_end(1000, fx::fade_to(Color::Black, Color::Black, 1));
        let paint = |c: Color| fx::prolong_end(1000, fx::fade_to(c, c, 1));
        let draw_tfx = |timer: u32| fx::parallel(&[
            fx::effect_fn((), timer, |_, _ctx, mut cells| {
                "[ T A C H Y O N F X ]".chars().enumerate().for_each(|(_, ch)| {
                    if let Some((_, cell)) = cells.next() {
                        cell.set_char(ch);
                        cell.set_fg(Color::Black);
                        cell.modifier |= ratatui::style::Modifier::BOLD;
                    }
                });
            }),
            fx::sequence(&[
                fx::sweep_in(LeftToRight, 40, 0, Color::from_u32(0xbcc0cc), 900),
                fx::sleep(timer - 1300),
                fx::dissolve(400),
            ])
        ]).with_filter(CellFilter::Area(Rect::new(16, 9, 21, 1)));

        use Interpolation::*;
        let target_color = Color::from_u32(0xbcc0cc);
        let split_layout = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        this.effects.add_effect(fx::repeating(
            fx::sequence(&[
                black_sleep(),
                fx::parallel(&[
                    // prepare colors for slide effect
                    paint(target_color),

                    fx::prolong_end(500, fx::slide_in(LeftToRight, 20, 0, Color::Black, 500)
                        .with_filter(CellFilter::Area(split_layout[0]))
                        .with_color_space(ColorSpace::Rgb)),

                    fx::prolong_start(500, fx::slide_in(RightToLeft, 20, 0, Color::Black, 500)
                        .with_filter(CellFilter::Area(split_layout[1]))
                        .with_color_space(ColorSpace::Rgb)),
                ]),

                fx::parallel(&[
                    fx::sequence(&[
                        fx::prolong_start(3150, fx::expand(
                            fx::ExpandDirection::Vertical,
                            Style::new().bg(target_color).fg(Color::Black),
                            (900, ExpoIn)
                        )),
                    ]),
                    draw_tfx(2750),
                ]),

                fx::prolong_start(300, fx::parallel(&[
                    fx::sweep_in(LeftToRight, 20, 0, Color::Black, 1000)
                        .with_color_space(ColorSpace::Rgb),
                    fx::prolong_start(1800,
                        fx::hsl_shift(Some([0.0, -100.0, -25.0]), None, (600, Linear))
                            .with_filter(CellFilter::Text.into_static())
                            .reversed()
                    )
                ])),

                fx::sleep(1000),
                fx::parallel(&[
                    fx::hsl_shift(Some([60.0, 10.0, 5.0]), None, 2500)
                        .with_filter(CellFilter::Text.into_static()),
                    fx::prolong_start(1900, fx::dissolve((600, ExpoOut)))
                ]),
            ])).with_area(area)
        );

        this
    }

    pub fn run(
        mut self,
        terminal: &mut Terminal<B>,
        notification: &mut Notification,
        button: &mut PinDriver<Gpio0, Input>,
    ) -> std::io::Result<()> {
        button.enable_interrupt().unwrap();

        let get_instant_ms = || unsafe {
            (esp_idf_svc::sys::esp_timer_get_time() / 1000) as u32
        };


        let mut instant = get_instant_ms();
        loop {
            if notification.wait(delay::NON_BLOCK).is_some() {
                return Ok(());
            }

            let old_instant = instant;
            instant = get_instant_ms();
            let elapsed = Duration::from_millis(instant.wrapping_sub(old_instant));

            self.fps_widget.fps.tick();
            terminal.draw(|frame| {
                let area = frame.area();
                frame.render_widget(&self, area);
                self.effects.process_effects(elapsed, frame.buffer_mut(), area);
            }).unwrap();
        }
    }
}

impl<B: Backend> Default for Nonsense<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Backend> Widget for &Nonsense<B> {
    #[allow(clippy::similar_names)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.worm_buffer.cached_render(area, buf, |buf| {
            let layout = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(1),
            ]).split(area);

            self.render_header(layout[0], buf);
            self.render_content(layout[1], buf);
        });

        self.render_footer(Rect::new(6, 23, 53 - 6, 1), buf);
    }
}

impl<B: Backend> Nonsense<B> {
    fn get_memory_info() -> (usize, usize) {
        unsafe {
            let free = heap_caps_get_free_size(MALLOC_CAP_8BIT) as usize;
            let total = heap_caps_get_total_size(MALLOC_CAP_8BIT) as usize;
            (free, total)
        }
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        render_header(area, buf, "Quantum Flibbertigibbet", CATPPUCCIN.mauve);
    }

    fn render_content(&self, area: Rect, buf: &mut Buffer) {
        let inner = area.inner(Margin::new(6, 1));
        self.content.clone().render(inner, buf);
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let fps_area = area.inner(Margin::new(8, 0));
        self.fps_widget.render(fps_area, buf);
    }
}