use std::fmt::Debug;

use crate::{Id, Identifiable};
use diesel::{backend::Backend, deserialize, serialize, sql_types::Int8};

impl<T: Identifiable + Debug, B: Backend> serialize::ToSql<Int8, B> for Id<T>
where
    u64: serialize::ToSql<Int8, B>,
{
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, B>) -> serialize::Result {
        <u64 as serialize::ToSql<Int8, B>>::to_sql(&self.value(), out)
    }
}

impl<T: Identifiable, B: Backend> deserialize::FromSql<Int8, B> for Id<T>
where
    u64: deserialize::FromSql<Int8, B>,
{
    fn from_sql(bytes: <B as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        <u64 as deserialize::FromSql<Int8, B>>::from_sql(bytes).map(|id| Id::new(id))
    }
}
