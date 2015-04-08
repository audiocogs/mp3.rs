#![allow(dead_code)]
#![allow(non_uppercase_statics)]

use std::io;

use peeker::Peeker;

#[deriving(Show)]
#[allow(non_camel_case_types)]
pub enum MpegVersion {
  MPEG1_0,
  MPEG2_0,
  MPEG2_5,
  MPEGReserved
}

#[deriving(Show,PartialEq)]
pub enum MpegLayer {
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

bitflags! {
  flags BinaryHeader: u32 {
    const Sync                 = 0xffe00000,
    const Version              = 0x00180000,
    const Layer                = 0x00060000,
    const CRC                  = 0x00010000,
    const Bitrate              = 0x0000f000,
    const Samplerate           = 0x00000c00,
    const Padding              = 0x00000200,
    const Private              = 0x00000100,
    const ChannelMode          = 0x000000c0,
    const ChannelModeExtension = 0x00000030,
    const Copyright            = 0x00000008,
    const Original             = 0x00000004,
    const Emphasis             = 0x00000003
  }
}

#[deriving(Show)]
pub struct Header {
  pub version: MpegVersion,
  pub layer: MpegLayer,
  pub crc: bool,
  pub bitrate: Option<u32>,
  pub samplerate: Option<u32>,
  pub padding: bool,
  pub private: bool,
  pub channel_mode: u32,
  pub channel_mode_extension: u32,
  pub copyright: bool,
  pub original: bool,
  pub emphasis: u32
}

impl Header {
  pub fn read_from(reader: &mut Peeker) -> io::IoResult<Option<Header>> {
    return match reader.peek_be_u32() {
      Ok(v) => match Header::from_binary(&BinaryHeader { bits: v }) {
        Some(s) => {
          match reader.seek(if s.crc { 6 } else { 4 }, io::SeekCur) {
            Ok(()) => {}, Err(e) => return Err(e)
          };

          Ok(Some(s))
        },
        None => Ok(None)
      },
      Err(e) => Err(e)
    }
  }

  pub fn from_binary(bin: &BinaryHeader) -> Option<Header> {
    if !bin.contains(Sync) {
      return None;
    }

    let version = new_mpeg_version((bin.bits & Version.bits) >> 19);
    let layer = new_mpeg_layer((bin.bits & Layer.bits) >> 17);
    let bitrate = new_mpeg_bitrate(version, layer, (bin.bits & Bitrate.bits) >> 12);
    let samplerate = new_mpeg_samplerate(version, (bin.bits & Samplerate.bits) >> 10);

    return Some(Header {
      version: version,
      layer: layer,
      crc: !bin.contains(CRC),
      bitrate: bitrate,
      samplerate: samplerate,
      padding: bin.contains(Padding),
      private: bin.contains(Private),
      channel_mode: (bin.bits & ChannelMode.bits) >> 6,
      channel_mode_extension: (bin.bits & ChannelModeExtension.bits) >> 4,
      copyright: bin.contains(Copyright),
      original: bin.contains(Original),
      emphasis: bin.bits & Original.bits
    });
  }

  pub fn slot_size(&self) -> u32 {
    return if self.layer == LayerI { 4 } else { 1 };
  }

  pub fn frame_samples(&self) -> Option<u32> {
    return new_mpeg_frame_samples(self.version, self.layer);
  }

  pub fn frame_size(&self) -> Option<u32> {
    let b = match self.bitrate { Some(v) => v as f64, None => return None };
    let s = match self.samplerate { Some(v) => v as f64, None => return None };
    let f = match self.frame_samples() { Some(v) => v as f64, None => return None };

    let bps = 1000.0 * f as f64 / 8.0;
    let size = bps * b / s + if self.padding { self.slot_size() as f64 } else { 0.0 };

    return Some(size as u32);
  }
}
