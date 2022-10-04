use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, is_not, tag, take_while_m_n},
    character::complete::one_of,
    combinator::{complete, cut, map, map_res, recognize, value, verify},
    error::context,
    multi::{many0, many1},
    sequence::{preceded, terminated},
    IResult,
};

use super::Span;

pub fn binary(input: Span) -> IResult<Span, u32> {
    map_res(
        preceded(
            alt((tag("0b"), tag("0B"))),
            recognize(many1(terminated(one_of("01"), many0(tag("_"))))),
        ),
        |number: Span| {
            let plain_bin = number.replace("_", "");
            u32::from_str_radix(&plain_bin, 2)
        },
    )(input)
}

pub fn hexadecimal(input: Span) -> IResult<Span, u32> {
    map_res(
        preceded(
            alt((tag("0x"), tag("0X"))),
            recognize(many1(terminated(
                one_of("0123456789abcdefABCDEF"),
                many0(tag("_")),
            ))),
        ),
        |number: Span| {
            let plain_hex = number.replace("_", "");
            u32::from_str_radix(&plain_hex, 16)
        },
    )(input)
}

macro_rules! hex_char {
    () => {
        preceded(
            tag("x"),
            take_while_m_n(1, 2, |c: char| c.is_ascii_hexdigit()),
        )
    };
}

macro_rules! escaped_section {
    ($delimiter: expr, $normal_chars: expr) => {
        preceded(
            nom::character::streaming::char($delimiter),
            cut(terminated(
                escaped_transform(
                    is_not($normal_chars),
                    '\\',
                    alt((
                        map(tag("\\"), |_| '\\'),
                        map(tag("\""), |_| '\"'),
                        map(tag("'"), |_| '\''),
                        map(tag("n"), |_| '\n'),
                        map(tag("r"), |_| '\r'),
                        map(tag("t"), |_| '\t'),
                        map_res(hex_char!(), |hex: Span| {
                            match u8::from_str_radix(&hex, 16) {
                                Ok(num) => Ok(num as char),
                                Err(err) => Err(err),
                            }
                        }),
                        // TODO: Add octal characters
                        map(tag("0"), |_| '\0'),
                    )),
                ),
                nom::character::streaming::char($delimiter),
            )),
        )
    };
}

