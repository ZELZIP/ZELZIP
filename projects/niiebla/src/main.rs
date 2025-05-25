use std::fs::File;
use std::fs::OpenOptions;
use std::path::PathBuf;

use niiebla::wad::Wad;
use niiebla::Dump;

fn main() {
    let wad_path = PathBuf::from("dirty/forecast-channel.wad");
    let mut wad_file = File::open(wad_path).unwrap();
    let wad = Wad::try_installable(&mut wad_file).unwrap();

    let ticket_path = PathBuf::from("dirty/niiebla.ticket.bin");
    let mut ticket_bin = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(ticket_path)
        .unwrap();

    let ticket = wad.ticket(&mut wad_file).unwrap();

    ticket.dump(&mut ticket_bin).unwrap();
}
