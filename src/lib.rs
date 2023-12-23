use std::{
    collections::HashSet,
    io::{self, Write},
    sync::OnceLock,
};
use thiserror::Error;
use utf8_read::Error as Utf8Error;

mod parse;

use parse::{Group, Record, Unit};

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
    next_sep: bool,
}

impl<W: Write> From<W> for AdelWriter<W> {
    fn from(value: W) -> Self {
        Self {
            inner: value,
            buffer: [0; 4],
            next_sep: false,
        }
    }
}

impl<W: Write> AdelWriter<W> {
    fn write_char(&mut self, c: char) -> io::Result<&mut Self> {
        let sl = c.encode_utf8(&mut self.buffer);
        self.inner.write_all(sl.as_bytes()).map(|_| self)
    }

    fn check_next_char(&mut self, next_sep: bool) -> io::Result<()> {
        if next_sep != self.next_sep {
            let msg = if self.next_sep {
                "Expected next char to be a separator"
            } else {
                "Expected next char to be a non-separator"
            };
            Err(io::Error::new(io::ErrorKind::InvalidInput, msg))
        } else {
            Ok(())
        }
    }

    fn write_unit(&mut self, unit: &Unit) -> io::Result<&mut Self> {
        self.check_next_char(false)?;
        self.next_sep = true;
        self.inner.write_all(unit.as_str().as_bytes()).map(|_| self)
    }

    fn write_record<'a, T: IntoIterator<Item = &'a Unit>>(
        &mut self,
        item: T,
    ) -> io::Result<&mut Self> {
        self.check_next_char(false)?;
        let mut it = item.into_iter();
        let Some(u) = it.next() else { return Ok(self) };
        self.write_unit(u)?;
        for i in it {
            self.write_us()?;
            self.write_unit(i)?;
        }
        Ok(self)
    }

    fn write_group<'a, T: IntoIterator<Item = &'a Record>>(
        &mut self,
        item: T,
    ) -> io::Result<&mut Self> {
        self.check_next_char(false)?;
        let mut it = item.into_iter();
        let Some(u) = it.next() else { return Ok(self) };
        self.write_record(u)?;
        for i in it {
            self.write_gs()?;
            self.write_record(i)?;
        }
        Ok(self)
    }

    fn write_file<'a, T: IntoIterator<Item = &'a Group>>(
        &mut self,
        item: T,
    ) -> io::Result<&mut Self> {
        self.check_next_char(false)?;
        let mut it = item.into_iter();
        let Some(u) = it.next() else { return Ok(self) };
        self.write_group(u)?;
        for i in it {
            self.write_fs()?;
            self.write_group(i)?;
        }
        Ok(self)
    }

    fn write_us(&mut self) -> io::Result<&mut Self> {
        self.check_next_char(true)?;
        self.next_sep = false;
        self.write_char(US)
    }

    fn write_rs(&mut self) -> io::Result<&mut Self> {
        self.check_next_char(true)?;
        self.next_sep = false;
        self.write_char(RS)
    }

    fn write_gs(&mut self) -> io::Result<&mut Self> {
        self.check_next_char(true)?;
        self.next_sep = false;
        self.write_char(GS)
    }

    fn write_fs(&mut self) -> io::Result<&mut Self> {
        self.check_next_char(true)?;
        self.next_sep = false;
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
