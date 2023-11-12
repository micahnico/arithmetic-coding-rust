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
- download the test files from the OneDrive folder and place them in the files/input directory
    - some of the smaller sizes are already there as an example
- move executables/arithmetic-coding to the root folder
- then run that file
#### On Windows
- download the test files from the OneDrive folder and place them in the files/input directory
    - some of the smaller sizes are already there as an example
- move executables/arithmetic-coding.exe to the root folder
- then run that file
## Notes
- it will take quite a while to run the code
- the input files that the code runs are located in the files/input directory
- for the first instance of each file size:
	- the encoded (compressed) file is written to the files/encoded directory
	- the decoded file is written to the files/decoded directory to make it easy to check that the code is doing what it should
