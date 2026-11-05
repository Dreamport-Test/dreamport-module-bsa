//! BsaListing.cs: every file a BSA names, its folder records, each folder's name and file records where its record
//! says they are, then the names, held to the counts the header declares. Nothing of the files' data is read.

use super::bytes::{slashed, u32_at, u64_at};
use super::header::{self, Header};
use super::record::{self, Record};
use super::unpack;
use super::Failed;
use crate::host::{Container, Refused, TOO_LARGE};

const FILE_RECORD: usize = 16;
const SIZE_BITS: u32 = 0x3FFF_FFFF;
const TOGGLE: u32 = 0x4000_0000;

pub struct Listing {
    pub header: Header,
    pub folders: Vec<Vec<u8>>,
    pub names: Vec<u8>,
    pub records: Vec<Record>,
    /// Where the names end: the header and every table lie before it, the files' data after.
    pub names_end: i64,
}

impl Listing {
    pub fn path(&self, r: &Record) -> Vec<u8> {
        record::path(&self.folders[r.folder], &self.names[r.name_at..r.name_at + r.name_length])
    }
}

pub fn read(file: &Container) -> Result<Listing, Failed> {
    let header = header::read(file)?;

    let folder_bytes = header.folders as i64 * header.folder_record();
    let table = if folder_bytes > i32::MAX as i64 { None } else { file.at(header::LENGTH, folder_bytes)? };
    let table = table.ok_or(unpack::TABLES_OUTSIDE)?;

    let mut listing = Listing { header, folders: Vec::new(), names: Vec::new(), records: Vec::new(), names_end: 0 };
    let mut at = listing.header.folders_end();
    for f in 0..listing.header.folders as usize {
        folder(file, &mut listing, &table, f, &mut at)?;
    }

    if listing.records.len() as u64 != listing.header.files as u64 {
        return Err(unpack::MISCOUNTED.into());
    }
    named(file, at, &mut listing)?;
    listing.names_end = at + listing.header.file_names_length as i64;
    Ok(listing)
}

fn folder(file: &Container, listing: &mut Listing, table: &[u8], f: usize, at: &mut i64) -> Result<(), Failed> {
    let header = &listing.header;
    let record = f * header.folder_record() as usize;
    let count = u32_at(table, record + 8);
    let declared = if header.version == 105 { u64_at(table, record + 16) as i64 } else { u32_at(table, record + 12) as i64 };

    if declared.wrapping_sub(header.file_names_length as i64) != *at {
        return Err(unpack::MISPLACED.into());
    }
    if count > (i32::MAX as u32 - 256) / FILE_RECORD as u32 {
        return Err(unpack::TABLES_OUTSIDE.into());
    }

    let length = file.at(*at, 1)?.ok_or(unpack::TABLES_OUTSIDE)?;
    let name_length = length[0] as usize;
    let count = count as usize;
    let block = file.at(*at + 1, (name_length + FILE_RECORD * count) as i64)?.ok_or(unpack::TABLES_OUTSIDE)?;
    if name_length == 0 || block[name_length - 1] != 0 {
        return Err(unpack::UNENDED_NAME.into());
    }

    let compressed = header.compressed();
    listing.folders.push(slashed(&block[..name_length - 1]));
    let folder = listing.folders.len() - 1;
    listing.records.try_reserve(count).map_err(|_| Refused(TOO_LARGE))?;
    for i in 0..count {
        let r = name_length + FILE_RECORD * i;
        let size = u32_at(&block, r + 8);
        let toggled = size & TOGGLE != 0;
        listing.records.push(Record {
            folder,
            name_at: 0,
            name_length: 0,
            offset: u32_at(&block, r + 12),
            size: size & SIZE_BITS,
            compressed: compressed != toggled,
        });
    }

    *at += 1 + name_length as i64 + (FILE_RECORD * count) as i64;
    Ok(())
}

/// The names follow the records in their order, each ended by a NUL; some writers pad the table after the last with NULs.
fn named(file: &Container, at: i64, listing: &mut Listing) -> Result<(), Failed> {
    let length = listing.header.file_names_length;
    let names = if length > i32::MAX as u32 { None } else { file.at(at, length as i64)? };
    let names = names.ok_or(unpack::TABLES_OUTSIDE)?;

    let (mut start, mut next) = (0usize, 0usize);
    let mut i = 0usize;
    while i < names.len() && next < listing.records.len() {
        if names[i] == 0 {
            let one = &mut listing.records[next];
            one.name_at = start;
            one.name_length = i - start;
            next += 1;
            start = i + 1;
        }
        i += 1;
    }

    if next < listing.records.len() || names[start..].iter().any(|&b| b != 0) {
        return Err(unpack::MISCOUNTED.into());
    }
    listing.names = names;
    Ok(())
}
