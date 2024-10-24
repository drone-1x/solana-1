use std::cmp::{min, max};
use std::collections::HashMap;

use cassowary::{Solver, Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::{WEAK, REQUIRED};

use buffer::Buffer;

pub enum Alignment {
    Top,
    Left,
    Center,
    Bottom,
    Right,
}

pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Default for Rect {
    fn default() -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Rect {
        Rect {
            x: x,
            y: y,
            width: width,
            height: height,
        }
    }

    pub fn area(&self) -> u16 {
        self.width * self.height
    }

    pub fn inner(&self, margin: u16) -> Rect {
        if self.width < 2 * margin || self.height < 2 * margin {
            Rect::default()
        } else {
            Rect {
                x: self.x + margin,
                y: self.y + margin,
                width: self.width - 2 * margin,
                height: self.height - 2 * margin,
            }
        }
    }

    pub fn union(&self, other: &Rect) -> Rect {
        let x1 = min(self.x, other.x);
        let y1 = min(self.y, other.y);
        let x2 = max(self.x + self.width, other.x + other.width);
        let y2 = max(self.y + self.height, other.y + other.height);
        Rect {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        }
    }

    pub fn intersection(&self, other: &Rect) -> Rect {
        let x1 = max(self.x, other.x);
        let y1 = max(self.y, other.y);
        let x2 = min(self.x + self.width, other.x + other.width);
        let y2 = min(self.y + self.height, other.y + other.height);
        Rect {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        }
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width && self.x + self.width > other.x &&
        self.y < other.y + other.height && self.y + self.height > other.y
    }
}

#[derive(Debug, Clone, Hash)]
pub enum Size {
    Fixed(u16),
    Max(u16),
    Min(u16),
}

/// # Examples
/// ```
/// extern crate tui;
/// use tui::layout::{Rect, Size, Alignment, Direction, split};
///
/// fn main() {
///     let chunks = split(&Rect{x: 2, y: 2, width: 10, height: 10},
///                        &Direction::Vertical,
///                        &Alignment::Left,
///                        0,
///                        &[Size::Fixed(5), Size::Min(5)]);
/// }
///
/// ```
#[allow(unused_variables)]
pub fn split(area: &Rect,
             dir: &Direction,
             align: &Alignment,
             margin: u16,
             sizes: &[Size])
             -> Vec<Rect> {
    let mut solver = Solver::new();
    let mut vars: HashMap<Variable, (usize, usize)> = HashMap::new();
    let elements = sizes.iter().map(|_| Element::new()).collect::<Vec<Element>>();
    let mut results = sizes.iter().map(|_| Rect::default()).collect::<Vec<Rect>>();
    let dest_area = area.inner(margin);
    for (i, e) in elements.iter().enumerate() {
        vars.insert(e.x, (i, 0));
        vars.insert(e.y, (i, 1));
        vars.insert(e.width, (i, 2));
        vars.insert(e.height, (i, 3));
    }
    let mut constraints: Vec<Constraint> = Vec::new();
    if let Some(first) = elements.first() {
        constraints.push(match *dir {
            Direction::Horizontal => first.x | EQ(REQUIRED) | dest_area.x as f64,
            Direction::Vertical => first.y | EQ(REQUIRED) | dest_area.y as f64,
        })
    }
    if let Some(last) = elements.last() {
        constraints.push(match *dir {
            Direction::Horizontal => {
                (last.x + last.width) | EQ(WEAK) | (dest_area.x + dest_area.width) as f64
            }
            Direction::Vertical => {
                (last.y + last.height) | EQ(WEAK) | (dest_area.y + dest_area.height) as f64
            }
        })
    }
    match *dir {
        Direction::Horizontal => {
            for pair in elements.windows(2) {
                constraints.push((pair[0].x + pair[0].width) | EQ(REQUIRED) | pair[1].x);
            }
            for (i, size) in sizes.iter().enumerate() {
                let cs = [elements[i].y | EQ(REQUIRED) | dest_area.y as f64,
                          elements[i].height | EQ(REQUIRED) | dest_area.height as f64,
                          match *size {
                              Size::Fixed(v) => elements[i].width | EQ(REQUIRED) | v as f64,
                              Size::Min(v) => elements[i].width | GE(REQUIRED) | v as f64,
                              Size::Max(v) => elements[i].width | LE(REQUIRED) | v as f64,
                          }];
                constraints.extend_from_slice(&cs);
            }
        }
        Direction::Vertical => {
            for pair in elements.windows(2) {
                constraints.push((pair[0].y + pair[0].height) | EQ(REQUIRED) | pair[1].y);
            }
            for (i, size) in sizes.iter().enumerate() {
                let cs = [elements[i].x | EQ(REQUIRED) | dest_area.x as f64,
                          elements[i].width | EQ(REQUIRED) | dest_area.width as f64,
                          match *size {
                              Size::Fixed(v) => elements[i].height | EQ(REQUIRED) | v as f64,
                              Size::Min(v) => elements[i].height | GE(REQUIRED) | v as f64,
                              Size::Max(v) => elements[i].height | LE(REQUIRED) | v as f64,
                          }];
                constraints.extend_from_slice(&cs);
            }
        }
    }
    solver.add_constraints(&constraints).unwrap();
    // TODO: Find a better way to handle overflow error
    for &(var, value) in solver.fetch_changes() {
        let (index, attr) = vars[&var];
        match attr {
            0 => {
                results[index].x = value as u16;
            }
            1 => {
                results[index].y = value as u16;
            }
            2 => {
                let mut v = value as u16;
                if v > area.width {
                    v = 0;
                }
                results[index].width = v;
            }
            3 => {
                let mut v = value as u16;
                if v > area.height {
                    v = 0;
                }
                results[index].height = v;
            }
            _ => {}
        }
    }
    results
}

struct Element {
    x: Variable,
    y: Variable,
    width: Variable,
    height: Variable,
}

impl Element {
    fn new() -> Element {
        Element {
            x: Variable::new(),
            y: Variable::new(),
            width: Variable::new(),
            height: Variable::new(),
        }
    }
}

pub struct Group {
    direction: Direction,
    alignment: Alignment,
    margin: u16,
    chunks: Vec<Size>,
}

impl Default for Group {
    fn default() -> Group {
        Group {
            direction: Direction::Horizontal,
            alignment: Alignment::Left,
            margin: 0,
            chunks: Vec::new(),
        }
    }
}

impl Group {
    pub fn direction(&mut self, direction: Direction) -> &mut Group {
        self.direction = direction;
        self
    }

    pub fn alignment(&mut self, alignment: Alignment) -> &mut Group {
        self.alignment = alignment;
        self
    }

    pub fn margin(&mut self, margin: u16) -> &mut Group {
        self.margin = margin;
        self
    }

    pub fn chunks(&mut self, chunks: &[Size]) -> &mut Group {
        self.chunks = Vec::from(chunks);
        self
    }
    pub fn render<F>(&self, area: &Rect, mut f: F)
        where F: FnMut(&[Rect])
    {
        let chunks = split(area,
                           &self.direction,
                           &self.alignment,
                           self.margin,
                           &self.chunks);
        f(&chunks);
    }
}
