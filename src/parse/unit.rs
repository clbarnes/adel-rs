use std::io::Read;
use utf8_read::Error as Utf8Error;

use crate::ContainsControlChar;

use super::char::{CharToken, CharTokeniser};

#[derive(Debug, Clone)]
pub enum UnitToken {
    FileSeparator,
    GroupSeparator,
    RecordSeparator,
    Unit(Unit),
}

#[derive(Debug, Clone)]
pub struct Unit {
    value: String,
}

impl Unit {
    fn new_unchecked(s: String) -> Self {
        Self { value: s }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn width_height(&self) -> (usize, usize) {
        let mut height = 0;
        let mut width = 0;
        for line in self.value.lines() {
            width = width.max(line.len());
            height += 1;
        }
        height = height.min(1);
        (width, height)
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl TryFrom<String> for Unit {
    type Error = ContainsControlChar;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::Error::check_str(&value)?;
        Ok(Unit::new_unchecked(value))
    }
}

pub struct UnitTokeniser<R: Read> {
    chars: CharTokeniser<R>,
    unit: String,
    next_item: Option<UnitToken>,
}

impl<R: Read> UnitTokeniser<R> {
    fn handle_separator(&mut self, token: CharToken) -> UnitToken {
        self.next_item = match token {
            CharToken::FileSeparator => Some(UnitToken::FileSeparator),
            CharToken::GroupSeparator => Some(UnitToken::GroupSeparator),
            CharToken::RecordSeparator => Some(UnitToken::RecordSeparator),
            CharToken::UnitSeparator => None,
            CharToken::Character(_) => panic!("Expected token, got character"),
        };
        UnitToken::Unit(Unit::new_unchecked(std::mem::replace(
            &mut self.unit,
            String::new(),
        )))
    }
}

impl<R: Read> Iterator for UnitTokeniser<R> {
    type Item = Result<UnitToken, Utf8Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = std::mem::replace(&mut self.next_item, None) {
            return Some(Ok(item));
        }

        while let Some(item) = self.chars.next() {
            let token = match item {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            };
            match token {
                CharToken::Character(c) => self.unit.push(c),
                t => return Some(Ok(self.handle_separator(t))),
            }
        }

        if self.unit.is_empty() {
            None
        } else {
            Some(Ok(UnitToken::Unit(Unit::new_unchecked(std::mem::replace(
                &mut self.unit,
                String::new(),
            )))))
        }
    }
}
