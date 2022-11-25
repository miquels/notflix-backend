use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

///
/// JVec<T> is a transparent wrapper for Vec<T>.
///
/// It implements the `sqlx` Encode/Decode traits so that it can be retrieved
/// from the database or stored in the database as JSON, and it also implements
/// the `poem_openapi` traits that are normally implemented by `derive(Object)`.
///
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JVec<T>(pub Vec<T>);

impl<T> JVec<T> {
    /// Constructs a new, empty `Vec<T>`.
    pub fn new() -> JVec<T> {
        JVec(Vec::new())
    }

    /// Constructs a new, empty `Vec<T>` with at least the specified capacity.
    pub fn with_capacity(cap: usize) -> JVec<T> {
        JVec(Vec::with_capacity(cap))
    }
}

impl<T> Deref for JVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for JVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> AsRef<JVec<T>> for JVec<T> {
    fn as_ref(&self) -> &JVec<T> {
        self
    }
}

impl<T> AsMut<JVec<T>> for JVec<T> {
    fn as_mut(&mut self) -> &mut JVec<T> {
        self
    }
}

//
// Implementation of `sqlx` traits, so that the type gets
// encoded/decodec in JSON to/from the dabase.
//
// Mostly copied from
// https://github.com/launchbadge/sqlx/blob/main/sqlx-core/src/sqlite/types/json.rs
//
impl<T> sqlx::Type<sqlx::Sqlite> for JVec<T> {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        // sqlx::sqlite::SqliteTypeInfo(sqlx::sqlite::type_info::DataType::Text)
        <sqlx::types::Json<T> as sqlx::Type<sqlx::Sqlite>>::type_info()
    }

    fn compatible(ty: &sqlx::sqlite::SqliteTypeInfo) -> bool {
        // <&str as sqlx::Type<sqlx::sqlite::Sqlite>>::compatible(ty)
        <sqlx::types::Json<T> as sqlx::Type<sqlx::Sqlite>>::compatible(ty)
    }
}

impl<T> sqlx::Encode<'_, sqlx::sqlite::Sqlite> for JVec<T>
where
    T: serde::Serialize,
{
    fn encode_by_ref(
        &self,
        buf: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'_>>,
    ) -> sqlx::encode::IsNull {
        let json_string_value =
            serde_json::to_string(&self.0).expect("serde_json failed to convert to string");

        sqlx::Encode::<sqlx::sqlite::Sqlite>::encode(json_string_value, buf)
    }
}

impl<'r, T> sqlx::Decode<'r, sqlx::sqlite::Sqlite> for JVec<T>
where
    T: 'r + serde::Deserialize<'r>,
{
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let string_value = <&str as sqlx::Decode<sqlx::sqlite::Sqlite>>::decode(value)?;

        serde_json::from_str(&string_value).map(JVec).map_err(Into::into)
    }
}

//
// Implementation of `poem_openapi` traits. Normally you'd just slap
// #derive(Object>` on the JVec<T> type, but the derive macro doesn't
// understand generics so we have to implement the traits manually.
//
// Mostly copied from
// https://github.com/poem-web/poem/blob/master/poem-openapi/src/types/external/vec.rs
//
use poem::web::Field as PoemField;
use poem_openapi::{
    registry::{MetaSchema, MetaSchemaRef, Registry},
    types::{
        ParseError, ParseFromJSON, ParseFromMultipartField, ParseFromParameter, ParseResult,
        ToJSON, Type,
    },
};
use serde_json::Value;
use std::borrow::Cow;

impl<T: Type> Type for JVec<T> {
    const IS_REQUIRED: bool = true;

    type RawValueType = Self;

    type RawElementValueType = T::RawValueType;

    fn name() -> Cow<'static, str> {
        format!("[{}]", T::name()).into()
    }

    fn schema_ref() -> MetaSchemaRef {
        MetaSchemaRef::Inline(Box::new(MetaSchema {
            items: Some(Box::new(T::schema_ref())),
            ..MetaSchema::new("array")
        }))
    }

    fn register(registry: &mut Registry) {
        T::register(registry);
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        Box::new(self.iter().filter_map(|item| item.as_raw_value()))
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: ParseFromJSON> ParseFromJSON for JVec<T> {
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        let value = value.unwrap_or_default();
        match value {
            Value::Array(values) => {
                let mut res = JVec::with_capacity(values.len());
                for value in values {
                    res.push(T::parse_from_json(Some(value)).map_err(ParseError::propagate)?);
                }
                Ok(res)
            },
            _ => Err(ParseError::expected_type(value)),
        }
    }
}

impl<T: ParseFromParameter> ParseFromParameter for JVec<T> {
    fn parse_from_parameter(_value: &str) -> ParseResult<Self> {
        unreachable!()
    }

    fn parse_from_parameters<I: IntoIterator<Item = A>, A: AsRef<str>>(
        iter: I,
    ) -> ParseResult<Self> {
        let mut values = JVec::new();
        for s in iter {
            values.push(
                T::parse_from_parameters(std::iter::once(s.as_ref()))
                    .map_err(ParseError::propagate)?,
            );
        }
        Ok(values)
    }
}

#[poem::async_trait]
impl<T: ParseFromMultipartField> ParseFromMultipartField for JVec<T> {
    async fn parse_from_multipart(field: Option<PoemField>) -> ParseResult<Self> {
        match field {
            Some(field) => {
                let item =
                    T::parse_from_multipart(Some(field)).await.map_err(ParseError::propagate)?;
                Ok(JVec(vec![item]))
            },
            None => Ok(JVec::new()),
        }
    }

    async fn parse_from_repeated_field(mut self, field: PoemField) -> ParseResult<Self> {
        let item = T::parse_from_multipart(Some(field)).await.map_err(ParseError::propagate)?;
        self.push(item);
        Ok(self)
    }
}

impl<T: ToJSON> ToJSON for JVec<T> {
    fn to_json(&self) -> Option<Value> {
        let mut values = Vec::with_capacity(self.len());
        for item in &self.0 {
            if let Some(value) = item.to_json() {
                values.push(value);
            }
        }
        Some(Value::Array(values))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_from_parameters() {
        let values = JVec::<i32>::parse_from_parameters(vec!["100", "200", "300"]).unwrap();
        assert_eq!(values, vec![100, 200, 300]);
    }
}
