/// The literal `*` URI.
///
/// ## Serde
///
/// For convience, `Asterisk` implements `Serialize` and `Deserialize`.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Asterisk;

impl std::fmt::Display for Asterisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "*".fmt(f)
    }
}

#[cfg(feature = "serde")]
mod serde {
    use std::fmt;

    use super::Asterisk;
    use _serde::{ser::{Serialize, Serializer}, de::{Deserialize, Deserializer, Error, Visitor}};

    impl Serialize for Asterisk {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str("*")
        }
    }

    struct AsteriskVistor;

    impl<'a> Visitor<'a> for AsteriskVistor {
        type Value = Asterisk;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "asterisk Uri")
        }

        // This method should be the only one that needs to be implemented, since the
        // other two methods (`visit_string` & `visit_borrowed_str`) have default implementations
        // that just call this one. We don't benefit from taking ownership or borrowing from the
        // deserializer, so this should be perfect.
        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
            if v == "*" {
                Ok(Asterisk)
            }else {
                Err(E::custom(format!("`{}` is not a valid asterisk uri", v)))
            }
        }
    }

    impl<'de> Deserialize<'de> for Asterisk {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_str(AsteriskVistor)
        }
    }
}
