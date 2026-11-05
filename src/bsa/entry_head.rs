//! BsaEntryHead.cs, read in place: the name a file's data may start with, held to its tables' name, and the size a
//! compressed file declares. The host holds that size to its limits when the file is read.

use super::bytes::{same_ignoring_case, slashed, u32_at};
use super::header::Header;
use super::placed::{Placed, STORED};
use super::record::Record;
use super::unpack;
use super::Failed;
use crate::host::Container;

const DECLARED_SIZE: usize = 4;
const LONGEST_NAME: usize = 1 + 255;

pub fn place(file: &Container, header: &Header, record: &Record, path: &[u8]) -> Result<Placed, Failed> {
    if record.offset as i64 + record.size as i64 > file.len() {
        return Err(unpack::OUTSIDE.into());
    }

    let wanted = (if header.embeds_names() { LONGEST_NAME } else { 0 }) + (if record.compressed { DECLARED_SIZE } else { 0 });
    let head = file.at(record.offset as i64, wanted.min(record.size as usize) as i64)?.ok_or(unpack::OUTSIDE)?;

    let mut used = 0usize;
    if header.embeds_names() {
        if head.is_empty() || head[0] as usize + 1 > head.len() {
            return Err(unpack::OUTSIDE.into());
        }
        let embedded = slashed(&head[1..1 + head[0] as usize]);
        if !same_ignoring_case(&embedded, path) {
            return Err(unpack::OTHER_NAME.into());
        }
        used = 1 + head[0] as usize;
    }

    let placed = Placed {
        data_at: record.offset as i64 + used as i64,
        stored: record.size as i32 - used as i32,
        size: record.size as i64 - used as i64,
        decoder: STORED,
    };

    if record.compressed {
        compressed(placed, &head, used, header.decoder())
    } else {
        Ok(placed)
    }
}

/// A block's size is a whole number, and a block of no bytes decodes to nothing.
fn compressed(mut placed: Placed, head: &[u8], used: usize, decoder: &'static str) -> Result<Placed, Failed> {
    if head.len() < used + DECLARED_SIZE {
        return Err(unpack::OUTSIDE.into());
    }

    placed.size = u32_at(head, used) as i64;
    placed.data_at += DECLARED_SIZE as i64;
    placed.stored -= DECLARED_SIZE as i32;
    placed.decoder = decoder;

    if placed.size > i32::MAX as i64 {
        return Err(unpack::TOO_LARGE_COMPRESSED.into());
    }
    if placed.size > 0 && placed.stored == 0 {
        return Err(unpack::EXPANDS.into());
    }
    Ok(placed)
}
