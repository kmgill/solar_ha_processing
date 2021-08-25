

use solar_ha_processing::{
    ser,
    constants,
    print,
    path,
    vprintln,
    util,
    imagebuffer,
    error,
    enums,
    mean
};

#[macro_use]
extern crate clap;

use clap::{Arg, App};
use std::process;


struct HaProcessing {
    pub flat_field:imagebuffer::ImageBuffer,
    pub dark_field:imagebuffer::ImageBuffer,
    pub width:usize,
    pub height:usize,
    pub buffer:imagebuffer::ImageBuffer,
    pub frame_count:u32,
    pub obj_detect_threshold:f32
}

impl HaProcessing {

    fn is_ser_file(ser_file_path:&str) -> bool {
        match path::get_extension(ser_file_path) {
            Some("ser") | Some("SER") => true,
            _ => false
        }
    }

    fn create_mean_from_ser(ser_file_path:&str) -> error::Result<imagebuffer::ImageBuffer> {
        if ! HaProcessing::is_ser_file(ser_file_path) {
            Err("Not a SER file")
        } else {
            let input_files:Vec<&str> =vec![ser_file_path];
            let mean_stack = mean::compute_mean(&input_files, true).expect("Failed to calculate mean");
            Ok(mean_stack)
        }
    }

    pub fn init_new(flat_path:&str, dark_path:&str, crop_width:usize, crop_height:usize, obj_detect_threshold:f32) -> error::Result<HaProcessing> {
        let flat = match flat_path.len() {
            0 => imagebuffer::ImageBuffer::new_empty().unwrap(),
            _ => {
                if ! path::file_exists(flat_path) {
                    panic!("File not found: {}", flat_path);
                }

                if HaProcessing::is_ser_file(flat_path) {
                    HaProcessing::create_mean_from_ser(flat_path).unwrap()
                } else {
                    imagebuffer::ImageBuffer::from_file(flat_path).unwrap()
                }
                
            }
        };
    
        let dark = match dark_path.len() {
            0 => imagebuffer::ImageBuffer::new_empty().unwrap(),
            _ => {
                if ! path::file_exists(dark_path) {
                    panic!("File not found: {}", dark_path);
                }

                if HaProcessing::is_ser_file(dark_path) {
                    HaProcessing::create_mean_from_ser(dark_path).unwrap()
                } else {
                    imagebuffer::ImageBuffer::from_file(dark_path).unwrap()
                }
            }
        };
    
        Ok(
            HaProcessing {
                flat_field:flat,
                dark_field:dark,
                width:crop_width,
                height:crop_height,
                buffer:imagebuffer::ImageBuffer::new_as_mode(crop_width, crop_height, enums::ImageMode::U8BIT).unwrap(),
                frame_count:0,
                obj_detect_threshold:obj_detect_threshold
            }
        )
    }

    fn apply_dark_flat_on_buffer(&self, buffer:&imagebuffer::ImageBuffer) -> error::Result<imagebuffer::ImageBuffer> {

        let mut frame_buffer = buffer.clone();
        if ! self.flat_field.is_empty() && ! self.dark_field.is_empty() {
            let darkflat = self.flat_field.subtract(&self.dark_field).unwrap();
            let mean_flat = darkflat.mean();
            let frame_minus_dark = frame_buffer.subtract(&self.dark_field).unwrap();
            frame_buffer = frame_minus_dark.scale(mean_flat).unwrap().divide(&self.flat_field).unwrap();
        } else if ! self.flat_field.is_empty() && self.dark_field.is_empty() {
            let mean_flat = self.flat_field.mean();
            frame_buffer = frame_buffer.scale(mean_flat).unwrap().divide(&self.flat_field).unwrap();
        } else if self.flat_field.is_empty() && ! self.dark_field.is_empty() {
            frame_buffer = frame_buffer.subtract(&self.dark_field).unwrap();
        }

        Ok(frame_buffer)
    }


    pub fn add_frame(&mut self, buffer:&imagebuffer::ImageBuffer) {

        let mut frame_buffer = buffer.clone();

        frame_buffer = self.apply_dark_flat_on_buffer(&frame_buffer).unwrap();

        // TODO: Apply time-based rotation for images taken on an az-el mount

        let com = frame_buffer.calc_center_of_mass_offset(40.0).unwrap();
        frame_buffer = frame_buffer.shift(com.h, com.v).unwrap();

        if self.width > 0 && self.height > 0 {
            frame_buffer = frame_buffer.crop(self.width, self.height).unwrap();
        }

        self.buffer = self.buffer.add(&frame_buffer).unwrap();
        self.frame_count += 1;
    }

    pub fn finalize(&self, out_path:&str) -> error::Result<&str> {

        if self.frame_count > 0 {
            let mean_buffer = self.buffer.scale(1.0 / self.frame_count as f32).unwrap();
            let stackmm = mean_buffer.get_min_max().unwrap();
            vprintln!("    Stack Min/Max : {}, {} ({} images)", stackmm.min, stackmm.max, self.frame_count);

            mean_buffer.save(out_path).expect("Error: Error saving output image");

            solar_ha_processing::ok!()
        } else {
            Err("No frames processed, not saving an empty buffer")
        }

    }

}


fn process_ser_file(ser_file_path:&str, proc_ha:&mut HaProcessing) {

    if ! path::file_exists(ser_file_path) {
        panic!("File not found: {}", ser_file_path);
    }

    let ser_file = ser::SerFile::load_ser(ser_file_path).expect("Unable to load SER file");
    ser_file.validate();

    for i in 0..ser_file.frame_count {
        // if i >= 3 {
        //     break;
        // }
        let frame_buffer = ser_file.get_frame(i).unwrap();

        // TODO: Detect and reject glitch frames

        proc_ha.add_frame(&frame_buffer.buffer);
    }

}





