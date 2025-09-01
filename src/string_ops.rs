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
use compact_str::{format_compact, CompactString, ToCompactString};
use mousefood::embedded_graphics::mono_font::MonoFont;
use ratatui::layout::{Alignment, Margin};
use ratatui::text::Text;
use crate::catpuccin::CATPPUCCIN;
use crate::embedded_str::EmbeddedStr;
use crate::fps::FpsWidget;
use crate::header::render_header;

const ITERATIONS: u32 = 1_000_000;

#[derive(Debug, Clone, Default)]
struct StringBenchmarkResults {
    from_ascii_str: Option<u32>,
    from_ascii_ch: Option<u32>,
    from_block_str: Option<u32>,
    from_block_ch: Option<u32>,
    as_str: Option<u32>,
}


#[derive(Debug, Clone, Default)]
struct BenchmarkResults {
    compact_str: StringBenchmarkResults,
    embedded_str: StringBenchmarkResults,
}

impl BenchmarkResults {
    fn new() -> Self {
        Self::default()
    }
}


#[derive(Debug)]
pub struct StringOpsApp<'a, B: Backend> {
    atlas_font: &'a MonoFont<'a>,
    results: BenchmarkResults,
    current_benchmark: usize,
    fps_widget: FpsWidget,
    _marker: PhantomData<B>,
}

impl<'a, B: Backend> StringOpsApp<'a, B> {
    pub fn new(atlas_font: &'a MonoFont<'a>) -> Self {
        Self {
            atlas_font,
            results: BenchmarkResults::new(),
            current_benchmark: 0,
            fps_widget: FpsWidget::new().with_label(true).with_style(CATPPUCCIN.green),
            _marker: PhantomData,
        }
    }

    fn run_next_benchmark(&mut self) {
        if self.current_benchmark >= 10 {
            return;
        }

        let duration_ms = match self.current_benchmark {
            // CompactString benchmarks
            0 => self.benchmark_from_ascii_str(|s| CompactString::from(s)),
            1 => self.benchmark_from_ascii_char(|c| c.to_compact_string()),
            2 => self.benchmark_from_block_str(|s| CompactString::from(s)),
            3 => self.benchmark_from_block_char(|c| c.to_compact_string()),
            4 => self.benchmark_as_str_compact(),
            // EmbeddedStr benchmarks
            5 => self.benchmark_from_ascii_str(|s|EmbeddedStr::from(s)),
            6 => self.benchmark_from_ascii_char(EmbeddedStr::from),
            7 => self.benchmark_from_block_str(|s| EmbeddedStr::from(s)),
            8 => self.benchmark_from_block_char(EmbeddedStr::from),
            9 => self.benchmark_as_str_embedded(),
            _ => 0,
        };

        match self.current_benchmark {
            // CompactString results
            0 => self.results.compact_str.from_ascii_str = Some(duration_ms),
            1 => self.results.compact_str.from_ascii_ch = Some(duration_ms),
            2 => self.results.compact_str.from_block_str = Some(duration_ms),
            3 => self.results.compact_str.from_block_ch = Some(duration_ms),
            4 => self.results.compact_str.as_str = Some(duration_ms),
            // EmbeddedStr results
            5 => self.results.embedded_str.from_ascii_str = Some(duration_ms),
            6 => self.results.embedded_str.from_ascii_ch = Some(duration_ms),
            7 => self.results.embedded_str.from_block_str = Some(duration_ms),
            8 => self.results.embedded_str.from_block_ch = Some(duration_ms),
            9 => self.results.embedded_str.as_str = Some(duration_ms),
            _ => {}
        };

        self.current_benchmark += 1;
    }

    fn benchmark_from_ascii_char<T>(&self, f: impl Fn(char) -> T) -> u32 {
        let chars: Vec<char> = (0x20..0x7F)
            .map(|v| char::from_u32(v).unwrap())
            .collect();

        let mut input = chars.iter().cycle();
        
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let ch = input.next().copied().unwrap();
            core::hint::black_box(f(ch));
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_from_ascii_str<T>(&self, f: impl Fn(&str) -> T) -> u32 {
        let chars: Vec<String> = (0x20..0x7F)
            .map(|v| char::from_u32(v).unwrap())
            .map(|c| c.to_string())
            .collect();

        let mut input = chars.iter().cycle();

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let s = input.next().unwrap();
            core::hint::black_box(f(s));
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_from_block_str<T>(&self, f: impl Fn(&str) -> T) -> u32 {
        let chars: Vec<String> = (0x2580..0x259F)
            .filter_map(char::from_u32)
            .map(|c| c.to_string())
            .collect();

        let mut input = chars.iter().cycle();

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let s = input.next().unwrap();
            core::hint::black_box(f(s));
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_from_block_char<T>(&self, f: impl Fn(char) -> T) -> u32 {
        let chars: Vec<char> = (0x2580..0x259F)
            .filter_map(char::from_u32)
            .collect();

        let mut input = chars.iter().cycle();
        
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let ch = input.next().copied().unwrap();
            core::hint::black_box(f(ch));
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_as_str_compact(&self) -> u32 {
        let strings: Vec<CompactString> = (0x20..0x7F)
            .map(|v| char::from_u32(v).unwrap())
            .map(|c| c.to_compact_string())
            .collect();

        let mut input = strings.iter().cycle();
        
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let s = input.next().unwrap();
            core::hint::black_box(s.as_str());
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_as_str_embedded(&self) -> u32 {
        let strings: Vec<EmbeddedStr> = (0x20..0x7F)
            .map(|v| char::from_u32(v).unwrap())
            .map(EmbeddedStr::from)
            .collect();

        let mut input = strings.iter().cycle();
        
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let s = input.next().unwrap();
            core::hint::black_box(s.as_str());
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
            if self.current_benchmark < 10 {
                self.run_next_benchmark();
            }
            
            self.fps_widget.fps.tick();
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))
                .unwrap();
            
            thread::sleep(Duration::from_millis(16)); // ~60 FPS
        }
    }
}


impl<'a, B: Backend> Widget for &StringOpsApp<'a, B> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Percentage(100),
            Constraint::Length(3),
        ]).split(area);

        self.render_header(layout[0], buf);
        self.render_results(layout[1], buf);
        self.render_footer(layout[2], buf);
    }
}

