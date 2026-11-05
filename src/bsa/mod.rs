//! The reader `bsa`: every file of one Bethesda archive (versions 103, 104, 105) and where its bytes lie, nothing
//! decoded. A translation of Somnium's C# reader (`Unpackers/Bsa/*.cs`), one Rust file per C# file; selection stays in
//! the host, and the host (or a decoder module) decodes what is compressed.

pub mod bytes;
pub mod entry_head;
pub mod header;
pub mod listing;
pub mod locate;
pub mod placed;
pub mod record;
pub mod unpack;

use crate::host::Refused;

/// A C# problem string, or a read the host refused, which ends the whole listing.
pub enum Failed {
    Problem(&'static str),
    Host(Refused),
}

impl From<&'static str> for Failed {
    fn from(problem: &'static str) -> Failed {
        Failed::Problem(problem)
    }
}

impl From<Refused> for Failed {
    fn from(refused: Refused) -> Failed {
        Failed::Host(refused)
    }
}
