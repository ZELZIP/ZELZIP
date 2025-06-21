//! TODO

use niiebla::{TitleMetadata, Wad};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Cursor, Read, Seek, Write};
use std::path::PathBuf;

#[allow(clippy::unwrap_used)]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut wad_file = File::options()
        .read(true)
        .write(true)
        .open(&args[1])
        .unwrap();

    let mut new_wad_file = File::options()
        .read(true)
        .write(true)
        .open(&args[2])
        .unwrap();

    io::copy(&mut wad_file, &mut new_wad_file).unwrap();
    wad_file.rewind().unwrap();

    let mut wad = niiebla::Wad::try_new_installable(&wad_file).unwrap();
    let mut ticket = wad.ticket(&wad_file).unwrap();
    let mut tmd = wad.title_metadata(&wad_file).unwrap();
    let mut cert_chain = wad.certificate_chain(&wad_file).unwrap();

    let data = Cursor::new([0xFE; 1024]);

    wad.add_content_wii(
        &mut new_wad_file,
        data,
        &ticket,
        &mut tmd,
        None,
        100,
        100,
        niiebla::TitleMetadataContentEntryKind::NormalWiiUKind1,
    )
    .unwrap();
}
