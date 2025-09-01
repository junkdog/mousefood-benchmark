use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::gpio::{Gpio0, Input, PinDriver};
use esp_idf_svc::hal::task::notification::Notification;
use ratatui::prelude::{Backend, Color, Terminal};
use ratatui::widgets::BorderType;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Gauge, Padding, Widget},
};
use std::marker::PhantomData;
use crate::fps::FpsWidget;
use crate::catpuccin::CATPPUCCIN;

#[derive(Debug)]
pub struct GaugeApp<B: Backend> {
    progress1: f64,
    progress2: f64,
    fps_widget: FpsWidget,
    _marker: PhantomData<B>,
}

impl<B: Backend> GaugeApp<B> {
    pub fn new() -> Self {
        Self {
            progress1: 20.0,
            progress2: 20.0,
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
            self.progress1 = (self.progress1 + 0.1).clamp(0.0, 100.0);
            self.progress2 = (self.progress2 + 0.1).clamp(0.0, 100.0);
        }
    }
}

impl<B: Backend> Default for GaugeApp<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Backend> Widget for &GaugeApp<B> {
    #[allow(clippy::similar_names)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min, Ratio};
        let layout = Layout::vertical([Min(0), Length(1)]);
        let [gauge_area, footer_area] = layout.areas(area);

        let layout = Layout::vertical([Ratio(1, 2); 2]);
        let [gauge1_area, gauge2_area] = layout.areas(gauge_area);

        self.render_footer(footer_area, buf);

        self.render_gauge1(gauge1_area, buf);
        self.render_gauge2(gauge2_area, buf);
    }
}


impl<B: Backend> GaugeApp<B> {
    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::layout::Margin;
        
        let fps_area = area.inner(Margin::new(8, 0));
        self.fps_widget.render(fps_area, buf);
    }

    fn render_gauge1(&self, area: Rect, buf: &mut Buffer) {
        let title = title_block("Gauge (no unicode)");
        let label = format!("{:.1}%", self.progress1);
        Gauge::default()
            .block(title)
            .gauge_style(Color::Gray)
            .ratio(self.progress1 / 100.0)
            .label(label)
            .render(area, buf);
    }

    fn render_gauge2(&self, area: Rect, buf: &mut Buffer) {
        let title = title_block("Gauge (unicode)");
        let label = format!("{:.1}%", self.progress1);
        Gauge::default()
            .block(title)
            .gauge_style(Color::Yellow)
            .ratio(self.progress2 / 100.0)
            .label(label)
            .use_unicode(true)
            .render(area, buf);
    }
}

fn title_block(title: &str) -> Block {
    let title = Line::from(title).centered();
    Block::bordered()
        .border_type(BorderType::Double)
        .padding(Padding::vertical(1))
        .title(title)
}
