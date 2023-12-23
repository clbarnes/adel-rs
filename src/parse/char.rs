use std::io::Read;
use utf8_read::{Error as UError, Reader as UReader};

use super::{map_opt_res, Token};

pub enum CharToken {
    FileSeparator,
    GroupSeparator,
    RecordSeparator,
    UnitSeparator,
    Character(char),
}

impl Token for CharToken {
    fn is_separator(&self) -> bool {
        match self {
            Self::Character(_) => false,
            _ => true,
        }
    }
}

pub struct CharTokeniser<R: Read> {
    inner: UReader<R>,
}

impl<R: Read> From<R> for CharTokeniser<R> {
    fn from(value: R) -> Self {
        Self {
            inner: UReader::new(value),
        }
    }
}

impl<R: Read> Iterator for CharTokeniser<R> {
    type Item = Result<CharToken, UError>;

    fn next(&mut self) -> Option<Self::Item> {
        use CharToken::*;
        map_opt_res((&mut self.inner).next(), |item| match item {
            '\u{241C}' => FileSeparator,
            '\u{241D}' => GroupSeparator,
            '\u{241E}' => RecordSeparator,
            '\u{241F}' => UnitSeparator,
            c => Character(c),
        })
    }
}
