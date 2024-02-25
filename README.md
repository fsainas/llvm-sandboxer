# LLVM Sandboxer
At this stage, the LLVM Sandboxer is expected to effectively identify a function
call, specifically `utx()`, remove it from the code, and subsequently generate
the updated LLVM IR code without its presence.

## Structure
- [`sandboxer/`](./sandboxer): This directory contains the LLVM
  Sandboxer, which is written in Rust.
- [`simple_add/`](./simple_add): This directory contains a
  simple C code that contains a call to a function named
  `utx()`. It is compiled into LLVM IR code.

## How to compile
To compile all the components and run a simple demonstration use the following
command:
```
$ make
```
This will run the sandboxer on the compiled `simple_add` example. More
information [here](.sandboxer/README.md).
