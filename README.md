# sanitize-filename-reader-friendly

A filename sanitizer aiming to produce reader friendly filenames,
while preserving as much information as possible.

[![Cargo](https://img.shields.io/crates/v/sanitize-filename-reader-friendly.svg)](
https://crates.io/crates/sanitize-filename-reader-friendly)
[![Documentation](https://docs.rs/sanitize-filename-reader-friendly/badge.svg)](
https://docs.rs/sanitize-filename-reader-friendly)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](
https://gitlab.com/getreu/sanitize-filename-reader-friendly)

Unlike Node's [sanitize-filename], this library replaces
non-file-system-compatible characters with underscore and space. Both are
trimmed when they appear at the beginning or at the end of a line or when they
repeat within. Unprintable punctuation marks are replaced by underscores, other
unprintable characters by spaces.  Newlines are replaced by dashes.

[sanitize-filename]: https://www.npmjs.com/package/sanitize-filename

Sample usage:

```rust
extern crate sanitize_filename_reader_friendly;
use crate::sanitize_filename_reader_friendly::sanitize;

fn main() {
    println!("{}",
        sanitize("Read: http://blog.getreu.net/projects/tp-note/"));
    // Prints: "Read_ http_blog.getreu.net_projects_tp-note"
}
```

This library comes with a simple command-line application. Usage:

```bash
cargo install sanitize-filename-reader-friendly
sanitize-filename <input.txt >output.txt
```


Version 1: 

