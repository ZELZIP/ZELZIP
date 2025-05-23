use std::fs::File;
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

            let ticket_toml = serde_json::to_string(&ticket).unwrap();
            eprintln!("TICKET JSON:\n{ticket_toml}");

            let certificate_chain = installable.certificate_chain(&mut wad_file).unwrap();
            eprintln!("CERT CHAIN: {certificate_chain:?}");
        }

        Wad::BackUp => (),
    }
}
