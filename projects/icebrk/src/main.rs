//! TODO REMOVE

fn main() {
    println!("GOO");
    let master_key =
        zelzip_icebrk::calculate_v2_master_key(zelzip_icebrk::Platform::The3ds, 5213456789, 5, 8)
            .unwrap();

    println!("{master_key}");
}
