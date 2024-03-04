CC = clang

CFLAGS = -emit-llvm -O2

# List of C source files
SOURCES := $(wildcard *.c)

# List of LLVM bitcode files
BITCODES := $(SOURCES:.c=.bc)
LLVMTXT := $(SOURCES:.c=.ll)

.PHONY: all clean

all: $(BITCODES) $(LLVMTXT)

# Rule to compile each C source file into LLVM bitcode
%.bc: %.c target_dir
	$(CC) $(CFLAGS) -c -o ../../target/tests/$@ $<

# Rule to compile each C source file into LLVM 
%.ll: %.c target_dir
	$(CC) $(CFLAGS) -S -o ../../target/tests/$@ $<

target_dir:
	mkdir -p ../../target/tests/

clean:
	rm -f ../../target/tests/*.bc ../../target/tests/*.ll

