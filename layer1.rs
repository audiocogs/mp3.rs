use std::io;

use bitreader;

static scale_factors_table: [f64, ..64] = [
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

// linear scaling table
static linear_table: [f64, ..14] = [
  1.33333333333333, 1.14285714285714, 1.06666666666667,
  1.03225806451613, 1.01587301587302, 1.00787401574803,
  1.00392156862745, 1.00195694716243, 1.00097751710655,
  1.00048851978505, 1.00024420024420, 1.00012208521548,
  1.00006103888177, 1.00003051850948
];



pub fn decode_layer1(reader: &mut io::Reader) -> Vec<f64> {
  let mut bit_reader = bitreader::BitReader::new(reader);
  let nb_subbands = 32;

  let allocations = decode_bit_allocations(&mut bit_reader, nb_subbands);
  let scale_factors = decode_scale_factors(&mut bit_reader, nb_subbands, &allocations);
  let samples = decode_samples(&mut bit_reader, nb_subbands, &allocations, scale_factors);
  samples
}

fn decode_bit_allocations(bit_reader: &mut bitreader::BitReader, nb_subbands: uint) -> Vec<u32>{
  let mut allocations = Vec::new();
  let n_bits_to_read = 4;

  for i in range(0, nb_subbands) {
    let n = match bit_reader.read_bits(n_bits_to_read) {
      Ok(0) => 0,
      Ok(15) => 15,
      Ok(n) => n + 1,
      Err(_) => 0
    };

    allocations.push(n);
  }

  allocations
}

fn decode_scale_factors(bit_reader: &mut bitreader::BitReader, nb_subbands: uint, allocations: &Vec<u32>) -> Vec<u32> {
  let mut scale_factors = Vec::new();
  let n_bits_to_read = 6;

  for i in range(0, nb_subbands) {
    let factor = if allocations[i] != 0 {
      match bit_reader.read_bits(n_bits_to_read) {
        Ok(n) => n,
        Err(_) => 0
      }
    }else{ 0 };
    scale_factors.push(factor)
  }

  scale_factors
}

fn decode_samples(bit_reader: &mut bitreader::BitReader, nb_subbands: uint, allocations: &Vec<u32>, scale_factors: Vec<u32>) -> Vec<f64> {
  let mut samples = Vec::new();
  let nb_samples = 12i;

  for i in range(0, nb_samples) {
    for j in range(0, nb_subbands) {
      let allocation = allocations[j];
      let sample = if allocation == 0 {
        0f64
      }else{
        sample(bit_reader, allocation as uint) * scale_factors_table[scale_factors[j] as uint]
      };

      samples.push(sample);
    }
  }

  samples
}

fn sample(bit_reader: &mut bitreader::BitReader, nb: uint) -> f64 {
  match bit_reader.read_bits(nb) {
    Ok(s) => {
      let mut sample = s;
      // invert most significant bit, and form a 2's complement sample
      sample ^= 1 << (nb - 1);
      sample |= -(sample & (1 << (nb - 1)));
      sample /= (1 << (nb - 1));

      // requantize the sample
      // s'' = (2^nb / (2^nb - 1)) * (s''' + 2^(-nb + 1))
      sample += 1 >> (nb - 1);
      return (sample as f64) * linear_table[nb - 2];
    },
    Err(_) => 0.0
  }

}

