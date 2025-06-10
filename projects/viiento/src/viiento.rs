//! TODO

use niiebla::Wad;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

#[allow(clippy::unwrap_used)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let first_arg = &args[1];

    let mut wad_file = File::options()
        .read(true)
        .write(true)
        .open(first_arg)
        .unwrap();

    let wad = Wad::try_new_installable(&mut wad_file).unwrap();
    let ticket = wad.ticket(&mut wad_file).unwrap();
    let mut content = wad.decrypted_content_view(&mut wad_file, 0).unwrap();

    let mut decrypted_content = Vec::new();
    content.read_to_end(&mut decrypted_content).unwrap();

    let mut decrypted_content = Cursor::new(decrypted_content);

    wad.write_content(&mut decrypted_content, &mut wad_file, 0, None, None)
        .unwrap();
}
