use peeker;
use header;

#[deriving(Show)]
pub struct MpegFrame {
  pub header: header::Header
}

impl MpegFrame {
  pub fn read_from(reader: &mut peeker::Peeker) -> Option<MpegFrame> {
    return match header::Header::read_from(reader) {
      Some(h) => Some(MpegFrame { header: h }),
      None => None
    }
  }
}
