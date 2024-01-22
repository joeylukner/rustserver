use readbuf::ReadBuf;
use std::io::{self, Cursor, Read, Write};

use crate::{error::ReadError, frame::Frame};
pub struct FrameReader<T> {
    r: T,
    buf: ReadBuf,
}

pub struct Guard<'a> {
    refer: &'a mut ReadBuf,
    bytes: usize,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.refer.consume(self.bytes);
    }
}

impl Guard<'_> {
    pub fn frame(&self) -> Frame<'_> {
        let mut src: Cursor<&[u8]> = Cursor::new(self.refer.buf());
        let res = Frame::decode(&mut src);
        res.expect("frame was checked")
    }
}

impl<R: Read> FrameReader<R> {
    pub fn new(reader: R) -> FrameReader<R> {
        FrameReader {
            r: reader,
            buf: ReadBuf::new(),
        }
    }
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.r
    }

    // pub fn read_frame(&mut self) -> Result<Frame, ReadError> {
    //     self.buf.read(&mut self.r)?;
    //     let mut src = Cursor::new(self.buf.buf());
    //     Frame::check(&mut src)?;
    //     src.set_position(0);
    //     let return_val: Frame = Frame::decode(&mut src)?;
    //     self.buf.consume(src.position() as usize);
    //     Ok(return_val)
    // }

    pub fn read_frame(&mut self) -> Result<Guard<'_>, ReadError> {
        self.buf.read(&mut self.r)?;
        let mut src = Cursor::new(self.buf.buf());
        Frame::check(&mut src)?;
        let num_bytes = src.position() as usize;

        Ok(Guard {
            refer: &mut self.buf,
            bytes: num_bytes,
        })
    }
}

pub trait WriteFrame {
    fn write_frame(&mut self, fr: &Frame) -> io::Result<()>;
}

impl<W: Write> WriteFrame for W {
    fn write_frame(&mut self, fr: &Frame) -> io::Result<()> {
        match fr {
            Frame::Simple(the_string) => {
                write!(self, "+{the_string}\r\n")?;
                Ok(())
            }
            Frame::Errors(the_error) => {
                write!(self, "-{the_error}\r\n")?;
                Ok(())
            }
            Frame::Integer(the_integer) => {
                write!(self, ":{the_integer}\r\n")?;
                Ok(())
            }
            Frame::BulkString(bulk) => {
                write!(self, "${} \r\n", bulk.len())?;
                self.write_all(bulk)?;
                write!(self, "\r\n")?;
                Ok(())
            }
            Frame::Vecs(vec) => {
                write!(self, "*{}\r\n", vec.len())?;

                for frame in vec {
                    self.write_frame(frame)?;
                }

                Ok(())
            }
            Frame::Slice(vec) => {
                write!(self, "*{}\r\n", vec.len())?;

                for frame in vec.iter() {
                    self.write_frame(frame)?;
                }

                Ok(())
            }
        }
    }
}
