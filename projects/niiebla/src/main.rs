use std::fs::File;
use std::path::PathBuf;

use niiebla::wad::Wad;

fn main() {
    let wad_path = PathBuf::from("dirty/forecast-channel.wad");
    let mut wad_file = File::open(wad_path).unwrap();

    let wad = Wad::from_reader(&mut wad_file).unwrap();
    eprintln!("{wad:?}");

    match wad {
        Wad::Installable { installable } => {
            let ticket = installable.ticket(&mut wad_file).unwrap();
            eprintln!("{ticket:?}");

            eprintln!("DECRYPTED TITLE KEY {:?}", ticket.decrypt_title_key());
        }
    }
}
