// A simple single-frame extraction tool. Does no preprocessing. 

use solar_ha_processing::{
    ser,
    constants,
    print,
    path
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
                        .help("Input")
                        .required(true)
                        .takes_value(true))
                    .arg(Arg::with_name("frame")
                        .short("f")
                        .long("frame")
                        .value_name("frame")
                        .help("Frame number (beginning at 1)")
                        .required(true)
                        .takes_value(true))  
                    .arg(Arg::with_name(constants::param::PARAM_OUTPUT)
                        .short(constants::param::PARAM_OUTPUT_SHORT)
                        .long(constants::param::PARAM_OUTPUT)
                        .value_name("OUTPUT")
                        .help("Output")
                        .required(true)
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

    let ser_file_path = matches.value_of(constants::param::PARAM_INPUTS).unwrap();
    let output_file_path = matches.value_of(constants::param::PARAM_OUTPUT).unwrap();


    if ! path::file_exists(ser_file_path) {
        eprintln!("Error: Specified file not found: {}", ser_file_path);
        process::exit(2);
    }

    let mut frame_num : usize = 0;
    if matches.is_present("frame") {
        let s = matches.value_of("frame").unwrap();
        if solar_ha_processing::is_valid_number!(s, usize) {
            frame_num = s.parse::<usize>().unwrap() - 1; // Subtract one to convert to zero indexing
        } else {
            eprintln!("Error: Invalid number specified");
            process::exit(1);
        }
    }

    let ser_file = ser::SerFile::load_ser(ser_file_path).expect("Unable to load SER file");
    ser_file.validate();

    if frame_num >= ser_file.frame_count {
        eprintln!("Error: Requested frame {} exceeds available frames {}", (frame_num + 1), ser_file.frame_count);
        process::exit(5);
    }

    let frame = ser_file.get_frame(frame_num).expect("Failed extracting frame");

    if ! path::parent_exists_and_writable(output_file_path) {
        eprintln!("Error: Output file path cannot be found or is unwritable");
        process::exit(3);
    }

    frame.buffer.save(output_file_path).expect("Failed to save output frame image");
}