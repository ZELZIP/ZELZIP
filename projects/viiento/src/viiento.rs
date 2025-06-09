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
    let mut wad = Wad::try_new_installable(&mut wad_file).unwrap();

    println!("OG SIZES:");
    println!("  TICKET: {}", wad.ticket_size);
    println!("  TMD: {}", wad.title_metadata_size);
    println!("  CERT: {}", wad.certificate_chain_size);

    let ticket = wad.ticket(&mut wad_file).unwrap();
    let tmd = wad.title_metadata(&mut wad_file).unwrap();
    let cert = wad.certificate_chain(&mut wad_file).unwrap();

    println!("WAD: {wad:?}");
    //println!("TICKET: {ticket:?}");
    //println!("TMD: {tmd:?}");
    //println!("CERT: {cert:?}");

    let synthetic_path = PathBuf::from("wad.viiento.synthetic.bin");
    let mut synthetic = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(synthetic_path)
        .unwrap();

    wad.dump(&mut synthetic).unwrap();
    wad.write_ticket(&ticket, &mut synthetic).unwrap();
    wad.write_title_metadata(&tmd, &mut synthetic).unwrap();
    wad.write_certificate_chain(&cert, &mut synthetic).unwrap();

    println!("RECALCULATED SIZES:");
    println!("  TICKET: {}", wad.ticket_size);
    println!("  TMD: {}", wad.title_metadata_size);
    println!("  CERT: {}", wad.certificate_chain_size);
}
