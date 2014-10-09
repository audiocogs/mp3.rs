use std::io;

use bitreader;
use header;

static SCALE_FACTORS_TABLE: [f64, ..64] = [
  2.000000000000, 1.587401051968, 1.259921049895, 1.000000000000,
  0.793700525984, 0.629960524947, 0.500000000000, 0.396850262992,
  0.314980262474, 0.250000000000, 0.198425131496, 0.157490131237,
  0.125000000000, 0.099212565748, 0.078745065618, 0.062500000000,
  0.049606282874, 0.039372532809, 0.031250000000, 0.024803141437,
  0.019686266405, 0.015625000000, 0.012401570719, 0.009843133202,
  0.007812500000, 0.006200785359, 0.004921566601, 0.003906250000,
  0.003100392680, 0.002460783301, 0.001953125000, 0.001550196340,
  0.001230391650, 0.000976562500, 0.000775098170, 0.000615195825,
  0.000488281250, 0.000387549085, 0.000307597913, 0.000244140625,
  0.000193774542, 0.000153798956, 0.000122070313, 0.000096887271,
  0.000076899478, 0.000061035156, 0.000048443636, 0.000038449739,
  0.000030517578, 0.000024221818, 0.000019224870, 0.000015258789,
  0.000012110909, 0.000009612435, 0.000007629395, 0.000006055454,
  0.000004806217, 0.000003814697, 0.000003027727, 0.000002403109,
  0.000001907349, 0.000001513864, 0.000001201554, 0.000000000000
];

static LINEAR_SCALING_TABLE: [f64, ..14] = [
  1.33333333333333, 1.14285714285714, 1.06666666666667,
  1.03225806451613, 1.01587301587302, 1.00787401574803,
  1.00392156862745, 1.00195694716243, 1.00097751710655,
  1.00048851978505, 1.00024420024420, 1.00012208521548,
  1.00006103888177, 1.00003051850948
];



pub fn decode_layer1(reader: &mut io::fs::File, frame_header: header::Header) -> Box<[[[f64, ..32], ..12], ..2]> {
  let mut bit_reader = bitreader::BitReader::new(reader);
  let nb_subbands = 32;
  let num_channels = if frame_header.channel_mode != 3 { 2 } else { 1 };

  let allocations = decode_bit_allocations(&mut bit_reader, nb_subbands, num_channels);
  let scale_factors = decode_scale_factors(&mut bit_reader, nb_subbands, num_channels, &allocations);
  let samples = decode_samples(&mut bit_reader, nb_subbands, num_channels, &allocations, &scale_factors);

  samples
}

fn decode_bit_allocations(bit_reader: &mut bitreader::BitReader, num_subbands: uint, num_channels: uint) -> Box<[[u32, ..32], ..2]> {
  let mut allocations = box [[0u32, ..32], ..2];

  for subband in range(0, num_subbands) {
    for channel in range(0, num_channels) {
      let g = bit_reader.read_bits(4);

      allocations[channel][subband] = g.unwrap();
    }
  }

  return allocations;
}

fn decode_scale_factors(bit_reader: &mut bitreader::BitReader, num_subbands: uint, num_channels: uint, allocations: &Box<[[u32, ..32], ..2]>) -> Box<[[u32, ..32], ..2]> {
  let mut scale_factors = box [[0u32, ..32], ..2];

  for subband in range(0, num_subbands) {
    for channel in range(0, num_channels) {
      let factor = if allocations[channel][subband] != 0 { bit_reader.read_bits(6).unwrap() } else { 0 };

      scale_factors[channel][subband] = factor;
    }
  }

  return scale_factors;
}

fn decode_samples(bit_reader: &mut bitreader::BitReader, num_subbands: uint, num_channels: uint, allocations: &Box<[[u32, ..32], ..2]>, scale_factors: &Box<[[u32, ..32], ..2]>) -> Box<[[[f64, ..32], ..12], ..2]> {
  let mut samples = box [[[0f64, ..32], ..12], ..2];

  for sample in range(0, 12u) {
    for subband in range(0, num_subbands) {
      for channel in range(0, num_channels) {
        let nb = allocations[channel][subband];

        samples[channel][sample][subband] = if nb > 0 {
          calculate_sample(bit_reader, nb as uint) * SCALE_FACTORS_TABLE[scale_factors[channel][subband] as uint]
        } else {
          0.0
        };
      }
    }
  }

  return samples;
}

