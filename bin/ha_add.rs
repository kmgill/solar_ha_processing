use solar_ha_processing::{
    constants,
    print,
    path,
    vprintln,
    imagebuffer
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

    let mut composite = imagebuffer::ImageBuffer::new_empty().unwrap();

    matches.values_of(constants::param::PARAM_INPUTS).unwrap().for_each(|input_file| {
        
        vprintln!("Loading input file {}", input_file);

        if ! path::file_exists(&input_file) {
            eprintln!("Error: Input file not found: {}", input_file);
            process::exit(1);
        }

        let input = imagebuffer::ImageBuffer::from_file(&input_file).expect("Error: failed to load file");

        if composite.is_empty() {
            composite = input;
        } else {
            composite.add_mut(&input);
        }

    });

    vprintln!("Writing output file to {}", output_file);
    composite.save(&output_file).expect("Error saving composite image");

}