impl<'a, B: Backend> StringOpsApp<'a, B> {
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let progress = self.current_benchmark.min(10);
        let title = format_compact!("String Operations Benchmark [{}/10]", progress);
        render_header(area, buf, &title, CATPPUCCIN.blue);
    }

    fn render_results(&self, area: Rect, buf: &mut Buffer) {
        let header = Row::new(vec![
            Cell::from("Operation").style(Style::default().fg(CATPPUCCIN.text)),
            Cell::from("Compact ").style(Style::default().fg(CATPPUCCIN.green)),
            Cell::from("Embedded").style(Style::default().fg(CATPPUCCIN.teal)),
            Cell::from(" Ratio ").style(Style::default().fg(CATPPUCCIN.peach)),
        ]);

        let rows = vec![
            self.create_string_row("AsciiStr ", self.results.compact_str.from_ascii_str, self.results.embedded_str.from_ascii_str, 0, 5),
            self.create_string_row("AsciiCh  ", self.results.compact_str.from_ascii_ch, self.results.embedded_str.from_ascii_ch, 1, 6),
            self.create_string_row("BlockStr ", self.results.compact_str.from_block_str, self.results.embedded_str.from_block_str, 2, 7),
            self.create_string_row("BlockCh  ", self.results.compact_str.from_block_ch, self.results.embedded_str.from_block_ch, 3, 8),
            self.create_string_row("AsStr    ", self.results.compact_str.as_str, self.results.embedded_str.as_str, 4, 9),
        ];

        let table = Table::new(rows, [Constraint::Length(9), Constraint::Length(8), Constraint::Length(8), Constraint::Length(7)])
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

    fn create_string_row<'b>(&self, operation_name: &'b str, compact_result: Option<u32>, embedded_result: Option<u32>, compact_idx: usize, embedded_idx: usize) -> Row<'b> {
        let compact_cell = self.format_benchmark_cell(compact_idx, compact_result, CATPPUCCIN.green);
        let embedded_cell = self.format_benchmark_cell(embedded_idx, embedded_result, CATPPUCCIN.teal);
        
        // Calculate speed ratio (compact is baseline 1.0x)
        let ratio_cell = if let (Some(compact_ms), Some(embedded_ms)) = (compact_result, embedded_result) {
            let ratio = compact_ms as f32 / embedded_ms as f32;
            let ratio_text = Text::from(format!("{:.1}x", ratio)).alignment(Alignment::Right);
            let color = if ratio > 1.0 { CATPPUCCIN.teal } else { CATPPUCCIN.green };
            Cell::from(ratio_text).style(Style::default().fg(color))
        } else {
            let color = if compact_idx < self.current_benchmark && embedded_idx < self.current_benchmark {
                CATPPUCCIN.surface2
            } else if compact_idx == self.current_benchmark || embedded_idx == self.current_benchmark {
                CATPPUCCIN.yellow
            } else {
                CATPPUCCIN.surface2
            };
            Cell::from("---").style(Style::default().fg(color))
        };
        
        Row::new(vec![
            Cell::from(operation_name).style(Style::default().fg(CATPPUCCIN.text)),
            compact_cell,
            embedded_cell,
            ratio_cell,
        ])
    }

    fn format_benchmark_cell(&self, bench_idx: usize, result: Option<u32>, completed_color: Color) -> Cell<'static> {
        let (text, color) = if bench_idx < self.current_benchmark {
            if let Some(duration) = result {
                (format!("{duration}"), completed_color)
            } else {
                ("Error".to_string(), CATPPUCCIN.red)
            }
        } else if bench_idx == self.current_benchmark {
            ("Running".to_string(), CATPPUCCIN.yellow)
        } else {
            ("Pending".to_string(), CATPPUCCIN.surface2)
        };
        
        Cell::from(Text::from(text).alignment(Alignment::Right)).style(Style::default().fg(color))
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let fps_area = area.inner(Margin::new(8, 0));
        self.fps_widget.render(fps_area, buf);
    }
}