use std::fmt;
use std::bitflags;

use std::io;
use std::io::File;

trait Peeker {
  fn peek_be_u32(&mut self) -> io::IoResult<u32>;
}

impl<T: io::Reader + io::Seek> Peeker for T {
  fn peek_be_u32(&mut self) -> io::IoResult<u32> {
    let result = self.read_be_u32();

    self.seek(-4, io::SeekCur);

    return result;
  }
}

bitflags!(
  flags Header: u32 {
    static Sync       = 0xffe00000,
    static Version    = 0x00180000,
    static Layer      = 0x00060000,
    static CRC        = 0x00010000,
    static Bitrate    = 0x0000f000,
    static Samplerate = 0x00000c00,
    static Padding    = 0x00000200,
    static Private    = 0x00000100,
    static Channel    = 0x000000c0,
    static ChanEx     = 0x00000030,
    static Copyright  = 0x00000008,
    static Original   = 0x00000004,
    static Emphasis   = 0x00000003
  }
)

impl fmt::Show for Header {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
     return write!(f, "Header {{ sync: {}, version: {}, layer: {}, crc: {}, bitrate: {}, samplerate: {}, padding: {}, private: {}, channel_mode: {}, mode_extension: {}, copyright: {}, original: {}, emphasis: {} }}", self.contains(Sync), (self.bits & Version.bits) >> 19, (self.bits & Layer.bits) >> 17, self.contains(CRC), (self.bits & Bitrate.bits) >> 12, (self.bits & Samplerate.bits) >> 10, self.contains(Padding), self.contains(Private), (self.bits & Channel.bits) >> 6, (self.bits & ChanEx.bits) >> 4, self.contains(Copyright), self.contains(Original), self.bits & Emphasis.bits);
  }
}

#[deriving(Show)]
pub struct MpegFrame {
  header: Header
}

impl MpegFrame {
  fn read_from(reader: &mut Peeker) -> Option<MpegFrame> {
    return match reader.peek_be_u32() {
      Ok(v) => {
        let h = Header { bits: v };

        return if h.contains(Sync) { Some(MpegFrame { header: h }) } else { None };
      },
      Err(e) => None
    }
  }
}

fn main() {
  let mut f = File::open(&Path::new("layer1/fl1.mp1"));

  let mut i = 0i32;
  let mut working = true;
  let mut reader = f.unwrap();
  
  while working {
    match MpegFrame::read_from(&mut reader) {
      Some(h) => println!("{} at {}", h, i),
      None => {}
    }

    match reader.read_u8() {
      Ok(_) => i += 1, Err(_) => working = false
    };
  }
  
}
