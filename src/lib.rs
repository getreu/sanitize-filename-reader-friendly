//! Converts strings in a file system friendly and human readable form.
//!
//! The algorithm replaces or deletes characters from the input stream using
//! various filters that are applied in the following sequential order:
//!
//! 1. Replace all whitespace with space.
//! 2. Filter all control characters.
//! 3. `REPLACE_ORIG_WITH_UNDERSCORE`
//! 4. `REPLACE_ORIG_WITH_SPACE`
//! 5. `FILTER_PROCESSED_AFTER_LAST_PROCESSED_WAS_SPACE`
//! 6. `FILTER_PROCESSED_AFTER_LAST_PROCESSED_WAS_UNDERSCORE`
//! 7. `FILTER_ORIG_AFTER_LAST_PROCESSED_WAS_WHITESPACE`
//! 8. `FILTER_ORIG_NON_PRINTING_CHARS`
//! 9. `TRIM_LINE`
//! 10. `INSERT_LINE_SEPARATOR`
//! 11. `TRIM_END_LINES`
//!
//! For details see the definition and documentation of the above (private) constants.
//!
//! # Rationale
//!
//! Exclude NTFS critical characters:       `<>:"\/|?*` \
//! <https://msdn.microsoft.com/en-us/library/windows/desktop/aa365247%28v=vs.85%29.aspx>
//!
//! These are considered unsafe in URLs:    `<>#%{}|\^~[]` ` \
//! <https://perishablepress.com/stop-using-unsafe-characters-in-urls/>
//!
//! New in version 2.0.0:
//! Do **not** exclude restricted in FAT32:    `+,;=[]`  \
//! <https://en.wikipedia.org/wiki/Filename#Reserved_characters_and_words>
//!
//! ```
//! use sanitize_filename_reader_friendly::sanitize;
//! let output = sanitize("Read: http://blog.getreu.net/projects/tp-note/");
//! assert_eq!(output, "Read_ http_blog.getreu.net_projects_tp-note");
//! ```
//! The output string's length is guaranteed to be shorter or equal than the input
//! string's length.

/// Start value for the algorithm. We pretend the last was just a regular letter
/// to which no `LAST_PROCESSED_WAS` rule applies.
const LAST_PROCESSED_START_CHAR: char = 'A';

/// Replace the set of quoted characters with underscore:
///
/// * Replace any whitespace by a space.
/// * Filter any control character.
/// * Replace any of the quoted characters with underscore.
const REPLACE_ORIG_WITH_UNDERSCORE: &str = r#":\/|?~"#;

/// * Replace any of the quoted characters with space.
const REPLACE_ORIG_WITH_SPACE: &str = "<>\"*#%{}^`";

/// * Filter the resulting character if it is in the quoted set and the last
///   processed character was a space.
const FILTER_PROCESSED_AFTER_LAST_PROCESSED_WAS_SPACE: &str = " ";

/// * Filter the resulting character if it is in the quoted set and the last
///   processed character was an underscore.
const FILTER_PROCESSED_AFTER_LAST_PROCESSED_WAS_UNDERSCORE: &str = "_";

/// * Filter if the current character is in the quoted set and the last
///   processed character was a whitespace. Ignore all former replacements of
///   the current character.
const FILTER_ORIG_AFTER_LAST_PROCESSED_WAS_WHITESPACE: &str = "_.\\/,;";

/// * Filter if the * current character is in the set of the quoted non printing
///   characters. Ignore all former replacements of the current character.
///
/// End of character loop.
const FILTER_ORIG_NON_PRINTING_CHARS: &str = "\u{200b}";

/// Group characters into lines (separated by newlines) and trim both sides of
/// all lines by the set of the quoted characters. In addition to the listed
/// characters whitespace is trimmed too.
const TRIM_LINE: &str = "_-.,;";

/// Insert the character below between lines.
const INSERT_LINE_SEPARATOR: char = '-';

/// Converts strings in a file system friendly and human readable form.
pub fn sanitize(s: &str) -> String {
    // This is used in a closure later.
    let mut last_processed_chr = LAST_PROCESSED_START_CHAR;

    // Proceed line by line.
    s.lines()
        .map(|l| {
            let mut s = l
                .chars()
                // Replace tab with space.
                .map(|c| if c.is_whitespace() { ' ' } else { c })
                // Delete control characters.
                .filter(|c| !c.is_control())
                .map(|c_orig| {
                    // Replace `:\\/|?~,;=` with underscore.
                    if REPLACE_ORIG_WITH_UNDERSCORE.find(c_orig).is_some() {
                        (c_orig, '_')
                    } else if REPLACE_ORIG_WITH_SPACE.find(c_orig).is_some() {
                        (c_orig, ' ')
                    } else {
                        (c_orig, c_orig)
                    }
                })
                .filter(|&(c_orig, c)| {
                    let discard = (FILTER_PROCESSED_AFTER_LAST_PROCESSED_WAS_SPACE
                        .find(c)
                        .is_some()
                        && last_processed_chr == ' ')
                        || (FILTER_PROCESSED_AFTER_LAST_PROCESSED_WAS_UNDERSCORE
                            .find(c)
                            .is_some()
                            && last_processed_chr == '_')
                        || (FILTER_ORIG_AFTER_LAST_PROCESSED_WAS_WHITESPACE
                            .find(c_orig)
                            .is_some()
                            && last_processed_chr.is_whitespace())
                        || FILTER_ORIG_NON_PRINTING_CHARS.find(c_orig).is_some();
                    if !discard {
                        last_processed_chr = c;
                    };
                    !discard
                })
                .map(|(_, c)| c)
                .collect::<String>()
                // Trim whitespace and `_-.,;` at the beginning and the end of the line.
                .trim_matches(|c: char| c.is_whitespace() || TRIM_LINE.find(c).is_some())
                .to_string();
            // Filter newline and insert line separator `-`.
            s.push(INSERT_LINE_SEPARATOR);
            s
        })
        .collect::<String>()
        // Trim whitespace and `_-.,;` at the beginning and the end of the whole string.
        .trim_matches(|c: char| c.is_whitespace() || TRIM_LINE.find(c).is_some())
        .to_string()
}
// TODO
// Should these be handled?
// RegexBuilder::new(r#"(?i)^(con|prn|aux|nul|com[0-9]|lpt[0-9])(\..*)?$"#)

