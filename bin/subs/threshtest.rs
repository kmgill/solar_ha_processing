// Test an object detection threshold against an uncalibrated input frame

use crate::subs::runnable::RunnableSubcommand;

use solar_ha_processing::{
    path,
    vprintln,
    ser,
    threshtest
};

use std::process;


#[derive(clap::Args)]
#[clap(author, version, about = "Add images", long_about = None)]
pub struct ThreshTest {
    #[clap(long, short, help = "Input images", multiple_values(false))]
    input_file: String,

    #[clap(long, short, help = "Output image")]
    output: String,

    #[clap(long, short, help = "Object detection threshold")]
    threshold: f32
}   

impl RunnableSubcommand for ThreshTest {
    fn run(&self) {
        if ! path::file_exists(&self.input_file) {
            eprintln!("Error: File not found: {}", self.input_file);
            process::exit(1);
        }

        if ! path::parent_exists_and_writable(&self.output) {
            eprintln!("Error: Invalid path for output image: {}", self.output);
            process::exit(2);
        }

        vprintln!("Loading SER file from {}", self.input_file);
        let ser_file = ser::SerFile::load_ser(&self.input_file).expect("Failed to load SER file");

        if ser_file.frame_count == 0 {
            eprintln!("Error: Input file has no frames");
            process::exit(3);
        }

        let frame = ser_file.get_frame(0).expect("Failed to retrieve frame");

        let out_img = threshtest::threshtest(&frame.buffer, self.threshold);

        out_img.save_8bit(&self.output);
    }
}