use solar_ha_processing::{
    constants,
    print,
    path,
    vprintln,
    mean
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


    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    let mean_stack = mean::compute_mean(&input_files, true).expect("Failed to calculate mean");

    vprintln!("Saving stack buffer to {}", output_file_path);
    mean_stack.save(output_file_path).expect("Failed to save output frame image");
}