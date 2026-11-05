//! BsaRecord.cs: one file as a BSA's tables name it, its folder and name kept as indexes into the listing's tables.

pub struct Record {
    pub folder: usize,
    pub name_at: usize,
    pub name_length: usize,
    pub offset: u32,
    pub size: u32,
    pub compressed: bool,
}

/// Its path with '/' between folders: the folder's name, then the file's.
pub fn path(folder: &[u8], name: &[u8]) -> Vec<u8> {
    if folder.is_empty() {
        return name.to_vec();
    }
    let mut path = Vec::with_capacity(folder.len() + 1 + name.len());
    path.extend_from_slice(folder);
    path.push(b'/');
    path.extend_from_slice(name);
    path
}
