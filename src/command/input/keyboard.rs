pub(crate) mod mapping;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Press {
    pub key: Key,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    pub key: Key,
}

crate::command::request!(Press, KeyboardPress, ());
crate::command::request!(Release, KeyboardRelease, ());

macro_rules! key_types {
    ($($variant:ident => ($token:literal, $code:ident, $logical:expr)),+ $(,)?) => {
        /// Layout-independent physical keys with fixed logical mappings.
        /// Keyboard events carry no text; text entry is a separate command.
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum Key {
            $(#[doc = concat!("Wire token `", $token, "`.")]
              $variant,)+
            /// Unsupported tokens are retained so the game can reject them as `invalid_key`.
            Unknown(String),
        }

        impl Key {
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $token,)+
                    Self::Unknown(token) => token,
                }
            }

        }

        impl<'de> Deserialize<'de> for Key {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let token = String::deserialize(deserializer)?;
                Ok(match token.as_str() {
                    $($token => Self::$variant,)+
                    _ => Self::Unknown(token),
                })
            }
        }

    };
}

impl Serialize for Key {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

mapping::keys!(key_types);
