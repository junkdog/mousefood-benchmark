use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::gpio::{Gpio0, Input, PinDriver};
use esp_idf_svc::hal::task::notification::Notification;
use ratatui::prelude::{Backend, Terminal};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Widget, Table, Row, Cell},
};
use std::marker::PhantomData;
use std::thread;
use std::time::{Duration, Instant};
use compact_str::format_compact;
use ratatui::layout::Margin;
use crate::catpuccin::CATPPUCCIN;
use crate::fps::FpsWidget;
use crate::header::render_header;

const ITERATIONS: u32 = 5_000_000;
const ALLOC_ITERATIONS: u32 = 100_000;

#[derive(Debug, Clone)]
struct BenchmarkResults {
    u32_add: Option<u32>,
    u32_mul: Option<u32>,
    u32_div: Option<u32>,
    f32_add: Option<u32>,
    f32_mul: Option<u32>,
    f32_div: Option<u32>,
    alloc_compact: Option<u32>,
    alloc_format: Option<u32>,
}

impl BenchmarkResults {
    fn new() -> Self {
        Self {
            u32_add: None,
            u32_mul: None,
            u32_div: None,
            f32_add: None,
            f32_mul: None,
            f32_div: None,
            alloc_compact: None,
            alloc_format: None,
        }
    }
}

#[derive(Debug)]
pub struct ComputeApp<B: Backend> {
    results: BenchmarkResults,
    current_benchmark: usize,
    fps_widget: FpsWidget,
    _marker: PhantomData<B>,
}

impl<B: Backend> ComputeApp<B> {
    pub fn new() -> Self {
        Self {
            results: BenchmarkResults::new(),
            current_benchmark: 0,
            fps_widget: FpsWidget::new().with_label(true).with_style(CATPPUCCIN.green),
            _marker: PhantomData,
        }
    }

    fn run_next_benchmark(&mut self) {
        if self.current_benchmark >= 8 {
            return;
        }

        let duration_ms = match self.current_benchmark {
            0 => self.benchmark_u32_add(),
            1 => self.benchmark_u32_mul(),
            2 => self.benchmark_u32_div(),
            3 => self.benchmark_f32_add(),
            4 => self.benchmark_f32_mul(),
            5 => self.benchmark_f32_div(),
            6 => self.benchmark_alloc_compact(),
            7 => self.benchmark_alloc_format(),
            _ => 0,
        };

        match self.current_benchmark {
            0 => self.results.u32_add = Some(duration_ms),
            1 => self.results.u32_mul = Some(duration_ms),
            2 => self.results.u32_div = Some(duration_ms),
            3 => self.results.f32_add = Some(duration_ms),
            4 => self.results.f32_mul = Some(duration_ms),
            5 => self.results.f32_div = Some(duration_ms),
            6 => self.results.alloc_compact = Some(duration_ms),
            7 => self.results.alloc_format = Some(duration_ms),
            _ => {}
        };

        self.current_benchmark += 1;
    }

