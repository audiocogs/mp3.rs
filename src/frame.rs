use std::io;

use peeker;
use header;

#[derive(Debug)]
pub struct MpegFrame {
  pub header: header::Header
}

impl MpegFrame {
  pub fn read_from(reader: &mut peeker::Peeker) -> io::IoResult<Option<MpegFrame>> {
    return match header::Header::read_from(reader) {
      Ok(h) => match h {
        Some(h) => Ok(Some(MpegFrame { header: h })),
        None => Ok(None)
      },
      Err(e) => Err(e)
    }
  }
}
