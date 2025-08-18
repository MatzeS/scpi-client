#![feature(pattern)]

use std::str::pattern::{Pattern, Searcher};
use thiserror::Error;

pub mod enumerations;
pub mod primitives;

#[derive(Error, Debug)]
pub enum Error {
    // TODO build more errors for decoding of values (string to f32 conversion)
    //  and unexpected symbols
    #[error("Received data does not match expected format: {0}")]
    ResponseDecoding(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait ScpiSerialize {
    fn serialize(&self, out: &mut String);

    fn serialize_to_string(&self) -> String {
        let mut out = String::new();
        self.serialize(&mut out);
        out
    }
}

pub trait ScpiDeserialize
where
    Self: Sized,
{
    // TODO maybe this should have an associated type so the implementer
    // can choose the error type.

    fn deserialize(input: &mut &str) -> Result<Self>;

    fn deserialize_complete(mut input: &str) -> Result<Self> {
        let result = Self::deserialize(&mut input)?;
        check_empty(input).unwrap();
        Ok(result)
    }
}

pub trait ScpiRequest: ScpiSerialize {
    // Note, that the response does intentionally not depend on ScpiDeserialize
    // because an empty response cannot be deserialized.
    // TODO maybe this should be modeled better by splitting scpi commands and queries, one with response, one without.
    type Response;
}

// TODO remove? is thits truly universal?
impl<T: ScpiSerialize> ScpiSerialize for Option<T> {
    fn serialize(&self, out: &mut String) {
        if let Some(inner) = self {
            inner.serialize(out);
        }
    }
}

/// Response type to indicate that no answer is expected.
/// The communication driver will not attempt to receive a
/// response for an associated request.
pub struct EmptyResponse;

#[macro_export]
macro_rules! impl_scpi_serialize {
    ($type:ty, [ $( $part:tt ),* $(,)? ]) => {
        impl $crate::ScpiSerialize for $type {
            fn serialize(&self, out: &mut String) {
                $(
                    impl_scpi_serialize!(@part self, out, $part);
                )*
            }
        }
    };

    // Handle string literals
    (@part $self:ident, $out:ident, $lit:literal) => {
        $out.push_str($lit);
    };

    // Handle field names
    (@part $self:ident, $out:ident, $field:ident) => {
        $self.$field.serialize($out);
    };

}

// TODO naming is bad here with request and structs FooRequest...
#[macro_export]
macro_rules! impl_scpi_request {
    ($request:ty, $response:ty) => {
        impl $crate::ScpiRequest for $request {
            type Response = $response;
        }
    };
}

pub fn match_literal(input: &mut &str, literal: &'static str) -> Result<()> {
    if let Some(rest) = input.strip_prefix(literal) {
        *input = rest;
        Ok(())
    } else {
        Err(Error::ResponseDecoding(format!(
            "Expected literal `{literal}` not matched `{input}`"
        )))
    }
}

pub fn read_until<'a>(input: &mut &'a str, delimiter: char) -> Result<&'a str> {
    if let Some(index) = input.find(delimiter) {
        let (head, tail) = input.split_at(index);
        *input = &tail[1..]; // from 1 to skip delimiter
        Ok(head)
    } else {
        Err(Error::ResponseDecoding(format!(
            "Expected `{delimiter}` in `{input}`"
        )))
    }
}

pub fn read_while<'a, P>(input: &mut &'a str, pattern: P) -> &'a str
where
    P: Pattern,
{
    let mut searcher = pattern.into_searcher(input);

    let split = searcher
        .next_reject()
        .map(|(split, _end)| split)
        .unwrap_or(input.len());

    let (head, tail) = input.split_at(split);
    *input = tail;
    head
}

pub fn read_exact<'a>(input: &mut &'a str, len: usize) -> Result<&'a str> {
    if input.len() < len {
        return Err(Error::ResponseDecoding(format!(
            "Failed to read {len} characters from `{input}`"
        )));
    }

    let (head, tail) = input.split_at(len);
    *input = tail;
    Ok(head)
}

pub fn read_all(input: &mut &str) -> Result<String> {
    let result = input.to_string();
    *input = "";
    Ok(result)
}

pub fn check_empty(input: &str) -> Result<()> {
    if input.is_empty() {
        Ok(())
    } else {
        Err(Error::ResponseDecoding(format!(
            "Response should be empty/fully deserialized, but still has content: `{input}`"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_empty() {
        assert!(check_empty("").is_ok());
        assert!(check_empty("x").is_err());
    }

    #[test]
    fn test_read_exact() {
        let input = &mut "1234";
        assert_eq!(read_exact(input, 2).unwrap(), "12");
        assert!(read_exact(input, 3).is_err());
        assert_eq!(read_exact(input, 2).unwrap(), "34");
        assert!(check_empty(input).is_ok());
    }

    #[test]
    fn test_match_literal() {
        let input = &mut "1234";
        assert!(match_literal(input, "12").is_ok());
        assert!(match_literal(input, "12").is_err());
        assert!(match_literal(input, "34").is_ok());
        assert!(check_empty(input).is_ok());
    }

    #[test]
    fn test_read_until() {
        let input = &mut "12,34";
        assert_eq!(read_until(input, ',').unwrap(), "12");
        assert!(match_literal(input, "34").is_ok());
        assert!(check_empty(input).is_ok());
    }

    #[test]
    fn test_read_while() {
        let input = &mut "12,34";
        assert_eq!(read_while(input, char::is_numeric), "12");
        assert!(match_literal(input, ",").is_ok());
        assert_eq!(read_while(input, char::is_numeric), "34");
        assert!(check_empty(input).is_ok());
    }

    #[test]
    fn test_read_all() {
        let input = &mut "12,34\nasdf";
        assert_eq!(read_all(input).unwrap(), "12,34\nasdf");
    }
}