fn calculate_sample(bit_reader: &mut bitreader::BitReader, nb: uint) -> f64 {
  match bit_reader.read_bits(nb) {
    Ok(s) => {
      let sample = (s as f64) / ((1u << nb) as f64) - 0.5;

      let table = if nb == 0 {
        LINEAR_SCALING_TABLE[0]
      } else {
        LINEAR_SCALING_TABLE[nb - 1]
      };

      return (sample as f64) * table;
    },
    Err(_) => 0.0
  }
}

#[cfg(test)]
fn generate_test_allocations() -> Box<[[u32, ..32], ..2]> {
  let buf = [0xED, 0x99, 0x88, 0x88, 0x88, 0x88, 0x77, 0x77, 0x66, 0x77, 0x55, 0x66, 0x55, 0x55, 0x55, 0x55, 0x44, 0x44, 0x44, 0x33, 0x44, 0x22, 0x33, 0x22, 0x22, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
  let mut br = io::BufReader::new(buf);
  let mut r = bitreader::BitReader::new(&mut br);

  return decode_bit_allocations(&mut r, 32, 2);
}

#[test]
fn test_bit_allocations() {
  let samples = generate_test_allocations();

  assert_eq!(samples[0][0], 0xE);
  assert_eq!(samples[0][1], 0x9);
  assert_eq!(samples[0][2], 0x8);
  assert_eq!(samples[0][3], 0x8);
  assert_eq!(samples[0][4], 0x8);
  assert_eq!(samples[0][5], 0x8);
  assert_eq!(samples[0][6], 0x7);
  assert_eq!(samples[0][7], 0x7);
  assert_eq!(samples[0][8], 0x6);
  assert_eq!(samples[0][9], 0x7);
  assert_eq!(samples[0][10], 0x5);
  assert_eq!(samples[0][11], 0x6);
  assert_eq!(samples[0][12], 0x5);
  assert_eq!(samples[0][13], 0x5);
  assert_eq!(samples[0][14], 0x5);
  assert_eq!(samples[0][15], 0x5);
  assert_eq!(samples[0][16], 0x4);
  assert_eq!(samples[0][17], 0x4);
  assert_eq!(samples[0][18], 0x4);
  assert_eq!(samples[0][19], 0x3);
  assert_eq!(samples[0][20], 0x4);
  assert_eq!(samples[0][21], 0x2);
  assert_eq!(samples[0][22], 0x3);
  assert_eq!(samples[0][23], 0x2);
  assert_eq!(samples[0][24], 0x2);
  assert_eq!(samples[0][25], 0x0);
  assert_eq!(samples[0][26], 0x0);
  assert_eq!(samples[0][27], 0x0);
  assert_eq!(samples[0][28], 0x0);
  assert_eq!(samples[0][29], 0x0);
  assert_eq!(samples[0][30], 0x0);
  assert_eq!(samples[0][31], 0x0);
  
  assert_eq!(samples[1][0], 0xD);
  assert_eq!(samples[1][1], 0x9);
  assert_eq!(samples[1][2], 0x8);
  assert_eq!(samples[1][3], 0x8);
  assert_eq!(samples[1][4], 0x8);
  assert_eq!(samples[1][5], 0x8);
  assert_eq!(samples[1][6], 0x7);
  assert_eq!(samples[1][7], 0x7);
  assert_eq!(samples[1][8], 0x6);
  assert_eq!(samples[1][9], 0x7);
  assert_eq!(samples[1][10], 0x5);
  assert_eq!(samples[1][11], 0x6);
  assert_eq!(samples[1][12], 0x5);
  assert_eq!(samples[1][13], 0x5);
  assert_eq!(samples[1][14], 0x5);
  assert_eq!(samples[1][15], 0x5);
  assert_eq!(samples[1][16], 0x4);
  assert_eq!(samples[1][17], 0x4);
  assert_eq!(samples[1][18], 0x4);
  assert_eq!(samples[1][19], 0x3);
  assert_eq!(samples[1][20], 0x4);
  assert_eq!(samples[1][21], 0x2);
  assert_eq!(samples[1][22], 0x3);
  assert_eq!(samples[1][23], 0x2);
  assert_eq!(samples[1][24], 0x2);
  assert_eq!(samples[1][25], 0x1);
  assert_eq!(samples[1][26], 0x0);
  assert_eq!(samples[1][27], 0x0);
  assert_eq!(samples[1][28], 0x0);
  assert_eq!(samples[1][29], 0x0);
  assert_eq!(samples[1][30], 0x0);
  assert_eq!(samples[1][31], 0x0);
}

#[test]
fn test_scale_factors() {
  let allocations = generate_test_allocations();
  
  let buf = [0x30, 0xC8, 0x61, 0xA6, 0x9A, 0xAA, 0xBA, 0xEB, 0x6D, 0xCB, 0x2C, 0x30, 0xD3, 0x4C, 0xB2, 0xDB, 0x6D, 0x34, 0xE3, 0x8D, 0x75, 0xDF, 0x7D, 0xF7, 0xDF, 0x7E, 0x79, 0xDF, 0x7E, 0xBA, 0xDF, 0x7E, 0xBA, 0xE3, 0x8E, 0xBA, 0xE3, 0x8E, 0xDF, 0xFF, 0xBF, 0xFE, 0xFF, 0xBF, 0xEF, 0xF7, 0xFB, 0xFD];
  let mut br = io::BufReader::new(buf);
  let mut r = bitreader::BitReader::new(&mut br);

  let samples = decode_scale_factors(&mut r, 32, 2, &allocations);

  assert_eq!(samples[0][0], 12);
  assert_eq!(samples[1][0], 12);
  assert_eq!(samples[0][1], 33);
  assert_eq!(samples[1][1], 33);
  assert_eq!(samples[0][2], 41);
  assert_eq!(samples[1][2], 41);
  assert_eq!(samples[0][3], 42);
  assert_eq!(samples[1][3], 42);
  assert_eq!(samples[0][4], 46);
  assert_eq!(samples[1][4], 46);
  assert_eq!(samples[0][5], 45);
  assert_eq!(samples[1][5], 45);
  assert_eq!(samples[0][6], 50);
  assert_eq!(samples[1][6], 50);
  assert_eq!(samples[0][7], 48);
  assert_eq!(samples[1][7], 48);
  assert_eq!(samples[0][8], 52);
  assert_eq!(samples[1][8], 52);
  assert_eq!(samples[0][9], 50);
  assert_eq!(samples[1][9], 50);
  assert_eq!(samples[0][10], 54);
  assert_eq!(samples[1][10], 54);
  assert_eq!(samples[0][11], 52);
  assert_eq!(samples[1][11], 52);
  assert_eq!(samples[0][12], 56);
  assert_eq!(samples[1][12], 56);
  assert_eq!(samples[0][13], 53);
  assert_eq!(samples[1][13], 53);
  assert_eq!(samples[0][14], 55);
  assert_eq!(samples[1][14], 55);
  assert_eq!(samples[0][15], 55);
  assert_eq!(samples[1][15], 55);
  assert_eq!(samples[0][16], 55);
  assert_eq!(samples[1][16], 55);
  assert_eq!(samples[0][17], 57);
  assert_eq!(samples[1][17], 57);
  assert_eq!(samples[0][18], 55);
  assert_eq!(samples[1][18], 55);
  assert_eq!(samples[0][19], 58);
  assert_eq!(samples[1][19], 58);
  assert_eq!(samples[0][20], 55);
  assert_eq!(samples[1][20], 55);
  assert_eq!(samples[0][21], 58);
  assert_eq!(samples[1][21], 58);
  assert_eq!(samples[0][22], 56);
  assert_eq!(samples[1][22], 56);
  assert_eq!(samples[0][23], 58);
  assert_eq!(samples[1][23], 58);
  assert_eq!(samples[0][24], 56);
  assert_eq!(samples[1][24], 56);
  assert_eq!(samples[0][25], 0);
  assert_eq!(samples[1][25], 59);
  assert_eq!(samples[0][26], 0);
  assert_eq!(samples[1][26], 0);
  assert_eq!(samples[0][27], 0);
  assert_eq!(samples[1][27], 0);
  assert_eq!(samples[0][28], 0);
  assert_eq!(samples[1][28], 0);
  assert_eq!(samples[0][29], 0);
  assert_eq!(samples[1][29], 0);
  assert_eq!(samples[0][30], 0);
  assert_eq!(samples[1][30], 0);
  assert_eq!(samples[0][31], 0);
  assert_eq!(samples[1][31], 0);
}
