/// This macro implements the sqlx::Decode, sqlx::Encode, and sqlx::Type traits
/// for any type so that JSON is used to read/write the type from/to the database.
///
/// Mostly copied from
/// https://github.com/launchbadge/sqlx/blob/main/sqlx-core/src/sqlite/types/json.rs
macro_rules! impl_sqlx_traits_for {
    ($ty:ty) => {
        impl_sqlx_traits_for!($ty, serde_json);
    };
    ($ty:ty, json) => {
        impl_sqlx_traits_for!($ty, serde_json);
    };
    ($ty:ty, text) => {
        impl_sqlx_traits_for!($ty, serde_plain);
    };
    ($ty:ty, $codec:ident) => {
        impl sqlx::Type<sqlx::Sqlite> for $ty {
            fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
                <sqlx::types::Json<$ty> as sqlx::Type<sqlx::Sqlite>>::type_info()
            }

            fn compatible(ty: &sqlx::sqlite::SqliteTypeInfo) -> bool {
                <sqlx::types::Json<$ty> as sqlx::Type<sqlx::Sqlite>>::compatible(ty)
            }
        }

        impl sqlx::Encode<'_, sqlx::sqlite::Sqlite> for $ty
        where
            $ty: serde::Serialize,
        {
            fn encode_by_ref(
                &self,
                buf: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'_>>,
            ) -> sqlx::encode::IsNull {
                let json_string_value =
                    $codec::to_string(self).expect("serde failed to convert to string");

                sqlx::Encode::<sqlx::sqlite::Sqlite>::encode(json_string_value, buf)
            }
        }

        impl<'r> sqlx::Decode<'r, sqlx::sqlite::Sqlite> for $ty
        where
            $ty: 'r + serde::Deserialize<'r>,
        {
            fn decode(
                value: sqlx::sqlite::SqliteValueRef<'r>,
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let string_value = <&str as sqlx::Decode<sqlx::sqlite::Sqlite>>::decode(value)?;

                $codec::from_str(&string_value).map_err(Into::into)
            }
        }
    };
}
pub(crate) use impl_sqlx_traits_for;
