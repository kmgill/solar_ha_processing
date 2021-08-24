use solar_ha_processing::{
    ser,
    constants,
    print,
    path,
    vprintln
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
                        .takes_value(true)
                        .multiple(true)) 
                    .arg(Arg::with_name(constants::param::PARAM_OUTPUT)
                        .short(constants::param::PARAM_OUTPUT_SHORT)
                        .long(constants::param::PARAM_OUTPUT)
                        .value_name("OUTPUT")
                        .help("Output directory")
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
    for ser_file_path in input_files.iter() {
        if ! path::file_exists(ser_file_path) {
            eprintln!("Error: Specified file not found: {}", ser_file_path);
            process::exit(2);
        }

        let output_directory = match matches.is_present(constants::param::PARAM_OUTPUT) {
            true => String::from(matches.value_of(constants::param::PARAM_OUTPUT).unwrap()),
            false => path::get_parent(ser_file_path)
        };

        let ser_file = ser::SerFile::load_ser(ser_file_path).expect("Unable to load SER file");
        ser_file.validate();

        for f in 0..ser_file.frame_count {
            let frame = ser_file.get_frame(f).expect("Failed extracting frame");

            let new_extension = format!("_{:0width$}.png", f, width = 5);
            let new_output_parent = format!("{}/{}", output_directory, path::basename(ser_file_path));
            let frame_output_path = new_output_parent.replace(".ser", &new_extension).replace(".SER", &new_extension);
            
            vprintln!("Frame #{} Output: {}", f, frame_output_path);
            

            if ! path::parent_exists_and_writable(&frame_output_path) {
                eprintln!("Error: Output file path cannot be found or is unwritable");
                process::exit(3);
            }


            frame.buffer.save(&frame_output_path).expect("Failed to save output frame image");

        }
    }

    

 



}