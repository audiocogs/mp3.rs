use std::io;
use std::io::SeekFrom;

pub trait Peeker : io::Seek + io::Read {
  fn peek_be_u32(&mut self) -> io::Result<u32>;
}

impl<T: io::Read + io::Seek> Peeker for T {
  fn peek_be_u32(&mut self) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    match self.read(&mut buf) {
      Ok(_) => {},
      Err(e) => return Err(e),
    };

    match self.seek(SeekFrom::Current(-4)) {
      Ok(_) => {}, Err(e) => return Err(e)
    };

    let mut return_value = 0u32;
    for idx in 0..4 {
      return_value = (return_value << 8) + (buf[idx] as u32);
    }
    Ok(return_value)
  }
}

#[test]
fn test_peek_in_small_buf() {
  let mut buf = Cursor::new(vec![0xFFu8, 0xAA, 0x44, 0xA3]);
  let mut pkr = Box::new(buf) as Box<Peeker>;
  assert_eq!(pkr.peek_be_u32().unwrap(), 0xFFAA44A3);
  assert_eq!(pkr.peek_be_u32().unwrap(), 0xFFAA44A3);
}

#[test]
fn test_peek_in_medium_buf() {
  let mut buf = Cursor::new(vec![0xFFu8, 0xAA, 0x44, 0xA3, 0x34, 0x99, 0x44]);
  let mut pkr = Box::new(buf) as Box<Peeker>;
  assert_eq!(pkr.peek_be_u32().unwrap(), 0xFFAA44A3);
  assert_eq!(pkr.peek_be_u32().unwrap(), 0xFFAA44A3);
}
