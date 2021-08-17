
use crate::{
    imagebuffer,
    constants,
    error,
    vprintln,
    print
};

use std::convert::TryInto;

use memmap::Mmap;
use std::fs::File;


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
    BayerMyyc = 19
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
            _ => panic!(format!("Invalid color format enum value: {}", v))

        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Endian {
    BigEndian = 0,
    LittleEndian = 1
}

impl Endian {
    pub fn from_i32(v:i32) -> Endian {
        match v {
            0 => Endian::BigEndian,
            1 => Endian::LittleEndian,
            _ => panic!("Invalid endian enum value")
        }
    }
}

// Variable size of pixel_depth * image_width * image_height
// Frames block is frame_size * num_images
// Frames block starts off at byte 178
pub struct SerFrame {
    pub buffer:imagebuffer::ImageBuffer
}

// Header is a fixed size of 178 bytes
// Optional trailer starts at num_images * pixel_depth * image_width * image_height
// Trailer size is 8 byte (i64) time stamps for each frame, size is 8 * num_images
pub struct SerFile {
    pub file_id: String,            // 14 bytes
    pub camera_series_id: i32,      // 4 bytes
    pub color_id: ColorFormatId,    // 4 bytes
    pub endian: Endian,             // 4 bytes
    pub image_width: i32,           // 4 bytes
    pub image_height: i32,          // 4 bytes
    pub pixel_depth: i32,           // 4 bytes
    pub frame_count: i32,           // 4 bytes
    pub observer: String,           // 40 bytes
    pub instrument: String,         // 40 bytes
    pub telescope: String,          // 40 bytes
    pub date_time: f64              // 4 bytes
}


fn read_string(map:&Mmap, start:usize, len:usize) -> error::Result<String> {
    let v:Vec<u8> = map[start..(start + len)].iter().map(|x| x.clone()).collect();
    let s = String::from_utf8(v).expect("Failed reading string value");
    Ok(s)
}

fn read_i32(map:&Mmap, start:usize) -> error::Result<i32> {
    let v: [u8; 4] = map[start..(start + 4)].try_into().expect("slice with incorrect length");
    let val = i32::from_ne_bytes(v);
    Ok(val)
}

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
        println!("Date/Time: {}", self.date_time);
    }

    pub fn load_ser(file_path:&str) -> error::Result<SerFile> {

        let ser_file_ptr = File::open(file_path).expect("Error opening file");

        let map = unsafe { 
            Mmap::map(&ser_file_ptr).expect("Error creating memory map")
        };

        let ser = SerFile {
            file_id: read_string(&map, 0, 14).expect("Failed to load SER header value"),            // 14 bytes
            camera_series_id: read_i32(&map, 14).expect("Failed to load SER header value"),         // 4 bytes, start at 14
            color_id: ColorFormatId::from_i32(read_i32(&map, 18).expect("Failed to load SER header value")),    // 4 bytes, start at 18
            endian: Endian::from_i32(read_i32(&map, 22).expect("Failed to load SER header value")),             // 4 bytes, start at 22
            image_width: read_i32(&map, 26).expect("Failed to load SER header value"),           // 4 bytes, start at 26
            image_height: read_i32(&map, 30).expect("Failed to load SER header value"),          // 4 bytes, start at 30
            pixel_depth: read_i32(&map, 34).expect("Failed to load SER header value"),           // 4 bytes, start at 34
            frame_count: read_i32(&map, 38).expect("Failed to load SER header value"),           // 4 bytes, start at 38
            observer: read_string(&map, 42, 40).expect("Failed to load SER header value"),           // 40 bytes, start at 42
            instrument: read_string(&map, 82, 40).expect("Failed to load SER header value"),         // 40 bytes, start at 82
            telescope: read_string(&map, 122, 40).expect("Failed to load SER header value"),          // 40 bytes, start at 122
            date_time: read_i32(&map, 162).expect("Failed to load SER header value") as f64             // 4 bytes, start at 162
        };

        if print::is_verbose() {
            ser.print_header_details();
        }

        Ok(ser)
    }

}