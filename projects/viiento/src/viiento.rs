//! TODO

use niiebla::{TitleMetadata, Wad};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

#[allow(clippy::unwrap_used)]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut tik_file = File::options()
        .read(true)
        .write(true)
        .open(&args[1])
        .unwrap();

    let tik = niiebla::PreSwitchTicket::new(&tik_file).unwrap();

    println!("{:?}", tik);
    println!("SIZE: {:?}", tik.size());

    let mut new_tik_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args[2])
        .unwrap();

    tik.dump(&mut new_tik_file).unwrap();
}
