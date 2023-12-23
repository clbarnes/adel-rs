use std::{
    collections::HashSet,
    io::{self, Write},
    sync::OnceLock,
};
use thiserror::Error;
use utf8_read::Error as Utf8Error;

mod parse;

use parse::Unit;

// this may not be worth the lock hassle
static CONTROL_CHARS: OnceLock<HashSet<char>> = OnceLock::new();

const FS: char = '\u{241C}';
const GS: char = '\u{241D}';
const RS: char = '\u{241E}';
const US: char = '\u{241F}';

pub(crate) fn control_chars() -> &'static HashSet<char> {
    CONTROL_CHARS.get_or_init(|| vec![FS, GS, RS, US].into_iter().collect())
}

#[derive(Debug, Error)]
#[error("Unit contains control character: {string:?}")]
pub struct ContainsControlChar {
    string: String,
}

impl ContainsControlChar {
    fn check_str(s: &str) -> Result<(), Self> {
        let banned = control_chars();
        for char in s.chars() {
            if banned.contains(&char) {
                return Err(Self {
                    string: String::from(s),
                });
            }
        }
        Ok(())
    }
}

struct AdelWriter<W: Write> {
    inner: W,
    buffer: [u8; 4],
}

impl<W: Write> From<W> for AdelWriter<W> {
    fn from(value: W) -> Self {
        Self {
            inner: value,
            buffer: [0; 4],
        }
    }
}

impl<W: Write> AdelWriter<W> {
    fn write_char(&mut self, c: char) -> io::Result<&mut Self> {
        let sl = c.encode_utf8(&mut self.buffer);
        self.inner.write_all(sl.as_bytes()).map(|_| self)
    }

    fn write_unit(&mut self, unit: Unit) -> io::Result<&mut Self> {
        self.inner.write_all(unit.as_str().as_bytes()).map(|_| self)
    }

    fn write_us(&mut self) -> io::Result<&mut Self> {
        self.write_char(US)
    }

    fn write_rs(&mut self) -> io::Result<&mut Self> {
        self.write_char(RS)
    }

    fn write_gs(&mut self) -> io::Result<&mut Self> {
        self.write_char(GS)
    }

    fn write_fs(&mut self) -> io::Result<&mut Self> {
        self.write_char(FS)
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
