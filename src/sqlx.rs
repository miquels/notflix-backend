/// This macro implements the sqlx::Decode, sqlx::Encode, and sqlx::Type traits
/// for any type so that JSON is used to read/write the type from/to the database.
macro_rules! impl_sqlx_traits_for {
    ($ty:ty) => {
        impl_sqlx_traits_for!($ty, serde_plain);
    };
    ($ty:ty, json) => {
        impl_sqlx_traits_for!($ty, serde_json);
    };
    ($ty:ty, text) => {
        impl_sqlx_traits_for!($ty, serde_plain);
    };
    ($ty:ty, $codec:ident) => {
        impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for $ty
        where
            &'r str: sqlx::Decode<'r, DB>
        {
            fn decode(
                value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
            ) -> Result<$ty, Box<dyn std::error::Error + 'static + Send + Sync>> {
                let value = <&str as sqlx::Decode<DB>>::decode(value)?;
                Ok($codec::from_str(value)?)
            }
        }

        impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for $ty {
            fn encode(self, args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>) -> sqlx::encode::IsNull {
                self.encode_by_ref(args)
            }

            fn encode_by_ref(&self, args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>) -> sqlx::encode::IsNull {
                let json = $codec::to_string(self).unwrap_or(String::from(
                        r#"{"error":"failed to encode"}"#
                ));
                args.push(sqlx::sqlite::SqliteArgumentValue::Text(std::borrow::Cow::Owned(json)));

                sqlx::encode::IsNull::No
            }
        }

        impl sqlx::Type<sqlx::Sqlite> for $ty {
            fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
                String::type_info()
            }
        }
    };
}
pub(crate) use impl_sqlx_traits_for;
