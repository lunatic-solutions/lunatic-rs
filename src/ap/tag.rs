use crate::Tag;

/// Unique tags that also hold additional `u6` data used to dispatch to the
/// correct handler function.
///
/// The reason only `u6` is used is that the first 2 bits are reserved for
/// future use cases. `AbstractProcesses` can have at most 16 handler functions
/// and this should be enough space to encode all of them.
pub(crate) struct AbstractProcessTag;

impl AbstractProcessTag {
    /// Returns a [`Tag`] with `u6` data encoded into it.
    #[track_caller]
    pub(crate) fn from_u6(data: u8) -> Tag {
        assert!(data < 64, "Only values less than 64 can fit into a `u6`");
        let tag = Tag::new();
        let id = ((data as i64) << 56) | tag.id(); // Fit data into top of i64.
        Tag::from(id)
    }

    /// Extracts `u6` data encoded into the [`Tag`].
    ///
    /// The returned `Tag` doesn't contain the data anymore.
    pub(crate) fn extract_u6_data(tag: Tag) -> (Tag, u8) {
        let data = (tag.id() >> 56) as u8; // extract data
        let tag = tag.id() & 0xFFFFFFFFFFFFFF; // remove data from first byte
        (Tag::from(tag), data)
    }
}
