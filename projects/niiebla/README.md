# ZEL.ZEL NiiEBLA

[Usage guide](https://docs.zel.zip/niiebla/niiebla.html) | [Reference](https://docs.rs/zelzip_niiebla) | [ZEL.ZIP website](https://zel.zip)

A parsing library for various Nintendo file formats. With accurate support for multiple niche entries and extensions (TikV1, Wii Savegame data, etc).

## Quickstart
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

For more advanced use cases visit the [usage guide](https://docs.zel.zip/niiebla/niiebla.html)
