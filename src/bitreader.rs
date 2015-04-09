use std::old_io;

pub struct BitReader<'a> {
  pub cache: u8,
  pub cache_length: u32,
  reader: &'a mut (old_io::Reader + 'a)
}

impl<'a> BitReader<'a> {
  pub fn new(reader: &'a mut old_io::Reader) -> BitReader<'a> {
    return BitReader { cache: 0, cache_length: 0, reader: reader };
  }

  pub fn read_bits(&mut self, n: u32) -> old_io::IoResult<u32> {
    if n > 32 {
      panic!("You cannot request more than 32 bits into a u32");
    }

    if n <= self.cache_length {
      let result = self.cache >> (self.cache_length - n);

      self.cache_length -= n;
      self.cache = self.cache & (0xFF >> (8 - self.cache_length));

      return Ok(result as u32);
    } else {
      let n_to_read = n - self.cache_length;
      let b_to_read = n_to_read / 8 + if n_to_read % 8 > 0 { 1 } else { 0 };

      let read = match self.reader.read_be_uint_n(b_to_read) { Ok(n) => n, Err(e) => return Err(e) };

      let sum = ((self.cache as u64) << (b_to_read * 8)) | (read as u64);

      self.cache_length = b_to_read * 8 - n_to_read;

      let result = sum >> self.cache_length;

      self.cache = (sum & (0xFF >> (8 - self.cache_length))) as u8;

      return Ok(result as u32);
    }
  }
}

#[test]
fn test_short_reads() {
  let buf = [0xFF, 0xAA, 0x44];
  let mut br = old_io::BufReader::new(buf);
  let mut r = BitReader::new(&mut br);

  assert_eq!(r.read_bits(8).unwrap(), 0xFF);
  assert_eq!(r.read_bits(4).unwrap(), 0x0A);
  assert_eq!(r.read_bits(2).unwrap(), 0x02);
  assert_eq!(r.read_bits(1).unwrap(), 0x01);
  assert_eq!(r.read_bits(1).unwrap(), 0x00);
  assert_eq!(r.read_bits(3).unwrap(), 0x02);
  assert_eq!(r.read_bits(3).unwrap(), 0x01);
  assert_eq!(r.read_bits(2).unwrap(), 0x00);

  match r.read_bits(1) { Err(e) => assert_eq!(e.kind, old_io::EndOfFile), _ => panic!("Shouldn't be here!") };
}

#[test]
fn test_medium_reads() {
  let buf = [0xFF, 0xAA, 0x44, 0xA3];
  let mut br = old_io::BufReader::new(buf);
  let mut r = BitReader::new(&mut br);

  assert_eq!(r.read_bits(16).unwrap(), 0xFFAA);
  assert_eq!(r.read_bits(12).unwrap(), 0x44A);
  assert_eq!(r.read_bits(4).unwrap(), 0x3);

  match r.read_bits(1) { Err(e) => assert_eq!(e.kind, old_io::EndOfFile), _ => panic!("Shouldn't be here!") };
}

#[test]
fn test_large_reads() {
  let buf = [0xFF, 0xAA, 0x44, 0xA3, 0x34, 0x99, 0x44];
  let mut br = old_io::BufReader::new(buf);
  let mut r = BitReader::new(&mut br);

  assert_eq!(r.read_bits(24).unwrap(), 0xFFAA44);
  assert_eq!(r.read_bits(32).unwrap(), 0xA3349944);

  match r.read_bits(1) { Err(e) => assert_eq!(e.kind, old_io::EndOfFile), _ => panic!("Shouldn't be here!") };
}

#[test]
fn test_stream() {
  let buf = [0xEA, 0xBD, 0x21];
  let mut br = old_io::BufReader::new(buf);
  let mut r = BitReader::new(&mut br);

  assert_eq!(r.read_bits(4).unwrap(), 0xE);
  assert_eq!(r.read_bits(4).unwrap(), 0xA);
  assert_eq!(r.read_bits(4).unwrap(), 0xB);
  assert_eq!(r.read_bits(4).unwrap(), 0xD);
  assert_eq!(r.read_bits(4).unwrap(), 0x2);
  assert_eq!(r.read_bits(4).unwrap(), 0x1);
}

#[test]
fn test_stream2() {
  let buf = [0x30, 0xC8, 0x61];
  let mut br = old_io::BufReader::new(buf);
  let mut r = BitReader::new(&mut br);

  assert_eq!(r.read_bits(6).unwrap(), 12);
  assert_eq!(r.read_bits(6).unwrap(), 12);
  assert_eq!(r.read_bits(6).unwrap(), 33);
  assert_eq!(r.read_bits(6).unwrap(), 33);
}
