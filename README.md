# LLVM Sandboxer


## Tests 

To run tests, run the following command: 
``` 
cargo test 
```

## Modules and Functions

### Static Checks

- `static_checks.rs`: This module contains functions to perform static analisys
  of LLVM code.
    - `_is_address_protected()`: This function checks whether a given memory
      address (represented by a pointer and an offset) is protected, meaning it
      falls within a range of protected memory addresses. 
    - `verify()`: This function statically verifies the memory accesses of a
      given function to ensure they are safe. It specifically looks for
      functions named `utx1` to identify memory addresses to protect and checks
      load and store instructions for compliance.

#### Challenges

It's quite hard to tell which memory location a pointer is pointing to. In many
cases it's not even possible at compile time (see
[good_entry_1](tests/c_files/good_entry_1.c)).

### Runtime Instrumentation

Runtime instrumentation ensures that only protected memory addresses are
accessed during program execution.

Within the `runtime.rs` module:
- `instrument()`: It substitutes calls to `utx1()` with stores to global
  variables `@protected_ptr` and `@protected_offset`. Whenever a `Load` or
  `Store` operation is identified, it inserts checks to validate that the
  memory being accessed is safeguarded.

## Todo

### Static checks

- Write a function to compute the memory address pointed by a pointer. The
  function should return None if the address cannot be known at compile time.

- Using the function implemented in the former point, improve
  `_is_address_protected` by non-trivially checking if a pointer points inside
  a range of protected memory.
