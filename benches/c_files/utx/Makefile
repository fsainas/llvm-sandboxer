CC = clang

CFLAGS = -emit-llvm -O2 -fno-inline-functions

# List of C source files
SOURCES := $(wildcard *.c)

# List of LLVM bitcode files
BITCODES := $(SOURCES:.c=.bc)
#LLVMTXT := $(SOURCES:.c=.ll)

.PHONY: all clean

all: $(BITCODES) 

# Rule to compile each C source file into LLVM bitcode
%.bc: %.c target_dir
	$(CC) $(CFLAGS) -c -o ../../../target/bench/utx/$@ $<

# Rule to compile each C source file into LLVM 
#%.ll: %.c target_dir
#	$(CC) $(CFLAGS) -S -o ../../target/bench/utx/$@ $<

target_dir:
	mkdir -p ../../../target/bench/utx/
	mkdir -p ../../../target/bench/instrumented/

clean:
	rm -f ../../../target/bench/utx/*.bc ../../../target/bench/utx/*.ll
