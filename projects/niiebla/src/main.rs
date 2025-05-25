use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;

use niiebla::wad::Wad;

fn main() {
    let wad_path = PathBuf::from("dirty/forecast-channel.wad");
    let mut wad_file = File::open(wad_path).unwrap();

    let wad = Wad::from_reader(&mut wad_file).unwrap();
    //eprintln!("{wad:?}");

    let wad_toml = serde_json::to_string(&wad).unwrap();
    eprintln!("WAD JSON:\n{wad_toml}");

    match wad {
        Wad::Installable(installable) => {
            let mut ticket = installable.ticket(&mut wad_file).unwrap();
            ticket.limit_entries[0] = niiebla::ticket::TicketLimitEntry::TimeLimit { minutes: 32 };
            //eprintln!("{ticket:?}");

            eprintln!("DECRYPTED TITLE KEY {:?}", ticket.decrypt_title_key());

            eprintln!("TICKET :\n{ticket:?}");

            let certificate_chain = installable.certificate_chain(&mut wad_file).unwrap();
            eprintln!("CERT CHAIN: {certificate_chain:?}");

            let title_metadata = installable.title_metadata(&mut wad_file).unwrap();
            eprintln!("TITLE METADATA: {title_metadata:?}");

            let mut decrypted_content = installable.take_decrpyted_content(wad_file, 0).unwrap();

            decrypted_content.seek_relative(2).unwrap();

            let mut buf = [0; 35];
            decrypted_content.read_exact(&mut buf).unwrap();

            eprintln!("BUF {buf:?}");

            /*
            let mut index_0_take = installable
                .take_encrypted_content(&mut wad_file, 0)
                .unwrap();

            let index_0_bin_path = PathBuf::from("dirty/index_0.bin.niiebla");
            let mut index_0_bin = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(index_0_bin_path)
                .unwrap();

            std::io::copy(&mut index_0_take, &mut index_0_bin).unwrap();
            */
        }

        Wad::BackUp => (),
    }
}
