use sqlx::{encode::IsNull, Database, Decode, Encode, Type};

use crate::{Id, Identifiable};

impl<T: Identifiable, DB: Database> Type<DB> for Id<T>
where
    u64: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <u64 as Type<DB>>::type_info()
    }

    fn compatible(ty: &<DB as Database>::TypeInfo) -> bool {
        <u64 as Type<DB>>::compatible(ty)
    }
}

impl<'q, T: Identifiable, DB: Database> Encode<'q, DB> for Id<T>
where
    u64: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <u64 as Encode<'q, DB>>::encode_by_ref(&u64::from_be_bytes(self.value()), buf)
    }

    fn encode(
        self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        <u64 as Encode<'q, DB>>::encode(u64::from_be_bytes(self.value()), buf)
    }

    fn produces(&self) -> Option<<DB as Database>::TypeInfo> {
        <u64 as Encode<'q, DB>>::produces(&u64::from_be_bytes(self.value()))
    }
}

impl<'r, T: Identifiable, DB: Database> Decode<'r, DB> for Id<T>
where
    u64: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let int = <u64 as Decode<'r, DB>>::decode(value)?;

        Ok(Self::from(int))
    }
}
