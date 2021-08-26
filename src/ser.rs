
// Technical specification: http://www.grischa-hahn.homepage.t-online.de/astro/ser/SER%20Doc%20V3b.pdf

use crate::{
    imagebuffer,
    error,
    vprintln,
    print,
    enums,
    timestamp
};

use std::convert::TryInto;

use memmap::Mmap;
use std::fs::File;

const HEADER_SIZE_BYTES : usize = 178;
const TIMESTAMP_SIZE_BYTES : usize = 8;


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ColorFormatId {
    Mono = 0,
    BayerRggb = 8,
    BayerGrbg = 9,
    BayerGbrg = 10,
    BayerBggr = 11,
    BayerCyym = 16,
    BayerYcmy = 17,
    BayerYmcy = 18,
    BayerMyyc = 19,
    Rgb       = 100,
    Bgr       = 101
}

impl ColorFormatId {
    pub fn from_i32(v:i32) -> ColorFormatId {
        match v {
            0 => ColorFormatId::Mono,
            8 => ColorFormatId::BayerRggb,
            9 => ColorFormatId::BayerGrbg,
            10 => ColorFormatId::BayerGbrg,
            11 => ColorFormatId::BayerBggr,
            16 => ColorFormatId::BayerCyym,
            17 => ColorFormatId::BayerYcmy,
            18 => ColorFormatId::BayerYmcy,
            19 => ColorFormatId::BayerMyyc,
            100 => ColorFormatId::Rgb,
            101 => ColorFormatId::Bgr,
            _ => panic!("Invalid color format enum value: {}", v)
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Endian {
    BigEndian = 1,
    LittleEndian = 0,
    NativeEndian = 100
}

impl Endian {
    pub fn from_i32(v:i32) -> Endian {
        match v {
            1 => Endian::BigEndian,
            0 => Endian::LittleEndian,
            100 => Endian::NativeEndian,
            _ => panic!("Invalid endian enum value")
        }
    }
}


// Variable size of pixel_depth * image_width * image_height
// Frames block is frame_size * num_images
// Frames block starts off at byte 178
pub struct SerFrame {
    pub buffer:imagebuffer::ImageBuffer,
    pub timestamp: timestamp::TimeStamp
}

// Header is a fixed size of 178 bytes
// Optional trailer starts at num_images * pixel_depth * image_width * image_height
// Trailer size is 8 byte (i64) time stamps for each frame, size is 8 * num_images
pub struct SerFile {
    pub file_id: String,            // 14 bytes
    pub camera_series_id: i32,      // 4 bytes
    pub color_id: ColorFormatId,    // 4 bytes
    pub endian: Endian,             // 4 bytes
    pub image_width: usize,         // 4 bytes
    pub image_height: usize,        // 4 bytes
    pub pixel_depth: usize,         // 4 bytes
    pub frame_count: usize,         // 4 bytes
    pub observer: String,           // 40 bytes
    pub instrument: String,         // 40 bytes
    pub telescope: String,          // 40 bytes
    pub date_time: timestamp::TimeStamp,       // 8 bytes,
    pub date_time_utc: timestamp::TimeStamp,   // 8 bytes,
    pub total_size: usize,          // Total file size (used for validation)
    map: Mmap,
    source_file:String
}


fn read_string(map:&Mmap, start:usize, len:usize) -> String {
    let v:Vec<u8> = map[start..(start + len)].iter().map(|x| x.clone()).collect();
    String::from_utf8(v).expect("Failed reading string value")
}

macro_rules! bytes_to_primitive {
    ($bytes:ident, $type:ident, $endian:expr) => {
        match $endian {
            Endian::BigEndian => $type::from_be_bytes($bytes),
            Endian::LittleEndian => $type::from_le_bytes($bytes),
            Endian::NativeEndian => $type::from_ne_bytes($bytes)
        }
    };
}

fn read_u64(map:&Mmap, start:usize) -> u64 {
    let v: [u8; 8] = map[start..(start + 8)].try_into().expect("slice with incorrect length");
    bytes_to_primitive!(v, u64, Endian::LittleEndian)
}

fn read_i32(map:&Mmap, start:usize) -> i32 {
    let v: [u8; 4] = map[start..(start + 4)].try_into().expect("slice with incorrect length");
    bytes_to_primitive!(v, i32, Endian::LittleEndian)
}

impl SerFrame {
    pub fn new(buffer:imagebuffer::ImageBuffer, timestamp:u64) -> SerFrame {
        SerFrame {
            buffer:buffer,
            timestamp:timestamp::TimeStamp::from_u64(timestamp)
        }
    }
}

// Full implementation of the SER specification is sorta impractical at this time
// since I lack both the requisite test data and the motivation to actually do it. 
impl SerFile {

    pub fn print_header_details(&self) {
        println!("SER Header Values:");
        println!("File Id: {}", self.file_id);
        println!("Camera Series Id: {}", self.camera_series_id);
        println!("Color Id: {:?}", self.color_id);
        println!("Endian: {:?}", self.endian);
        println!("Image Width: {}", self.image_width);
        println!("Image Height: {}", self.image_height);
        println!("Pixel Depth: {}", self.pixel_depth);
        println!("Frame Count: {}", self.frame_count);
        println!("Observer: {}", self.observer);
        println!("Instrument: {}", self.instrument);
        println!("Telescope: {}", self.telescope);
        println!("Date/Time: {:?}", self.date_time);
        println!("Date/Time UTC: {:?}", self.date_time_utc);
        println!("Total File Size: {}", self.total_size);
        println!("Bytes per image: {}", self.image_width * self.image_height * (self.pixel_depth / 8));
    }

    pub fn load_ser(file_path:&str) -> error::Result<SerFile> {

        let ser_file_ptr = File::open(file_path).expect("Error opening file");

        let map: Mmap = unsafe { 
            Mmap::map(&ser_file_ptr).expect("Error creating memory map")
        };

        let ser = SerFile {
            file_id: read_string(&map, 0, 14),                               // 14 bytes
            camera_series_id: read_i32(&map, 14),                            // 4 bytes, start at 14
            color_id: ColorFormatId::from_i32(read_i32(&map, 18)),           // 4 bytes, start at 18
            endian: Endian::from_i32(read_i32(&map, 22)),                    // 4 bytes, start at 22
            image_width: read_i32(&map, 26) as usize,                        // 4 bytes, start at 26
            image_height: read_i32(&map, 30) as usize,                       // 4 bytes, start at 30
            pixel_depth: read_i32(&map, 34) as usize,                        // 4 bytes, start at 34
            frame_count: read_i32(&map, 38) as usize,                        // 4 bytes, start at 38
            observer: read_string(&map, 42, 40),                             // 40 bytes, start at 42
            instrument: read_string(&map, 82, 40),                           // 40 bytes, start at 82
            telescope: read_string(&map, 122, 40),                           // 40 bytes, start at 122
            date_time: timestamp::TimeStamp::from_u64(read_u64(&map, 162)),      // 8 bytes, start at 162
            date_time_utc: timestamp::TimeStamp::from_u64(read_u64(&map, 170)),  // 8 bytes, start at 170
            total_size: map.len(),
            map: map,
            source_file: file_path.to_string()
        };

        if print::is_verbose() {
            ser.print_header_details();
        }

        Ok(ser)
    }

    pub fn image_frame_size_bytes(&self) -> usize {
        self.image_width * self.image_height * (self.pixel_depth / 8)
    }

    pub fn image_frame_start_index(&self, frame_num:usize) -> usize {
        HEADER_SIZE_BYTES + (self.image_frame_size_bytes() * frame_num)
    }

    pub fn has_timestamps(&self) -> bool {
        self.total_size > self.timestamp_block_start_index()
    }

    pub fn timestamp_block_start_index(&self) -> usize {
        HEADER_SIZE_BYTES +  // Header
                (self.image_frame_size_bytes() * self.frame_count) // Frames
    }

    pub fn timestamp_start_index(&self, frame_num:usize) -> usize {
        let block_start = self.timestamp_block_start_index();
        block_start + (frame_num * TIMESTAMP_SIZE_BYTES)
    }

    pub fn expected_size(&self) -> usize {
        let has_ts = if self.has_timestamps() { 1 } else { 0 };

        HEADER_SIZE_BYTES +  // Header
            (self.image_frame_size_bytes() * self.frame_count) +   // Frames
            (8 * self.frame_count * has_ts) // Timestamps
    }

    pub fn validate(&self) {
        let expected_size = self.expected_size();
        assert_eq!(self.total_size, expected_size);
    }


    pub fn get_frame_timestamp(&self, frame_num:usize) -> error::Result<u64> {
        if frame_num >= self.frame_count {
            return Err("Frame number out of range");
        }

        if ! self.has_timestamps() {
            return Ok(0);
        }

        let timestamp_start_index = self.timestamp_start_index(frame_num);
        let timestamp_bytes : [u8; 8] = self.map[timestamp_start_index..(timestamp_start_index+TIMESTAMP_SIZE_BYTES)].try_into().expect("slice with incorrect length");

        Ok(
            bytes_to_primitive!(timestamp_bytes, u64, Endian::NativeEndian)
        )
    }

    pub fn get_frame(&self, frame_num:usize) -> error::Result<SerFrame> {

        if frame_num >= self.frame_count {
            return Err("Frame number out of range");
        }

        let image_frame_size_bytes = self.image_frame_size_bytes();
        let image_frame_start_index = self.image_frame_start_index(frame_num);

        vprintln!("Extracting image frame #{} of {} from {}. Size {} at byte index {}", frame_num, self.frame_count, self.source_file, image_frame_size_bytes, image_frame_start_index);

        let bytes:Vec<u8> = self.map[image_frame_start_index..(image_frame_start_index + image_frame_size_bytes)].iter().map(|x| x.clone()).collect();

        let mut values:Vec<f32> = Vec::with_capacity(self.image_width * self.image_height);
        values.resize(self.image_width * self.image_height, 0.0);

        let bytes_per_pixel = self.pixel_depth / 8;
        for y in 0..self.image_height {
            for x in 0..self.image_width {
                
                let pixel_start = (x + (y * self.image_width)) * bytes_per_pixel;
                let pixel_value:f32;

                if self.pixel_depth == 8 {
                    let pixel_bytes : u8 = bytes[pixel_start];
                    pixel_value = pixel_bytes as f32;
                } else if self.pixel_depth == 16 {
                    let pixel_bytes : [u8; 2] = bytes[pixel_start..(pixel_start+2)].try_into().expect("slice with incorrect length");
                    pixel_value = bytes_to_primitive!(pixel_bytes, u16, self.endian) as f32;
                } else {
                    panic!("Encountered unsupported pixel depth: {}", self.pixel_depth);
                }

                values[x + (y * self.image_width)] = pixel_value;
            }
        }
        
        Ok(
            SerFrame::new(
                imagebuffer::ImageBuffer::from_vec_as_mode(values, 
                    self.image_width, 
                    self.image_height,
                    match self.pixel_depth {
                        8 => enums::ImageMode::U8BIT,
                        _ => enums::ImageMode::U16BIT
                    }
                ).expect("Failed to allocate image buffer"),
                self.get_frame_timestamp(frame_num).expect("Failed to extract frame timestamp")
            )
        )
    }

}