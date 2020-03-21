mod lib;
use crate::lib::sanitize;
use std::io;
use std::io::Read;

/// Minimal application that reads some lines from stdin and prints
/// the sanitized string.
fn main() -> Result<(), ::std::io::Error> {
    let mut buffer = String::new();
    Read::read_to_string(&mut io::stdin(), &mut buffer)?;

    let output = sanitize(&buffer);
    println!("{}", &output);
    Ok(())
}
