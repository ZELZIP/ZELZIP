//! TODO

use niiebla::{
    ContentSelector, CryptographicMethod, TitleMetadata, TitleMetadataContentEntryKind, Wad,
};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Cursor, Read, Seek, Write};
use std::path::PathBuf;

#[allow(clippy::unwrap_used)]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut wad_file = File::options()
        .read(true)
        .write(false)
        .open(&args[1])
        .unwrap();

    let mut new_wad_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args[2])
        .unwrap();

    io::copy(&mut wad_file, &mut new_wad_file).unwrap();
    wad_file.rewind().unwrap();

    let mut wad = Wad::try_new_installable(&wad_file).unwrap();
    let mut ticket = wad.ticket(&wad_file).unwrap();
    let mut tmd = wad.title_metadata(&wad_file).unwrap();
    let mut cert_chain = wad.certificate_chain(&wad_file).unwrap();

    println!("----- PREV ----- ");
    println!("{wad:?}");
    println!("{ticket:?}");
    println!("{tmd:?}");
    //println!("{cert_chain:?}");

    let mut data = Cursor::new([0xFE; 1024]);

    wad.modify_content(&mut new_wad_file)
        .crypotgraphy(&ticket, CryptographicMethod::Wii)
        .set_id(420)
        .set_index(69)
        .set_kind(TitleMetadataContentEntryKind::Dlc)
        .safe(true)
        .trim_if_file(true)
        .add(&mut data, &mut tmd)
        .unwrap();

    new_wad_file.rewind().unwrap();

    let mut wad = Wad::try_new_installable(&new_wad_file).unwrap();
    let mut ticket = wad.ticket(&new_wad_file).unwrap();
    let mut tmd = wad.title_metadata(&new_wad_file).unwrap();
    let mut cert_chain = wad.certificate_chain(&new_wad_file).unwrap();

    println!("----- POST ----- ");
    println!("{wad:?}");
    println!("{ticket:?}");
    println!("{tmd:?}");
    //println!("{cert_chain:?}");
}
