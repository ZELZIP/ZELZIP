//! TODO REMOVE

fn main() {
    println!("GOO");
    let master_key = zelzip_icebrk::calculate_v1_master_key(1000000000, 5, 8).unwrap();

    println!("{master_key}");
}
