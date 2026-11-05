//! BsaHeader.cs: a BSA's header, its version, its flags and the sizes of the tables after it.

use super::bytes::u32_at;
use super::unpack;
use super::Failed;
use crate::host::Container;

pub const LENGTH: i64 = 36;

const MAGIC: u32 = 0x00415342;
const FOLDER_NAMES: u32 = 0x1;
const FILE_NAMES: u32 = 0x2;
const COMPRESSED_BY_DEFAULT: u32 = 0x4;
const FOR_XBOX: u32 = 0x40;
const NAMES_EMBEDDED: u32 = 0x100;
const XBOX_CODEC: u32 = 0x200;

/// DreamportDecoders' names for a compressed file of each version.
pub const LZ4: &str = "lz4";
pub const ZLIB: &str = "zlib";

pub struct Header {
    pub version: u32,
    pub flags: u32,
    pub folders: u32,
    pub files: u32,
    pub file_names_length: u32,
}

impl Header {
    pub fn compressed(&self) -> bool {
        self.flags & COMPRESSED_BY_DEFAULT != 0
    }

    /// Version 103 gives the bit no such meaning.
    pub fn embeds_names(&self) -> bool {
        self.version != 103 && self.flags & NAMES_EMBEDDED != 0
    }

    pub fn folder_record(&self) -> i64 {
        if self.version == 105 {
            24
        } else {
            16
        }
    }

    pub fn decoder(&self) -> &'static str {
        if self.version == 105 {
            LZ4
        } else {
            ZLIB
        }
    }

    pub fn folders_end(&self) -> i64 {
        LENGTH + self.folders as i64 * self.folder_record()
    }

    fn problem(&self) -> Option<&'static str> {
        if !(103..=105).contains(&self.version) {
            return Some(unpack::OTHER_VERSION);
        }
        if self.flags & (FOR_XBOX | XBOX_CODEC) != 0 {
            return Some(unpack::XBOX);
        }
        if self.flags & FOLDER_NAMES == 0 || self.flags & FILE_NAMES == 0 {
            return Some(unpack::UNNAMED);
        }
        None
    }
}

pub fn read(file: &Container) -> Result<Header, Failed> {
    let h = match file.at(0, LENGTH)? {
        Some(h) if u32_at(&h, 0) == MAGIC && u32_at(&h, 8) == LENGTH as u32 => h,
        _ => return Err(unpack::NOT_BSA.into()),
    };

    let header = Header {
        version: u32_at(&h, 4),
        flags: u32_at(&h, 12),
        folders: u32_at(&h, 16),
        files: u32_at(&h, 20),
        file_names_length: u32_at(&h, 28),
    };

    match header.problem() {
        Some(problem) => Err(problem.into()),
        None => Ok(header),
    }
}
