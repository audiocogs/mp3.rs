use std::old_io;

pub trait Peeker : old_io::Seek + old_io::Reader {
  fn peek_be_u32(&mut self) -> old_io::IoResult<u32>;
}

impl<T: old_io::Reader + old_io::Seek> Peeker for T {
  fn peek_be_u32(&mut self) -> old_io::IoResult<u32> {
    let result = self.read_be_u32();

    match self.seek(-4, old_io::SeekCur) {
      Ok(()) => {}, Err(e) => return Err(e)
    };

    return result;
  }
}
