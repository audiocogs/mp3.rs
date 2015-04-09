use std::os;
use std::env;
use std::path::Path;
use std::old_io;
use std::old_io::File;

#[macro_use]
extern crate bitflags;

mod frame;
mod header;
mod peeker;
mod bitreader;
mod layer1;

fn main() {
  let f = File::open(&Path::new(env::args()[1].clone()));

  let mut working = true;
  let mut reader = f.unwrap();

  while working {
    working = false;
    match frame::MpegFrame::read_from(&mut reader) {
      Ok(h) => match h {
        Some(h) => {
          let samples = layer1::decode_layer1(&mut reader, h.header);
          // for i in range(0, 2) {
          //   for j in range(0, 12) {
          //     for k in range(0, 32) {
          //       println!("ch = {}, sample = {}, sb = {}: {}", i, j, k, samples[i][j][k]);
          //     }
          //   }
          // }
        },
        None => {
          reader.seek(1, old_io::SeekCur).unwrap();
        }
      },
      Err(e) =>  if e.kind == old_io::EndOfFile { working = false }
    }
  }
}
