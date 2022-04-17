use solar_ha_processing::{
    constants,
    print,
    path,
    vprintln
};

use sciimg::{
    imagebuffer,
    enums::ImageMode
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
                        .help("Input(s)")
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

    
    let output_file = match matches.is_present(constants::param::PARAM_OUTPUT) {
        true => String::from(matches.value_of(constants::param::PARAM_OUTPUT).unwrap()),
        false => {
            println!("{}", matches.usage());
            process::exit(1);
        }
    };

    if ! path::parent_exists_and_writable(&output_file) {
        eprintln!("Error: Output parent directory does not exist or is unwritable: {}", path::get_parent(&output_file));
        process::exit(2);
    }


    let first = matches.values_of(constants::param::PARAM_INPUTS).unwrap().nth(0).unwrap();
    let second = matches.values_of(constants::param::PARAM_INPUTS).unwrap().nth(1).unwrap();

    vprintln!("Loading input file {}", first);
    if ! path::file_exists(&first) {
        eprintln!("Error: Input file not found: {}", first);
        process::exit(1);
    }

    vprintln!("Loading input file {}", second);
    if ! path::file_exists(&second) {
        eprintln!("Error: Input file not found: {}", second);
        process::exit(1);
    }

    let mut first_buff = imagebuffer::ImageBuffer::from_file(&first).expect("Error: failed to load file");
    let mut second_buff = imagebuffer::ImageBuffer::from_file(&second).expect("Error: failed to load file");

    second_buff.scale_mut(0.6);
    first_buff.subtract_mut(&second_buff);
    
    vprintln!("Writing output file to {}", output_file);
    first_buff.save(&output_file, ImageMode::U16BIT);
}