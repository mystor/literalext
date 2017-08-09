use {RawInt, IntLit, FloatLit};

use std::char;
use std::ops::{Index, RangeFrom};
use std::ascii::AsciiExt;

/// Filter the input string, removing all bytes which match the given input
/// byte in place, without allocation.
///
/// # Panics
///
/// Panics if the filter byte is not a valid ASCII character.
fn string_filter(string: String, remove: u8) -> String {
    assert!(remove.is_ascii());
    let mut bytes = string.into_bytes();
    let mut write = 0;
    for read in 0..bytes.len() {
        if bytes[read] == remove {
            continue; // Don't increase write
        } else if write != read {
            let x = bytes[read];
            bytes[write] = x;
        }
        write += 1;
    }
    bytes.truncate(write);
    String::from_utf8(bytes)
        .expect("Transformation should not have impacted utf-8 validity")
}

#[cfg(test)]
#[test]
fn test_string_filter() {
    assert_eq!(string_filter("1921381902_1231___23___333".to_string(), b'_'),
               "1921381902123123333");

    assert_eq!(string_filter("_________".to_string(), b'_'), "");
    assert_eq!(string_filter("12837912837192837129387129_".to_string(), b'_'),
               "12837912837192837129387129");
    assert_eq!(string_filter("_12837912837192837129387129".to_string(), b'_'),
               "12837912837192837129387129");
}

/// Get the byte at offset idx, or a default of `b'\0'` if we're looking past
/// the end of the input buffer.
fn byte<S: AsRef<[u8]> + ?Sized>(s: &S, idx: usize) -> u8 {
    let s = s.as_ref();
    if idx < s.len() {
        s[idx]
    } else {
        0
    }
}

fn next_chr(s: &str) -> char {
    s.chars().next().unwrap_or('\0')
}

fn raw_str(s: &str) -> &str {
    // NOTE: This makes very very strong assumptions about the formatting of raw
    // string literals. If you pass a malformed literal into this function it is
    // very likely to panic or do the wrong thing.
    let begin = s.find('"').expect("Raw string must begin with \" char");
    let end = s.rfind('"').expect("Raw string must end with \" char");
    &s[begin + 1..end]
}

fn backslash_x<S>(s: &S) -> (&S, u8)
    where S: Index<RangeFrom<usize>, Output=S> + AsRef<[u8]> + ?Sized
{
    let mut ch = 0;
    let b0 = byte(s, 0);
    let b1 = byte(s, 1);
    ch += 0x10 * match b0 {
        b'0'...b'9' => b0 - b'0',
        b'a'...b'f' => 10 + (b0 - b'a'),
        b'A'...b'F' => 10 + (b0 - b'A'),
        _ => panic!("unexpected non-hex character after \\x"),
    };
    ch += 0x1 * match b1 {
        b'0'...b'9' => b1 - b'0',
        b'a'...b'f' => 10 + (b1 - b'a'),
        b'A'...b'F' => 10 + (b1 - b'A'),
        _ => panic!("unexpected non-hex character after \\x"),
    };
    (&s[2..], ch)
}

fn backslash_u(mut s: &str) -> (&str, char) {
    if byte(s, 0) != b'{' {
        panic!("expected {{ after \\u");
    }
    s = &s[1..];

    let mut ch = 0;
    for _ in 0..6 {
        let b = byte(s, 0);
        match b {
            b'0'...b'9' => {
                ch *= 0x10;
                ch += (b - b'0') as u32;
                s = &s[1..];
            }
            b'a'...b'f' => {
                ch *= 0x10;
                ch += (10 + b - b'a') as u32;
                s = &s[1..];
            }
            b'A'...b'F' => {
                ch *= 0x10;
                ch += (10 + b - b'A') as u32;
                s = &s[1..];
            }
            b'}' => break,
            _ => panic!("unexpected non-hex character after \\u"),
        }
    }
    assert!(byte(s, 0) == b'}');
    s = &s[1..];

    if let Some(ch) = char::from_u32(ch) {
        (s, ch)
    } else {
        panic!("character code {:x} is not a valid unicode character", ch);
    }
}

