use crate::error::ParseError;
use cursor::{byte, integer, line, size, slice};

use std::io::Cursor;

use std::str::from_utf8;

#[derive(Debug, PartialEq)]
pub enum Frame<'a> {
    Simple(&'a str),
    Errors(&'a str),
    Integer(i64),
    BulkString(&'a [u8]),
    Vecs(Vec<Frame<'a>>),
    Slice(&'a [Frame<'a>]),
}

impl<'a> Frame<'a> {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), ParseError> {
        match byte(src)? {
            b'+' => {
                let slice: &[u8] = line(src)?;
                let _validated: &str = from_utf8(slice)?;
            }
            b'-' => {
                let slice: &[u8] = line(src)?;
                let _validated: &str = from_utf8(slice)?;
            }
            b':' => {
                integer(src)?;
            }
            b'$' => {
                let bulk_size = size(src)?;
                let _bulk_string = slice(src, bulk_size)?;
                // let bulk_slice = match slice(src, bulk_size) {
                //     Ok(bulk_slice) => bulk_slice,
                //     Err(E) => return(Err(E.into())),
                // };
                let check = slice(src, 2)?;

                // let line_slice:&[u8] = line(src)?;
                // let borrowed_string: &str = from_utf8(line_slice)?;
                // let owned_string: String = borrowed_string.to_string();
                if check != "\r\n".as_bytes() {
                    return Err(ParseError::SizeMismatch(bulk_size));
                }
                //else correct length
            }
            b'*' => {
                let array_size = size(src)?;
                let mut counter = 0;
                for _ in 0..array_size {
                    Frame::check(src)?;
                    counter += 1;
                }
                // unsigned so will not be negative
                if counter != array_size {
                    return Err(ParseError::SizeMismatch(array_size));
                }
            }
            invalidbyte => {
                return Err(ParseError::FrameType(invalidbyte));
            } //throw frametype error
        }
        Ok(())
    }

    pub fn decode(src: &mut Cursor<&'a [u8]>) -> Result<Self, ParseError> {
        match byte(src)? {
            b'+' => {
                let slice: &[u8] = line(src)?;
                let _validated: &str = from_utf8(slice)?;
                Ok(Frame::Simple(_validated))
            }
            b'-' => {
                let slice: &[u8] = line(src)?;
                let _validated: &str = from_utf8(slice)?;
                Ok(Frame::Errors(_validated))
            }
            b':' => Ok(Frame::Integer(integer(src)?)),
            b'$' => {
                let slice_size = size(src)?;
                //compare size to length of slice
                //line will advance to the end of the bulk string

                let line_slice: &[u8] = line(src)?;
                if line_slice.len() as u64 != slice_size {
                    return Err(ParseError::SizeMismatch(slice_size));
                } //else correct length

                Ok(Frame::BulkString(line_slice))
            }
            b'*' => {
                let array_size = size(src)?;
                let mut counter = 0;
                let mut ret: Vec<Frame> = Vec::new();
                for _ in 0..array_size {
                    ret.push(Frame::decode(src)?);
                    counter += 1;
                }
                if counter != array_size {
                    return Err(ParseError::SizeMismatch(array_size));
                }
                Ok(Frame::Vecs(ret))
            }
            invalidbyte => Err(ParseError::FrameType(invalidbyte)), //throw frametype error
        }
    }
    pub fn as_array(&self) -> Option<&[Frame<'a>]> {
        match self {
            Frame::Vecs(vec) => Some(vec),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rw::FrameReader;

    use super::*;

    #[test]
    fn test_check() {
        let mut src = Cursor::new(":100\r\n".as_bytes());

        assert!(Frame::check(&mut src).is_ok());
    }

    #[test]
    fn string_check() {
        let mut src = Cursor::new("+I love Rust\r\n".as_bytes());

        assert!(Frame::check(&mut src).is_ok());
    }
    #[test]
    fn error_check() {
        let mut src = Cursor::new("-NOT FOUND\r\n".as_bytes());

        assert!(Frame::check(&mut src).is_ok());
    }
    #[test]
    fn integer_check() {
        let mut src = Cursor::new(":123\r\n".as_bytes());

        assert!(Frame::check(&mut src).is_ok());
    }
    #[test]
    fn bulk_string_check() {
        let mut src = Cursor::new("$4\r\nƒoo\r\n".as_bytes());

        assert!(Frame::check(&mut src).is_ok());
    }
    #[test]
    fn arr_check() {
        let mut src = Cursor::new("*2\r\n+hello\r\n:100\r\n".as_bytes());

        assert!(Frame::check(&mut src).is_ok());
    }

    #[test]
    fn invalid_frame() {
        let mut src = Cursor::new("<100\r\n".as_bytes());

        assert!(Frame::check(&mut src).is_err());
    }

    //decode tests
    #[test]
    fn string_decode_check() {
        let mut src = Cursor::new("+I love Rust\r\n".as_bytes());

        assert!(Frame::decode(&mut src).is_ok());
    }
    #[test]
    fn error_decode_check() {
        let mut src = Cursor::new("-NOT FOUND\r\n".as_bytes());

        assert!(Frame::decode(&mut src).is_ok());
    }
    #[test]
    fn integerdecode_check() {
        let mut src = Cursor::new(":123\r\n".as_bytes());

        assert!(Frame::decode(&mut src).is_ok());
    }
    #[test]
    fn bulk_stringdecode_check() {
        let mut src = Cursor::new("$4\r\naaoo\r\n".as_bytes());

        assert!(Frame::decode(&mut src).is_ok());
    }
    #[test]
    fn arrdecode_check() {
        let mut src = Cursor::new("*2\r\n+hello\r\n:100\r\n".as_bytes());

        assert!(Frame::decode(&mut src).is_ok());
    }

    #[test]
    fn invaliddecode_frame() {
        let mut src = Cursor::new("<100\r\n".as_bytes());

        assert!(Frame::decode(&mut src).is_err());
    }
    #[test]
    fn test_read_frame() {
        let data = "+Hello, world!\r\n".as_bytes();

        // &[u8] implements `Read`
        let mut reader = FrameReader::new(data);
        let result = reader.read_frame().unwrap();
        assert_eq!(result.frame(), Frame::Simple("Hello, world!"));
    }

    #[test]
    fn test_read_frame_2() {
        let data = ":100\r\n".as_bytes();

        // &[u8] implements `Read`
        let mut reader = FrameReader::new(data);
        let result = reader.read_frame().unwrap();
        assert_eq!(result.frame(), Frame::Integer(100));
    }

    #[test]
    fn test_read_frame_3() {
        let data = "-NOT FOUND\r\n".as_bytes();

        // &[u8] implements `Read`
        let mut reader = FrameReader::new(data);
        let result = reader.read_frame().unwrap();
        assert_eq!(result.frame(), Frame::Errors("NOT FOUND"));
    }

    #[test]
    fn test_read_frame_4() {
        let data = "$4\r\nafoo\r\n".as_bytes();

        // &[u8] implements `Read`
        let mut reader = FrameReader::new(data);
        let result = reader.read_frame().unwrap();
        let control = "afoo".as_bytes();
        assert_eq!(result.frame(), Frame::BulkString(control));
    }

    #[test]
    fn test_read_frame_5() {
        let data = "*2\r\n+hello\r\n:100\r\n".as_bytes();

        // &[u8] implements `Read`
        let mut reader = FrameReader::new(data);
        let result = reader.read_frame().unwrap();
        let vec: Vec<Frame> = vec![Frame::Simple("hello"), Frame::Integer(100)];
        assert_eq!(result.frame(), Frame::Vecs(vec));
    }
}
