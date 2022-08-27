use std::fmt;
use std::ops::Deref;

/// Any function pointer that can be converted to and from usize
pub trait Fn<T> {
    fn id(&self) -> usize;
    fn from_id(id: usize) -> T;
}

macro_rules! impl_types {
    ( $($ty:ident),* $(,)? ) => {
        impl<Res, $($ty,)*> Fn<Self> for fn($($ty,)*) -> Res
        {
            fn from_id(id: usize) -> Self {
                unsafe { std::mem::transmute(id) }
            }

            fn id(&self) -> usize {
                *self as usize
            }
        }
    };
}

// impl Fn<T> for most fn pointers
impl_types!();
impl_types!(T1);
impl_types!(T1, T2);
impl_types!(T1, T2, T3);
impl_types!(T1, T2, T3, T4);
impl_types!(T1, T2, T3, T4, T5);
impl_types!(T1, T2, T3, T4, T5, T6);
impl_types!(T1, T2, T3, T4, T5, T6, T7);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

/// Reference to a function that can be sent to another process
#[derive(Copy, Clone, Debug)]
pub struct FuncRef<T>(T);

impl<T> FuncRef<T>
where
    T: Fn<T> + Copy,
{
    pub fn new(func: T) -> Self {
        FuncRef(func)
    }

    pub fn get(&self) -> T {
        self.0
    }
}

impl<T> Deref for FuncRef<T>
where
    T: Fn<T>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> serde::Serialize for FuncRef<T>
where
    T: Fn<T>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.id() as u64)
    }
}

impl<'de, T> serde::Deserialize<'de> for FuncRef<T>
where
    T: Fn<T>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct U64Visitor;
        impl<'de> serde::de::Visitor<'de> for U64Visitor {
            type Value = u64;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("u64 representing a function pointer")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v)
            }
        }

        let id = deserializer.deserialize_u64(U64Visitor)? as usize;
        Ok(FuncRef(T::from_id(id)))
    }
}
