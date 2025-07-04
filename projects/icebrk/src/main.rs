//! TODO REMOVE

fn main() {
    println!("GOO");
    let master_key = zelzip_icebrk::calculate_v3_master_key(1234567890, 5, 8).unwrap();

    println!("{master_key}");
}