pub fn parse_string(input: Span) -> IResult<Span, Vec<u8>> {
    context(
        "string",
        map(
            verify(
                alt((
                    value(String::from(""), tag(r#""""#)),
                    cut(complete(escaped_section!('"', "\\\"\n"))),
                )),
                |string: &str| string.is_ascii() || string.len() == 0,
            ),
            |string: String| {
                let mut bytes = string.as_bytes().to_vec();
                bytes.push(0u8); // zero-terminate ascii string
                bytes
            },
        ),
    )(input)
}

pub fn parse_char(input: Span) -> IResult<Span, u8> {
    context(
        "char",
        map(
            verify(
                cut(complete(escaped_section!('\'', "\\\'\n"))),
                |string: &str| string.is_ascii() && string.len() == 1,
            ),
            |string: String| string.as_bytes()[0],
        ),
    )(input)
}

pub fn float(s: Span) -> IResult<Span, f32> {
    // Regular float string
    // TODO: add support for hexadecimal float literals
    nom::number::complete::float(s)
}

pub fn double(s: Span) -> IResult<Span, f64> {
    // Regular float string
    // TODO: add support for hexadecimal float literals
    nom::number::complete::double(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::HashMap;

    // Bin tests

    #[test]
    fn test_bin() {
        let (remain, num) = binary(Span::new("0b10000")).unwrap();
        assert_eq!("", *remain.fragment());
        assert_eq!(7, remain.location_offset());
        assert_eq!(0b10000, num);

        let (remain, num) = binary(Span::new("0B10001")).unwrap();
        assert_eq!("", *remain.fragment());
        assert_eq!(7, remain.location_offset());
        assert_eq!(0b10001, num);

        let (remain, num) = binary(Span::new("0b1_0_0_0_0")).unwrap();
        assert_eq!("", *remain.fragment());
        assert_eq!(0b10000, num);
    }

    #[test]
    fn test_bin_invalid_character() {
        let (remain, num) = binary(Span::new("0b1A")).unwrap();
        assert_eq!("A", *remain.fragment());
        assert_eq!(3, remain.location_offset());
        assert_eq!(0b1, num);
    }

    #[test]
    fn test_bin_invalid_prefix() {
        let nom_error = binary(Span::new("0x10000")).unwrap_err();
        match nom_error {
            nom::Err::Error(error) => {
                assert_eq!("0x10000", *error.input)
            }
            _ => panic!("Error expected, but not received!"),
        }
    }

    #[test]
    fn test_bin_no_binary_literals() {
        let nom_error = binary(Span::new("0b")).unwrap_err();
        match nom_error {
            nom::Err::Error(error) => {
                assert_eq!("", *error.input)
            }
            _ => panic!("Error expected, but not received!"),
        }
    }

    // Hex Tests

    #[test]
    fn test_hex() {
        let (remain, num) = hexadecimal(Span::new("0x1ABCD")).unwrap();
        assert_eq!("", *remain.fragment());
        assert_eq!(7, remain.location_offset());
        assert_eq!(0x1ABCD, num);

        let (remain, num) = hexadecimal(Span::new("0X2012A")).unwrap();
        assert_eq!("", *remain.fragment());
        assert_eq!(7, remain.location_offset());
        assert_eq!(0x2012A, num);

        let (remain, num) = hexadecimal(Span::new("0x2_1_A_a_C")).unwrap();
        assert_eq!("", *remain.fragment());
        assert_eq!(0x2_1_A_a_C, num);
    }

    #[test]
    fn test_hex_invalid_character() {
        let (remain, num) = hexadecimal(Span::new("0x1G")).unwrap();
        assert_eq!("G", *remain.fragment());
        assert_eq!(3, remain.location_offset());
        assert_eq!(0x1, num);
    }

    #[test]
    fn test_hex_invalid_prefix() {
        let nom_error = hexadecimal(Span::new("0b10000")).unwrap_err();
        match nom_error {
            nom::Err::Error(error) => {
                assert_eq!("0b10000", *error.input)
            }
            _ => panic!("Error expected, but not received!"),
        }
    }

    #[test]
    fn test_hex_no_hex_literals() {
        let nom_error = hexadecimal(Span::new("0x")).unwrap_err();
        match nom_error {
            nom::Err::Error(error) => {
                assert_eq!("", *error.input)
            }
            _ => panic!("Error expected, but not received!"),
        }
    }

    // Char Tests

    #[test]
    fn test_special_char() {
        let test_cases = hashmap! {
            // Normal
            r#"'A'"# => (b'A', ""),

            // Special characters
            r#"'\n'"# => (b'\n', ""),
            r#"'\t'"# => (b'\t', ""),
            r#"'\r'"# => (b'\r', ""),
            r#"'"'"# => (b'"', ""),
            r#"'\''"# => (b'\'', ""),
            r#"'\x00'"# => (b'\x00', ""),
            r#"'\x01'"# => (b'\x01', ""),
            r#"'\\'"# => (b'\\', ""),
            r#"'\"'"# => (b'\"', ""),
            r#"'\''"# => (b'\'', ""),
            r#"'\0'"# => (b'\0', ""),
        };

        for (test_case, (expected_byte, expected_remaining)) in test_cases.iter() {
            let (span, byte) = parse_char(Span::new(test_case)).unwrap();
            assert_eq!(*expected_byte, byte);
            assert_eq!(*expected_remaining, *span);
        }
    }

    #[test]
    fn test_unicode_char_literal() {
        if let nom::Err::Error(byte) = parse_char(Span::new("'§'")).unwrap_err() {
            assert_eq!("'§'", *byte.input);
        } else {
            panic!("Unexpected error")
        }
    }

    #[test]
    fn test_incomplete_char_literal() {
        match parse_char(Span::new("'a")).unwrap_err() {
            err @ nom::Err::Error(_) => {
                panic!("Unexpected incomplete: {}", err)
            }
            err @ nom::Err::Incomplete(_) => {
                panic!("Unexpected incomplete: {}", err)
            }
            _err @ nom::Err::Failure(_) => {
                // Expected failure since quote is unmatched
            }
        }
    }

    #[test]
    fn test_empty_char_literal() {
        match parse_char(Span::new("''")).unwrap_err() {
            err @ nom::Err::Error(_) => {
                panic!("Unexpected incomplete: {}", err)
            }
            err @ nom::Err::Incomplete(_) => {
                panic!("Unexpected incomplete: {}", err)
            }
            _err @ nom::Err::Failure(_) => {
                // Expected failure since quote is unmatched
            }
        }
    }

    #[test]
    fn test_multiple_char_literal() {
        if let nom::Err::Error(byte) = parse_char(Span::new("'as'")).unwrap_err() {
            assert_eq!("'as'", *byte.input);
        } else {
            panic!("Unexpected error")
        }
    }

    // String tests

    #[test]
    fn test_special_strings() {
        let test_cases = hashmap! {
            // Normal
            r#""A""# => ("A", ""),

            // Empty
            r#""""# => ("", ""),

            // Special characters
            r#""\n""# => ("\n", ""),
            r#""\t""# => ("\t", ""),
            r#""\r""# => ("\r", ""),
            r#""'""# => ("'", ""),
            r#""\x01""# => ("\x01", ""),
            r#""\x00""# => ("\x00", ""),
            r#""\\""# => ("\\", ""),
            r#""\"""# => ("\"", ""),
            r#""\'""# => ("\'", ""),
            r#""\0""# => ("\0", ""),

            // Multi-character strings
            r#""ABC""# => ("ABC", ""),
            r#""A\0C""# => ("A\0C", ""),
            r#""\x123""# => ("\x123", ""),
            r#""'''""# => ("'''", ""),
        };

        for (test_case, (expected_byte, expected_remaining)) in test_cases.iter() {
            let (span, bytes) = parse_string(Span::new(test_case)).unwrap();
            let mut expected_vec = expected_byte.as_bytes().to_vec();
            expected_vec.push(0u8); // Add null termination
            assert_eq!(
                expected_vec, bytes,
                "Test Case: {} | expected bytes: {:?} | bytes: {:?}",
                test_case, expected_vec, bytes
            );
            assert_eq!(*expected_remaining, *span);
        }
    }

    #[test]
    fn test_unicode_string_literal() {
        if let nom::Err::Error(byte) = parse_string(Span::new(r#""§""#)).unwrap_err() {
            assert_eq!(r#""§""#, *byte.input);
        } else {
            panic!("Unexpected error")
        }
    }

    #[test]
    fn test_incomplete_string_literal() {
        match parse_string(Span::new(r#""a"#)).unwrap_err() {
            err @ nom::Err::Error(_) => {
                panic!("Unexpected incomplete: {}", err)
            }
            err @ nom::Err::Incomplete(_) => {
                panic!("Unexpected incomplete: {}", err)
            }
            _err @ nom::Err::Failure(_) => {
                // Expected failure since quote is unmatched
            }
        }
    }

    // #[test]
    // #[should_panic(expected = "Expected only 1 byte from Ascii string: as")]
    // fn test_multiple_char_literal() {
    //     let byte = parse_char_literal(Span::new("as")).unwrap_err();
    //     assert_eq!((), byte);
    // }

    #[test]
    fn test_float() {
        let test_cases = hashmap! {
            // Normal
            r#"1.24"# => (1.24, ""),
            r#"2."# => (2., ""),
            r#"+2.0"# => (2.0, ""),
            r#"-2.0"# => (-2.0, ""),
            r#"-2e10"# => (-2.0e10, ""),
            r#"-2E+10"# => (-2.0e10, ""),
            r#"-2E-10"# => (-2.0e-10, ""),
        };

        for (test_case, (expected_float, expected_remaining)) in test_cases.iter() {
            let (span, decimal) = float(Span::new(test_case)).unwrap();
            assert_eq!(decimal, *expected_float);
            assert_eq!(*expected_remaining, *span);
        }
    }

    #[test]
    fn test_double() {
        let test_cases: HashMap<&str, (f64, &str)> = hashmap! {
            // Normal
            r#"1.24"# => (1.24, ""),
            r#"2."# => (2., ""),
            r#"+2.0"# => (2.0, ""),
            r#"-2.0"# => (-2.0, ""),
            r#"-2e10"# => (-2.0e10, ""),
            r#"-2E+10"# => (-2.0e10, ""),
            r#"-2E-10"# => (-2.0e-10, ""),
        };

        for (test_case, (expected_float, expected_remaining)) in test_cases.iter() {
            let (span, decimal) = double(Span::new(test_case)).unwrap();
            assert_eq!(decimal, *expected_float);
            assert_eq!(*expected_remaining, *span);
        }
    }
}