fn main() {
    let matches = App::new(crate_name!())
                    .version(crate_version!())
                    .author(crate_authors!())
                    .arg(Arg::with_name(constants::param::PARAM_INPUTS)
                        .short(constants::param::PARAM_INPUTS_SHORT)
                        .long(constants::param::PARAM_INPUTS)
                        .value_name("INPUT")
                        .help("Input SER files")
                        .required(true)
                        .takes_value(true)
                        .multiple(true)) 
                    .arg(Arg::with_name(constants::param::PARAM_OUTPUT)
                        .short(constants::param::PARAM_OUTPUT_SHORT)
                        .long(constants::param::PARAM_OUTPUT)
                        .value_name("OUTPUT")
                        .help("Output file")
                        .required(true)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_FLAT_FRAME)
                        .short(constants::param::PARAM_FLAT_FRAME_SHORT)
                        .long(constants::param::PARAM_FLAT_FRAME)
                        .value_name("FLAT")
                        .help("Flat frame image")
                        .required(false)
                        .takes_value(true)) 
                    .arg(Arg::with_name(constants::param::PARAM_DARK_FRAME)
                        .short(constants::param::PARAM_DARK_FRAME_SHORT)
                        .long(constants::param::PARAM_DARK_FRAME)
                        .value_name("DARK")
                        .help("Dark frame image")
                        .required(false)
                        .takes_value(true)) 
                    .arg(Arg::with_name(constants::param::PARAM_CROP_WIDTH)
                        .short(constants::param::PARAM_CROP_WIDTH_SHORT)
                        .long(constants::param::PARAM_CROP_WIDTH)
                        .value_name("WIDTH")
                        .help("Crop width")
                        .required(false)
                        .takes_value(true))    
                    .arg(Arg::with_name(constants::param::PARAM_CROP_HEIGHT)
                        .short(constants::param::PARAM_CROP_HEIGHT_SHORT)
                        .long(constants::param::PARAM_CROP_HEIGHT)
                        .value_name("HEIGHT")
                        .help("Crop height")
                        .required(false)
                        .takes_value(true))    
                    .arg(Arg::with_name(constants::param::PARAM_OBJ_DETECT_THRESHOLD)
                        .short(constants::param::PARAM_OBJ_DETECT_THRESHOLD_SHORT)
                        .long(constants::param::PARAM_OBJ_DETECT_THRESHOLD)
                        .value_name("THRESHOLD")
                        .help("Object detection threshold")
                        .required(false)
                        .takes_value(true))          
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
                    .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    // If, for some weird reason, clap misses the missing parameter...
    if ! matches.is_present(constants::param::PARAM_INPUTS) {
        println!("{}", matches.usage());
        process::exit(1);
    }
    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    let obj_detect_threshold = match matches.is_present(constants::param::PARAM_OBJ_DETECT_THRESHOLD) {
        true => {
            let s = matches.value_of(constants::param::PARAM_CROP_WIDTH).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for red scalar");
                process::exit(1);
            }
        },
        false => {
            40.0
        }
    };

    let crop_width = match matches.is_present(constants::param::PARAM_CROP_WIDTH) {
        true => {
            let s = matches.value_of(constants::param::PARAM_CROP_WIDTH).unwrap();
            if util::string_is_valid_usize(&s) {
                s.parse::<usize>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for red scalar");
                process::exit(1);
            }
        },
        false => 0
    };

    let crop_height = match matches.is_present(constants::param::PARAM_CROP_HEIGHT) {
        true => {
            let s = matches.value_of(constants::param::PARAM_CROP_HEIGHT).unwrap();
            if util::string_is_valid_usize(&s) {
                s.parse::<usize>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for red scalar");
                process::exit(1);
            }
        },
        false => 0
    };

    if crop_width == 0 && crop_height > 0 || crop_width > 0 && crop_height == 0 {
        eprintln!("Error: Both width and height need to be specified if any are");
        process::exit(1);
    }

    let output_file = match matches.is_present(constants::param::PARAM_OUTPUT) {
        true => {
            let d = String::from(matches.value_of(constants::param::PARAM_OUTPUT).unwrap());
            if ! path::parent_exists_and_writable(&d) {
                eprintln!("Error: Output directory not found or is not writable");
                process::exit(1);
            }

            d
        },
        false => {
            panic!("Error: Output file path not specified");
        }
    };

    let flat_frame = match matches.is_present(constants::param::PARAM_FLAT_FRAME) {
        true => {
            let f = String::from(matches.value_of(constants::param::PARAM_FLAT_FRAME).unwrap());
            if ! path::file_exists(&f) {
                eprintln!("Error: Flat file not found: {}", f);
            }
            f
        },
        false => String::from("")
    };

    let dark_frame = match matches.is_present(constants::param::PARAM_DARK_FRAME) {
        true => {
            let f = String::from(matches.value_of(constants::param::PARAM_DARK_FRAME).unwrap());
            if ! path::file_exists(&f) {
                eprintln!("Error: Dark file not found: {}", f);
            }
            f
        },
        false => String::from("")
    };

    let mut ha_processing = HaProcessing::init_new(&flat_frame, &dark_frame, crop_width, crop_height, obj_detect_threshold).expect("Failed to create processing context");

    for ser_file_path in input_files.iter() {
        process_ser_file(ser_file_path, &mut ha_processing);
    }

    ha_processing.finalize(&output_file).expect("Failed to finalize buffer");

}