
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

    if ! path::file_exists(ser_file_path) {
        eprintln!("Specified file not found: {}", ser_file_path);
        process::exit(2);
    }

    let ser_file = ser::SerFile::load_ser(ser_file_path).expect("Unable to load SER file");
    ser_file.validate();

    ser_file.print_header_details();
}
