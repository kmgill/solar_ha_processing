
use crate::subs::runnable::RunnableSubcommand;

use solar_ha_processing::{
    path,
    ldcorrect
};

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Limb Darkening Correction", long_about = None)]
pub struct LdCorrect {
    #[clap(long, short,  help = "Input image")]
    input_file: String,

    #[clap(long, short,  help = "Solar radius in pixels")]
    radius_pixels: usize,

    #[clap(long, short, help = "Output image")]
    output: String,
}   


impl RunnableSubcommand for LdCorrect {
    fn run(&self) {
        if ! path::file_exists(&self.input_file) {
            eprintln!("ERROR: File not found: {}", &self.input_file);
            process::exit(1);
        }

        if ! path::parent_exists_and_writable(&self.output) {
            eprintln!("ERROR: Output directory not found or is not writable");
            process::exit(2);
        }

        ldcorrect::limb_darkening_correction(&self.input_file, &self.output, self.radius_pixels);
    }
}