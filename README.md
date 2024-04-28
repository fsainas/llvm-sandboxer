# LLVM Sandboxer

## Modules and Functions

### Static Checks

- `static_checks.rs`: This module contains functions to perform static analysis
  of LLVM code.
    - `_is_address_protected()`: This function checks whether a given memory
      address (represented by a pointer and an offset) is protected, meaning it
      falls within a range of protected memory addresses. 
    - `verify()`: This function statically verifies the memory accesses of a
      given function to ensure they are safe. It specifically looks for
      functions named `utx1` to identify memory addresses to protect and checks
      load and store instructions for compliance.

### Runtime Instrumentation

Runtime instrumentation ensures that only protected memory addresses are
accessed during program execution.

Within the `runtime.rs` module:
- `instrument()`: It substitutes calls to `utx1()` with stores to global
  variables `@protected_ptr` and `@protected_offset`. Whenever a `Load` or
  `Store` operation is identified, it inserts checks to validate that the
  memory being accessed is safeguarded.

## Tests 

To run tests, run the following command: 
``` 
cargo test 
```

## Todo

### Static checks

- Notify bug in verify phi_0
