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
%.bc: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

# Rule to compile each C source file into LLVM 
%.ll: %.c
	$(CC) $(CFLAGS) -S -o $@ $<

clean:
	rm -f *.bc *.ll

