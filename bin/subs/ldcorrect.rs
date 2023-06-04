use crate::subs::runnable::RunnableSubcommand;
use sciimg::path;
use solhat::ldcorrect;
use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Limb Darkening Correction", long_about = None)]
pub struct LdCorrect {
    #[clap(long, short, help = "Input image")]
    input_file: String,

    #[clap(long, short, help = "Solar radius in pixels")]
    radius_pixels: usize,

    #[clap(
        long,
        short,
        help = "Limb darkening coefficient",
        multiple_values(true)
    )]
    ld_coefficient: Option<Vec<f64>>,

    #[clap(long = "margin", short = 'm', help = "Composite margin blur")]
    composite_gradient_margin: Option<f64>,

    #[clap(long, short = 'I', help = "Invert chromosphere luminance")]
    inverted_chromosphere: bool,

    #[clap(long, short, help = "Output image")]
    output: String,
}

impl RunnableSubcommand for LdCorrect {
    fn run(&self) {
        if !path::file_exists(&self.input_file) {
            error!("ERROR: File not found: {}", &self.input_file);
            process::exit(1);
        }

        if !path::parent_exists_and_writable(&self.output) {
            error!("ERROR: Output directory not found or is not writable");
            process::exit(2);
        }

        let ld_coefficient = match &self.ld_coefficient {
            Some(v) => v.clone(),
            None => vec![0.0_f64], // Zero value (0.0) will trigger the function to attempt to calculate it
        };

        let composite_gradient_margin =
            if let Some(composite_gradient_margin) = self.composite_gradient_margin {
                composite_gradient_margin
            } else {
                0.0
            };

        match ldcorrect::limb_darkening_correction(
            &self.input_file,
            &self.output,
            self.radius_pixels,
            &ld_coefficient,
            composite_gradient_margin,
            self.inverted_chromosphere,
        ) {
            Ok(_) => {
                vprintln!("Done")
            }
            Err(why) => eprintln!("Error: {}", why),
        };
    }
}
