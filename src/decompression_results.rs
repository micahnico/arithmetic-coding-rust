use std::time::Duration;

pub struct DecompressionResults {
    pub decoded_bytes: Vec<u8>,
    duration: Duration,
}

impl DecompressionResults {
    pub fn new(decoded_bytes: Vec<u8>, duration: Duration) -> DecompressionResults {
        DecompressionResults {
            decoded_bytes,
            duration,
        }
    }

    pub fn print_info(&self, bottom_spacer: bool) {
        println!("Decompression done in {} ms", self.duration.as_millis());
        if bottom_spacer {
            println!();
        }
    }
}
