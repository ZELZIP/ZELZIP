//! TODO

use niiebla::TitleMetadata;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

#[allow(clippy::unwrap_used)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let first_arg = &args[1];

    let tmd_file = File::options()
        .read(true)
        .write(true)
        .open(first_arg)
        .unwrap();

    let tmd = TitleMetadata::new(tmd_file).unwrap();

    println!("{tmd:?}");
}
