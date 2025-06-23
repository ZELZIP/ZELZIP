use super::*;
use std::io::Cursor;
use std::io::Read;

// Seeking to an negative offset with [SeekFrom::Start] is impossible

#[test]
fn seek_start() {
    let buffer = Cursor::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    let mut view = View::new(buffer, 3).unwrap();
    let new_position = view.seek(SeekFrom::Start(1)).unwrap();
    assert_eq!(new_position, 1);

    let mut data = [0; 2];
    view.read_exact(&mut data).unwrap();

    assert_eq!(data, [2, 3]);
}

#[test]
fn seek_start_out_of_bounds() {
    let buffer = Cursor::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    let mut view = View::new(buffer, 3).unwrap();
    view.seek(SeekFrom::Start(200)).unwrap();

    let mut data = [0; 5];
    let bytes_read = view.read(&mut data).unwrap();

    assert_eq!(bytes_read, 0);
    assert_eq!(data, [0, 0, 0, 0, 0]);
}