#[cfg(test)]
mod tests {
    use super::sanitize;
    #[test]
    fn test_sanitize() {
        // Test filter tabs.
        assert_eq!(sanitize("\tabc\tefg\t"), "abc efg".to_string());
        // Test filter control characters.
        assert_eq!(sanitize("abc\u{0019}efg"), "abcefg".to_string());
        // Test filter special characters, replace with _.
        assert_eq!(sanitize("abc:\\/|?~=efg"), "abc_=efg".to_string());
        // Test filter special characters, replace with _.
        assert_eq!(
            sanitize("abc<>\"*<>#%{}^[]+[]`efg"),
            "abc []+[] efg".to_string()
        );
        // Test trim before and after newline.
        assert_eq!(
            sanitize("-_ \tabc \t >_-\n   efg \t_-"),
            "abc-efg".to_string()
        );
        // Test replace Unix newline.
        assert_eq!(sanitize("abc\nefg"), "abc-efg".to_string());
        // Test replace Windows newline.
        assert_eq!(sanitize("abc\r\nefg"), "abc-efg".to_string());
        // Test double '_' or ' '.
        assert_eq!(sanitize("abc_ __  efg __hij"), "abc_ efg hij".to_string());
        // Test hyperlink.
        assert_eq!(
            sanitize("https://blog.getreu.net/projects/"),
            "https_blog.getreu.net_projects".to_string()
        );
    }

    // File stem examples are taken from:
    // https://github.com/parshap/node-sanitize-filename/blob/master/test.js
    // (the extension is usually added after sanitzing the file stem.)
    static INPUT: &'static [&'static str] = &[
        "the quick brown fox jumped over the lazy dog",
        "résumé",
        "hello\u{0000}world",
        "hello\nworld",
        ";-_hello.,\n,.world_-;",
        "semi;colon",
        ";leading-semi",
        "com,ma",
        "equals=",
        "slash\\",
        "slash/",
        "col:on",
        "star*",
        "question?",
        "quote\"",
        "singlequote'",
        "brack<e>ts",
        "p|pes",
        "plus+",
        "'five and six<seven'",
        " space at front",
        "space at end ",
        ".period",
        "period.",
        "relative/path/to/some/dir",
        "/abs/path/to/some/dir",
        "~/.\u{0000}notssh/authorized_keys",
        "",
        "h?w",
        "h/w",
        "h*w",
        ".",
        "..",
        "./",
        "../",
        "/..",
        "/../",
        "*.|.",
        "./",
        "./foobar",
        "../foobar",
        "../../foobar",
        "./././foobar",
        "|*.what",
        "LPT9.asdf",
        "author| title",
        "author | title",
        "author: title",
        "auteur : titre",
        "author, title",
        "no , enumeration",
        "Any questions? Or not?",
        "Des questions ? Ou pas ?",
        "Hello!",
        "filename(1).ext",
        "1,23",
        "1.23",
        "foo\u{200b}bar",
    ];

    // Optimized for reading and keeping and much information as possible.
    // Compare with:
    // https://github.com/parshap/node-sanitize-filename/blob/master/test.js
    static EXPECTED_OUTPUT: &'static [&'static str] = &[
        "the quick brown fox jumped over the lazy dog",
        "résumé",
        "helloworld",
        "hello-world",
        "hello-world",
        "semi;colon",
        "leading-semi",
        "com,ma",
        "equals=",
        "slash",
        "slash",
        "col_on",
        "star",
        "question",
        "quote",
        "singlequote'",
        "brack e ts",
        "p_pes",
        "plus+",
        "'five and six seven'",
        "space at front",
        "space at end",
        "period",
        "period",
        "relative_path_to_some_dir",
        "abs_path_to_some_dir",
        "notssh_authorized_keys",
        "",
        "h_w",
        "h_w",
        "h w",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "foobar",
        "foobar",
        "foobar",
        "foobar",
        "what",
        "LPT9.asdf",
        "author_ title",
        "author _ title",
        "author_ title",
        "auteur _ titre",
        "author, title",
        "no enumeration",
        "Any questions_ Or not",
        "Des questions _ Ou pas",
        "Hello!",
        "filename(1).ext",
        "1,23",
        "1.23",
        "foobar",
    ];

    #[test]
    fn test_string_list() {
        for (i, s) in INPUT.iter().enumerate() {
            assert_eq!(EXPECTED_OUTPUT[i], super::sanitize(s));
        }
    }
}
