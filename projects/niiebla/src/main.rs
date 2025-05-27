use niiebla::wad::Wad;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Cursor;
use std::io::{Read, Seek};
use std::path::PathBuf;

fn main() {
    let wad_path = PathBuf::from("dirty/forecast-channel.wad");
    let mut wad_content = Cursor::new(std::fs::read(wad_path).unwrap());
    let mut wad = Wad::try_new_installable(&mut wad_content).unwrap();

    let mut ticket = wad.ticket(&mut wad_content).unwrap();
    ticket.title_id = niiebla::title_id::TitleId::new(0x0001000248414741);

    wad.write_ticket(&ticket, &mut wad_content).unwrap();

    let modified_wad_path = PathBuf::from("dirty/forecast-channel.modified.wad");
    let mut modified_wad_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(modified_wad_path)
        .unwrap();

    wad_content.rewind().unwrap();
    std::io::copy(&mut wad_content, &mut modified_wad_file).unwrap();

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
}
