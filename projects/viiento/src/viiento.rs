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

    let ticket_path = PathBuf::from("ticket.viiento.bin");
    let mut ticket_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(ticket_path)
        .unwrap();

    let mut ticket_view = wad.ticket_view(&mut wad_file).unwrap();
    io::copy(&mut ticket_view, &mut ticket_file).unwrap();
}
