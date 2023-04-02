// Test an object detection threshold against an input frame. Optionally calibrate

use crate::subs::runnable::RunnableSubcommand;

use solhat::{processing::HaProcessing, ser, threshtest, vprintln};

use sciimg::path;
use sciimg::prelude::*;
use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Add images", long_about = None)]
pub struct ThreshTest {
    #[clap(long, short, help = "Input images", multiple_values(false))]
    input_file: String,

    #[clap(long, short, help = "Output image")]
    output: String,

    #[clap(long, short, help = "Object detection threshold")]
    threshold: f32,

    #[clap(long, short, help = "Flat frame file")]
    flat: Option<String>,

    #[clap(long, short, help = "Dark frame file")]
    dark: Option<String>,

    #[clap(long, short = 'D', help = "dark Flat frame file")]
    darkflat: Option<String>,
}

impl RunnableSubcommand for ThreshTest {
    fn run(&self) {
        if !path::file_exists(&self.input_file) {
            eprintln!("Error: File not found: {}", self.input_file);
            process::exit(1);
        }

        if !path::parent_exists_and_writable(&self.output) {
            eprintln!("Error: Invalid path for output image: {}", self.output);
            process::exit(2);
        }

        let flat_frame = match &self.flat {
            Some(f) => {
                if !path::file_exists(f) {
                    eprintln!("Error: Flat file not found: {}", f);
                }
                if HaProcessing::is_ser_file(f) {
                    HaProcessing::create_mean_from_ser(f).unwrap()
                } else {
                    Image::open_str(f).unwrap()
                }
            }
            None => Image::new_empty().unwrap(),
        };

        let dark_frame = match &self.dark {
            Some(f) => {
                if !path::file_exists(f) {
                    eprintln!("Error: Dark file not found: {}", f);
                }
                if HaProcessing::is_ser_file(f) {
                    HaProcessing::create_mean_from_ser(f).unwrap()
                } else {
                    Image::open_str(f).unwrap()
                }
            }
            None => Image::new_empty().unwrap(),
        };

        let dark_flat_frame = match &self.darkflat {
            Some(f) => {
                if !path::file_exists(f) {
                    eprintln!("Error: Dark flat file not found: {}", f);
                }
                if HaProcessing::is_ser_file(f) {
                    HaProcessing::create_mean_from_ser(f).unwrap()
                } else {
                    Image::open_str(f).unwrap()
                }
            }
            None => Image::new_empty().unwrap(),
        };

        vprintln!("Loading SER file from {}", self.input_file);
        let ser_file = ser::SerFile::load_ser(&self.input_file).expect("Failed to load SER file");

        if ser_file.frame_count == 0 {
            eprintln!("Error: Input file has no frames");
            process::exit(3);
        }

        let frame = ser_file.get_frame(0).expect("Failed to retrieve frame");
        let mut buffer = frame.buffer;
        buffer.calibrate(&flat_frame, &dark_frame, &dark_flat_frame);

        let out_img = threshtest::threshtest(&buffer, self.threshold);

        out_img.save_8bit(&self.output);
    }
}
