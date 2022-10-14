use solhat::{path, ser, timestamp};

#[test]
fn test_load_ser() {
    let test_ser_file = "testdata/Sun_150729.ser";

    // Validate file exists
    assert_eq!(path::file_exists(test_ser_file), true);

    // Load SER file and validate
    let ser_file = ser::SerFile::load_ser(test_ser_file).expect("Unable to load SER file");
    ser_file.validate();

    // Print header details to stdout
    ser_file.print_header_details();

    // Validate header values match expectations
    assert_eq!(ser_file.file_id, "LUCAM-RECORDER");
    assert_eq!(ser_file.camera_series_id, 0);
    assert_eq!(ser_file.color_id, ser::ColorFormatId::Mono);
    assert_eq!(ser_file.image_width, 1936);
    assert_eq!(ser_file.image_height, 1216);
    assert_eq!(ser_file.pixel_depth, 8);
    assert_eq!(ser_file.frame_count, 114);
    assert_eq!(
        ser_file.observer,
        "                                        "
    );
    assert_eq!(
        ser_file.instrument,
        "ASI=ZWO ASI174MMtemp=46.5               "
    );
    assert_eq!(
        ser_file.telescope,
        "fps=92.17gain=160exp=1.50               "
    );
    assert_eq!(
        ser_file.date_time,
        timestamp::TimeStamp::from_u64(637648348476600000)
    );
    assert_eq!(
        ser_file.date_time_utc,
        timestamp::TimeStamp::from_u64(637648348476340000)
    );
    assert_eq!(ser_file.total_size, 268377154);

    // Validate timestamps are present in file
    assert_eq!(ser_file.has_timestamps(), true);

    // Validate image size in bytes, but calculated and expected
    let image_frame_size_bytes =
        ser_file.image_width * ser_file.image_height * (ser_file.pixel_depth / 8);
    assert_eq!(image_frame_size_bytes, 2354176);
    assert_eq!(ser_file.image_frame_size_bytes(), 2354176);

    // Validate bytes per pixel
    let bytes_per_pixel = ser_file.pixel_depth / 8;
    assert_eq!(bytes_per_pixel, 1);

    // Validate image size in bytes via bytes per pixel, width, and height
    let expected_image_size = bytes_per_pixel * ser_file.image_width * ser_file.image_height;
    assert_eq!(expected_image_size, image_frame_size_bytes);

    // Validate image start byte index
    assert_eq!(ser_file.image_frame_start_index(0), 178);
    assert_eq!(
        ser_file.image_frame_start_index(1),
        178 + expected_image_size
    );

    // Validate SER file size matches expectations
    let has_ts = if ser_file.has_timestamps() { 1 } else { 0 };
    let expected_size = 178 +  // Header
                        (image_frame_size_bytes * ser_file.frame_count) +   // Frames
                        (8 * ser_file.frame_count * has_ts); // Timestamps

    println!("Expected File Size: {}", expected_size);
    assert_eq!(ser_file.total_size, expected_size);
    assert_eq!(ser_file.expected_size(), expected_size);

    // Validate expected timestamp block start byte index
    let expected_timestamp_block_start = 178 + (image_frame_size_bytes * ser_file.frame_count);
    assert_eq!(
        ser_file.timestamp_block_start_index(),
        expected_timestamp_block_start
    );

    // Validate expected individual timestamp start byte index
    assert_eq!(
        ser_file.timestamp_start_index(0),
        expected_timestamp_block_start
    );
    assert_eq!(
        ser_file.timestamp_start_index(1),
        expected_timestamp_block_start + 8
    );
}

#[test]
fn test_fetch_frame() {
    let test_ser_file = "testdata/Sun_150729.ser";

    // Validate file exists
    assert_eq!(path::file_exists(test_ser_file), true);

    // Load SER file
    let ser_file = ser::SerFile::load_ser(test_ser_file).expect("Unable to load SER file");

    // Validate byte index of first frame
    assert_eq!(178, ser_file.image_frame_start_index(0));

    // Fetch first frame
    let frame_0 = ser_file
        .get_frame(0)
        .expect("Failed extracting frame at index 0");

    // Validate expected frame width matches buffer
    assert_eq!(frame_0.buffer.width, ser_file.image_width);

    // Validate expected frame height matches buffer
    assert_eq!(frame_0.buffer.height, ser_file.image_height);

    println!("Timestamp: {:?}", frame_0.timestamp);
    assert_eq!(
        frame_0.timestamp,
        timestamp::TimeStamp::from_u64(637648348476340000)
    ); // Need to validate this value

    // Validate frame saves to disk. Check output manually
    //frame_0.buffer.save_8bit("testdata/test_frame_0.png").expect("Failed to save test frame to testdata directory");
}
