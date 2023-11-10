## Implementation of Arithmetic Coding in Rust
- mainly followed the tutorial in [this](https://marknelson.us/posts/2014/10/19/data-compression-with-arithmetic-coding.html) article by Mark Nelson
- uses a simple adaptive model to encode/compress the data
# Running the Code
### On Mac
- run executables/arithmetic-coding
### On Windows
- run executables/arithmetic-coding.exe
# Notes
- the input files that the code runs are located in the files/input directory
- when the code is run:
	- the encoded (compressed) files are written to the files/encoded directory
	- the smaller decoded files are written to the files/decoded directory to make it easy to check that the code is doing what it should
