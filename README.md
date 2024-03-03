# LLVM Sandboxer

## Tests 

To run tests, run the following command: 
``` 
cargo test 
```

## Modules and Functions

### `sandboxer`

#### `_is_address_protected` 

This function checks whether a given memory address (represented by a pointer
and an offset) is protected, meaning it falls within a range of protected
memory addresses. 

It takes three parameters:

- `protected_ptrs`: A reference to a vector containing tuples of protected
  memory addresses along with their offsets. 
- `ptr`: The pointer value to be checked.
- `offset`: The offset associated with the pointer value. 

#### `verify` 

This function statically verifies the memory accesses of a given function to
ensure they are safe. It specifically looks for functions named `utx1` to
identify memory addresses to protect and checks load and store instructions for
compliance.

It takes one parameter:

- `function`: The LLVM FunctionValue to be verified. 

## Challenges

It's quite hard to tell which memory location a pointer is pointing to. In many
cases it's not even possible at compile time (see
[good_entry_1](tests/c_files/good_entry_1.c)).

## What I did

- Setup the test environment.
- Parse Call, Load, Store.
- Create the data structure to keep protected addresses.
- Write a function that checks if a pointer is protected.

## Todo

- Write a function to compute the memory address pointed by a pointer. The
  function should return None if the address cannot be known at compile time.

- Using the function implemented in the former point, improve
  `_is_address_protected` by non-trivially checking if a pointer points inside
  a range of protected memory.

