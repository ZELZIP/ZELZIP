//! TODO

use niiebla::Wad;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::PathBuf;

#[allow(clippy::unwrap_used)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let first_arg = &args[1];

    let mut wad_file = File::open(first_arg).unwrap();
    let wad = Wad::try_new_installable(&mut wad_file).unwrap();
    let ticket = wad.ticket(&mut wad_file).unwrap();
    let tmd = wad.title_metadata(&mut wad_file).unwrap();
    let cert = wad.certificate_chain(&mut wad_file).unwrap();

    println!("WAD: {wad:?}");
    println!("TICKET: {ticket:?}");
    println!("TMD: {tmd:?}");
    println!("CERT: {cert:?}");

    let content_path = PathBuf::from("content.i0.viiento.decrypted.bin");
    let mut content_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(content_path)
        .unwrap();

    println!("CREATING DECRYPTION VIEW");
    let mut content_view = wad.decrypted_content_view(&mut wad_file, 0).unwrap();

    println!("COPYING AND DECRYPTING");
    io::copy(&mut content_view, &mut content_file).unwrap();
}
