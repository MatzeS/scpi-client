use regex::Regex;
use std::str::FromStr;

use crate::{Error, Result, ScpiDeserialize, ScpiSerialize, read_exact};

pub struct SerializeToString<T: ToString>(T);

impl<T: ToString> ScpiSerialize for SerializeToString<T> {
    fn serialize(&self, out: &mut String) {
        out.push_str(self.0.to_string().as_str());
    }
}

pub trait DeserializedWithParse: FromStr {
    /// Return the number of characters to consume from `input` and parse to Self.
    fn prefix_len(input: &str) -> usize;
}

impl<T> ScpiDeserialize for T
where
    T: DeserializedWithParse + FromStr<Err: std::fmt::Display>, // TODO relax
{
    fn deserialize(input: &mut &str) -> Result<Self> {
        let len = T::prefix_len(input);
        let prefix = read_exact(input, len)?;
        let result = prefix
            .parse()
            .map_err(|e| Error::ResponseDecoding(format!("Failed to parse: {e}")))?;
        Ok(result)
    }
}

macro_rules! impl_serialize_to_string {
    ($type:ty) => {
        impl ScpiSerialize for $type {
            fn serialize(&self, out: &mut String) {
                SerializeToString(self).serialize(out);
            }
        }
    };
}

macro_rules! for_numeric_primitives {
    ($callback_macro:ident) => {
        $callback_macro!(u8);
        $callback_macro!(u16);
        $callback_macro!(u32);
        $callback_macro!(u64);
        $callback_macro!(u128);
        $callback_macro!(i8);
        $callback_macro!(i16);
        $callback_macro!(i32);
        $callback_macro!(i64);
        $callback_macro!(i128);
        $callback_macro!(f32);
        $callback_macro!(f64);
    };
}

for_numeric_primitives!(impl_serialize_to_string);

macro_rules! impl_deserialize_with_parse_from_regex {
    ($type:ty, $regex:ident) => {
        impl DeserializedWithParse for $type {
            fn prefix_len(input: &str) -> usize {
                $regex.find(input).map_or(0, |m| m.end())
            }
        }
    };
}

lazy_static::lazy_static! {
    static ref REGEX_UNSIGNED_INT: Regex = Regex::new(r"^\d+").unwrap();
    static ref REGEX_SIGNED_INT: Regex = Regex::new(r"^[+-]?\d+").unwrap();
    static ref REGEX_FLOATING_POINT: Regex = Regex::new(r"^[+-]?(?:\d+)?(?:\.\d+)?(?:[eE][+-]?\d+)?").unwrap();
}

impl_deserialize_with_parse_from_regex!(u8, REGEX_UNSIGNED_INT);
impl_deserialize_with_parse_from_regex!(u16, REGEX_UNSIGNED_INT);
impl_deserialize_with_parse_from_regex!(u32, REGEX_UNSIGNED_INT);
impl_deserialize_with_parse_from_regex!(u64, REGEX_UNSIGNED_INT);
impl_deserialize_with_parse_from_regex!(u128, REGEX_UNSIGNED_INT);
impl_deserialize_with_parse_from_regex!(i8, REGEX_SIGNED_INT);
impl_deserialize_with_parse_from_regex!(i16, REGEX_SIGNED_INT);
impl_deserialize_with_parse_from_regex!(i32, REGEX_SIGNED_INT);
impl_deserialize_with_parse_from_regex!(i64, REGEX_SIGNED_INT);
impl_deserialize_with_parse_from_regex!(i128, REGEX_SIGNED_INT);
impl_deserialize_with_parse_from_regex!(f32, REGEX_FLOATING_POINT);
impl_deserialize_with_parse_from_regex!(f64, REGEX_FLOATING_POINT);

#[cfg(test)]
mod tests {

    macro_rules! serialize {
        ($input:expr) => {{
            use crate::ScpiSerialize;
            let mut out = String::new();
            $input.serialize(&mut out);
            out
        }};
    }

    macro_rules! deserialize {
        ($input:expr, $type:ty) => {{
            use crate::{ScpiDeserialize, check_empty};
            let mut data: &str = $input;
            let result: $type = ScpiDeserialize::deserialize(&mut data).unwrap();
            check_empty(data).unwrap();
            result
        }};
    }

    #[test]
    fn serialize_primitives() {
        assert_eq!(serialize!(123u8), "123");
        assert_eq!(serialize!(1234u16), "1234");
        assert_eq!(serialize!(123456u32), "123456");
        assert_eq!(serialize!(123456789u64), "123456789");
        assert_eq!(serialize!(1234567890u128), "1234567890");

        assert_eq!(serialize!(-123i8), "-123");
        assert_eq!(serialize!(-1234i16), "-1234");
        assert_eq!(serialize!(-123456i32), "-123456");
        assert_eq!(serialize!(-123456789i64), "-123456789");
        assert_eq!(serialize!(-1234567890i128), "-1234567890");

        assert_eq!(serialize!(123i8), "123");

        assert_eq!(serialize!(1.2345f32), "1.2345");
        assert_eq!(serialize!(2e-8f32), "0.00000002");
        assert_eq!(serialize!(-0.2e0f32), "-0.2");
    }

    #[test]
    fn deserialize_primitives() {
        assert_eq!(deserialize!("123", u8), 123u8);
        assert_eq!(deserialize!("1234", u16), 1234u16);
        assert_eq!(deserialize!("123456", u32), 123456u32);
        assert_eq!(deserialize!("123456789", u64), 123456789u64);
        assert_eq!(deserialize!("1234567890", u128), 1234567890u128);
        assert_eq!(deserialize!("-123", i8), -123i8);
        assert_eq!(deserialize!("-1234", i16), -1234i16);
        assert_eq!(deserialize!("-123456", i32), -123456i32);
        assert_eq!(deserialize!("-123456789", i64), -123456789i64);
        assert_eq!(deserialize!("-1234567890", i128), -1234567890i128);
        assert_eq!(deserialize!("123", i8), 123i8);
        assert_eq!(deserialize!("1.2345", f32), 1.2345f32);
        assert_eq!(deserialize!("0.00000002", f32), 2e-8f32);
        assert_eq!(deserialize!("-0.2", f64), -0.2e0f64);
    }
}
