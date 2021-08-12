/// Converts strings in a file system friendly and human readable form.
///
/// * Replace tab with one space.
/// * Filter control characters.
/// * Replace `:\\/|?~` with underscore.
/// * Replace `<>:"#%{}^\`` with space.
/// * Filter replaced space after replaced space.
/// * Filter period after period, replaced space, replaced underscore or at the beginning of string.
/// * Filter replaced underscore after replaced underscore.
/// * Filter `_.\/,;` after whitespace.
/// * Filter non-printing space `U+200b`.
/// * Trim whitespace and `_-.,;` at the beginning and the end of the line.
/// * Filter newline and insert line separator `-`.
/// * Trim whitespace and `_-.,;` at the beginning and the end of the whole string.
///
/// ```
/// use sanitize_filename_reader_friendly::sanitize;
/// let output = sanitize("Read: http://blog.getreu.net/projects/tp-note/");
/// assert_eq!(output, "Read_ http_blog.getreu.net_projects_tp-note");
/// ```
/// The output string's length is guaranteed to be shorter or equal than the input
/// string's length.
///
/// Change log:
///
/// * Version 2.0.0: drop FAT32 restrictions and allow: `+,;=[]`
///                  (the output is stays eFAT compatible).

pub fn sanitize(s: &str) -> String {
    // This is used in a closure later.
    // To avoid the period as first character, we pretend that there had been
    // a period already.
    let mut last_replaced_chr = '.';

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
                    //
                    // Exclude NTFS critical characters:       `<>:"\\/|?*`
                    // https://msdn.microsoft.com/en-us/library/windows/desktop/aa365247%28v=vs.85%29.aspx
                    // New in version 2.0.0:
                    // Do **not** exclude restricted in FAT32:    `+,;=[]`
                    // https://en.wikipedia.org/wiki/Filename#Reserved_characters_and_words
                    // These are considered unsafe in URLs:    `<>#%{}|\^~[]\``
                    // https://perishablepress.com/stop-using-unsafe-characters-in-urls/
                    if c_orig == ':'
                        || c_orig == '\\'
                        || c_orig == '/'
                        || c_orig == '|'
                        || c_orig == '?'
                        || c_orig == '~'
                    {
                        (c_orig, '_')
                    } else if
                    // Replace `<>:"#%{}^[]+\`` with space.
                    //
                    // Exclude NTFS critical characters:       `<>:"\\/|?*`
                    // https://msdn.microsoft.com/en-us/library/windows/desktop/aa365247%28v=vs.85%29.aspx
                    // Do **not** exclude restricted in fat32: `+,;=[]`
                    // https://en.wikipedia.org/wiki/Filename#Reserved_characters_and_words
                    // These are considered unsafe in URLs:    `<>#%{}|\^~[]\``
                    // https://perishablepress.com/stop-using-unsafe-characters-in-urls/
                    c_orig == '<'
                        || c_orig == '>'
                        || c_orig == '"'
                        || c_orig == '*'
                        || c_orig == '#'
                        || c_orig == '%'
                        || c_orig == '{'
                        || c_orig == '}'
                        || c_orig == '^'
                        || c_orig == '`'
                    {
                        (c_orig, ' ')
                    } else {
                        (c_orig, c_orig)
                    }
                })
                // Filter replaced space after replaced space.
                // Filter period after period, replaced space, replaced underscore or at the beginning of string.
                // Filter replaced underscore after replaced underscore.
                // Filter `_.\/,;` after whitespace.
                // Filter non-printing space `U+200b`.
                .filter(|&(c_orig, c)| {
                    let discard = (c == ' ' && last_replaced_chr == ' ')
                        || (c == '_' && last_replaced_chr == '_')
                        || ((c_orig == '_'
                            || c_orig == '.'
                            || c_orig == '\\'
                            || c_orig == '/'
                            || c_orig == ','
                            || c_orig == ';')
                            && last_replaced_chr.is_whitespace())
                        || c_orig == '\u{200b}';
                    if !discard {
                        last_replaced_chr = c;
                    };
                    !discard
                })
                .map(|(_, c)| c)
                .collect::<String>()
                // Trim whitespace and `_-.,;` at the beginning and the end of the line.
                .trim_matches(|c: char| {
                    c.is_whitespace() || c == '_' || c == '-' || c == '.' || c == ',' || c == ';'
                })
                .to_string();
            // Filter newline and insert line separator `-`.
            s.push('-');
            s
        })
        .collect::<String>()
        // Trim whitespace and `_-.,;` at the beginning and the end of the whole string.
        .trim_matches(|c: char| {
            c.is_whitespace() || c == '_' || c == '-' || c == '.' || c == ',' || c == ';'
        })
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
        // test filter 1
        assert_eq!(sanitize("\tabc\tefg\t"), "abc efg".to_string());
        // test filter 2
        assert_eq!(sanitize("abc\u{0019}efg"), "abcefg".to_string());
        // test filter 3
        assert_eq!(sanitize("abc:\\/|?~=efg"), "abc_=efg".to_string());
        // test filter4
        assert_eq!(
            sanitize("abc<>\"*<>#%{}^[]+[]`efg"),
            "abc []+[] efg".to_string()
        );
        // test trim before and after newline
        assert_eq!(
            sanitize("-_ \tabc \t >_-\n   efg \t_-"),
            "abc-efg".to_string()
        );
        // test replace Unix newline
        assert_eq!(sanitize("abc\nefg"), "abc-efg".to_string());
        // test replace Window newline
        assert_eq!(sanitize("abc\r\nefg"), "abc-efg".to_string());
        // test double '_' or ' '
        assert_eq!(sanitize("abc_ __  efg __hij"), "abc_ efg hij".to_string());
        // test link
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
