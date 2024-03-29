use crate::subs::runnable::RunnableSubcommand;
use sciimg::{enums::ImageMode, imagebuffer, path};
use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Add images", long_about = None)]
pub struct Add {
    #[clap(long, short, help = "Input images", multiple_values(true))]
    input_files: Vec<String>,

    #[clap(long, short, help = "Output image")]
    output: String,
}

impl RunnableSubcommand for Add {
    fn run(&self) {
        if !path::parent_exists_and_writable(self.output.as_str()) {
            error!(
                "Error: Output parent directory does not exist or is unwritable: {}",
                path::get_parent(self.output.as_str())
            );
            process::exit(2);
        }

        let mut composite = imagebuffer::ImageBuffer::new_empty().unwrap();

        self.input_files.iter().for_each(|input_file| {
            vprintln!("Loading input file {}", input_file);

            if !path::file_exists(input_file) {
                error!("Error: Input file not found: {}", input_file);
                process::exit(1);
            }

            let input = imagebuffer::ImageBuffer::from_file(input_file)
                .expect("Error: failed to load file");

            if composite.is_empty() {
                composite = input;
            } else {
                composite.add_mut(&input);
            }
        });

        info!("Writing output file to {}", self.output);
        composite.mode = ImageMode::U16BIT;
        composite
            .save(self.output.as_str())
            .expect("Failed to save image");
    }
}
