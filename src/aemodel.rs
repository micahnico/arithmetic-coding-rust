use std::time::Instant;

use crate::compression_results::CompressionResults;
use crate::decompression_results::DecompressionResults;

// FreqType should have at least half the bits that ValueType has. Using the unsigned integer type that is a step down from ValueType (aka 1/2 the bits)
// results in the best case scenario for speed while making sure there can't be any overflow
// due to how TOTAL_BITS are split between CODE_VALUE_BITS and FREQ_BITS.
type ValueType = u64; // The type used to store the values throughout the calculations
type FreqType = u32; // The type used to store the frequency counts

// To ensure that we never get values that will overflow or underflow the valueType,
// we need to make sure that CODE_VALUE_BITS + FREQ_BITS <= TOTAL_BITS.
// So MAX_CODE and FREQ_BITS work together to ensure that we never get values that will overflow or underflow the ValueType.

const TOTAL_BITS: usize = std::mem::size_of::<ValueType>() * 8; // The number of bits in a ValueType
const CODE_VALUE_BITS: usize = (TOTAL_BITS / 2) + 1; // The number of bits used to store the code value
const FREQ_BITS: usize = (TOTAL_BITS / 2) - 1; // The number of bits used to store the frequency count

// This implementation can handle all ASCII characters, plus a special EOF character (so 257 characters).
// The FREQ_ARRAY_LEN index (i = 257) is used to store the upper bound for the EOF symbol and is also the total number of symbols.

const FREQ_ARRAY_LEN: usize = 258; // 256 ASCII values + 1 for EOF + 1 for total number of symbols
const TOTAL_FREQ_COUNT_IDX: usize = FREQ_ARRAY_LEN - 1; // Index of total number of symbols (aka upper bound of EOF symbol probability range)
const MAX_FREQ: FreqType = (1 << FREQ_BITS) - 1; // Max value that can be stored in FREQ_BITS bits
const EOF: u32 = 256; // End of file character

// ONE_FOURTH, ONE_HALF, and THREE_FOURTHS are calculated from MAX_CODE.
// They are used in the encode/decode functions to calculate the range and in the encode function to determine when to output bits.

const MAX_CODE: ValueType = (1 << CODE_VALUE_BITS) - 1; // Max value that can be stored in CODE_VALUE_BITS bits
const ONE_FOURTH: ValueType = (1 << CODE_VALUE_BITS) / 4; // 1/4 of MAX_CODE
const ONE_HALF: ValueType = (1 << CODE_VALUE_BITS) / 2; // 1/2 of MAX_CODE
const THREE_FOURTHS: ValueType = 3 * (1 << CODE_VALUE_BITS) / 4; // 3/4 of MAX_CODE

// Represents the probability of a symbol
struct Probability {
    start: FreqType,
    end: FreqType,
    denom: FreqType,
}

// The Arithmetic Encoding model
pub struct AEModel {
    cumulative_frequencies: [FreqType; FREQ_ARRAY_LEN],
    frozen: bool,
}

impl AEModel {
    // Create a new AEModel and initialize the cumulative frequencies
    pub fn new() -> AEModel {
        let mut m = AEModel {
            cumulative_frequencies: [0; FREQ_ARRAY_LEN],
            frozen: false,
        };
        m.reset();
        m
    }

    // Sets the cumulative frequencies to their initial values and unfreezes the model
    fn reset(&mut self) {
        for i in 0..FREQ_ARRAY_LEN {
            self.cumulative_frequencies[i] = i as FreqType;
        }
        self.frozen = false;
    }

    // Updates the cumulative frequencies for all symbols with index > c.
    // Freezes the model if the total frequency count is >= MAX_FREQ.
    fn update_cumulative_frequencies(&mut self, c: usize) {
        for i in (c + 1)..FREQ_ARRAY_LEN {
            self.cumulative_frequencies[i] += 1;
        }
        if self.cumulative_frequencies[TOTAL_FREQ_COUNT_IDX] >= MAX_FREQ {
            self.frozen = true;
        }
    }

    // Gets the probability of a symbol and updates the cumulative frequencies
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

    // Get the character and probability for the given scaled value
    fn get_char(&self, scaled_value: ValueType) -> char {
        for i in 0..TOTAL_FREQ_COUNT_IDX {
            if scaled_value < self.cumulative_frequencies[i + 1] as ValueType {
                return char::from_u32(i as u32).unwrap();
            }
        }
        char::from_u32(0).unwrap() // this line should never happen
    }

    // Write the bit and any pending bits to encoded_bits
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

