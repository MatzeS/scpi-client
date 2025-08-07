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
    use crate::{ScpiDeserialize, ScpiSerialize};

    #[test]
    fn serialize_primitives() {
        assert_eq!(123u8.serialize_to_string(), "123");
        assert_eq!(1234u16.serialize_to_string(), "1234");
        assert_eq!(123456u32.serialize_to_string(), "123456");
        assert_eq!(123456789u64.serialize_to_string(), "123456789");
        assert_eq!(1234567890u128.serialize_to_string(), "1234567890");

        assert_eq!((-123i8).serialize_to_string(), "-123");
        assert_eq!((-1234i16).serialize_to_string(), "-1234");
        assert_eq!((-123456i32).serialize_to_string(), "-123456");
        assert_eq!((-123456789i64).serialize_to_string(), "-123456789");
        assert_eq!((-1234567890i128).serialize_to_string(), "-1234567890");

        assert_eq!(123i8.serialize_to_string(), "123");

        assert_eq!((1.2345f32).serialize_to_string(), "1.2345");
        assert_eq!((2e-8f32).serialize_to_string(), "0.00000002");
        assert_eq!((-0.2e0f32).serialize_to_string(), "-0.2");
    }

    #[test]
    fn deserialize_primitives() {
        assert_eq!(u8::deserialize_complete("123").unwrap(), 123u8);
        assert_eq!(u16::deserialize_complete("1234").unwrap(), 1234u16);
        assert_eq!(u32::deserialize_complete("123456").unwrap(), 123456u32);
        assert_eq!(
            u64::deserialize_complete("123456789").unwrap(),
            123456789u64
        );
        assert_eq!(
            u128::deserialize_complete("1234567890").unwrap(),
            1234567890u128
        );
        assert_eq!(i8::deserialize_complete("-123").unwrap(), -123i8);
        assert_eq!(i16::deserialize_complete("-1234").unwrap(), -1234i16);
        assert_eq!(i32::deserialize_complete("-123456").unwrap(), -123456i32);
        assert_eq!(
            i64::deserialize_complete("-123456789").unwrap(),
            -123456789i64
        );
        assert_eq!(
            i128::deserialize_complete("-1234567890").unwrap(),
            -1234567890i128
        );
        assert_eq!(i8::deserialize_complete("123").unwrap(), 123i8);
        assert_eq!(f32::deserialize_complete("1.2345").unwrap(), 1.2345f32);
        assert_eq!(f32::deserialize_complete("0.00000002").unwrap(), 2e-8f32);
        assert_eq!(f64::deserialize_complete("-0.2").unwrap(), -0.2e0f64);
    }
}
