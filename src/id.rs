use std::fmt;
use std::ops::Deref;

use anyhow::Result;
use rand::Rng;
use serde::{Serialize, Deserialize};

use crate::sqlx::impl_sqlx_traits_for;

type AString = arraystring::ArrayString<typenum::U22>;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash, Default, Debug)]
#[serde(transparent)]
pub struct Id(AString);

impl Id {
    /// Generate a base62 encoded unique (random) id of `len` characters.
    /// Maximum length is 22, which can encode 128 bits of data.
    /// Length 20 contains about 125 bits of random data.
    pub fn new_with_len(len: usize) -> Id {
        let mut id = AString::new();

        let len = std::cmp::min(len, 22);
        let max = if len < 22 {
            62u128.pow(len as u32)
        } else {
            u128::MAX
        };
        let mut rng = rand::thread_rng();
        let mut n = rng.gen_range(0 ..= max);

        // into base62.
        for _ in 0 .. len {
            let m = (n % 62) as u8;
            n /= 62;
            let c = match m {
                0 ..= 9 => m + b'0',
                10 ..= 35 => m - 10 + b'A',
                36 ..= 61 => m - 36 + b'a',
                _ => unreachable!(),
            };
            let _ = id.try_push(c.into());
        }

        Id(id)
    }

    /// Default length is 20.
    pub fn new() -> Id {
        Id::new_with_len(20)
    }

    pub fn from_str(s: &str) -> Result<Id> {
        if s.len() > 22 {
            bail!("Id::from_str: invalid Id (too long)");
        }
        Ok(Id(AString::try_from_str(s).unwrap()))
    }
}
impl_sqlx_traits_for!(Id, text);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl Deref for Id {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

use std::borrow::Cow;
use serde_json::Value;
use poem_openapi::{
    registry::{MetaSchema, MetaSchemaRef},
    types::{ToJSON, Type},
};

impl Type for Id {
    const IS_REQUIRED: bool = true;

    type RawValueType = AString;

    type RawElementValueType = AString;

    fn name() -> Cow<'static, str> {
        "string".into()
    }

    fn schema_ref() -> MetaSchemaRef {
        MetaSchemaRef::Inline(Box::new(MetaSchema::new("string")))
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(&self.0)
    }

    fn raw_element_iter<'b>(
        &'b self,
    ) -> Box<dyn Iterator<Item = &'b Self::RawElementValueType> + 'b> {
        Box::new(self.as_raw_value().into_iter())
    }
}

impl ToJSON for Id {
    fn to_json(&self) -> Option<Value> {
        Some(Value::String(self.to_string()))
    }
}
