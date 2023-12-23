use crate::Utf8Error;
use std::io::Read;

use super::group::{Group, GroupToken, GroupTokeniser};

#[derive(Debug, Clone)]
pub struct File {
    groups: Vec<Group>,
}

impl File {
    pub fn groups(&self) -> &[Group] {
        &self.groups
    }
}

impl From<Vec<Group>> for File {
    fn from(value: Vec<Group>) -> Self {
        Self { groups: value }
    }
}

pub struct FileTokeniser<R: Read> {
    groups: GroupTokeniser<R>,
    file: Vec<Group>,
}

impl<R: Read> Iterator for FileTokeniser<R> {
    type Item = Result<File, Utf8Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.groups.next() {
            let token = match item {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            };
            match token {
                GroupToken::Group(g) => self.file.push(g),
                t => return Some(Ok(std::mem::replace(&mut self.file, vec![]).into())),
            }
        }

        if self.file.is_empty() {
            None
        } else {
            Some(Ok(File::from(std::mem::replace(&mut self.file, vec![]))))
        }
    }
}
