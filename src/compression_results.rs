use std::time::Duration;

pub struct CompressionResults {
    pub encoded_bits: Vec<bool>,
    original_bits_len: usize,
    duration: Duration,
}

impl CompressionResults {
    pub fn new(
        original_bits_len: usize,
        encoded_bits: Vec<bool>,
        duration: Duration,
    ) -> CompressionResults {
        let cr = CompressionResults {
            original_bits_len,
            encoded_bits,
            duration,
        };
        cr
    }

    pub fn encoded_bytes(&self) -> Vec<u8> {
        let encoded_bits_len = self.encoded_bits.len();
        let mut encoded_bytes = Vec::new();

        let mut curr_byte: u8 = 0;
        for i in 0..encoded_bits_len {
            if i % 8 == 7 {
                encoded_bytes.push(curr_byte);
                curr_byte = 0;
            } else {
                curr_byte = curr_byte << 1;
                if self.encoded_bits[i] {
                    curr_byte += 1; // append a 1
                }
            }
        }
        if encoded_bits_len % 8 != 0 {
            curr_byte <<= 8 - (encoded_bits_len % 8);
            encoded_bytes.push(curr_byte);
        }

        encoded_bytes
    }

    fn ratio(&self) -> f32 {
        self.encoded_bits.len() as f32 / self.original_bits_len as f32
    }

    pub fn print_info(&self, bottom_spacer: bool) {
        println!("Compression done in {} ms", self.duration.as_millis());
        println!("Compression ratio: {}", self.ratio());
        if bottom_spacer {
            println!();
        }
    }
}
