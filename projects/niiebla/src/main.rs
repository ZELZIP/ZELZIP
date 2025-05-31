use niiebla::wad::Wad;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::PathBuf;

fn main() {
    let wad_path = PathBuf::from("dirty/forecast-channel.wad");
    let mut wad_file = File::open(wad_path).unwrap();
    let wad = Wad::try_new_installable(&mut wad_file).unwrap();

    let cert_chain_path = PathBuf::from("dirty/cert-chain.niiebla.bin");
    let mut cert_chain_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(cert_chain_path)
        .unwrap();

    let cert_chain = wad.certificate_chain(&mut wad_file).unwrap();
    cert_chain.dump(&mut cert_chain_file).unwrap();

    /*
    std::io::copy(
        &mut wad.take_certificate_chain(&mut wad_file).unwrap(),
        &mut cert_chain_file,
    )
    .unwrap();
    */
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
