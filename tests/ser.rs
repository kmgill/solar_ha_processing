
use solar_ha_processing::{
    ser,
    path
};


#[test]
fn test_load_ser() {
    let test_ser_file = "testdata/Sun_150729.ser";
    assert_eq!(path::file_exists(test_ser_file), true);

    let ser_file = ser::SerFile::load_ser(test_ser_file).expect("Unable to load SER file");
    ser_file.print_header_details();

    assert_eq!(ser_file.file_id, "LUCAM-RECORDER");
    assert_eq!(ser_file.camera_series_id, 0);
    assert_eq!(ser_file.color_id, ser::ColorFormatId::Mono);
    assert_eq!(ser_file.endian, ser::Endian::BigEndian);
    assert_eq!(ser_file.image_width, 1936);
    assert_eq!(ser_file.image_height, 1216);
    assert_eq!(ser_file.pixel_depth, 8);
    assert_eq!(ser_file.frame_count, 114);
    assert_eq!(ser_file.observer, "                                        ");
    assert_eq!(ser_file.instrument, "ASI=ZWO ASI174MMtemp=46.5               ");
    assert_eq!(ser_file.telescope, "fps=92.17gain=160exp=1.50               ");
    assert_eq!(ser_file.date_time, 1720708800.0);

    
}