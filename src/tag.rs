/// A `i64` value used as a message tag.
///
/// Processes can selectively receive messages based on the message's tag. This mechanism can be
/// used to handle messages in a different order from their arrival.
///
/// Creating a new tag will return a process-unique value. Some tag values are reserved for
/// internal use only, but the range from 64 to 128 can be used by the developer to assign
/// application specific meaning.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Tag(i64);

impl Tag {
    // Create tag of any value.
    pub(crate) fn from(id: i64) -> Tag {
        Tag(id)
    }

    /// Returns a unique tag inside this process.
    ///
    /// Two calls to `Tag::new()` are guaranteed to return a unique tag only if they occurred
    /// inside the same process.
    pub fn new() -> Tag {
        unsafe {
            COUNTER += 1;
            Tag(COUNTER)
        }
    }

    /// Returns a special tag that is used by [`Process::send`](crate::Process) and awaited on by
    /// [Mailbox::receive](crate::Mailbox).
    ///
    /// Most messages where the order is not significant use this tag.
    pub fn none() -> Tag {
        Tag(1)
    }

    /// Create a special purpose tag.
    ///
    /// The `id` must be in the range between 64 and 128 or the function will return `None`.
    pub fn special(id: i64) -> Option<Tag> {
        if (64..=128).contains(&id) {
            Some(Tag(id))
        } else {
            None
        }
    }

    pub fn id(&self) -> i64 {
        self.0
    }
}

// Reserve first 128 tags for special purposes.
static mut COUNTER: i64 = 128;

impl Tag {}

impl Default for Tag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Tag;
    use lunatic_test::test;

    #[test]
    fn tag_increments() {
        assert_eq!(Tag::new(), Tag(129));
        assert_eq!(Tag::new(), Tag(130));
        assert_eq!(Tag::new(), Tag(131));
        assert_eq!(Tag::new(), Tag(132));
    }

    #[test]
    fn test_special_tag() {
        assert!(Tag::special(64).is_some());
        assert!(Tag::special(128).is_some());

        assert!(Tag::special(63).is_none());
        assert!(Tag::special(129).is_none());

        assert_eq!(Tag::none(), Tag::none());
    }
}
