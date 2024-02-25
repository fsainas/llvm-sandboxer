CC = clang
LLC = llc

CFLAGS = -emit-llvm

.PHONY: all clean

all: simple_add.bc simple_add.ll

simple_add.bc: main.c
	$(CC) $(CFLAGS) -c $< -o $@

simple_add.ll: main.c
	$(CC) $(CFLAGS) -S $< -o $@

clean:
	rm -f *.bc *.ll
