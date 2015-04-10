use std::os;
use std::env;
use std::path::Path;
use std::io;
use std::fs::File;
use std::io::Seek;

#[macro_use]
extern crate bitflags;
extern crate byteorder;

mod frame;
mod header;
mod peeker;
mod bitreader;
mod layer1;

fn main() {
  let args: Vec<String> = env::args().collect();
  let path = Path::new(&args[1]);
  let f = File::open(&path);

  let mut working = true;
  let mut reader = f.unwrap();
/*
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
          reader.seek(1, io::SeekCur).unwrap();
        }
      },
      Err(e) =>  if e.kind == io::EndOfFile { working = false }
    }
  }
*/
}
