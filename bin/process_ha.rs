

use solar_ha_processing::{
    constants,
    print,
    path,
    util,
    processing
};

#[macro_use]
extern crate clap;

use clap::{Arg, App};
use std::process;

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
                    .arg(Arg::with_name(constants::param::PARAM_RED_WEIGHT)
                        .short(constants::param::PARAM_RED_WEIGHT_SHORT)
                        .long(constants::param::PARAM_RED_WEIGHT)
                        .value_name("RED")
                        .help("Red weight")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_GREEN_WEIGHT)
                        .short(constants::param::PARAM_GREEN_WEIGHT_SHORT)
                        .long(constants::param::PARAM_GREEN_WEIGHT)
                        .value_name("GREEN")
                        .help("Green weight")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_BLUE_WEIGHT)
                        .short(constants::param::PARAM_BLUE_WEIGHT_SHORT)
                        .long(constants::param::PARAM_BLUE_WEIGHT)
                        .value_name("BLUE")
                        .help("Blue weight")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_LATITUDE)
                        .short(constants::param::PARAM_LATITUDE_SHORT)
                        .long(constants::param::PARAM_LATITUDE)
                        .value_name("LATITUDE")
                        .help("Observer latitude")
                        .required(true)
                        .allow_hyphen_values(true)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_LONGITUDE)
                        .short(constants::param::PARAM_LONGITUDE_SHORT)
                        .long(constants::param::PARAM_LONGITUDE)
                        .value_name("LONGITUDE")
                        .help("Observer longitude")
                        .required(true)
                        .allow_hyphen_values(true)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_OBJ_DETECT_THRESHOLD)
                        .short(constants::param::PARAM_OBJ_DETECT_THRESHOLD_SHORT)
                        .long(constants::param::PARAM_OBJ_DETECT_THRESHOLD)
                        .value_name("THRESHOLD")
                        .help("Object detection threshold")
                        .required(false)
                        .takes_value(true))     
                    .arg(Arg::with_name(constants::param::PARAM_MASK)
                        .short(constants::param::PARAM_MASK_SHORT)
                        .long(constants::param::PARAM_MASK)
                        .value_name("MASK")
                        .help("Image mask")
                        .required(false)
                        .takes_value(true))   
                    .arg(Arg::with_name(constants::param::PARAM_QUALITY)
                        .short(constants::param::PARAM_QUALITY_SHORT)
                        .long(constants::param::PARAM_QUALITY)
                        .value_name("QUALITY")
                        .help("Quality limit (top % frames)")
                        .required(false)
                        .takes_value(true))        
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
                    .get_matches();

    print::set_verbose(matches.is_present(constants::param::PARAM_VERBOSE));

    // If, for some weird reason, clap misses the missing parameter...
    if ! matches.is_present(constants::param::PARAM_INPUTS) {
        println!("{}", matches.usage());
        process::exit(1);
    }
    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    let obj_detect_threshold = match matches.is_present(constants::param::PARAM_OBJ_DETECT_THRESHOLD) {
        true => {
            let s = matches.value_of(constants::param::PARAM_OBJ_DETECT_THRESHOLD).unwrap();
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

    let mask_file = match matches.is_present(constants::param::PARAM_MASK) {
        true => {
            let f = String::from(matches.value_of(constants::param::PARAM_MASK).unwrap());
            if ! path::file_exists(&f) {
                eprintln!("Error: Mask file not found: {}", f);
            }
            f
        },
        false => String::from("")
    };

    let mut red_scalar = constants::DEFAULT_RED_WEIGHT;
    let mut green_scalar = constants::DEFAULT_GREEN_WEIGHT;
    let mut blue_scalar = constants::DEFAULT_BLUE_WEIGHT;

    if matches.is_present(constants::param::PARAM_RED_WEIGHT) {
        let s = matches.value_of(constants::param::PARAM_RED_WEIGHT).unwrap();
        if util::string_is_valid_f32(&s) {
            red_scalar = s.parse::<f32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified for red scalar");
            process::exit(1);
        }
    }

    if matches.is_present(constants::param::PARAM_GREEN_WEIGHT) {
        let s = matches.value_of(constants::param::PARAM_GREEN_WEIGHT).unwrap();
        if util::string_is_valid_f32(&s) {
            green_scalar = s.parse::<f32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified for green scalar");
            process::exit(1);
        }
    }

    if matches.is_present(constants::param::PARAM_BLUE_WEIGHT) {
        let s = matches.value_of(constants::param::PARAM_BLUE_WEIGHT).unwrap();
        if util::string_is_valid_f32(&s) {
            blue_scalar = s.parse::<f32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified for blue scalar");
            process::exit(1);
        }
    }


    let obs_latitude = match matches.is_present(constants::param::PARAM_LATITUDE) {
        true => {
            let s = matches.value_of(constants::param::PARAM_LATITUDE).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for observer latitude");
                process::exit(1);
            }
        },
        false => {
            eprintln!("Error: Observer latitude not specified");
            process::exit(1);
        }
    };

    let obs_longitude = match matches.is_present(constants::param::PARAM_LONGITUDE) {
        true => {
            let s = matches.value_of(constants::param::PARAM_LONGITUDE).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for observer longitude");
                process::exit(1);
            }
        },
        false => {
            eprintln!("Error: Observer longitude not specified");
            process::exit(1);
        }
    };
    
    let limit_top_pct = match matches.is_present(constants::param::PARAM_QUALITY) {
        true => {
            let s = matches.value_of(constants::param::PARAM_QUALITY).unwrap();
            if util::string_is_valid_u8(&s) {
                let p = s.parse::<u8>().unwrap();
                if p <= 0 {
                    panic!("Error: Quality limit percentage cannot be zero or below");
                } else if p > 100 {
                    panic!("Error: Quality limit percentage cannot exceed 100%");
                }

                p
            } else {
                eprintln!("Error: Invalid number specified for observer longitude");
                process::exit(1);
            }
        },
        false => 100
    };

    let mut ha_processing = processing::HaProcessing::init_new(&flat_frame, 
                                                    &dark_frame, 
                                                    &mask_file,
                                                    crop_width, 
                                                    crop_height, 
                                                    obj_detect_threshold, 
                                                    red_scalar, 
                                                    green_scalar, 
                                                    blue_scalar,
                                                    obs_latitude,
                                                    obs_longitude).expect("Failed to create processing context");
    ha_processing.process_ser_files(&input_files, limit_top_pct);
    ha_processing.finalize(&output_file).expect("Failed to finalize buffer");
}