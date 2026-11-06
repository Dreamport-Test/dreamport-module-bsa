//! Somnium's Bethesda .bsa reader as a WebAssembly module (ABI v4): the reader `bsa`, versions 103, 104 and 105.
//! A translation of the client's `Unpackers/Bsa/*.cs` listing and in-place locating. It decodes nothing: the host,
//! or a decoder module, decodes what it lists. Its only import is `dp_host.read`, the container being listed.

mod bsa;
mod host;
mod lines;

use std::alloc::{alloc, dealloc, Layout};
use std::cell::UnsafeCell;

const ABI_VERSION: u32 = 4;
const DESCRIPTION: &[u8] = br#"{"readers":["bsa"],"decoders":[]}"#;
const ALIGN: usize = 16;

struct State(UnsafeCell<Vec<u8>>);

// The module is single-threaded: one instance, one call at a time.
unsafe impl Sync for State {}

static LISTING: State = State(UnsafeCell::new(Vec::new()));

#[no_mangle]
pub extern "C" fn dp_abi() -> u32 {
    ABI_VERSION
}

#[no_mangle]
pub extern "C" fn dp_alloc(len: u32) -> u32 {
    if len == 0 {
        return 0;
    }
    match Layout::from_size_align(len as usize, ALIGN) {
        Ok(layout) => unsafe { alloc(layout) as usize as u32 },
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn dp_free(ptr: u32, len: u32) {
    if ptr == 0 || len == 0 {
        return;
    }
    if let Ok(layout) = Layout::from_size_align(len as usize, ALIGN) {
        unsafe { dealloc(ptr as usize as *mut u8, layout) }
    }
}

#[no_mangle]
pub extern "C" fn dp_list(container_len: i64) -> i64 {
    let listing = unsafe { &mut *LISTING.0.get() };
    *listing = Vec::new();
    if container_len < 0 {
        return host::BAD_ARGUMENTS;
    }
    match lines::list_bsa(&host::Container::new(container_len)) {
        Ok(made) => {
            *listing = made;
            listing.len() as i64
        }
        Err(refused) => refused.0,
    }
}

#[no_mangle]
pub extern "C" fn dp_output() -> u32 {
    unsafe { (*LISTING.0.get()).as_ptr() as usize as u32 }
}

/// What this module answers, in its own words, at dp_output until the next dp_list or dp_describe.
#[no_mangle]
pub extern "C" fn dp_describe() -> i64 {
    let listing = unsafe { &mut *LISTING.0.get() };
    *listing = DESCRIPTION.to_vec();
    listing.len() as i64
}
