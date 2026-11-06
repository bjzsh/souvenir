#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
use crate::Souvenir;
#[cfg(any(feature = "postgres", feature = "mysql"))]
use diesel::serialize;

/// Fail the build if `T`'s declared storage doesn't fit the column type.
macro_rules! require_storage {
    ($p:ty, $($pat:pat_param)|+ , $msg:literal) => {
        const {
            assert!(matches!(<$p as $crate::Tagged>::STORAGE, $($pat)|+), $msg);
        }
    };
}

/// `ToSql` for `Text` and `Binary`; only the output buffer differs between
/// backends, so each backend supplies how to write a value.
#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
macro_rules! backend_to_sql {
    ($backend:ty, |$out:ident, $value:ident| $write:expr) => {
        impl ::diesel::serialize::ToSql<::diesel::sql_types::Text, $backend> for $crate::Souvenir {
            fn to_sql<'b>(
                &'b self,
                $out: &mut ::diesel::serialize::Output<'b, '_, $backend>,
            ) -> ::diesel::serialize::Result {
                let $value = $crate::integration::diesel::support::Value::Text(*self);
                $write
            }
        }

        impl<T: $crate::Tagged> ::diesel::serialize::ToSql<::diesel::sql_types::Text, $backend>
            for $crate::Id<T>
        {
            fn to_sql<'b>(
                &'b self,
                $out: &mut ::diesel::serialize::Output<'b, '_, $backend>,
            ) -> ::diesel::serialize::Result {
                require_storage!(
                    T,
                    $crate::Storage::Text,
                    "a Text column needs a Tagged type with Storage::Text"
                );
                let $value = $crate::integration::diesel::support::Value::Text(self.to_untyped());
                $write
            }
        }

        impl<T: $crate::Tagged> ::diesel::serialize::ToSql<::diesel::sql_types::Binary, $backend>
            for $crate::Id<T>
        {
            fn to_sql<'b>(
                &'b self,
                $out: &mut ::diesel::serialize::Output<'b, '_, $backend>,
            ) -> ::diesel::serialize::Result {
                require_storage!(
                    T,
                    $crate::Storage::Bytes | $crate::Storage::Uuid,
                    "a Binary column needs a Tagged type with Storage::Bytes or Storage::Uuid"
                );
                let stored = $crate::integration::stored::StoredBinary::of(self);
                let $value = $crate::integration::diesel::support::Value::Binary(stored.as_slice());
                $write
            }
        }
    };
}

/// A value about to be written by `backend_to_sql!`.
#[derive(Clone, Copy)]
#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
pub(crate) enum Value<'a> {
    Text(Souvenir),
    Binary(&'a [u8]),
}

#[cfg(any(feature = "postgres", feature = "mysql"))]
pub(crate) fn write_value(out: &mut impl std::io::Write, value: Value<'_>) -> serialize::Result {
    match value {
        Value::Text(id) => {
            out.write_all(id.encode(&mut [0; crate::spec::MAX_TEXT_LEN]).as_bytes())?
        }
        Value::Binary(bytes) => out.write_all(bytes)?,
    }
    Ok(serialize::IsNull::No)
}
