//! BsaBytes.cs: the little-endian numbers and one-byte-per-character names a BSA is written in. Names stay bytes here;
//! each byte is one character (U+0000 to U+00FF) when the listing is written.

pub fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

pub fn u64_at(b: &[u8], at: usize) -> u64 {
    u32_at(b, at) as u64 | (u32_at(b, at + 4) as u64) << 32
}

/// The archive's '\' between folders, written as '/'.
pub fn slashed(b: &[u8]) -> Vec<u8> {
    b.iter().map(|&c| if c == b'\\' { b'/' } else { c }).collect()
}

/// .NET's OrdinalIgnoreCase over one-byte characters: each compared by its simple uppercase.
pub fn same_ignoring_case(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(&x, &y)| upper(x) == upper(y))
}

fn upper(c: u8) -> u8 {
    match c {
        b'a'..=b'z' => c - 0x20,
        0xE0..=0xFE if c != 0xF7 => c - 0x20,
        _ => c,
    }
}
