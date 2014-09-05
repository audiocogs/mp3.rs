use std::io;

pub trait Peeker {
  fn peek_be_u32(&mut self) -> io::IoResult<u32>;
}

impl<T: io::Reader + io::Seek> Peeker for T {
  fn peek_be_u32(&mut self) -> io::IoResult<u32> {
    let result = self.read_be_u32();

    match self.seek(-4, io::SeekCur) {
      Ok(()) => {}, Err(e) => return Err(e)
    };

    return result;
  }
}
