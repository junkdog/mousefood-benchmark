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
use embedded_graphics_unicodefonts::MONO_6X10;
use mousefood::embedded_graphics::mono_font::MonoFont;
use ratatui::layout::{Alignment, Margin};
use ratatui::text::Text;
use crate::catpuccin::CATPPUCCIN;
use crate::fps::FpsWidget;
use crate::header::render_header;
use crate::worm_buffer::WormBuffer;

const ITERATIONS: u32 = 100_000;

#[derive(Debug, Clone, Default)]
struct MappingBenchmarkResults {
    ascii: Option<u32>,
    latin1: Option<u32>,
    block: Option<u32>,
    braille: Option<u32>,
    quadrant: Option<u32>,
    box_drawing: Option<u32>,
}


#[derive(Debug, Clone, Default)]
struct BenchmarkResults {
    atlas_mapping: MappingBenchmarkResults,
    str_mapping: MappingBenchmarkResults,
}

impl BenchmarkResults {
    fn new() -> Self {
        Self::default()
    }
}


#[derive(Debug)]
pub struct GlyphMappingApp<'a, B: Backend> {
    atlas_font: &'a MonoFont<'a>,
    results: BenchmarkResults,
    current_benchmark: usize,
    fps_widget: FpsWidget,
    worm_buffer: WormBuffer,
    _marker: PhantomData<B>,
}

impl<'a, B: Backend> GlyphMappingApp<'a, B> {
    pub fn new(atlas_font: &'a MonoFont<'a>) -> Self {
        Self {
            atlas_font,
            results: BenchmarkResults::new(),
            current_benchmark: 0,
            fps_widget: FpsWidget::new().with_label(true).with_style(CATPPUCCIN.green),
            _marker: PhantomData,
            worm_buffer: WormBuffer::new(),
        }
    }

    fn run_next_benchmark(&mut self) {
        if self.current_benchmark >= 12 {
            return;
        }

        let str_atlas = MONO_6X10;

        let duration_ms = match self.current_benchmark {
            // String mapping benchmarks
            0 => self.benchmark_ascii(&str_atlas),
            1 => self.benchmark_latin1(&str_atlas),
            2 => self.benchmark_block_elements(&str_atlas),
            3 => self.benchmark_braille(&str_atlas),
            4 => self.benchmark_quadrants(&str_atlas),
            5 => self.benchmark_box_drawing(&str_atlas),
            // Atlas mapping benchmarks
            6 => self.benchmark_ascii(&self.atlas_font),
            7 => self.benchmark_latin1(&self.atlas_font),
            8 => self.benchmark_block_elements(&self.atlas_font),
            9 => self.benchmark_braille(&self.atlas_font),
            10 => self.benchmark_quadrants(&self.atlas_font),
            11 => self.benchmark_box_drawing(&self.atlas_font),
            _ => 0,
        };

        match self.current_benchmark {
            // String mapping results
            0 => self.results.str_mapping.ascii = Some(duration_ms),
            1 => self.results.str_mapping.latin1 = Some(duration_ms),
            2 => self.results.str_mapping.block = Some(duration_ms),
            3 => self.results.str_mapping.braille = Some(duration_ms),
            4 => self.results.str_mapping.quadrant = Some(duration_ms),
            5 => self.results.str_mapping.box_drawing = Some(duration_ms),
            // Atlas mapping results
            6 => self.results.atlas_mapping.ascii = Some(duration_ms),
            7 => self.results.atlas_mapping.latin1 = Some(duration_ms),
            8 => self.results.atlas_mapping.block = Some(duration_ms),
            9 => self.results.atlas_mapping.braille = Some(duration_ms),
            10 => self.results.atlas_mapping.quadrant = Some(duration_ms),
            11 => self.results.atlas_mapping.box_drawing = Some(duration_ms),
            _ => {}
        };

        self.current_benchmark += 1;
        self.worm_buffer.reset();
    }

