use std::process::Command;
use std::path::Path;

/// Test rejection of unprotected memory.
/// bad_entry_0 has no call to utx0() or utx1(),
/// therefore no memory protection.
#[test]
fn test_bad_entry_0() {
    // Makefile to comiple c code
    let makefile_path = Path::new("tests/c_files/Makefile");
    let c_files_dir_path = Path::new("tests/c_files/");

    // Compile the C source file
    let output = Command::new("make")
        .args(["-C", &c_files_dir_path.to_string_lossy()])
        .output()
        .expect("Failed to compile C source file");

    // TODO: Verify bad_entry_0.bc code..
    // compare result
}
