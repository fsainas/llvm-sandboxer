CC = clang

CFLAGS = -emit-llvm -O2 -fno-inline-functions

# List of C source files
SOURCES := $(wildcard *.c)

# List of LLVM bitcode files
SOURCES := $(SOURCES:.c=.o)
LLVMTXT := $(SOURCES:.c=.ll)

.PHONY: all clean

all: $(LLVMTXT) $(SOURCES)

%.o: %.c target_dir 
	$(CC) -o ../../../target/bench/no_utx/$@ $<

# Rule to compile each C source file into LLVM 
%.ll: %.c target_dir
	$(CC) $(CFLAGS) -S -o ../../../target/bench/no_utx/$@ $<

target_dir:
	mkdir -p ../../../target/bench/no_utx/

clean:
	rm -f ../../../target/bench/no_utx/*.o ../../../target/bench/no_utx/*.ll
