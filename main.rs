use std::io;
use std::io::File;

mod frame;
mod header;
mod peeker;

fn main() {
  let mut f = File::open(&Path::new("layer1/fl1.mp1"));

  let mut i = 0i32;
  let mut working = true;
  let mut reader = f.unwrap();
  
  while working {
    match frame::MpegFrame::read_from(&mut reader) {
      Some(h) => {
        println!("{} at {}", h, i)

        let s = h.header.frame_size().unwrap();
        i += s as i32;
        reader.seek(s as i64, io::SeekCur);
      }
      None => {
        i += 1;
        reader.seek(1, io::SeekCur);
      }
    }
  }
}
