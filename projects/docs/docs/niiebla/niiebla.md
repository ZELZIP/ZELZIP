# The NiiEBLA library

Rust parsing library for various Nintendo file formats. With accurate support for multiple niche entries and extensions (TikV1, Wii Savegame data, etc). This documentation presents a brief explanation and "How-to" guides of the library, an [online API reference is also available](https://docs.rs/zelzip_niiebla) for a more complete understanding.

NiiEBLA works around the concept of data streams, this opens the door to use more than just in-memory arrays: network sockets, virtual filesystems, just-in-place data extraction of container files (ZIP, TAR, ISO, etc), mmap files, etc; anything that implementes the traits [Read](https://doc.rust-lang.org/stable/std/io/trait.Read.html), [Write](https://doc.rust-lang.org/stable/std/io/trait.Write.html) and/or [Seek](https://doc.rust-lang.org/stable/std/io/trait.Seek.html) will work.

For the sake of maintaining the code snippets easy to follow files as streams will be used on all the documentation.

## Getting Started

First of all add the library into your Rust project:

```sh
$ cargo add zelzip_niiebla
```

### Parsing

From this you can parse any format you want using the `new(...)` method on the proper struct:

```rust
use zelzip_niiebla::PreSwitchTicket;
use std::fs::File;

let mut ticket_file = File::open("/just/any/path").unwrap();

let ticket = PreSwitchTicket::new(&mut ticket_file).unwrap();

println!(ticket.title_id);
// 00000001-00000002
```

### Dumping

After making any change, let's say on the metadata of a title, you can compose the data again with the `dump(...)` method.

```rust
use zelzip_niiebla::TitleMetadata;
use std::fs::File;

let mut tmd_file = File::open("/just/any/path").unwrap();

let mut tmd = TitleMetadata::new(&mut tmd_file).unwrap();
tmd.boot_content_index = 1;

let mut new_tmd_file = File::open("/just/any/other/path").unwrap();
tmd.dump(&mut new_tmd_file).unwrap();
```

### Regard WAD/TAD files

Be aware that **WAD/TAD files have a different API**, and explanation on this archive format can be found [on this documentation](./wad).

### Title IDs

You may be also interested on information about the support of [Title IDs](./title_ids).
