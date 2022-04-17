
use solar_ha_processing::{
    constants,
    print,
    path,
    util,
    ser,
    lunar, 
    solar,
    vprintln,
    parallacticangle,
    enums::Target
};

use sciimg::{
    quality
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
                    .arg(Arg::with_name(constants::param::PARAM_TARGET)
                        .short(constants::param::PARAM_TARGET_SHORT)
                        .long(constants::param::PARAM_TARGET)
                        .value_name("PARAM_TARGET")
                        .help("Target (Moon, Sun)")
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

    let target = match matches.is_present(constants::param::PARAM_TARGET) {
        true => {
            match Target::from(matches.value_of(constants::param::PARAM_TARGET).unwrap()) {
                Some(t) => t,
                None => {
                    eprintln!("Error: Unrecognized target value: {}", matches.value_of(constants::param::PARAM_TARGET).unwrap());
                    process::exit(1);
                }
            }
        },
        false => Target::Sun
    };


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

    println!("{:11} {:26} {:8}    {:9}    {:5} {:5}", "Frame Num:", "Date/Time:", "Sigma:", "Rotation:", "Min DN:", "Max DN:");
    for sf in input_files.iter() {
        if ! path::file_exists(sf) {
            panic!("File not found: {}", sf);
        }

        let ser_file = ser::SerFile::load_ser(sf).expect("Unable to load SER file");
        for i in 0..ser_file.frame_count {

            let frame_buffer = ser_file.get_frame(i).unwrap();
            let (alt, az) = match target {
                Target::Moon => {
                    vprintln!("Calculating position for Moon");
                    lunar::position_from_lat_lon_and_time(obs_latitude as f64, obs_longitude as f64, &frame_buffer.timestamp)
                },
                Target::Sun => {
                    vprintln!("Calculating position for Sun");
                    solar::position_from_lat_lon_and_time(obs_latitude as f64, obs_longitude as f64, &frame_buffer.timestamp)
                }
            };
    
            let rotation = parallacticangle::from_lat_azimuth_altitude(obs_latitude as f64, az, alt);
            let (min, max) = frame_buffer.buffer.get_min_max_all_channel();

            let qual = quality::get_quality_estimation(&frame_buffer.buffer);

            println!("{:>10}  {}-{:02}-{:02} {:02}:{:02}:{:02}.{:4}   {:.4} {:>10.4} {:>7}    {:>7}", i, 
                                                    frame_buffer.timestamp.year, 
                                                    frame_buffer.timestamp.month, 
                                                    frame_buffer.timestamp.day,
                                                    frame_buffer.timestamp.hour,
                                                    frame_buffer.timestamp.minute,
                                                    frame_buffer.timestamp.second,
                                                    frame_buffer.timestamp.microsecond / 100,
                                                    qual,
                                                    rotation,
                                                    min, 
                                                    max);

        }

    }
}

