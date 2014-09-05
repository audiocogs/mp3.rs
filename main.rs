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

  let mut i = 0i32;
  let mut working = true;
  let mut reader = f.unwrap();

  let mut out = std::io::stdio::stdout();

  while working {
    match frame::MpegFrame::read_from(&mut reader) {
      Ok(h) => match h {
        Some(h) => {
          //println!("{} at {}", h, i);

          let s = h.header.frame_size().unwrap();
          i += s as i32;
          reader.seek(s as i64, io::SeekCur).unwrap();
          let samples = layer1::decode_layer1(&mut reader);
          for sample in samples.iter() {
              //out.write_be_f64(*sample);
          }
        },
        None => {
          i += 1;
          reader.seek(1, io::SeekCur).unwrap();
        }
      },
      Err(e) =>  if e.kind == io::EndOfFile { working = false }
    }
  }
}
