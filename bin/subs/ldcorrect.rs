
use crate::subs::runnable::RunnableSubcommand;

use solar_ha_processing::{
    path,
    ldcorrect,
    vprintln
};

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Limb Darkening Correction", long_about = None)]
pub struct LdCorrect {
    #[clap(long, short,  help = "Input image")]
    input_file: String,

    #[clap(long, short,  help = "Solar radius in pixels")]
    radius_pixels: usize,

    #[clap(long, short,  help = "Limb darkening coefficient")]
    ld_coefficient: Option<f64>,

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

        
        let ld_coefficient = match self.ld_coefficient {
            Some(v) => v,
            None => 0.0 // Zero value (0.0) will trigger the function to attempt to calculate it
        };

        match ldcorrect::limb_darkening_correction(&self.input_file, &self.output, self.radius_pixels, ld_coefficient) {
            Ok(_) => {
                vprintln!("Done")
            }, 
            Err(why) => eprintln!("Error: {}", why)
        };
    }
}