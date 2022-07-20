
use crate::subs::runnable::RunnableSubcommand;

use solar_ha_processing::{
    path,
    vprintln
};

use sciimg::{
    imagebuffer,
    enums::ImageMode
};

use std::process;


#[derive(clap::Args)]
#[clap(author, version, about = "Subtract image from another", long_about = None)]
pub struct Subtract {
    #[clap(long, short, help = "Input images", multiple_values(true))]
    input_files: Vec<String>,

    #[clap(long, short, help = "Output image")]
    output: String,
}   


impl RunnableSubcommand for Subtract {
    fn run(&self) {

        if ! path::parent_exists_and_writable(&self.output.as_str()) {
            eprintln!("Error: Output parent directory does not exist or is unwritable: {}", path::get_parent(&self.output.as_str()));
            process::exit(2);
        }

        if self.input_files.len() < 2 {
            eprintln!("Error: Two input files are required");
            process::exit(3);
        }
        // Assuming length of array!
        let first = self.input_files[0].clone();
        let second = self.input_files[1].clone();

        vprintln!("Loading input file {}", first);
        if ! path::file_exists(&first) {
            eprintln!("Error: Input file not found: {}", first);
            process::exit(1);
        }
    
        vprintln!("Loading input file {}", second);
        if ! path::file_exists(&second) {
            eprintln!("Error: Input file not found: {}", second);
            process::exit(1);
        }

        let mut first_buff = imagebuffer::ImageBuffer::from_file(&first).expect("Error: failed to load file");
        let mut second_buff = imagebuffer::ImageBuffer::from_file(&second).expect("Error: failed to load file");
    
        second_buff.scale_mut(0.6);
        first_buff.subtract_mut(&second_buff);
        
        vprintln!("Writing output file to {}", self.output);
        first_buff.save(&self.output, ImageMode::U16BIT);

        
    }
}