use niiebla::title_id::TitleId;
use niiebla::wad::Wad;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::PathBuf;

fn main() {
    let wad_path = PathBuf::from("dirty/forecast-channel.wad");
    let mut wad_file = File::open(wad_path).unwrap();
    let wad = Wad::try_new_installable(&mut wad_file).unwrap();

    let tmd_path = PathBuf::from("dirty/tmd.niiebla.bin");
    let mut tmd_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(tmd_path)
        .unwrap();

    let title_metadata = wad.title_metadata(&mut wad_file).unwrap();
    title_metadata.dump(&mut tmd_file).unwrap();
}

/*
let ticket_path = PathBuf::from("dirty/niiebla.ticket.bin");
let mut ticket_bin = OpenOptions::new()
    .create(true)
    .truncate(true)
    .write(true)
    .open(ticket_path)
    .unwrap();

let ticket = wad.ticket(&mut wad_file).unwrap();
ticket.dump(&mut ticket_bin).unwrap();
*/
