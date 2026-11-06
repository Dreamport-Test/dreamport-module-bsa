# Dreamport module: Bethesda .bsa

A module, made from [Dreamport-Test/dreamport-module](https://github.com/Dreamport-Test/dreamport-module), that
offers the reader `bsa` (every file of a Bethesda `.bsa` archive, versions 103, 104 and 105, and where its
bytes lie). It declares no decoder: the files it lists as compressed are decoded by the host's own `zlib` and
`lz4`, or by a decoder module the same pattern names. It is a Rust translation of the listing and in-place
locating of Somnium's C# reader (`Unpackers/Bsa`), checked file by file against it. It is not Bethesda's code.

What follows is the base's description.

---

A **Dreamport module** is code a Dreamport world can ask Somnium to use when it reads a game's files. It is
a WebAssembly module that offers **decoders** (bytes in, bytes out), **readers** (it lists the entries of
one container, such as an Unreal `.pak`), or both, in one `.wasm` with one SHA-256 and one consent. The
client runs it **isolated**: it cannot open files, use the network or see anything of the player's machine.
A reader can read the one container it is listing, through the host, and nothing else.

This repository is the base. **To make a module, fork it**, put your code in `src/`, list the names it
offers in `module.json`, and push a tag of your own name (`v1-lz4`, ...). The release workflow builds `module.wasm` and
publishes it with its SHA-256.

A world's pattern names a module by **its fork, its release tag and the SHA-256 of `module.wasm`**, once per
source, with every name it uses joined by `+`:

```
module=unrealpak+oodle:Owner/repo@tag#sha256
```

The client asks the player before downloading it, keeps one copy per SHA-256, and refuses a file whose hash
differs. A module you change gets a new tag and a new hash; patterns that name the old one keep working.

> Test repository (2026-11): a prototype of the design, not a Somnium service.

## The ABI (version 3)

`module.json`:

```json
{ "abi": 3, "decoders": ["oodle"], "readers": ["unrealpak"] }
```

`decoders` and `readers` are the names a pattern asks for; each must be one Somnium knows. The client checks
that every listed name has its export: `dp_decode` for decoders, `dp_list` for readers. The host runs only
version 3: `dp_abi` must answer 3.

### Exports

| Export | Signature | Meaning |
|---|---|---|
| `memory` | memory | the module's memory |
| `dp_abi` | `() -> i32` | 3 |
| `dp_alloc` | `(len: u32) -> u32` | a block of `len` bytes in the module's memory, 16-byte aligned; 0 when it cannot |
| `dp_free` | `(ptr: u32, len: u32)` | gives back a block from `dp_alloc`, with the same `len` |
| `dp_decode` | `(name_ptr: u32, name_len: u32, in_ptr: u32, in_len: u32, out_ptr: u32, out_len: u32) -> i64` | decodes `in_len` bytes into exactly `out_len` bytes with the decoder named (UTF-8, one of `decoders`); returns `out_len`, or a negative refusal |
| `dp_list` | `(container_len: i64) -> i64` | lists the one container the host has open; returns the listing's length in bytes, or a negative refusal |
| `dp_output` | `() -> i32` | where the listing of the last `dp_list` lies; valid until the next `dp_list` |

`dp_decode`'s input and output blocks must not overlap, and a refused call must leave zeros in the output.
Refusals are negative: -1 corrupt, -2 not supported (a decoder name it does not offer, encrypted, unknown
version, unknown codec), -3 too large, -5 bad arguments. The client gives each call a time limit and caps the module's memory; a module that
traps or runs out of time is treated as a refusal.

### The one import, for readers only

```
(import "dp_host" "read" (func (param i64 i32 i32) (result i32)))   ;; read(offset, dst_ptr, len) -> bytes read or negative
```

- It reads from **the container being listed, and nothing else**: there is no handle, no path and no file
  name, so a module cannot even name another file.
- The host refuses (-5) any read outside `[0, container_len)`, any `len` over 16 MiB, and more than 1 GiB in
  total per `dp_list`. Outside a `dp_list` call it always returns -5.
- A module importing anything else, or importing `dp_host.read` without declaring a reader, is refused before
  it runs. A module used only as a decoder gets an import that always returns -5.

`src/host.rs` wraps it safely: `Container::read` fills a buffer in reads of at most 16 MiB, and
`Container::at` returns `None` for bytes outside the container without asking the host. A module that never
calls it imports nothing.

### The listing (UTF-8 JSON Lines)

The first line describes the container; `indexAt` and `indexLength` are the region the host hashes to tell
one game build from another:

```json
{"archive":{"indexAt":123,"indexLength":456,"mount":"../../../"}}
```

Then one line per entry, **every entry, in the container's own order** (selection stays in the host). The
path is the container's own, before any mount strip; `decoder` is `stored`, `zlib`, `oodle`, ...:

```json
{"path":"Engine/Content/x.uasset","offset":1,"length":2,"method":1,"size":3,"decoder":"oodle","blockSize":65536,"blocks":[100,200]}
```

An entry the reader can name but not locate is `{"path":"...","problem":"..."}`; the host refuses the
container only if the pattern selects that entry.

## Making a module, step by step

1. **Fork this repository** on GitHub. An account (or organisation) can hold one fork of it, so a second module
   of yours needs another owner.
2. **Write your code** in `src/`: `decode` for a decoder, `dp_list` with `host::Container` for a reader. Keep
   `rust-toolchain.toml` as it is, so everyone builds with the same compiler.
3. **List what it offers** in `module.json`. Each name must be one Somnium knows:
   - readers: `zip`, `file`, `unrealpak`, `iostore`, `bsa`, `scs`, `unityfs`, `dat2`, `mpq`, `n64`;
   - decoders: `zlib`, `deflate`, `lz4`, `lz4block`, `oodle`.

   A source may take a module only where its reader allows it: `unrealpak`, `iostore`, `file`, `bsa` and `unityfs` today.
4. **Push a tag with your own name**, such as `v1-lz4`, never a bare `v1`. A fork inherits the base's tags, so `v1`
   already exists in your fork and pushing it again builds nothing.
5. **Take the SHA-256 from the release**, from `module.wasm.sha256` beside `module.wasm`, not from your own build.
   GitHub builds on Linux; a build on Windows differs by a few bytes of file paths inside panic messages, so its
   hash is not the one players download. Test the released file, not your local one.
6. **Name it in a pattern** with that tag and that hash: `module=<names>:<you>/<repo>@<tag>#<sha256>`. One module
   answers each name in a pattern; two forks can serve one source when they answer different names (for example
   a `bsa` reader from one fork and `lz4` from another).

**You are responsible for your module.** Somnium does not write, review or host modules: it only runs, isolated,
the exact file a pattern names by its hash.

## Build

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

The result is `target/wasm32-unknown-unknown/release/dreamport_module.wasm`. Use no crate that needs an
import (no `wasm-bindgen`, no WASI).


## This fork's reader

`src/lib.rs` exports the ABI and `src/lines.rs` writes the listing; `src/bsa/` holds one Rust file per file of
the C# reader it translates (the header, the folder and file records with the names, a file's embedded name and
declared size, and locating its bytes). The archive is read only through `src/host.rs`, the one import.

The listing's archive line proves the archive from its start to the end of its file names (`indexAt` 0,
`indexLength` the names' end) and has no mount (`""`). Each file is listed as the archive names it, `folder/name`
with `/` for the archive's `\`, one character per byte. A stored file is one range (`method` 0, `decoder`
`stored`); a compressed one is one block (`method` 1, `decoder` `zlib` for versions 103 and 104, `lz4` for 105,
`blockSize` its declared size, `blocks` its compressed length). A file whose embedded name is not its own, whose
bytes lie outside the archive, or whose declared size cannot be one block is a problem line. An archive that is
not a BSA, or whose tables do not add up, is refused (-1); another version, an Xbox 360 archive or one that keeps
no names is refused as not supported (-2). Which files a world takes, and the limits, stay in the host.

Measured 2026-11-05 in Wasmtime 48 from .NET 10: the listing equals the C# reader's, field by field, on every
`.bsa` of a shipped Bethesda game (the 23 archives of Skyrim Special Edition, 172,961 files) and on the C# suite's
own archives of versions 103, 104 and 105, each refusal it checks, and damaged copies of them.
