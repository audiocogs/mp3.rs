use std::io;

use bitreader;

pub fn decode_layer1(reader: &mut io::Reader) {
  let mut bit_reader = bitreader::BitReader::new(reader);

  let allocations = decode_bit_allocations(&mut bit_reader, 32);
  let scale_factors = decode_scale_factors(&mut bit_reader, 32, allocations);
  let samples = decode_samples();
}

fn decode_bit_allocations(bit_reader: &mut bitreader::BitReader, size: uint) -> Vec<u32>{
  let mut allocations = Vec::new();
  let n_bits_to_read = 4;

  for i in range(0, size) {
    let n = match bit_reader.read_bits(n_bits_to_read) {
      Ok(n) => n + 1,
      Err(_) => 0
    };

    allocations.push(n);
  }

  allocations
}

fn decode_scale_factors(bit_reader: &mut bitreader::BitReader, size: uint, allocations: Vec<u32>) -> Vec<u32> {
  let mut scale_factors = Vec::new();
  let n_bits_to_read = 6;

  for i in range(0, size) {
    if allocations[i] != 0 {
      match bit_reader.read_bits(n_bits_to_read) {
        Ok(n) => scale_factors.push(n),
        Err(_) => ()
      }
    }
  }

  scale_factors
}

fn decode_samples() {}

