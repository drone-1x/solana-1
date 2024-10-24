extern crate termion;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
extern crate cassowary;
extern crate unicode_segmentation;

mod buffer;
mod util;
pub mod symbols;
pub mod terminal;
pub mod widgets;
pub mod style;
pub mod layout;

pub use self::terminal::Terminal;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
