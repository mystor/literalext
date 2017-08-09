// NOTE: We need `DummyLiteral` to run our tests.
#![cfg(all(feature = "dummy", test))]

use {DummyLiteral, LiteralExt};

#[test]
fn ints() {
    macro_rules! test_int {
        ($i:tt) => {
            test_int!($i, as_u8, as_i8, as_u16, as_i16, as_u32, as_i32, as_u64, as_i64);
        };
        ($i:tt, $($f:ident),*) => {
            let dl = DummyLiteral(stringify!($i));
            let asint = dl.parse_int().expect(&format!("Unable to parse {} as an integer", stringify!($i)));
            $(
                assert_eq!(
                    asint
                        .$f()
                        .expect(&format!("Unable to extract {} with {}",
                                         stringify!($i), stringify!($f))),
                    $i
                );
            )*
            // NOTE: Some ints can also be parsed as floats, so we don't check that as_float fails.
            assert_eq!(dl.parse_float(), None);
            assert_eq!(dl.parse_string(), None);
            assert_eq!(dl.parse_char(), None);
            assert_eq!(dl.parse_bytes(), None);
            assert_eq!(dl.parse_byte(), None);
            assert_eq!(dl.parse_inner_doc(), None);
            assert_eq!(dl.parse_outer_doc(), None);
        }
    }

    test_int!(5);
    test_int!(5u32, as_u32);
    test_int!(5_0);
    test_int!(5_____0_____);
    test_int!(0x7f);
    test_int!(0x7F);
    test_int!(0b1001);
    test_int!(0o73);
    test_int!(0x7Fu8, as_u8);
    test_int!(0b1001i8, as_i8);
    test_int!(0o73u32, as_u32);
    test_int!(0x__7___f_);
    test_int!(0x__7___F_);
    test_int!(0b_1_0__01);
    test_int!(0o_7__3);
    test_int!(0x_7F__u8, as_u8);
    test_int!(0b__10__0_1i8, as_i8);
    test_int!(0o__7__________________3u32, as_u32);
}

#[test]
fn floats() {
    macro_rules! test_float {
        ($i:tt) => {
            test_float!($i, as_f32, as_f64);
        };
        ($i:tt, $($f:ident),*) => {
            let dl = DummyLiteral(stringify!($i));
            let asfloat = dl.parse_float()
                .expect(&format!("Unable to parse {} as a float", stringify!($i)));
            $(
                assert_eq!(
                    asfloat.$f().expect(&format!("Unable to extract {} with {}",
                                                 stringify!($i), stringify!($f))),
                    $i
                );
            )*
            assert_eq!(dl.parse_int(), None);
            assert_eq!(dl.parse_string(), None);
            assert_eq!(dl.parse_char(), None);
            assert_eq!(dl.parse_bytes(), None);
            assert_eq!(dl.parse_byte(), None);
            assert_eq!(dl.parse_inner_doc(), None);
            assert_eq!(dl.parse_outer_doc(), None);
        }
    }

    test_float!(5.5);
    test_float!(5.5E32);
    test_float!(5.5e32);
    test_float!(1.0__3e-23);
    test_float!(1.03e+23);
}

#[test]
fn chars() {
    macro_rules! test_char {
        ($i:tt) => {
            let dl = DummyLiteral(stringify!($i));
            assert_eq!(dl.parse_char(), Some($i));
            assert_eq!(dl.parse_int(), None);
            assert_eq!(dl.parse_float(), None);
            assert_eq!(dl.parse_string(), None);
            assert_eq!(dl.parse_bytes(), None);
            assert_eq!(dl.parse_byte(), None);
            assert_eq!(dl.parse_inner_doc(), None);
            assert_eq!(dl.parse_outer_doc(), None);
        }
    }

    test_char!('a');
    test_char!('\n');
    test_char!('\r');
    test_char!('\t');
    test_char!('🐕'); // NOTE: This is an emoji
    test_char!('\'');
    test_char!('"');
    test_char!('\u{1F415}');
}

#[test]
fn byte() {
    macro_rules! test_byte {
        ($i:tt) => {
            let dl = DummyLiteral(stringify!($i));
            assert_eq!(dl.parse_byte(), Some($i));
            assert_eq!(dl.parse_int(), None);
            assert_eq!(dl.parse_float(), None);
            assert_eq!(dl.parse_string(), None);
            assert_eq!(dl.parse_char(), None);
            assert_eq!(dl.parse_bytes(), None);
            assert_eq!(dl.parse_inner_doc(), None);
            assert_eq!(dl.parse_outer_doc(), None);
        }
    }

    test_byte!(b'a');
    test_byte!(b'\n');
    test_byte!(b'\r');
    test_byte!(b'\t');
    test_byte!(b'\'');
    test_byte!(b'"');
}

#[test]
fn string() {
    macro_rules! test_string {
        ($i:tt) => {
            let dl = DummyLiteral(stringify!($i));
            assert_eq!(dl.parse_string().unwrap(), $i);
            assert_eq!(dl.parse_int(), None);
            assert_eq!(dl.parse_float(), None);
            assert_eq!(dl.parse_char(), None);
            assert_eq!(dl.parse_bytes(), None);
            assert_eq!(dl.parse_byte(), None);
            assert_eq!(dl.parse_inner_doc(), None);
            assert_eq!(dl.parse_outer_doc(), None);
        }
    }

    test_string!("a");
    test_string!("\n");
    test_string!("\r");
    test_string!("\t");
    test_string!("🐕"); // NOTE: This is an emoji
    test_string!("\"");
    test_string!("'");
    test_string!("");
    test_string!("\u{1F415}");
    test_string!("This
           String contains\
           newlines and other such

things which make it more \


Interesting");
    test_string!(r#"This
Is
A
RAW STRING"#);
    test_string!(r######"This
Is
A r####"Raw string with another in it"####
RAW STRING"######);
}

#[test]
fn bytes() {
    macro_rules! test_bytes {
        ($i:tt) => {
            let dl = DummyLiteral(stringify!($i));
            // NOTE: We slice $i here to get it from &[u8; N] to &[u8]
            assert_eq!(dl.parse_bytes().unwrap(), &$i[..]);
            assert_eq!(dl.parse_int(), None);
            assert_eq!(dl.parse_float(), None);
            assert_eq!(dl.parse_string(), None);
            assert_eq!(dl.parse_char(), None);
            assert_eq!(dl.parse_byte(), None);
            assert_eq!(dl.parse_inner_doc(), None);
            assert_eq!(dl.parse_outer_doc(), None);
        }
    }

    test_bytes!(b"a");
    test_bytes!(b"\n");
    test_bytes!(b"\r");
    test_bytes!(b"\t");
    test_bytes!(b"\"");
    test_bytes!(b"'");
    test_bytes!(b"This
           String contains\
           newlines and other such

things which make it more \


Interesting");
    test_bytes!(br#"This
Is
A
RAW STRING"#);
    test_bytes!(br######"This
Is
A r####"Raw string with another in it"####
RAW STRING"######);
}
