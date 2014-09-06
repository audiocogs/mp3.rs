use std::os;

use std::io;
use std::io::File;

mod frame;
mod header;
mod peeker;
mod bitreader;
mod layer1;

fn main() {
  let f = File::open(&Path::new(os::args()[1].clone()));

  let mut working = true;
  let mut reader = f.unwrap();

  while working {
    match frame::MpegFrame::read_from(&mut reader) {
      Ok(h) => match h {
        Some(h) => {
          let s = h.header.frame_size().unwrap();
          println!("{}", reader.tell());
          reader.seek(s as i64, io::SeekCur).unwrap();
          let samples = layer1::decode_layer1(&mut reader, h.header);
        },
        None => {
          reader.seek(1, io::SeekCur).unwrap();
        }
      },
      Err(e) =>  if e.kind == io::EndOfFile { working = false }
    }
  }
}