pub(crate) fn str_lit(mut s: &str) -> Option<String> {
    match byte(s, 0) {
        b'"' => {
            s = &s[1..]
        }
        b'r' => {
            return Some(raw_str(s).to_string());
        }
        _ => return None,
    }

    let mut out = String::new();
    'outer: loop {
        let ch = match byte(s, 0) {
            b'"' => break,
            b'\\' => {
                let b = byte(s, 1);
                s = &s[2..];
                match b {
                    b'x' => {
                        let (rest, byte) = backslash_x(s);
                        s = rest;
                        assert!(byte <= 0x80, "Invalid \\x byte in string literal");
                        char::from_u32(byte as u32).unwrap()
                    }
                    b'u' => {
                        let (rest, chr) = backslash_u(&s);
                        s = rest;
                        chr
                    }
                    b'n' => '\n',
                    b'r' => '\r',
                    b't' => '\t',
                    b'\\' => '\\',
                    b'0' => '\0',
                    b'\'' => '\'',
                    b'"' => '"',
                    b'\r' | b'\n' => {
                        loop {
                            let ch = next_chr(s);
                            if ch.is_whitespace() {
                                s = &s[ch.len_utf8()..];
                            } else {
                                continue 'outer;
                            }
                        }
                    }
                    b => {
                        panic!("unexpected byte {:?} after \\ character in byte literal", b)
                    }
                }
            }
            b'\r' => {
                assert_eq!(byte(s, 1), b'\n', "Bare CR not allowed in string");
                s = &s[2..];
                '\n'
            }
            _ => {
                let ch = next_chr(s);
                s = &s[ch.len_utf8()..];
                ch
            }
        };
        out.push(ch);
    }

    assert_eq!(s, "\"");
    return Some(out);
}

pub(crate) fn byte_str_lit(mut s: &str) -> Option<Vec<u8>> {
    match (byte(s, 0), byte(s, 1)) {
        (b'b', b'"') => {
            s = &s[2..];
        }
        (b'b', b'r') => {
            return Some(raw_str(s).as_bytes().to_vec());
        }
        _ => return None,
    }
    // We're going to want to have slices which don't respect codepoint boundaries.
    let mut s = s.as_bytes();

    let mut out = Vec::new();
    'outer: loop {
        let byte = match byte(s, 0) {
            b'"' => break,
            b'\\' => {
                let b = byte(s, 1);
                s = &s[2..];
                match b {
                    b'x' => {
                        let (rest, b) = backslash_x(s);
                        s = rest;
                        b
                    }
                    b'n' => b'\n',
                    b'r' => b'\r',
                    b't' => b'\t',
                    b'\\' => b'\\',
                    b'0' => b'\0',
                    b'\'' => b'\'',
                    b'"' => b'"',
                    b'\r' | b'\n' => {
                        loop {
                            let byte = byte(s, 0);
                            let ch = char::from_u32(byte as u32).unwrap();
                            if ch.is_whitespace() {
                                s = &s[1..];
                            } else {
                                continue 'outer;
                            }
                        }
                    }
                    b => {
                        panic!("unexpected byte {:?} after \\ character in byte literal", b)
                    }
                }
            }
            b'\r' => {
                assert_eq!(byte(s, 1), b'\n', "Bare CR not allowed in string");
                s = &s[2..];
                b'\n'
            }
            b => {
                s = &s[1..];
                b
            }
        };
        out.push(byte);
    }

    assert_eq!(s, b"\"");
    return Some(out);
}

pub(crate) fn char_lit(mut s: &str) -> Option<char> {
    if byte(s, 0) != b'\'' {
        return None;
    }
    s = &s[1..];

    let ch = match byte(s, 0) {
        b'\\' => {
            let b = byte(s, 1);
            s = &s[2..];
            match b {
                b'x' => {
                    let (rest, byte) = backslash_x(s);
                    s = rest;
                    assert!(byte <= 0x80, "Invalid \\x byte in string literal");
                    char::from_u32(byte as u32).unwrap()
                }
                b'u' => {
                    let (rest, chr) = backslash_u(s);
                    s = rest;
                    chr
                }
                b'n' => '\n',
                b'r' => '\r',
                b't' => '\t',
                b'\\' => '\\',
                b'0' => '\0',
                b'\'' => '\'',
                b'"' => '"',
                b => {
                    panic!("unexpected byte {:?} after \\ character in byte literal", b)
                }
            }
        }
        _ => {
            let ch = next_chr(s);
            s = &s[ch.len_utf8()..];
            ch
        }
    };
    assert_eq!(s, "\'", "Expected end of char literal");
    Some(ch)
}

pub(crate) fn byte_lit(s: &str) -> Option<u8> {
    if byte(s, 0) != b'b' || byte(s, 1) != b'\'' {
        return None;
    }
    // We're going to want to have slices which don't respect codepoint boundaries.
    let mut s = s[2..].as_bytes();

    let b = match byte(s, 0) {
        b'\\' => {
            let b = byte(s, 1);
            s = &s[2..];
            match b {
                b'x' => {
                    let (rest, b) = backslash_x(s);
                    s = rest;
                    b
                }
                b'n' => b'\n',
                b'r' => b'\r',
                b't' => b'\t',
                b'\\' => b'\\',
                b'0' => b'\0',
                b'\'' => b'\'',
                b'"' => b'"',
                b => {
                    panic!("unexpected byte {:?} after \\ character in byte literal", b)
                }
            }
        }
        b => {
            s = &s[1..];
            b
        }
    };

    assert!(byte(s, 0) == b'\'');
    Some(b)
}

