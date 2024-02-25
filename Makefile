SANDBOXER = sandboxer/target/debug/sandboxer
LLVM_BITCODE = simple_add/simple_add.bc

.PHONY: all simple_add sandboxer

all: $(LLVM_BITCODE) $(SANDBOXER)
	./$(SANDBOXER) $(LLVM_BITCODE)

$(SANDBOXER):
	@echo "Compiling sandboxer..."
	@cd sandboxer && cargo build

$(LLVM_BITCODE):
	@echo "Compiling simple_add..."
	@cd simple_add && make

clean:
	cd simple_add && make clean
	cd sandboxer && cargo clean
