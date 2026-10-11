//! Удобная печать с форматированием.

use core::fmt::{self, Write};
use crate::vga;

struct Writer;

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        vga::print(s.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    let mut w = Writer;
    let _ = w.write_fmt(args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
