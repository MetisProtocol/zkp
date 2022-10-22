# Install Dependencies
## sudo apt install g++
## sudo apt install libssl-dev
## sudo apt install libboost-all-dev
## sudo apt install libjsoncpp-dev
## apt-get install libgtest-dev
## cd zkmetis
## make -j8
## make zkmetis-tests -j8
## ./zkmetis-tests .
# Test READ MIPS Instruction set
## ./zkmetis --asm ./examples-zkmips/read_test/read_test.zmips --tsteps 5 --pubtape ./examples-zkmips/read_test/read_test.pubtape --auxtape ./examples-zkmips/read_test/read_test.auxtape .
# Test Factorial MIPS instruction set
## ./zkmetis --asm ./examples-zkmips/factorial/fact.zmips --tsteps 5 --pubtape ./examples-zkmips/factorial/fact2.pubtape .

