//! TODO

use niiebla::{
    ContentSelector, CryptographicMethod, TitleMetadata, TitleMetadataContentEntryKind, Wad,
};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Cursor, Read, Seek, Write};
use std::path::PathBuf;

#[allow(clippy::unwrap_used)]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut wad_file = File::options()
        .read(true)
        .write(false)
        .open(&args[1])
        .unwrap();

    let mut new_wad_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args[2])
        .unwrap();

    io::copy(&mut wad_file, &mut new_wad_file).unwrap();
    wad_file.rewind().unwrap();

    let mut wad = Wad::try_new_installable(&wad_file).unwrap();
    let mut ticket = wad.ticket(&wad_file).unwrap();
    let mut tmd = wad.title_metadata(&wad_file).unwrap();
    let mut cert_chain = wad.certificate_chain(&wad_file).unwrap();

    println!("WAD OLD: {wad:?}");
    ticket.content_access_permissions = [0xFA; 64];
    wad.write_ticket_safe_file(&mut new_wad_file, &ticket, &tmd)
        .unwrap();

    cert_chain.certificates[0].identity = String::from("HELLO FROM SPAIN");
    wad.write_certificate_chain_safe_file(&mut new_wad_file, &cert_chain, &ticket, &tmd)
        .unwrap();

    let mut data = Cursor::new([0xFE; 1024]);
    let mut data2 = Cursor::new([0xAB; 1024]);

    new_wad_file.rewind().unwrap();

    new_wad_file.rewind().unwrap();

    wad.modify_content(&mut new_wad_file)
        .set_cryptography(&ticket, CryptographicMethod::Wii)
        .trim_if_file(true)
        .set_id(666)
        .set_index(444)
        .set_kind(TitleMetadataContentEntryKind::Dlc)
        .replace(&mut data, tmd.select_with_physical_position(1), &mut tmd)
        .unwrap();

    wad.modify_content(&mut new_wad_file)
        .set_cryptography(&ticket, CryptographicMethod::Wii)
        .trim_if_file(true)
        .remove(tmd.select_with_physical_position(2), &mut tmd)
        .unwrap();

    wad.modify_content(&mut new_wad_file)
        .set_cryptography(&ticket, CryptographicMethod::Wii)
        .trim_if_file(true)
        .set_id(222)
        .set_index(333)
        .set_kind(TitleMetadataContentEntryKind::Dlc)
        .add(&mut data2, &mut tmd)
        .unwrap();

    /*
    let mut content_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args[3])
        .unwrap();

    let mut view = wad
        .decrypted_content_view(
            &new_wad_file,
            &ticket,
            &tmd,
            CryptographicMethod::Wii,
            tmd.select_with_id(222),
        )
        .unwrap();

    io::copy(&mut view, &mut content_file).unwrap()
    */

    new_wad_file.rewind().unwrap();
    let mut wad = Wad::try_new_installable(&new_wad_file).unwrap();
    let mut ticket = wad.ticket(&new_wad_file).unwrap();
    let mut tmd = wad.title_metadata(&new_wad_file).unwrap();
    let mut cert_chain = wad.certificate_chain(&new_wad_file).unwrap();

    println!("WAD NEW: {wad:?}");
    println!("{ticket:?}");
    println!("{tmd:?}");
    //println!("{cert_chain:?}");
}
