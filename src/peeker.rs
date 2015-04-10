use std::io;
use std::io::SeekFrom;
use byteorder;
use byteorder::{BigEndian, ReadBytesExt};
use byteorder::Error::{UnexpectedEOF, Io};

pub trait Peeker : io::Seek + io::Read {
  fn peek_be_u32(&mut self) -> io::Result<u32>;
}

impl<T: io::Read + io::Seek> Peeker for T {
  fn peek_be_u32(&mut self) -> io::Result<u32> {
    let result = match self.read_u32::<BigEndian>() {
                   Ok(val) => Ok(val),
                   Err(Io(e)) => Err(e),
                   Err(UnexpectedEOF) => panic!("Unexpected EOF"),
                 };

    match self.seek(SeekFrom::Current(-4)) {
      Ok(_) => {}, Err(e) => return Err(e)
    };

    return result;
  }
}
