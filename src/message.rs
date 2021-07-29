/// A primitive for exchanging data between [processes](crate::process::Process).
///
/// When a [`Message`] is sent to another process all the data is copied, as processes don't
/// share any memory space. Every type that implements [`serde::Serialize`] **and**
/// [`serde::Deserialize`] automatically implements [`Message`].
///
/// Lunatic resources ([`Process`](crate::process::Process), [`TcpStream`](crate::net::TcpStream),
/// ...) are owned by processes and some extra steps need to be taken when sending them between
/// processes. To simplify implementing [`Message`] for types that hold lunatic resources, a derive
/// macro is provided [`lunatic::derive::Message`](lunatic_message_derive::Message).
pub trait Message {
    /// Returns bytes read from data and the deserialized value.
    fn from_bincode(data: &[u8], resources: &[u64]) -> (usize, Self);
    /// Consumes itself and writes the serialized representation to dest.
    ///
    /// ### Safety
    ///
    /// Some lunatic resources will add themself to the next message and only write their index
    /// inside of the resource array to `dest`. If not handled correctly, this can lead to resource
    /// leaks. This method should never be implemented manually, instead you should use:
    /// `#[derive(serde::Serialize, serde::Deserialize)]` or `#[derive(lunatic::derive::Message)]`
    unsafe fn to_bincode(self, dest: &mut Vec<u8>);
}

impl<T> Message for T
where
    T: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn from_bincode(data: &[u8], _: &[u64]) -> (usize, Self) {
        let value: T = bincode::deserialize(data).unwrap();
        let bytes_read = bincode::serialized_size(&value).unwrap();
        (bytes_read as usize, value)
    }

    #[allow(clippy::wrong_self_convention)]
    unsafe fn to_bincode(self, dest: &mut Vec<u8>) {
        let serialized = bincode::serialize(&self).unwrap();
        dest.extend(serialized);
    }
}
