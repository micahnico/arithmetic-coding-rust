# Implementation of Arithmetic Coding in Rust
- mainly followed the tutorial in [this](https://marknelson.us/posts/2014/10/19/data-compression-with-arithmetic-coding.html) article by Mark Nelson
- uses a simple adaptive model to encode/compress the data
## Source Code Overview
- src/main.rs -> the entrypoint that runs the program
- src/acmodel.rs -> the actual Arithmetic Coding code including both encode and decode, aka the meat of the project
- src/compression_results.rs -> struct and associated functions for the results of the encoding process
- src/decompression_results.rs -> struct and associated functions for the results of the decoding process
## Running the Code
#### On Mac
- move executables/arithmetic-coding to the root folder
- then run that file
#### On Windows
- move executables/arithmetic-coding.exe to the root folder
- then run that file
## Notes
- the input files that the code runs are located in the files/input directory
- when the code is run:
	- the encoded (compressed) files are written to the files/encoded directory
	- the smaller decoded files are written to the files/decoded directory to make it easy to check that the code is doing what it should
