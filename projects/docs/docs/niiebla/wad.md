# WAD/TAD files

WAD files are one of the many ways titles can be distributed, they consist of an ownership ticket, the metadata of the title, a certificate chain and a set of binary blobs called "contents".

Be aware that the usual `dump(...)` method **will only dump the header of the WAD** not all the content inside of it, to dump the data inside it you should use one of the following functions: `write_*_raw(...)`, `write_*_safe(...)` or `write_*_safe_file(...)` where `*` can be either `ticket`, `title_metadata` or `certificate_chain`, the suffix on the methods meain the following:

- `_raw`: The data will be writen without any safety check, usually data after it will be corrupted.
- `_safe`: The data after the new one will be stored on the heap and put after it safely.
  ` _safe_file`: In addition to the actions made by `_safe`, given that the parameters of the function only allows [`File`](https://doc.rust-lang.org/std/fs/struct.File.html)s the file will be trimmed to avoid useless or meaningless data at the end.

## Content manipulation

### Reading

Contents inside a WAD are encrypted with a method unique of the platform where the title is expected to run, both the encrypted and a in-place decrypted stream (a "subview" of the stream of the WAD) of the data can be get:

```rust
use zelzip_niiebla::{Wad, CryptographicMethod};

let wad = Wad::new(&mut wad_file).unwrap();
let tmd = wad.title_metadata(&mut wad_file).unwrap();
let tik = wad.ticket(&mut wad_file).unwrap();

// This is a standard Rust `Read` stream
let encrypted_view = wad.encrypted_view(
    &mut wad_file,
    tmd,
    tmd.select_with_id(0)
).unwrap();

// This is a standard Rust `Read` stream
let decrypted_view = wad.decrypted_view(
    &mut wad_file,
    tik,
    tmd,
    CryptographicMethod::Wii,
    tmd.select_with_id(0)
).unwrap();
```

### Writting

To avoid store contents in-memory (as they can have an arbitrary size) the following builder can be used:

```rust
// Modify the data stored inside a content
wad.modify_content(&mut wad_stream)
    .set_cryptography(&ticket, CryptographicMethod::Wii)
    .trim_if_file(true) // Will trim the file if `wad_stream` is a `File`.
    .set_id(666) // Optional
    .set_index(444) // Optional
    .set_kind(TitleMetadataContentEntryKind::Dlc)
    .replace(&mut data, tmd.select_with_physical_position(1), &mut tmd)
    .unwrap();

// Remove a content
wad.modify_content(&mut wad_stream)
    .set_cryptography(&ticket, CryptographicMethod::Wii)
    .trim_if_file(true) // Will trim the file if `wad_stream` is a `File`.
    .remove(tmd.select_with_physical_position(2), &mut tmd)
    .unwrap();

// Add new content at the end
wad.modify_content(&mut wad_stream)
    .set_cryptography(&ticket, CryptographicMethod::Wii)
    .trim_if_file(true)
    .set_id(222)
    .set_index(333)
    .set_kind(TitleMetadataContentEntryKind::Dlc)
    .add(&mut data2, &mut tmd)
    .unwrap();
```
