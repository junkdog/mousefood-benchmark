use std::cell::RefCell;
use std::hash::{BuildHasher, Hash, Hasher};
use foldhash::fast::RandomState;
use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::{Position, Rect};

/// Caches and replays ratatui buffer changes for optimized rendering
/// 
/// WormBuffer records only the cells that change during widget rendering, allowing
/// expensive layouts to be computed once and then replayed efficiently. On first
/// render, it captures which cells differ before and after widget rendering. On
/// subsequent renders, it replays only those changed cells without re-executing
/// the widget rendering logic.
/// 
/// Once populated, the buffer becomes read-only for rendering until [`reset()`](Self::reset)
/// is called to clear the cache and allow new content to be captured.
/// 
/// This is particularly useful for embedded systems where rendering performance
/// is critical and complex layouts should be cached when possible.
#[derive(Debug, Default, Clone)]
pub struct WormBuffer {
    cells: RefCell<Vec<(Position, Cell)>>,
    hasher_state: RandomState,
}

impl WormBuffer {
    /// Creates a new empty buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Combines this buffer with another, consuming self
    /// 
    /// # Arguments
    /// * `other` - Buffer to merge cells from
    pub fn combine(self, other: &Self) -> Self {
        self.cells.borrow_mut().extend_from_slice(&other.cells.borrow());
        self
    }
    
    /// Clears all cached cells
    pub fn reset(&mut self) {
        self.cells.borrow_mut().clear();
    }

    /// Renders widgets, caching only changed cells for future replays
    /// 
    /// # Arguments
    /// * `area` - Rectangular area to render within
    /// * `buf` - Target buffer to render into
    /// * `render_widgets` - Closure that performs the actual widget rendering (only called when buffer is empty)
    pub fn cached_render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        mut render_widgets: impl FnMut(&mut Buffer)
    ) {
        // replay the recorded cells if the buffer has already been captured
        if !self.cells.borrow().is_empty() {
            self.render(area, buf);
            return;
        }

        // make a record of the current state of the buffer by hashing each cell
        let area = area.intersection(buf.area);
        let cell_hashes: Vec<u64> = area.positions()
            .map(|pos| self.cell_hash(&buf[pos]))
            .collect();

        // render the widgets into the buffer
        render_widgets(buf);

        // compare the new state of the buffer to the old state,
        // and record any changed cells
        let mut cells = self.cells.borrow_mut();
        area.positions()
            .zip(cell_hashes)
            .filter(|&(pos, old_hash)| old_hash != self.cell_hash(&buf[pos]))
            .for_each(|(pos, _)| cells.push((pos, buf[pos].clone())));
    }

    fn render(&self, _area: Rect, buf: &mut Buffer) {
        self.cells
            .borrow()
            .iter()
            .cloned()
            .for_each(|(pos, cell)| buf[pos] = cell);
    }

    fn cell_hash(&self, cell: &Cell) -> u64 {
        self.hasher_state.hash_one(&cell)
    }
}

impl Into<WormBuffer> for &Buffer {
    fn into(self) -> WormBuffer {
        let cells = self
            .area
            .positions()
            .map(|pos| (pos, self[pos].clone()))
            .filter(|(_, cell)| *cell != Cell::EMPTY)
            .collect();

        WormBuffer {
            cells: RefCell::new(cells),
            hasher_state: RandomState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Color, Style};
    use ratatui::widgets::{Block, Borders, Paragraph, Widget};

    #[test]
    fn test_worm_buffer_equivalent_to_direct_render() {
        let area = Rect::new(0, 0, 10, 5);
        
        // Direct rendering without WormBuffer
        let mut direct_buf = Buffer::empty(area);
        Paragraph::new("Hello")
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL))
            .render(area, &mut direct_buf);

        // Rendering with WormBuffer
        let mut worm_buf = WormBuffer::new();
        let mut cached_buf = Buffer::empty(area);
        
        // First render - populate the WormBuffer
        worm_buf.cached_render(area, &mut cached_buf, |buf| {
            Paragraph::new("Hello")
                .style(Style::default().fg(Color::Red))
                .block(Block::default().borders(Borders::ALL))
                .render(area, buf);
        });

        // The replayed buffer should match the direct render
        assert_buffer_eq(&direct_buf, &cached_buf, area);
    }

    fn assert_buffer_eq(expected: &Buffer, actual: &Buffer, area: Rect) {
        for pos in area.positions() {
            let expected_cell = &expected[pos];
            let actual_cell = &actual[pos];
            assert_eq!(
                expected_cell, actual_cell,
                "Cell mismatch at {:?}: expected {:?}, got {:?}",
                pos, expected_cell, actual_cell
            );
        }
    }
}