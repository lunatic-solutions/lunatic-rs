// Represents a message tag.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Tag(pub(crate) i64);

static mut COUNTER: i64 = 0;

impl Tag {
    // Returns a unique tag inside of the process.
    pub(crate) fn new() -> Tag {
        unsafe {
            COUNTER += 1;
            Tag(COUNTER)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tag;

    #[test]
    fn tag_increments() {
        assert_eq!(Tag::new(), Tag(1));
        assert_eq!(Tag::new(), Tag(2));
        assert_eq!(Tag::new(), Tag(3));
        assert_eq!(Tag::new(), Tag(4));
    }
}
