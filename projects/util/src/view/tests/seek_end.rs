use super::*;
use byteorder::ReadBytesExt;
use std::io::Cursor;
use std::io::Read;

#[test]
fn test_view_seek_end() {
    let buffer = Cursor::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    let mut view = View::new(buffer, 5).unwrap();

    let new_position = view.seek(SeekFrom::End(-1)).unwrap();
    assert_eq!(new_position, 3);

    assert_eq!(view.read_u8().unwrap(), 4);
}

#[test]
fn test_view_seek_end_out_of_bounds() {
    let buffer = Cursor::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    let mut view = View::new(buffer, 3).unwrap();
    view.seek(SeekFrom::End(200)).unwrap();

    let mut data = [0; 5];
    let bytes_read = view.read(&mut data).unwrap();

    assert_eq!(bytes_read, 0);
    assert_eq!(data, [0, 0, 0, 0, 0]);
}

#[test]
fn test_view_seek_end_negative_offset_outside_buffer() {
    let buffer = Cursor::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    let mut view = View::new(buffer, 3).unwrap();
    assert!(view.seek(SeekFrom::End(-200)).is_err());
}

#[test]
fn test_view_seek_end_negative_offset_outside_view() {
    let mut buffer = Cursor::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    buffer.seek_relative(2).unwrap();

    let mut view = View::new(buffer, 3).unwrap();
    assert!(view.seek(SeekFrom::Current(-3)).is_err());
}