    fn benchmark_ascii(&self, font: &MonoFont) -> u32 {
        let chars: Vec<char> = (0x20..0x7F)
            .map(|v| char::from_u32(v).unwrap())
            .collect();

        let mut input = chars.iter().cycle();
        
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let ch = input.next().copied().unwrap();
            core::hint::black_box(font.glyph_mapping.index(ch));
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_latin1(&self, font: &MonoFont) -> u32 {
        let chars: Vec<char> = (0xA0..0xFF)
            .filter_map(char::from_u32)
            .collect();

        let mut input = chars.iter().cycle();
        
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let ch = input.next().copied().unwrap();
            core::hint::black_box(font.glyph_mapping.index(ch));
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_block_elements(&self, font: &MonoFont) -> u32 {
        let chars: Vec<char> = (0x2580..0x259F)
            .filter_map(char::from_u32)
            .collect();

        let mut input = chars.iter().cycle();
        
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let ch = input.next().copied().unwrap();
            core::hint::black_box(font.glyph_mapping.index(ch));
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_braille(&self, font: &MonoFont) -> u32 {
        let chars: Vec<char> = (0x2800..0x28FF)
            .filter_map(char::from_u32)
            .collect();

        let mut input = chars.iter().cycle();
        
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let ch = input.next().copied().unwrap();
            core::hint::black_box(font.glyph_mapping.index(ch));
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_quadrants(&self, font: &MonoFont) -> u32 {
        let chars: Vec<char> = vec!['▀', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█', '▉', '▊', '▋', '▌', '▍', '▎', '▏',
                                   '▐', '░', '▒', '▓', '▔', '▕', '▖', '▗', '▘', '▙', '▚', '▛', '▜', '▝', '▞', '▟'];

        let mut input = chars.iter().cycle();
        
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let ch = input.next().copied().unwrap();
            core::hint::black_box(font.glyph_mapping.index(ch));
        }
        start.elapsed().as_millis() as u32
    }

    fn benchmark_box_drawing(&self, font: &MonoFont) -> u32 {
        let chars: Vec<char> = (0x2500..0x257F)
            .filter_map(char::from_u32)
            .collect();

        let mut input = chars.iter().cycle();
        
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let ch = input.next().copied().unwrap();
            core::hint::black_box(font.glyph_mapping.index(ch));
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
            if self.current_benchmark < 12 {
                self.run_next_benchmark();
            }
            
            self.fps_widget.fps.tick();
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))
                .unwrap();
        }
    }
}


impl<'a, B: Backend> Widget for &GlyphMappingApp<'a, B> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.worm_buffer.cached_render(area, buf, |buf| {
            let layout = Layout::vertical([
                Constraint::Length(3),
                Constraint::Percentage(100),
                Constraint::Length(3),
            ]).split(area);

            self.render_header(layout[0], buf);
            self.render_results(layout[1], buf);
            // self.render_footer(layout[2], buf);
        });
        self.render_footer(Rect::new(6, 23, 53 - 6, 1), buf);
    }
}

impl<'a, B: Backend> GlyphMappingApp<'a, B> {
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let progress = self.current_benchmark.min(12);
        let title = format_compact!("Glyph Mapping Benchmark [{}/12]", progress);
        render_header(area, buf, &title, CATPPUCCIN.blue);
    }

    fn render_results(&self, area: Rect, buf: &mut Buffer) {
        let header = Row::new(vec![
            Cell::from(" ").style(Style::default().fg(CATPPUCCIN.text)),
            Cell::from(" str  ").style(Style::default().fg(CATPPUCCIN.green)),
            Cell::from(" atlas ").style(Style::default().fg(CATPPUCCIN.teal)),
            Cell::from(" Ratio ").style(Style::default().fg(CATPPUCCIN.peach)),
        ]);

        let rows = vec![
            self.create_glyph_row("ASCII   ", self.results.str_mapping.ascii, self.results.atlas_mapping.ascii, 0, 6),
            self.create_glyph_row("Latin1  ", self.results.str_mapping.latin1, self.results.atlas_mapping.latin1, 1, 7),
            self.create_glyph_row("Block   ", self.results.str_mapping.block, self.results.atlas_mapping.block, 2, 8),
            self.create_glyph_row("Braille ", self.results.str_mapping.braille, self.results.atlas_mapping.braille, 3, 9),
            self.create_glyph_row("Quadrant", self.results.str_mapping.quadrant, self.results.atlas_mapping.quadrant, 4, 10),
            self.create_glyph_row("BoxDraw ", self.results.str_mapping.box_drawing, self.results.atlas_mapping.box_drawing, 5, 11),
        ];

        let table = Table::new(rows, [Constraint::Length(8), Constraint::Length(8), Constraint::Length(8), Constraint::Length(6)])
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

    fn create_glyph_row<'b>(&self, glyph_name: &'b str, str_result: Option<u32>, atlas_result: Option<u32>, str_idx: usize, atlas_idx: usize) -> Row<'b> {
        let str_cell = self.format_benchmark_cell(str_idx, str_result, CATPPUCCIN.green);
        let atlas_cell = self.format_benchmark_cell(atlas_idx, atlas_result, CATPPUCCIN.teal);
        
        // Calculate speed ratio (str_mapping is baseline 1.0x)
        let ratio_cell = if let (Some(str_ms), Some(atlas_ms)) = (str_result, atlas_result) {
            let ratio = str_ms as f32 / atlas_ms as f32;
            let ratio_text = Text::from(format!("{:.1}x", ratio)).alignment(Alignment::Right);
            let color = if ratio > 1.0 { CATPPUCCIN.green } else { CATPPUCCIN.red };
            Cell::from(ratio_text).style(Style::default().fg(color))
        } else {
            let color = if str_idx < self.current_benchmark && atlas_idx < self.current_benchmark {
                CATPPUCCIN.surface2
            } else if str_idx == self.current_benchmark || atlas_idx == self.current_benchmark {
                CATPPUCCIN.yellow
            } else {
                CATPPUCCIN.surface2
            };
            Cell::from("---").style(Style::default().fg(color))
        };
        
        Row::new(vec![
            Cell::from(glyph_name).style(Style::default().fg(CATPPUCCIN.text)),
            str_cell,
            atlas_cell,
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