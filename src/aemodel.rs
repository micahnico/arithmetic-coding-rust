use std::time::Instant;

use crate::compression_results::CompressionResults;
use crate::decompression_results::DecompressionResults;

type ValueType = u64;

const TOTAL_BITS: usize = std::mem::size_of::<ValueType>() * 8;
const CODE_VALUE_BITS: usize = (TOTAL_BITS / 2) + 1;
const FREQ_BITS: usize = (TOTAL_BITS / 2) - 1;

const FREQ_ARRAY_LEN: usize = 258;
const TOTAL_FREQ_COUNT_IDX: usize = FREQ_ARRAY_LEN - 1;
const MAX_FREQ: ValueType = (1 << FREQ_BITS) - 1;
const EOF: u32 = 256;

const MAX_CODE: ValueType = (1 << CODE_VALUE_BITS) - 1;
const ONE_FOURTH: ValueType = (1 << CODE_VALUE_BITS) / 4;
const ONE_HALF: ValueType = (1 << CODE_VALUE_BITS) / 2;
const THREE_FOURTHS: ValueType = 3 * (1 << CODE_VALUE_BITS) / 4;

struct Probability {
    start: ValueType,
    end: ValueType,
    denom: ValueType,
}

pub struct AEModel {
    cumulative_frequencies: [ValueType; FREQ_ARRAY_LEN],
    frozen: bool,
}

impl AEModel {
    pub fn new() -> AEModel {
        let mut m = AEModel {
            cumulative_frequencies: [0; FREQ_ARRAY_LEN],
            frozen: false,
        };
        m.reset();
        m
    }

    fn reset(&mut self) {
        for i in 0..FREQ_ARRAY_LEN {
            self.cumulative_frequencies[i] = i as ValueType;
        }
        self.frozen = false;
    }

    fn update_cumulative_frequencies(&mut self, c: usize) {
        for i in (c + 1)..FREQ_ARRAY_LEN {
            self.cumulative_frequencies[i] += 1;
        }
        if self.cumulative_frequencies[TOTAL_FREQ_COUNT_IDX] >= MAX_FREQ {
            self.frozen = true;
        }
    }

    fn get_probability(&mut self, c: usize) -> Probability {
        let p = Probability {
            start: self.cumulative_frequencies[c],
            end: self.cumulative_frequencies[c + 1],
            denom: self.cumulative_frequencies[TOTAL_FREQ_COUNT_IDX],
        };
        if !self.frozen {
            self.update_cumulative_frequencies(c);
        }
        p
    }

    fn get_char(&self, scaled_value: ValueType) -> char {
        for i in 0..TOTAL_FREQ_COUNT_IDX {
            if scaled_value < self.cumulative_frequencies[i + 1] {
                return char::from_u32(i as u32).unwrap();
            }
        }
        char::from_u32(0).unwrap() // this line should never happen
    }

    fn output_bit_plus_pending(
        &mut self,
        encoded_bits: &mut Vec<bool>,
        bit: bool,
        pending_bits: u32,
    ) {
        encoded_bits.push(bit);
        for _ in 0..pending_bits {
            encoded_bits.push(!bit);
        }
    }

    pub fn encode(&mut self, mut s: String) -> CompressionResults {
        self.reset();
        let start_time = Instant::now();

        let mut encoded_bits: Vec<bool> = Vec::new();
        let mut pending_bits: u32 = 0;
        let mut low: ValueType = 0;
        let mut high: ValueType = MAX_CODE;

        s.push(char::from_u32(EOF).unwrap());
        for c in s.chars() {
            let p = self.get_probability(c as usize);
            let range = high - low + 1;

            high = low + (range * p.end / p.denom) - 1;
            low = low + (range * p.start / p.denom);

            loop {
                if high < ONE_HALF {
                    self.output_bit_plus_pending(&mut encoded_bits, false, pending_bits);
                    pending_bits = 0;
                } else if low >= ONE_HALF {
                    self.output_bit_plus_pending(&mut encoded_bits, true, pending_bits);
                    pending_bits = 0;
                } else if low >= ONE_FOURTH && high < THREE_FOURTHS {
                    low -= ONE_FOURTH;
                    high -= ONE_FOURTH;
                    pending_bits += 1;
                } else {
                    break;
                }
                low <<= 1;
                high <<= 1;
                high += 1;

                high &= MAX_CODE;
                low &= MAX_CODE;
            }
        }

        pending_bits += 1;
        if low < ONE_FOURTH {
            self.output_bit_plus_pending(&mut encoded_bits, false, pending_bits);
        } else {
            self.output_bit_plus_pending(&mut encoded_bits, true, pending_bits);
        }

        CompressionResults::new(s.len() * 8 - 8, encoded_bits, start_time.elapsed())
    }

    pub fn decode(&mut self, bits: Vec<bool>) -> DecompressionResults {
        let eof_char = char::from_u32(EOF).unwrap();

        self.reset();
        let start_time = Instant::now();

        let mut output_bytes: Vec<u8> = Vec::new();
        let mut value: ValueType = 0;
        let mut low: ValueType = 0;
        let mut high: ValueType = MAX_CODE;

        for i in 0..CODE_VALUE_BITS {
            value <<= 1;
            if bits[i] {
                value += 1;
            }
        }
        let mut next_bit = CODE_VALUE_BITS;

        loop {
            let range = high - low + 1;
            let scaled_value: ValueType =
                ((value - low + 1) * self.cumulative_frequencies[TOTAL_FREQ_COUNT_IDX] - 1) / range;

            let c = self.get_char(scaled_value);
            let p = self.get_probability(c as usize);
            if c == eof_char {
                break;
            }
            output_bytes.push(c as u8);

            high = low + (range * p.end / p.denom) - 1;
            low = low + (range * p.start / p.denom);

            loop {
                if high < ONE_HALF {
                    // do nothing
                } else if low >= ONE_HALF {
                    value -= ONE_HALF;
                    low -= ONE_HALF;
                    high -= ONE_HALF;
                } else if low >= ONE_FOURTH && high < THREE_FOURTHS {
                    value -= ONE_FOURTH;
                    low -= ONE_FOURTH;
                    high -= ONE_FOURTH;
                } else {
                    break;
                }
                low <<= 1;
                high <<= 1;
                high += 1;

                value <<= 1;
                if next_bit >= bits.len() {
                    value += 1;
                } else {
                    if bits[next_bit] {
                        value += 1;
                    }
                    next_bit += 1;
                }
            }
        }

        DecompressionResults::new(output_bytes, start_time.elapsed())
    }
}
