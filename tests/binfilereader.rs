use anyhow::Result;
use sciimg::binfilereader::*;

#[test]
fn test_on_ser_file() -> Result<()> {
    let test_ser_file = "testdata/Sun_130540_F0001-0005.ser";

    let file_reader = BinFileReader::new(&test_ser_file.to_string());

    assert_eq!(file_reader.read_i32(26)?, 1936); // Image width
    assert_eq!(file_reader.read_i32(30)?, 1216); // Image height
    assert_eq!(file_reader.read_i32(34)?, 16); // Pixel bit depth
    assert_eq!(file_reader.read_i32(38)?, 5); // Number of frames
    assert_eq!(
        file_reader.read_string(42, 40)?,
        "Kevin M. Gill\0\0\0\0\0\0\0@\u{7}\0\0\0\0\0\0\0\0\0\0\0\0\0\0\u{5}\0\0\0"
    );
    assert_eq!(
        file_reader.read_string(82, 40)?,
        "ASI=ZWO ASI174MMtemp=45.8\0P\0l\0a\0y\0e\0r\0\\\0"
    );
    Ok(())
}
