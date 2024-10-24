use std::cmp::min;

use layout::Rect;
use buffer::Buffer;
use widgets::{Widget, Block};
use style::Color;
use symbols::bar;

pub struct Sparkline<'a> {
    block: Option<Block<'a>>,
    fg: Color,
    bg: Color,
    data: Vec<u64>,
    max: Option<u64>,
}

impl<'a> Default for Sparkline<'a> {
    fn default() -> Sparkline<'a> {
        Sparkline {
            block: None,
            fg: Color::White,
            bg: Color::Black,
            data: Vec::new(),
            max: None,
        }
    }
}

impl<'a> Sparkline<'a> {
    pub fn block(&mut self, block: Block<'a>) -> &mut Sparkline<'a> {
        self.block = Some(block);
        self
    }

    pub fn fg(&mut self, fg: Color) -> &mut Sparkline<'a> {
        self.fg = fg;
        self
    }

    pub fn bg(&mut self, bg: Color) -> &mut Sparkline<'a> {
        self.bg = bg;
        self
    }


    pub fn data(&mut self, data: &[u64]) -> &mut Sparkline<'a> {
        self.data = data.to_vec();
        self
    }

    pub fn max(&mut self, max: u64) -> &mut Sparkline<'a> {
        self.max = Some(max);
        self
    }
}

impl<'a> Widget<'a> for Sparkline<'a> {
    fn buffer(&'a self, area: &Rect) -> Buffer<'a> {
        let (mut buf, spark_area) = match self.block {
            Some(ref b) => (b.buffer(area), b.inner(*area)),
            None => (Buffer::empty(*area), *area),
        };
        if spark_area.height < 1 {
            return buf;
        } else {
            let margin_x = spark_area.x - area.x;
            let margin_y = spark_area.y - area.y;
            let max = match self.max {
                Some(v) => v,
                None => *self.data.iter().max().unwrap_or(&1u64),
            };
            let max_index = min(spark_area.width as usize, self.data.len());
            let mut data = self.data
                .iter()
                .map(|e| e * spark_area.height as u64 * 8 / max)
                .collect::<Vec<u64>>();
            for j in (0..spark_area.height).rev() {
                for (i, d) in data.iter_mut().take(max_index).enumerate() {
                    buf.update_cell(margin_x + i as u16, margin_y + j, |c| {
                        c.symbol = match *d {
                            0 => " ",
                            1 => bar::ONE_EIGHTH,
                            2 => bar::ONE_QUATER,
                            3 => bar::THREE_EIGHTHS,
                            4 => bar::HALF,
                            5 => bar::FIVE_EIGHTHS,
                            6 => bar::THREE_QUATERS,
                            7 => bar::SEVEN_EIGHTHS,
                            _ => bar::FULL,
                        };
                        c.fg = self.fg;
                        c.bg = self.bg;
                    });
                    if *d > 8 {
                        *d -= 8;
                    } else {
                        *d = 0;
                    }
                }
            }
        }
        buf
    }
}
