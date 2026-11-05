//! BsaLocate.cs: each file located, never decoded. A compressed file is one block the host's decoder (or a module's)
//! reads back. The archive's proof covers its header and every table, up to the end of its names.

use super::entry_head;
use super::listing::Listing;
use super::placed::{Placed, STORED};
use super::record::Record;
use super::Failed;
use crate::host::{Container, Refused};

const AS_IT_LIES: i32 = 0;
const COMPRESSED: i32 = 1;

/// DreamportRange's fields, as a reader declares them.
pub struct Range {
    pub offset: i64,
    pub length: i64,
    pub method: i32,
    pub size: i64,
    pub decoder: &'static str,
    pub block_size: i32,
    pub blocks: Vec<i32>,
}

/// One file's range, or the problem the C# reader refuses it with when it is selected.
pub fn one(file: &Container, listing: &Listing, record: &Record, path: &[u8]) -> Result<Result<Range, &'static str>, Refused> {
    match entry_head::place(file, &listing.header, record, path) {
        Ok(placed) => Ok(Ok(take(&placed))),
        Err(Failed::Problem(problem)) => Ok(Err(problem)),
        Err(Failed::Host(refused)) => Err(refused),
    }
}

/// The archive's proof: from its start to the end of its names.
pub fn proven(listing: &Listing) -> (i64, i64) {
    (0, listing.names_end)
}

/// A compressed file of no bytes has no block: it is served as the nothing it declares.
fn take(placed: &Placed) -> Range {
    if placed.decoder == STORED || placed.size == 0 {
        Range { offset: placed.data_at, length: placed.size, method: AS_IT_LIES, size: placed.size, decoder: STORED, block_size: 0, blocks: Vec::new() }
    } else {
        Range {
            offset: placed.data_at,
            length: placed.stored as i64,
            method: COMPRESSED,
            size: placed.size,
            decoder: placed.decoder,
            block_size: placed.size as i32,
            blocks: vec![placed.stored],
        }
    }
}