    // encode the input string and return the compression results
    pub fn encode(&mut self, mut s: String) -> CompressionResults {
        self.reset();
        let start_time = Instant::now();

        let mut encoded_bits: Vec<bool> = Vec::new();
        let mut pending_bits: u32 = 0;
        let mut low: ValueType = 0;
        let mut high: ValueType = MAX_CODE;

        s.push(char::from_u32(EOF).unwrap()); // add the EOF character
        for c in s.chars() {
            let p = self.get_probability(c as usize);
            let range = high - low + 1;

            // p.start / p.denom is the lower bound of the probability range
            // p.end / p.denom is the upper bound of the probability range
            high = low + (range * (p.end as ValueType) / (p.denom as ValueType)) - 1;
            low = low + (range * (p.start as ValueType) / (p.denom as ValueType));

            loop {
                if high < ONE_HALF {
                    // both low and high start with a 0 bit, so we can output a 0 bit (and any other pending bits)
                    self.output_bit_plus_pending(&mut encoded_bits, false, pending_bits);
                    pending_bits = 0;
                } else if low >= ONE_HALF {
                    // both low and high start with a 1 bit, so we can output a 1 bit (and any other pending bits)
                    self.output_bit_plus_pending(&mut encoded_bits, true, pending_bits);
                    pending_bits = 0;
                } else if low >= ONE_FOURTH && high < THREE_FOURTHS {
                    // Avoids the near convergence problem.
                    // The bit to output is still unknown (since low and high haven't converged), so don't remove the MSB.
                    // Instead, remove the 2nd MSB by subtracting ONE_FOURTH from both low and high.
                    // Then when low and high are left shifted, the MSB is preserved (instead of being chopped off).
                    low -= ONE_FOURTH;
                    high -= ONE_FOURTH;
                    pending_bits += 1;
                } else {
                    break;
                }

                // scale up (expand) the range
                low <<= 1;
                high <<= 1;
                high += 1;

                // The next two lines of code prevent overflow by ensuring that low and high are always <= MAX_CODE.
                // Ex: In the case below, low is 34 bits long and MaxCode is 33 bits long, which could cause an overflow in future calculations.
                //     To prevent this, we chop off the leftmost bitlen(low) - bitlen(MAX_CODE) bits (in this case 34 - 33 = 1 bit).
                //           low = 0b1001100100100110100110010110011000
                //       MaxCode = 0b0111111111111111111111111111111111
                // low & MaxCode = 0b0001100100100110100110010110011000
                high &= MAX_CODE;
                low &= MAX_CODE;
            }
        }

        // Properly terminate the encoding.
        // Only 2 bits are needed to ensure that low <= value < high in the decoder.
        // if low starts with 00, we can write 01 (and any other pending bits)
        // if low starts with 01, we can write 10 (and any other pending bits)
        pending_bits += 1;
        self.output_bit_plus_pending(&mut encoded_bits, low >= ONE_FOURTH, pending_bits);

        // subtract 8 bits for the EOF character
        CompressionResults::new(s.len() * 8 - 8, encoded_bits, start_time.elapsed())
    }

    // Decode the encoded bits and return the decompression results
    pub fn decode(&mut self, bits: Vec<bool>) -> DecompressionResults {
        let eof_char = char::from_u32(EOF).unwrap();

        self.reset();
        let start_time = Instant::now();

        let mut output_bytes: Vec<u8> = Vec::new();
        let mut value: ValueType = 0;
        let mut low: ValueType = 0;
        let mut high: ValueType = MAX_CODE;

        // read the first CODE_VALUE_BITS bits into value
        for i in 0..CODE_VALUE_BITS {
            value <<= 1;
            if bits[i] {
                value += 1;
            }
        }
        let mut next_bit = CODE_VALUE_BITS;

        loop {
            let range = high - low + 1;
            // scaled_value turns the value into the cumulative frequency value we need to look for in the model's cumulative_frequencies
            // scaled_value = (distance from start of range) * (total number of symbols) / (size of range)
            let scaled_value: ValueType = ((value - low + 1)
                * (self.cumulative_frequencies[TOTAL_FREQ_COUNT_IDX] as ValueType)
                - 1)
                / range;

            // get the character and probability
            let c = self.get_char(scaled_value);
            let p = self.get_probability(c as usize);
            // check for EOF
            if c == eof_char {
                break;
            }
            // output the character
            output_bytes.push(c as u8);

            // p.start / p.denom is the lower bound of the probability range
            // p.end / p.denom is the upper bound of the probability range
            high = low + (range * (p.end as ValueType) / (p.denom as ValueType)) - 1;
            low = low + (range * (p.start as ValueType) / (p.denom as ValueType));

            loop {
                if high < ONE_HALF {
                    // do nothing (MSB for low, value, and high is a 0).
                    // won't overflow when low and high are shifted left (since MSB is a 0)
                } else if low >= ONE_HALF {
                    // MSB for low, value, and high is a 1.
                    // will overflow when low and high are shifted left (since MSB is a 1),
                    // so scale back range to prevent overflow
                    value -= ONE_HALF;
                    low -= ONE_HALF;
                    high -= ONE_HALF;
                } else if low >= ONE_FOURTH && high < THREE_FOURTHS {
                    // Avoids the near convergence problem.
                    // Remove the 2nd MSB by subtracting ONE_FOURTH from low, value, and high.
                    // This is the same operation from the encode function to keep the process the same.
                    value -= ONE_FOURTH;
                    low -= ONE_FOURTH;
                    high -= ONE_FOURTH;
                } else {
                    break;
                }

                // scale up (expand) the range
                low <<= 1;
                high <<= 1;
                high += 1;

                // read in the next bit
                // if there's no more bits, then set the next bit to 1
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
