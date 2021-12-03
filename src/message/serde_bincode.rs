use crate::{net::TcpStream, process::Process, Msg, Tag};
use std::io::{Read, Write};

use super::{DeserializeError, Serializer};

pub trait Bincode: serde::Serialize + serde::de::DeserializeOwned {}

impl<T: Bincode> Serializer<T> for T {
    fn serialize(data: &T, writer: &mut dyn Write) {
        bincode::serialize_into(writer, data).unwrap();
    }

    fn deserialize(reader: &mut dyn Read) -> Result<T, DeserializeError> {
        bincode::deserialize_from(reader).map_err(|e| e.into())
    }
}

impl Bincode for u8 {}
impl Bincode for u16 {}
impl Bincode for u32 {}
impl Bincode for u64 {}
impl Bincode for u128 {}
impl Bincode for usize {}
impl Bincode for i8 {}
impl Bincode for i16 {}
impl Bincode for i32 {}
impl Bincode for i64 {}
impl Bincode for i128 {}
impl Bincode for isize {}
impl Bincode for bool {}
impl Bincode for String {}
impl Bincode for () {}
impl<T: Bincode> Bincode for Vec<T> {}

//TODO why const generics are not supported? [T; N]
// impl <T: Bincode, const N: usize> Bincode for [T; N] {}
impl<T: Bincode> Bincode for [T; 0] {}
impl<T: Bincode> Bincode for [T; 1] {}
impl<T: Bincode> Bincode for [T; 2] {}
impl<T: Bincode> Bincode for [T; 3] {}
impl<T: Bincode> Bincode for [T; 4] {}
impl<T: Bincode> Bincode for [T; 5] {}
impl<T: Bincode> Bincode for [T; 6] {}
impl<T: Bincode> Bincode for [T; 7] {}
impl<T: Bincode> Bincode for [T; 8] {}
impl<T: Bincode> Bincode for [T; 9] {}

impl<A1, A2> Bincode for (A1, A2)
where
    A1: Bincode,
    A2: Bincode,
{
}
impl<A1, A2, A3> Bincode for (A1, A2, A3)
where
    A1: Bincode,
    A2: Bincode,
    A3: Bincode,
{
}
impl<A1, A2, A3, A4> Bincode for (A1, A2, A3, A4)
where
    A1: Bincode,
    A2: Bincode,
    A3: Bincode,
    A4: Bincode,
{
}

impl<T: Msg> Bincode for Process<T> {}
impl Bincode for TcpStream {}
impl Bincode for Tag {}
