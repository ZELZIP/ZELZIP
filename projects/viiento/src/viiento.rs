//! TODO

use niiebla::Wad;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

#[allow(clippy::unwrap_used)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let first_arg = &args[1];

    let mut wad_file = File::options()
        .read(true)
        .write(true)
        .open(first_arg)
        .unwrap();

    let mut wad = Wad::try_new_installable(&mut wad_file).unwrap();
    let ticket = wad.ticket(&mut wad_file).unwrap();

    let mut buf = Cursor::new(b"HELLO WORLD!");

    //let mut new_content = Cursor::new(b"HELLO WORLD!aaaa");

    wad.write_content_safe_file(&mut buf, &mut wad_file, 0, None, None)
        .unwrap();
}
