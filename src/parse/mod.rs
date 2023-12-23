use std::error::Error;
pub use utf8_read::Error as UError;

mod char;
mod file;
mod group;
mod record;
mod unit;

pub use self::unit::Unit;

pub trait Token {
    fn is_separator(&self) -> bool;
}

fn map_opt_res<I, O, E: Error>(
    value: Option<Result<I, E>>,
    mut f: impl FnMut(I) -> O,
) -> Option<Result<O, E>> {
    match value? {
        Ok(i) => Some(Ok(f(i))),
        Err(e) => Some(Err(e)),
    }
}
