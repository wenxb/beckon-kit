use std::{ops::Range, sync::Arc};

use gpui::{Bounds, Pixels, Point, WrappedLine, point, px, size};

use super::display::DisplayText;

pub(super) struct LayoutLine {
    pub line: WrappedLine,
    pub y: Pixels,
    start: usize,
}

pub(super) struct VisualRow {
    start: usize,
    end: usize,
    y: Pixels,
    start_x: Pixels,
    line: usize,
}

#[derive(Clone)]
pub(super) struct FieldGeometry {
    pub bounds: Bounds<Pixels>,
    pub origin: Point<Pixels>,
    pub lines: Arc<Vec<LayoutLine>>,
    rows: Arc<Vec<VisualRow>>,
    pub display: DisplayText,
    pub line_height: Pixels,
}

impl FieldGeometry {
    pub fn new(
        bounds: Bounds<Pixels>,
        lines: impl IntoIterator<Item = WrappedLine>,
        display: DisplayText,
        line_height: Pixels,
    ) -> Self {
        let mut layouts = Vec::new();
        let mut rows = Vec::new();
        let mut start = 0;
        let mut y = px(0.0);
        for line in lines {
            let line_ix = layouts.len();
            let mut row_start = 0;
            let first_row = rows.len();
            for end in line
                .wrap_boundaries
                .iter()
                .map(|boundary| line.runs()[boundary.run_ix].glyphs[boundary.glyph_ix].index)
                .chain([line.len()])
            {
                rows.push(VisualRow {
                    start: start + row_start,
                    end: start + end,
                    y: y + line_height * (rows.len() - first_row),
                    start_x: line.unwrapped_layout.x_for_index(row_start),
                    line: line_ix,
                });
                row_start = end;
            }
            let height = line.size(line_height).height;
            layouts.push(LayoutLine { line, y, start });
            start += layouts.last().unwrap().line.len() + 1;
            y += height;
        }
        Self {
            bounds,
            origin: bounds.origin,
            lines: Arc::new(layouts),
            rows: Arc::new(rows),
            display,
            line_height,
        }
    }

    pub fn height(&self) -> Pixels {
        self.line_height * self.rows.len()
    }

    pub fn width(&self) -> Pixels {
        self.lines
            .iter()
            .map(|line| line.line.width())
            .fold(px(0.0), Pixels::max)
    }

    fn row_for_index(&self, index: usize) -> usize {
        self.rows
            .iter()
            .enumerate()
            .find(|(ix, row)| {
                index <= row.end && self.rows.get(ix + 1).is_none_or(|next| next.start != index)
            })
            .map_or(self.rows.len() - 1, |(ix, _)| ix)
    }

    pub fn position(&self, source: usize) -> Point<Pixels> {
        let index = self.display.display_index(source);
        let row = &self.rows[self.row_for_index(index)];
        let line = &self.lines[row.line].line;
        self.origin
            + point(
                line.unwrapped_layout
                    .x_for_index(index.min(row.end) - self.line_start(row.line))
                    - row.start_x,
                row.y,
            )
    }

    fn line_start(&self, line: usize) -> usize {
        self.lines[line].start
    }

    pub fn index_at(&self, position: Point<Pixels>) -> usize {
        let local = position - self.origin;
        let row_ix =
            ((local.y / self.line_height).floor().max(0.0) as usize).min(self.rows.len() - 1);
        let row = &self.rows[row_ix];
        let line = &self.lines[row.line].line;
        let index = self.line_start(row.line)
            + line
                .unwrapped_layout
                .closest_index_for_x(local.x + row.start_x);
        self.display.source_index(index.clamp(row.start, row.end))
    }

    pub fn row_edge(&self, source: usize, end: bool) -> usize {
        let row = &self.rows[self.row_for_index(self.display.display_index(source))];
        self.display
            .source_index(if end { row.end } else { row.start })
    }

    pub fn selection_bounds(&self, range: Range<usize>, mut paint: impl FnMut(Bounds<Pixels>)) {
        let range = self.display.range(range);
        for row in self.rows.iter() {
            if range.end <= row.start || range.start > row.end {
                continue;
            }
            let line = &self.lines[row.line].line;
            let start = self.line_start(row.line);
            let left = line
                .unwrapped_layout
                .x_for_index(range.start.max(row.start) - start)
                - row.start_x;
            let mut right = line
                .unwrapped_layout
                .x_for_index(range.end.min(row.end) - start)
                - row.start_x;
            if range.end > row.end {
                right += px(4.0);
            }
            paint(Bounds::new(
                self.origin + point(left, row.y),
                size((right - left).max(px(1.0)), self.line_height),
            ));
        }
    }
}
