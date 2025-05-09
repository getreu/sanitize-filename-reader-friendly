use sanitize_filename_reader_friendly::sanitize;
use std::io;
use std::io::Read;

/// Minimal application that reads some lines from stdin and prints
/// the sanitized string to stdout.
fn main() -> Result<(), ::std::io::Error> {
    let mut buffer = String::new();
    Read::read_to_string(&mut io::stdin(), &mut buffer)?;

    let output = sanitize(&buffer);
    println!("{}", &output);
    Ok(())
}
