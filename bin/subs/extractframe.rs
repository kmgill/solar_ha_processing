use crate::subs::runnable::RunnableSubcommand;

use solhat::{path, ser};

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Exract single frame from SER file", long_about = None)]
pub struct ExtractFrame {
    #[clap(long, short, help = "Input ser file")]
    input_file: String,

    #[clap(long, short, help = "Frame number (beginning at 1)")]
    frame: usize,

    #[clap(long, short, help = "Output image")]
    output: String,
}

impl RunnableSubcommand for ExtractFrame {
    fn run(&self) {
        if !path::file_exists(&self.input_file) {
            eprintln!("Error: Specified file not found: {}", self.input_file);
            process::exit(1);
        }

        if !path::parent_exists_and_writable(&self.output.as_str()) {
            eprintln!(
                "Error: Output parent directory does not exist or is unwritable: {}",
                path::get_parent(&self.output.as_str())
            );
            process::exit(2);
        }

        let ser_file = ser::SerFile::load_ser(&self.input_file).expect("Unable to load SER file");
        ser_file.validate();

        if self.frame >= ser_file.frame_count {
            eprintln!(
                "Error: Requested frame {} exceeds available frames {}",
                (self.frame + 1),
                ser_file.frame_count
            );
            process::exit(5);
        }

        let frame = ser_file
            .get_frame(self.frame)
            .expect("Failed extracting frame");

        if !path::parent_exists_and_writable(&self.output) {
            eprintln!("Error: Output file path cannot be found or is unwritable");
            process::exit(3);
        }

        frame.buffer.save(&self.output);
    }
}
