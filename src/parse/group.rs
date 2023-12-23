use crate::Utf8Error;
use std::io::Read;

use super::record::{Record, RecordToken, RecordTokeniser};

#[derive(Debug, Clone)]
pub enum GroupToken {
    FileSeparator,
    Group(Group),
}

#[derive(Debug, Clone)]
pub struct Group {
    records: Vec<Record>,
}

impl Group {
    pub fn records(&self) -> &[Record] {
        &self.records
    }
}

impl From<Vec<Record>> for Group {
    fn from(value: Vec<Record>) -> Self {
        Self { records: value }
    }
}

pub struct GroupTokeniser<R: Read> {
    records: RecordTokeniser<R>,
    group: Vec<Record>,
    next_item: Option<GroupToken>,
}

impl<R: Read> GroupTokeniser<R> {
    fn handle_separator(&mut self, token: RecordToken) -> GroupToken {
        self.next_item = match token {
            RecordToken::FileSeparator => Some(GroupToken::FileSeparator),
            RecordToken::GroupSeparator => None,
            RecordToken::Record(_) => panic!("Expected separator, got record"),
        };
        GroupToken::Group(Group::from(std::mem::replace(&mut self.group, vec![])))
    }
}

impl<R: Read> Iterator for GroupTokeniser<R> {
    type Item = Result<GroupToken, Utf8Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = std::mem::replace(&mut self.next_item, None) {
            return Some(Ok(item));
        }

        while let Some(item) = self.records.next() {
            let token = match item {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            };
            match token {
                RecordToken::Record(r) => self.group.push(r),
                t => return Some(Ok(self.handle_separator(t))),
            }
        }

        if self.group.is_empty() {
            None
        } else {
            Some(Ok(GroupToken::Group(Group::from(std::mem::replace(
                &mut self.group,
                vec![],
            )))))
        }
    }
}
