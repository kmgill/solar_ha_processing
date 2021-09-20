// A simple frame extraction tool. Does no preprocessing. 

use solar_ha_processing::{
    ser,
    constants,
    print,
    path,
    vprintln,
    quality,
    util
};

use std::fs;

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
                    .arg(Arg::with_name(constants::param::PARAM_QUALITY)
                        .short(constants::param::PARAM_QUALITY_SHORT)
                        .long(constants::param::PARAM_QUALITY)
                        .value_name("QUALITY")
                        .help("Quality estimation sorting")
                        .required(false)
                        .takes_value(false))
                    .arg(Arg::with_name(constants::param::PARAM_MIN_SIGMA)
                        .short(constants::param::PARAM_MIN_SIGMA_SHORT)
                        .long(constants::param::PARAM_MIN_SIGMA)
                        .value_name("MINSIGMA")
                        .help("Minimum sigma value (quality)")
                        .required(false)
                        .takes_value(true)) 
                    .arg(Arg::with_name(constants::param::PARAM_MAX_SIGMA)
                        .short(constants::param::PARAM_MAX_SIGMA_SHORT)
                        .long(constants::param::PARAM_MAX_SIGMA)
                        .value_name("MAXSIGMA")
                        .help("Maximum sigma value (quality)")
                        .required(false)
                        .takes_value(true))  
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
                    .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let do_qual_sorting = matches.is_present(constants::param::PARAM_QUALITY);

    // If, for some weird reason, clap misses the missing parameter...
    if ! matches.is_present(constants::param::PARAM_INPUTS) {
        println!("{}", matches.usage());
        process::exit(1);
    }

    let min_sigma = match matches.is_present(constants::param::PARAM_MIN_SIGMA) {
        true => {
            let s = matches.value_of(constants::param::PARAM_MIN_SIGMA).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for minumum sigma");
                process::exit(1);
            }
        },
        false => 0.0
    };

    let max_sigma = match matches.is_present(constants::param::PARAM_MAX_SIGMA) {
        true => {
            let s = matches.value_of(constants::param::PARAM_MAX_SIGMA).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for maximum sigma");
                process::exit(1);
            }
        },
        false => 100.0
    };

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();
    for ser_file_path in input_files.iter() {
        if ! path::file_exists(ser_file_path) {
            eprintln!("Error: Specified file not found: {}", ser_file_path);
            process::exit(2);
        }

        let mut output_directory = match matches.is_present(constants::param::PARAM_OUTPUT) {
            true => String::from(matches.value_of(constants::param::PARAM_OUTPUT).unwrap()),
            false => path::get_parent(ser_file_path)
        };

        if input_files.len() > 1 {
            let bn = path::basename(ser_file_path);
            let out_file_base = bn.replace(".ser", "").replace(".SER", "");
            output_directory = format!("{}/{}", &output_directory, &out_file_base);
            if ! path::file_exists(&output_directory.as_str()) {
                let err = format!("Failed to create output directory {}", &output_directory);
                fs::create_dir(&output_directory).expect(err.as_str());
            }
        }


        let ser_file = ser::SerFile::load_ser(ser_file_path).expect("Unable to load SER file");
        ser_file.validate();

        

        for f in 0..ser_file.frame_count {
            let frame = ser_file.get_frame(f).expect("Failed extracting frame");
            let sd = quality::get_quality_estimation(&frame.buffer);

            if sd < min_sigma || sd > max_sigma {
                vprintln!("Frame #{} is outside of sigma range ({})", f, sd);
                continue;
            }

            let new_extension = match do_qual_sorting {
                true => {
                    format!("_{}_{:0width$}.png", (sd * 10000.0) as u32, f, width = 5)
                },
                false => format!("_{:0width$}.png", f, width = 5)
            };

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