use crate::subs::runnable::RunnableSubcommand;

use solhat::{processing, ser, vprintln};

use sciimg::{image, path, quality};

use rayon::prelude::*;
use std::fs;
use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Extract frames from SER file", long_about = None)]
pub struct Extract {
    #[clap(long, short, help = "Input images", multiple_values(true))]
    input_files: Vec<String>,

    #[clap(long, short, help = "Output directory")]
    output: Option<String>,

    #[clap(long, short, help = "Quality estimation sorting")]
    quality: bool,

    #[clap(long, short, help = "Minimum sigma value")]
    minsigma: Option<f32>,

    #[clap(long, short = 'M', help = "Maximum sigma value")]
    maxsigma: Option<f32>,

    #[clap(long, short, help = "Flat frame file")]
    flat: Option<String>,

    #[clap(long, short, help = "Dark frame file")]
    dark: Option<String>,

    #[clap(long, short = 'D', help = "dark Flat frame file")]
    darkflat: Option<String>,
}

impl RunnableSubcommand for Extract {
    fn run(&self) {
        let min_sigma = self.minsigma.unwrap_or(1.0);
        let max_sigma = self.maxsigma.unwrap_or(100000.0);

        let flat_frame = match &self.flat {
            Some(f) => {
                if !path::file_exists(f) {
                    eprintln!("Error: Flat file not found: {}", f);
                }

                processing::HaProcessing::create_mean_from_ser(f).unwrap()
            }
            None => image::Image::new_empty().unwrap(),
        };

        let dark_frame = match &self.dark {
            Some(f) => {
                if !path::file_exists(f) {
                    eprintln!("Error: Dark file not found: {}", f);
                }

                processing::HaProcessing::create_mean_from_ser(f).unwrap()
            }
            None => image::Image::new_empty().unwrap(),
        };

        let dark_flat_frame = match &self.darkflat {
            Some(f) => {
                if !path::file_exists(f) {
                    eprintln!("Error: Dark flat file not found: {}", f);
                }

                processing::HaProcessing::create_mean_from_ser(f).unwrap()
            }
            None => image::Image::new_empty().unwrap(),
        };

        self.input_files.iter().for_each(|ser_file_path| {
            if !path::file_exists(ser_file_path) {
                eprintln!("Error: Specified file not found: {}", ser_file_path);
                process::exit(2);
            }

            let mut output_directory = match &self.output {
                Some(o) => o.clone(),
                None => path::get_parent(ser_file_path),
            };

            if self.input_files.len() > 1 {
                let bn = path::basename(ser_file_path);
                let out_file_base = bn.replace(".ser", "").replace(".SER", "");
                output_directory = format!("{}/{}", &output_directory, &out_file_base);
                if !path::file_exists(output_directory.as_str()) {
                    let err = format!("Failed to create output directory {}", &output_directory);
                    fs::create_dir(&output_directory).unwrap_or_else(|_| panic!("{}", err));
                }
            }

            let ser_file = ser::SerFile::load_ser(ser_file_path).expect("Unable to load SER file");
            ser_file.validate();

            (0..ser_file.frame_count).into_par_iter().for_each(|i| {
                let mut frame = ser_file.get_frame(i).expect("Failed extracting frame");

                frame
                    .buffer
                    .calibrate(&flat_frame, &dark_frame, &dark_flat_frame);

                let sd = quality::get_quality_estimation(&frame.buffer);

                if sd < min_sigma || sd > max_sigma {
                    vprintln!("Frame #{} is outside of sigma range ({})", i, sd);
                    return;
                }

                let new_extension = match self.quality {
                    true => {
                        format!("_{}_{:0width$}.png", (sd * 10000.0) as u32, i, width = 5)
                    }
                    false => format!("_{:0width$}.png", i, width = 5),
                };

                let new_output_parent =
                    format!("{}/{}", output_directory, path::basename(ser_file_path));
                let frame_output_path = new_output_parent
                    .replace(".ser", &new_extension)
                    .replace(".SER", &new_extension);

                vprintln!("Frame #{} Output: {}", i, frame_output_path);

                if !path::parent_exists_and_writable(&frame_output_path) {
                    eprintln!("Error: Output file path cannot be found or is unwritable");
                    process::exit(3);
                }

                frame.buffer.save(&frame_output_path);
            });
        });
    }
}
