# LLVM Sandboxer
To compile the LLVM Sandboxer, run the following command:
```
$ cargo build
```
This command will compile the LLVM Sandboxer and generate the executable.


## Usage
You can use the LLVM Sandboxer to remove `utx()` function calls from LLVM IR
code. Here's how to use it:
```
$ ./sandboxer <llvm bitcode file path>
```
Replace `<llvm bitcode file path>` with the path to the LLVM bitcode file
containing the code you want to modify. Please note that the file to be
processed must be in LLVM IR bitcode format (**.bc** or **.o**) and not in a
textual representation (**.ll**). 

Upon execution, the sandboxer will identify any calls to `utx()` within the
`main()` function and remove them from the LLVM IR code. The modified LLVM IR
code, without the `utx()` call, will be printed to the standard output.
