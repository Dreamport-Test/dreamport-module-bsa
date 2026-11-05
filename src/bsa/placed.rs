//! BsaPlaced.cs: where one file's own bytes lie, past any embedded name and declared size, and what they decode to.

pub const STORED: &str = "stored";

pub struct Placed {
    pub data_at: i64,
    pub stored: i32,
    pub size: i64,
    /// DreamportBlocks.Stored, or the decoder of the archive's version.
    pub decoder: &'static str,
}
