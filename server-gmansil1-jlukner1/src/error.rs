use cursor::CursorError;
use std::{io, str::Utf8Error};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Cursor(CursorError),
    FrameType(u8),
    Utf8(Utf8Error),
    SizeMismatch(u64),
}

impl From<CursorError> for ParseError {
    fn from(err: CursorError) -> Self {
        Self::Cursor(err)
    }
}

impl From<Utf8Error> for ParseError {
    fn from(err: Utf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl From<u64> for ParseError {
    fn from(err: u64) -> Self {
        Self::SizeMismatch(err)
    }
}

impl From<u8> for ParseError {
    fn from(err: u8) -> Self {
        Self::FrameType(err)
    }
}

#[derive(Debug)]
pub enum ReadError {
    Parse(ParseError),
    Io(io::Error),
}

impl From<ParseError> for ReadError {
    fn from(err: ParseError) -> Self {
        ReadError::Parse(err)
    }
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> Self {
        ReadError::Io(err)
    }
}