    fn benchmark_u32_add(&self) -> u32 {
        let start = Instant::now();
        let mut result = 0u32;
        for i in 0..ITERATIONS {
            result = core::hint::black_box(result.wrapping_add(i));
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_u32_mul(&self) -> u32 {
        let start = Instant::now();
        let mut result = 1u32;
        for i in 1..=ITERATIONS {
            result = core::hint::black_box(result.wrapping_mul(i.wrapping_add(1)));
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_u32_div(&self) -> u32 {
        let start = Instant::now();
        let mut result = ITERATIONS;
        for i in 1..=ITERATIONS {
            result = core::hint::black_box(result / (i.wrapping_add(1)));
            if result == 0 { result = ITERATIONS; }
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_f32_add(&self) -> u32 {
        let start = Instant::now();
        let mut result = 0.0f32;
        for i in 0..ITERATIONS {
            result += core::hint::black_box(i as f32 * 0.1);
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_f32_mul(&self) -> u32 {
        let start = Instant::now();
        let mut result = 1.0f32;
        for i in 1..=ITERATIONS {
            result = core::hint::black_box(result * (i as f32 + 1.0) * 0.01);
            if result < 0.001 { result = 1.0; }
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_f32_div(&self) -> u32 {
        let start = Instant::now();
        let mut result = ITERATIONS as f32;
        for i in 1..=ITERATIONS {
            result = core::hint::black_box(result / (i as f32 + 1.0));
            if result < 0.0001 { result = ITERATIONS as f32; }
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_alloc_compact(&self) -> u32 {
        let start = Instant::now();
        for i in 0..ALLOC_ITERATIONS {
            let s = format_compact!("Test string {}", i);
            core::hint::black_box(s);
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_alloc_format(&self) -> u32 {
        let start = Instant::now();
        for i in 0..ALLOC_ITERATIONS {
            let s = format!("Test string {}", i);
            core::hint::black_box(s);
        }
        start.elapsed().as_millis() as u32
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
            
            // Run next benchmark if available
            if self.current_benchmark < 8 {
                self.run_next_benchmark();
            }
            
            self.fps_widget.fps.tick();
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))
                .unwrap();
            
            thread::sleep(Duration::from_millis(16)); // ~60 FPS
        }
    }
}

impl<B: Backend> Default for ComputeApp<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Backend> Widget for &ComputeApp<B> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Percentage(100),
            Constraint::Length(3),
        ]).split(area);

        self.render_header(layout[0], buf);
        self.render_results(layout[1], buf);
        self.render_footer(layout[2], buf);
    }
}

impl<B: Backend> ComputeApp<B> {
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let progress = self.current_benchmark.min(8);
        let title = format_compact!("Compute Benchmark [{}/8]", progress);
        render_header(area, buf, &title, CATPPUCCIN.blue);
    }

    fn render_results(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([
            Constraint::Length(6),
            Constraint::Length(4),
        ]).split(area);

        // Arithmetic table
        let header = Row::new(vec![
            Cell::from(" Op ").style(Style::default().fg(CATPPUCCIN.text)),
            Cell::from(" u32 ").style(Style::default().fg(CATPPUCCIN.green)),
            Cell::from(" f32 ").style(Style::default().fg(CATPPUCCIN.teal)),
        ]);

        let rows = vec![
            self.create_table_row(" ADD ", 0, 3, self.results.u32_add, self.results.f32_add),
            self.create_table_row(" MUL ", 1, 4, self.results.u32_mul, self.results.f32_mul),
            self.create_table_row(" DIV ", 2, 5, self.results.u32_div, self.results.f32_div),
        ];

        let table = Table::new(rows, [Constraint::Length(5), Constraint::Length(8), Constraint::Length(8)])
            .header(header)
            .block(Block::new());

        let table_area = Rect {
            x: layout[0].x + 6,
            y: layout[0].y,
            width: layout[0].width - 6,
            height: layout[0].height,
        };
        table.render(table_area, buf);

        // Allocation results
        self.render_allocation_results(layout[1], buf);
    }

    fn create_table_row<'a>(&self, op_name: &'a str, u32_idx: usize, f32_idx: usize, u32_result: Option<u32>, f32_result: Option<u32>) -> Row<'a> {
        let u32_cell = self.format_benchmark_cell(u32_idx, u32_result, CATPPUCCIN.green);
        let f32_cell = self.format_benchmark_cell(f32_idx, f32_result, CATPPUCCIN.teal);
        
        Row::new(vec![
            Cell::from(op_name).style(Style::default().fg(CATPPUCCIN.text)),
            u32_cell,
            f32_cell,
        ])
    }

    fn render_allocation_results(&self, area: Rect, buf: &mut Buffer) {
        let header = Row::new(vec![
            Cell::from("Alloc").style(Style::default().fg(CATPPUCCIN.text)),
            Cell::from("compact").style(Style::default().fg(CATPPUCCIN.peach)),
            Cell::from("format ").style(Style::default().fg(CATPPUCCIN.maroon)),
        ]);

        let compact_cell = self.format_benchmark_cell(6, self.results.alloc_compact, CATPPUCCIN.peach);
        let format_cell = self.format_benchmark_cell(7, self.results.alloc_format, CATPPUCCIN.maroon);
        
        let rows = vec![
            Row::new(vec![
                Cell::from("1M   ").style(Style::default().fg(CATPPUCCIN.text)),
                compact_cell,
                format_cell,
            ])
        ];

        let table = Table::new(rows, [Constraint::Length(5), Constraint::Length(8), Constraint::Length(8)])
            .header(header)
            .block(Block::new());

        let table_area = Rect {
            x: area.x + 6,
            y: area.y,
            width: area.width - 6,
            height: area.height,
        };
        table.render(table_area, buf);
    }

    fn format_benchmark_cell(&self, bench_idx: usize, result: Option<u32>, completed_color: Color) -> Cell<'static> {
        let (text, color) = if bench_idx < self.current_benchmark {
            if let Some(duration) = result {
                (format!("{} ms", duration), completed_color)
            } else {
                ("Error".to_string(), CATPPUCCIN.red)
            }
        } else if bench_idx == self.current_benchmark {
            ("Running...".to_string(), CATPPUCCIN.yellow)
        } else {
            ("Pending".to_string(), CATPPUCCIN.surface2)
        };
        
        Cell::from(text).style(Style::default().fg(color))
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let fps_area = area.inner(Margin::new(8, 0));
        self.fps_widget.render(fps_area, buf);
    }
}