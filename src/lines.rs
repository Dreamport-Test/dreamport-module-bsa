//! The ABI v2 listing: UTF-8 JSON Lines, the container first, then one line per entry.

use crate::bsa::locate::{self, Range};
use crate::bsa::{listing, unpack, Failed};
use crate::host::{Container, Refused, TOO_LARGE};
use std::fmt::Write;

pub fn list_bsa(file: &Container) -> Result<Vec<u8>, Refused> {
    let bsa = match listing::read(file) {
        Ok(bsa) => bsa,
        Err(Failed::Problem(problem)) => return Err(Refused(unpack::refusal(problem))),
        Err(Failed::Host(refused)) => return Err(refused),
    };

    let mut lines = Lines::default();
    let (index_at, index_length) = locate::proven(&bsa);
    let mut line = String::new();
    let _ = write!(line, "{{\"archive\":{{\"indexAt\":{},\"indexLength\":{},\"mount\":\"\"}}}}", index_at, index_length);
    lines.add(&line)?;

    for record in &bsa.records {
        let path = bsa.path(record);
        line.clear();
        line.push_str("{\"path\":");
        quote(&mut line, &path);
        match locate::one(file, &bsa, record, &path)? {
            Ok(range) => entry(&mut line, &range),
            Err(problem) => {
                line.push_str(",\"problem\":");
                quote(&mut line, problem.as_bytes());
            }
        }
        line.push('}');
        lines.add(&line)?;
    }

    Ok(lines.out)
}

fn entry(line: &mut String, r: &Range) {
    let _ = write!(
        line,
        ",\"offset\":{},\"length\":{},\"method\":{},\"size\":{},\"decoder\":\"{}\",\"blockSize\":{},\"blocks\":[",
        r.offset, r.length, r.method, r.size, r.decoder, r.block_size
    );
    for (i, b) in r.blocks.iter().enumerate() {
        if i > 0 {
            line.push(',');
        }
        let _ = write!(line, "{}", b);
    }
    line.push(']');
}

/// One character per byte, as BsaBytes.Latin reads a name.
fn quote(line: &mut String, latin: &[u8]) {
    line.push('"');
    for &b in latin {
        match b {
            b'"' => line.push_str("\\\""),
            b'\\' => line.push_str("\\\\"),
            b if b < 0x20 => {
                let _ = write!(line, "\\u{:04x}", b);
            }
            b => line.push(b as char),
        }
    }
    line.push('"');
}

#[derive(Default)]
struct Lines {
    out: Vec<u8>,
}

impl Lines {
    fn add(&mut self, line: &str) -> Result<(), Refused> {
        self.out.try_reserve(line.len() + 1).map_err(|_| Refused(TOO_LARGE))?;
        self.out.extend_from_slice(line.as_bytes());
        self.out.push(b'\n');
        Ok(())
    }
}
