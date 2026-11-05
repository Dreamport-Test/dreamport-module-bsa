//! BsaUnpack.cs: the problems a listing shares with the reader, word for word.

pub const NOT_BSA: &str = "it is not a BSA archive: it does not start with a BSA header";
pub const OTHER_VERSION: &str = "it is a BSA version this reader does not open; it opens 103, 104 and 105";
pub const XBOX: &str = "it is a BSA written for the Xbox 360, which this reader does not open";
pub const UNNAMED: &str = "it does not keep its folder and file names, so its files cannot be named";
pub const TABLES_OUTSIDE: &str = "its tables do not fit inside it";
pub const MISPLACED: &str = "a folder's files are not where its record says";
pub const MISCOUNTED: &str = "its folders, files and names do not add up to what its header declares";
pub const UNENDED_NAME: &str = "a folder's name does not end where its length says";

pub const OUTSIDE: &str = "a file's bytes do not lie inside the archive";
pub const OTHER_NAME: &str = "a file's own name is not the name the archive's tables give it";
pub const TOO_LARGE_COMPRESSED: &str = "a compressed file declares more than 64 MiB, more than this reader decodes at once";
pub const EXPANDS: &str = "a compressed file declares more bytes than its compressed bytes could hold";

/// What `dp_list` returns for an archive it cannot list: -2 for what this reader does not open, -1 for the rest.
pub fn refusal(problem: &str) -> i64 {
    if [OTHER_VERSION, XBOX, UNNAMED].contains(&problem) {
        -2
    } else {
        -1
    }
}
