mod acmodel;
mod compression_results;
mod decompression_results;

use std::fs::File;
use std::io::{Read, Write};

const NUM_SIZES: usize = 7;
const SIZES: [usize; NUM_SIZES] = [
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
];
const SAMPLES_PER_SIZE: usize = 10;

// Run the encoder and decoder on the input files
fn main() {
    let mut compression_ratios = [[0f32; SAMPLES_PER_SIZE]; NUM_SIZES];
    let mut runtimes = [[0u128; SAMPLES_PER_SIZE]; NUM_SIZES];

    for size in 0..NUM_SIZES {
        for sample in 0..SAMPLES_PER_SIZE {
            let file = format!("output_{}_{}.txt", SIZES[size], sample + 1);

            println!("-----------------------------------\n");
            println!("Processing {}\n", file);

            // read input from file
            let mut input_file = File::open(format!("files/input/{}_words/{}", SIZES[size], file))
                .expect("Could not open file");
            let mut input_str = String::new();
            let _ = input_file.read_to_string(&mut input_str);

            // create a new model
            let mut model = acmodel::ACModel::new();

            // encode/compress the input
            println!("Encoding...");
            let compression_result = model.encode(input_str);
            compression_ratios[size][sample] = compression_result.ratio();
            runtimes[size][sample] = compression_result.duration.as_millis();
            compression_result.print_info(true);

            // only do the following operations for the first sample of each size to save time
            if sample == 0 {
                // write the encoded data to a file
                let mut compressed_file =
                    File::create(format!("files/encoded/{}", file)).expect("Could not create file");
                let _ = compressed_file.write_all(&compression_result.encoded_bytes());

                // decode the encoded data
                println!("Decoding...");
                let decompression_result = model.decode(compression_result.encoded_bits.to_vec());
                decompression_result.print_info(true);

                // write the decoded data to a file
                let mut output_file =
                    File::create(format!("files/decoded/{}", file)).expect("Could not create file");
                let _ = output_file.write_all(&decompression_result.decoded_bytes);
            }
        }
    }

    println!("-----------------------------------\n");
    println!("Final results:\n");
    // Print the averages
    for size in 0..NUM_SIZES {
        let mut ratios_sum = 0f32;
        let mut encoding_durations_sum = 0u128;
        for sample in 0..SAMPLES_PER_SIZE {
            ratios_sum += compression_ratios[size][sample];
            encoding_durations_sum += runtimes[size][sample];
        }
        println!("{} words:", SIZES[size]);
        println!(
            "    Compression ratio: {}",
            ratios_sum / SAMPLES_PER_SIZE as f32
        );
        println!(
            "    Average encoding runtime: {} ms",
            encoding_durations_sum / SAMPLES_PER_SIZE as u128
        );
    }
    println!("\n-----------------------------------\n");

    let mut wait_for_input = String::new();
    std::io::stdin()
        .read_line(&mut wait_for_input)
        .expect("can not read user input");
}
