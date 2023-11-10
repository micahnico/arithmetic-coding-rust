mod aemodel;
mod compression_results;
mod decompression_results;

use std::fs::File;
use std::io::{Read, Write};

// Run the encoder and decoder on the input files
fn main() {
    let files = [
        "input.txt",
        "input2.txt",
        "input3.txt",
        "input4.txt",
        "input5.txt",
        "input6.txt",
        "input7.txt",
        "input8.txt",
        "Bible.txt",
        "enwik8.txt",
        // "enwik9.txt",
    ];

    for (i, file) in files.iter().enumerate() {
        println!("-----------------------------------\n");
        println!("Processing {}\n", file);

        // read input from file
        let mut input_file =
            File::open(format!("files/input/{}", file)).expect("Could not open file");
        let mut input_str = String::new();
        let _ = input_file.read_to_string(&mut input_str);

        // create a new model
        let mut model = aemodel::AEModel::new();

        // encode/compress the input
        println!("Encoding...");
        let compression_result = model.encode(input_str);
        compression_result.print_info(true);

        // write the compressed data to file
        let mut compressed_file =
            File::create(format!("files/encoded/{}", file)).expect("Could not create file");
        let _ = compressed_file.write_all(&compression_result.encoded_bytes());

        // decode the encoded data
        println!("Decoding...");
        let decompression_result = model.decode(compression_result.encoded_bits.to_vec());
        decompression_result.print_info(true);

        // Write the decoded data to output file.
        // Don't write the largest files. This is just so the output can be easily checked to see
        // if the code is actually working.
        if i < 8 {
            let mut output_file =
                File::create(format!("files/decoded/{}", file)).expect("Could not create file");
            let _ = output_file.write_all(&decompression_result.decoded_bytes);
        }
    }
}
