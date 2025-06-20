//! TODO

use niiebla::{TitleMetadata, Wad};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Cursor, Read, Seek, Write};
use std::path::PathBuf;

#[allow(clippy::unwrap_used)]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut wad_file = File::options()
        .read(true)
        .write(true)
        .open(&args[1])
        .unwrap();

    let mut wad = niiebla::Wad::try_new_installable(&wad_file).unwrap();
    let mut ticket = wad.ticket(&wad_file).unwrap();
    let mut tmd = wad.title_metadata(&wad_file).unwrap();
    let mut cert_chain = wad.certificate_chain(&wad_file).unwrap();

    println!("TIK: {ticket:?}");
    println!("CERTS: {cert_chain:?}");
    println!("TMD: {tmd:?}");

    let mut new_wad_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args[2])
        .unwrap();

    ticket.version_1_extension = Some(niiebla::ticket::v1::PreSwitchTicketV1ExtraData {
        sections: vec![niiebla::ticket::v1::PreSwitchTicketV1ExtraDataSection {
            flags: 0,
            records: niiebla::ticket::v1::PreSwitchTicketV1ExtraDataRecords::Subscription(vec![
                niiebla::ticket::v1::PreSwitchTicketV1ExtraDataRecordSubscription {
                    expiration_time: 0,
                    reference_id: niiebla::ticket::v1::PreSwitchTicketV1ExtraDataRefereceId {
                        id: [0xFC; 16],
                        attributes: 0,
                    },
                },
                niiebla::ticket::v1::PreSwitchTicketV1ExtraDataRecordSubscription {
                    expiration_time: 0,
                    reference_id: niiebla::ticket::v1::PreSwitchTicketV1ExtraDataRefereceId {
                        id: [0xFC; 16],
                        attributes: 0,
                    },
                },
                niiebla::ticket::v1::PreSwitchTicketV1ExtraDataRecordSubscription {
                    expiration_time: 0,
                    reference_id: niiebla::ticket::v1::PreSwitchTicketV1ExtraDataRefereceId {
                        id: [0xFC; 16],
                        attributes: 0,
                    },
                },
            ]),
        }],
        flags: 20,
    });

    for i in 0..2 {
        cert_chain
            .certificates
            .push(cert_chain.certificates[0].clone());
    }

    cert_chain.certificates[0].identity = String::from("HOLA MUNDO!");

    tmd.content_chunk_entries
        .push(niiebla::title_metadata::TitleMetadataContentEntry {
            id: 69,
            index: 420,
            kind: niiebla::title_metadata::TitleMetadataContentEntryKind::Normal,
            size: 1,
            hash: niiebla::title_metadata::TitleMetadataContentEntryHashKind::Version0([0; 20]),
        });

    wad.write_ticket_safe_file(&mut new_wad_file, &ticket, &tmd)
        .unwrap();

    wad.write_title_metadata_safe_file(&mut new_wad_file, &tmd);

    wad.write_certificate_chain_safe_file(&mut new_wad_file, &cert_chain, &ticket, &tmd);
}
