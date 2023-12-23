use crate::Utf8Error;
use std::io::Read;

use super::unit::{Unit, UnitToken, UnitTokeniser};

#[derive(Debug, Clone)]
pub enum RecordToken {
    FileSeparator,
    GroupSeparator,
    Record(Record),
}

#[derive(Debug, Clone)]
pub struct Record {
    units: Vec<Unit>,
}

impl Record {
    pub fn units(&self) -> &[Unit] {
        &self.units
    }

    fn widths_heights(&self) -> (Vec<usize>, Vec<usize>) {
        let mut widths = Vec::with_capacity(self.units.first().map_or(0, |u| u.len()));
        let mut heights = Vec::with_capacity(self.units.len());

        for (idx, unit) in self.units.iter().enumerate() {
            let (w, h) = unit.width_height();
            heights.push(h);
            if idx >= widths.len() {
                widths.push(w);
            } else {
                widths[idx] = widths[idx].max(w)
            }
        }

        (widths, heights)
    }
}

impl From<Vec<Unit>> for Record {
    fn from(value: Vec<Unit>) -> Self {
        Self { units: value }
    }
}

pub struct RecordTokeniser<R: Read> {
    units: UnitTokeniser<R>,
    record: Vec<Unit>,
    next_item: Option<RecordToken>,
}

impl<R: Read> RecordTokeniser<R> {
    fn handle_separator(&mut self, token: UnitToken) -> RecordToken {
        self.next_item = match token {
            UnitToken::FileSeparator => Some(RecordToken::FileSeparator),
            UnitToken::GroupSeparator => Some(RecordToken::GroupSeparator),
            UnitToken::RecordSeparator => None,
            UnitToken::Unit(_) => panic!("Expected separator, got unit"),
        };
        RecordToken::Record(Record::from(std::mem::replace(&mut self.record, vec![])))
    }
}

impl<R: Read> Iterator for RecordTokeniser<R> {
    type Item = Result<RecordToken, Utf8Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = std::mem::replace(&mut self.next_item, None) {
            return Some(Ok(item));
        }

        while let Some(item) = self.units.next() {
            let token = match item {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            };
            match token {
                UnitToken::Unit(u) => self.record.push(u),
                t => return Some(Ok(self.handle_separator(t))),
            }
        }

        if self.record.is_empty() {
            None
        } else {
            Some(Ok(RecordToken::Record(Record::from(std::mem::replace(
                &mut self.record,
                vec![],
            )))))
        }
    }
}
