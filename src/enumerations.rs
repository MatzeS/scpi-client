#[macro_export]
macro_rules! scpi_enum {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => $literal:expr
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        impl $crate::ScpiDeserialize for $name {
            fn deserialize(input: &mut &str) -> $crate::Result<Self> {
                $(
                    if let Ok(()) = $crate::match_literal(input, $literal) {
                        return Ok(Self::$variant);
                    }
                )*
                Err($crate::Error::ResponseDecoding(format!("Unexpected token for {}: `{}`", stringify!($name), input)))
            }
        }

        impl $crate::ScpiSerialize for $name {
            fn serialize(&self, out: &mut String) {
                match self {
                    $(
                        Self::$variant => out.push_str($literal),
                    )*
                }
            }
        }
    };
}
