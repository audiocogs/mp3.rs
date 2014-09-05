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

#[deriving(Show)]
enum MpegVersion {
  MPEG1_0,
  MPEG2_0,
  MPEG2_5,
  MPEGReserved
}

#[deriving(Show)]
enum MpegLayer {
  LayerI,
  LayerII,
  LayerIII,
  LayerReserved
}


fn new_mpeg_version(i: u32) -> MpegVersion {
  match i {
    0x0 => MPEG2_5, 0x2 => MPEG2_0, 0x3 => MPEG1_0, _ => MPEGReserved
  }
}

fn new_mpeg_layer(i: u32) -> MpegLayer {
  match i {
    0x3 => LayerI, 0x2 => LayerII, 0x1 => LayerIII, _ => LayerReserved
  }
}

fn new_mpeg_bitrate(v: MpegVersion, l: MpegLayer, bits: u32) -> Option<u32> {
  if bits == 0 {
    return None; /* Free bitrate */
  }
  
  if bits == 16 {
    return None;
  }

  return Some(match v {
    MPEG1_0 => {
      match l {
        LayerI => 32 * bits,
        LayerII => match bits {
          1 => 32,
          2 => 48,
          3 => 56,
          4 => 64,
          5 => 80,
          6 => 96,
          7 => 112,
          8 => 128,
          9 => 160,
          10 => 192,
          11 => 224,
          12 => 256,
          13 => 320,
          14 => 384,
          _ => return None
        },
        LayerIII => match bits {
          1 => 32,
          2 => 40,
          3 => 48,
          4 => 56,
          5 => 64,
          6 => 80,
          7 => 96,
          8 => 112,
          9 => 128,
          10 => 160,
          11 => 192,
          12 => 224,
          13 => 256,
          14 => 320,
          _ => return None
        },
        _ => return None
      }
    }
    MPEG2_0 | MPEG2_5 => match l {
      LayerI => match bits {
        1 => 32,
        2 => 48,
        3 => 56,
        4 => 64,
        5 => 80,
        6 => 96,
        7 => 112,
        8 => 128,
        9 => 144,
        10 => 160,
        11 => 176,
        12 => 192,
        13 => 224,
        14 => 256,
        _ => return None
      },
      LayerII | LayerIII => match bits {
        1 => 8,
        2 => 16,
        3 => 24,
        4 => 32,
        5 => 40,
        6 => 48,
        7 => 56,
        8 => 64,
        9 => 80,
        10 => 96,
        11 => 112,
        12 => 128,
        13 => 144,
        14 => 160,
        _ => return None
      },
      _ => return None
    },
    _ => return None
  })
}

fn new_mpeg_samplerate(v: MpegVersion, bits: u32) -> Option<u32> {
  return Some(match v {
    MPEG1_0 => match bits {
      0 => 44100, 1 => 48000, 2 => 32000, _ => return None
    },
    MPEG2_0 => match bits {
      0 => 22050, 1 => 24000, 2 => 16000, _ => return None
    },
    MPEG2_5 => match bits {
      0 => 11025, 1 => 12000, 2 => 8000, _ => return None
    },
    _ => return None
  });
}

fn new_mpeg_frame_samples(v: MpegVersion, l: MpegLayer) -> Option<u32> {
  return Some(match v {
    MPEG1_0 => match l {
      LayerI => 384, LayerII => 1152, LayerIII => 1152, _ => return None
    },
    MPEG2_0 => match l {
      LayerI => 384, LayerII => 1152, LayerIII => 576, _ => return None
    },
    MPEG2_5 => match l {
      LayerI => 384, LayerII => 1152, LayerIII => 576, _ => return None
    },
    _ => return None
  });
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
    let version = new_mpeg_version((self.bits & Version.bits) >> 19);
    let layer = new_mpeg_layer((self.bits & Layer.bits) >> 17);
    let bitrate = new_mpeg_bitrate(version, layer, (self.bits & Bitrate.bits) >> 12);
    let samplerate = new_mpeg_samplerate(version, (self.bits & Samplerate.bits) >> 10);
    let frame_samples = new_mpeg_frame_samples(version, layer);

    let b = bitrate.unwrap();
    let f = frame_samples.unwrap();
    let s = samplerate.unwrap();

    let length = (f / 8u32) * b / s + 0;
    return write!(f, "Header {{ sync: {}, version: {}, layer: {}, crc: {}, bitrate: {}, samplerate: {}, frame_samples: {}, padding: {}, private: {}, channel_mode: {}, mode_extension: {}, copyright: {}, original: {}, emphasis: {}, length: {} }}", self.contains(Sync), version, layer, self.contains(CRC), bitrate, samplerate, frame_samples, self.contains(Padding), self.contains(Private), (self.bits & Channel.bits) >> 6, (self.bits & ChanEx.bits) >> 4, self.contains(Copyright), self.contains(Original), self.bits & Emphasis.bits, length as uint);
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
      Some(h) => {
        println!("{} at {}", h, i)

        i += 300;
        reader.seek(300, io::SeekCur);
      }
      None => {
        i += 1;
        reader.seek(1, io::SeekCur);
      }
    }
  }
}