pub(crate) fn int_lit(mut s: &str) -> Option<IntLit> {
    let base = match (byte(s, 0), byte(s, 1)) {
        (b'0', b'x') => {
            s = &s[2..];
            16
        }
        (b'0', b'o') => {
            s = &s[2..];
            8
        }
        (b'0', b'b') => {
            s = &s[2..];
            2
        }
        (b'0'...b'9', _) => 10,
        _ => return None,
    };

    let mut value: Option<RawInt> = Some(0);
    loop {
        let b = byte(s, 0);
        let digit = match b {
            b'0'...b'9' => (b - b'0') as RawInt,
            b'a'...b'f' if base > 10 => 10 + (b - b'a') as RawInt,
            b'A'...b'F' if base > 10 => 10 + (b - b'A') as RawInt,
            b'_' => {
                s = &s[1..];
                continue;
            }
            // NOTE: Looking at a floating point literal, we don't want to
            // consider these integers.
            b'.' if base == 10 => return None,
            b'e' | b'E' if base == 10 => return None,
            _ => break,
        };

        if digit >= base {
            panic!("Unexpected digit {:x} out of base range", digit);
        }

        value = value
            .and_then(|v| v.checked_mul(base))
            .and_then(|v| v.checked_add(digit));
        s = &s[1..];
    }

    // Check if the suffix is one of our legal suffixes, if it is, return an
    // equal 'static string which we can store in the IntLit object.
    let suffix = match s {
        "u8" => "u8",
        "i8" => "i8",
        "u16" => "u16",
        "i16" => "i16",
        "u32" => "u32",
        "i32" => "i32",
        "u64" => "u64",
        "i64" => "i64",
        "usize" => "usize",
        "isize" => "isize",
        "" => "",
        _ => return None,
    };

    assert_eq!(suffix, s);

    Some(IntLit {
        val: value,
        suffix: suffix,
    })
}

pub(crate) fn float_lit(input: String) -> Option<FloatLit> {
    match (byte(&input, 0), byte(&input, 1)) {
        (b'0', b'x') | (b'0', b'o') | (b'0', b'b') => return None,
        (b'0'...b'9', _) => {}
        _ => return None,
    };

    // Rust's floating point literals are very similar to the ones parsed by the
    // standard library, except that rust's literals can contain ignorable
    // underscores. Let's remove those underscores in-place.
    let input = string_filter(input, b'_');
    let mut s = &input[..];

    let mut has_dot = false;
    let mut has_exp = false;
    loop {
        match byte(s, 0) {
            b'0'...b'9' => {
                s = &s[1..];
            }
            b'.' => {
                s = &s[1..];
                if has_dot {
                    panic!("Unexpected second dot while parsing float literal");
                }
                has_dot = true;
            }
            b'e' | b'E' => {
                s = &s[1..];
                loop {
                    match byte(s, 0) {
                        b'+' | b'-' if !has_exp => {
                            s = &s[1..];
                        }
                        b'0'...b'9' => {
                            s = &s[1..];
                            has_exp = true;
                        }
                        b'_' => {
                            s = &s[1..];
                        }
                        _ => break,
                    }
                }
                assert!(has_exp,
                        "Unexpected end of float literal after `E` char");
                break;
            }
            _ => break,
        };
    }

    let suffix = match s {
        "f32" => "f32",
        "f64" => "f64",
        "" => "",
        _ => return None,
    };
    assert_eq!(suffix, s);

    // If we don't have an exponent or a . and the suffix is empty, then we're
    // looking at an integer literal. Don't parse it as a float.
    if !has_exp && !has_dot && suffix == "" {
        return None;
    }

    Some(FloatLit {
        val: input[..input.len() - suffix.len()].parse::<f64>().unwrap(),
        suffix: suffix,
    })
}

pub(crate) fn outer_doc(s: String) -> Option<String> {
    if s.starts_with("///") || s.starts_with("/**") {
        Some(s)
    } else {
        None
    }
}

pub(crate) fn inner_doc(s: String) -> Option<String> {
    if s.starts_with("//!") || s.starts_with("/*!") {
        Some(s)
    } else {
        None
    }
}
