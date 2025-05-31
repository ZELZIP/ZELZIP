use super::*;
use std::io::Cursor;

#[test]
fn test_view_write() {
    let mut buffer = Cursor::new([1, 2, 3, 4, 5]);
    buffer.seek_relative(1).unwrap();

    let mut view = View::new(&mut buffer, 3).unwrap();

    view.write_all(&[20, 30]).unwrap();

    assert_eq!(buffer.into_inner(), [1, 20, 30, 4, 5]);
}

#[test]
fn test_view_write_insufficient_len() {
    let mut buffer = Cursor::new([1, 2, 3, 4, 5]);

    let mut view = View::new(&mut buffer, 1).unwrap();

    let bytes_written = view.write(&[10, 20, 30]).unwrap();

    assert_eq!(bytes_written, 1);
    assert_eq!(buffer.into_inner(), [10, 2, 3, 4, 5]);
}

#[test]
fn test_view_write_insufficient_buffer_size() {
    let mut buffer = Cursor::new([1, 2]);

    let mut view = View::new(&mut buffer, 100).unwrap();

    let bytes_written = view.write(&[10, 20, 30, 40, 50]).unwrap();

    assert_eq!(bytes_written, 2);
    assert_eq!(buffer.into_inner(), [10, 20]);
}
