// Represents a message tag.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Tag(i64);

impl Tag {
    pub(crate) fn from(id: i64) -> Tag {
        Tag(id)
    }

    pub fn id(&self) -> i64 {
        self.0
    }
}

// Reserve first 128 tags for special purposes.
static mut COUNTER: i64 = 128;

impl Tag {
    // Returns a unique tag inside of the process.
    pub fn new() -> Tag {
        unsafe {
            COUNTER += 1;
            Tag(COUNTER)
        }
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use lunatic_test::test;
    // use super::Tag;

    #[test]
    fn tag_increments() {
        // All tests run in the same process, this makes it impossible to run per process checks.
        // assert_eq!(Tag::new(), Tag(129));
        // assert_eq!(Tag::new(), Tag(130));
        // assert_eq!(Tag::new(), Tag(131));
        // assert_eq!(Tag::new(), Tag(132));
    }
}
