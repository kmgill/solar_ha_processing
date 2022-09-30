use solhat::binfilereader::*;

#[test]
fn test_on_ser_file() {

    let test_ser_file = "testdata/Sun_150729.ser";

    let file_reader = BinFileReader::new(&test_ser_file.to_string());

    assert_eq!(file_reader.read_i32(26), 1936);
    assert_eq!(file_reader.read_i32(30), 1216);
    assert_eq!(file_reader.read_i32(34), 8);
    assert_eq!(file_reader.read_i32(38), 114);
    assert_eq!(file_reader.read_string(42, 40), "                                        ");
    assert_eq!(file_reader.read_string(82, 40), "ASI=ZWO ASI174MMtemp=46.5               ");


}