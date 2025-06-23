use super::*;
use std::io::Cursor;
use std::io::Read;

#[test]
fn read() {
    let mut buffer = Cursor::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    buffer.seek_relative(4).unwrap();

    let mut view = View::new(buffer, 20).unwrap();

    let mut data = [0; 5];
    view.read_exact(&mut data).unwrap();

    assert_eq!(data, [5, 6, 7, 8, 9]);
}

#[test]
fn read_insufficient_len() {
    let mut buffer = Cursor::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    buffer.seek_relative(4).unwrap();

    let mut view = View::new(buffer, 2).unwrap();

    let mut data = [0; 5];
    let bytes_read = view.read(&mut data).unwrap();

    assert_eq!(bytes_read, 2);
    assert_eq!(data, [5, 6, 0, 0, 0]);
}

#[test]
fn read_insufficient_buffer_size() {
    let buffer = Cursor::new(vec![1, 2, 3]);
    let mut view = View::new(buffer, 100).unwrap();

    let mut data = [0; 5];
    let bytes_read = view.read(&mut data).unwrap();

    assert_eq!(bytes_read, 3);
    assert_eq!(data, [1, 2, 3, 0, 0]);
}
