mod aemodel;
mod compression_results;
mod decompression_results;

use std::fs::File;
use std::io::{Read, Write};

// Run the encoder and decoder
fn main() {
    // read input from file
    let mut input_file = File::open("files/enwik8.txt").expect("Could not open file");
    let mut input_str = String::new();
    let _ = input_file.read_to_string(&mut input_str);

    // create a new model
    let mut model = aemodel::AEModel::new();

    // encode/compress the input
    println!("Compressing...");
    let compression_result = model.encode(input_str);
    compression_result.print_info(true);

    // write the compressed data to file
    println!("Writing compressed data to file...");
    let mut compressed_file = File::create("files/compressed.txt").expect("Could not create file");
    let _ = compressed_file.write_all(&compression_result.encoded_bytes());
    println!("Writing done.\n");

    // decode the compressed data
    println!("Decompressing...");
    let decompression_result = model.decode(compression_result.encoded_bits.to_vec());
    decompression_result.print_info(true);

    // write the decoded data to output file
    println!("Writing decompressed data to file...");
    let mut output_file = File::create("files/output.txt").expect("Could not create file");
    let _ = output_file.write_all(&decompression_result.decoded_bytes);
    println!("Writing done.");
}
