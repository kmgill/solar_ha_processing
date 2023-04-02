use crate::subs::runnable::RunnableSubcommand;

use solhat::{medianblur, path, vprintln};

use sciimg::image::Image;

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Apply median blur", long_about = None)]
pub struct Median {
    #[clap(long, short, help = "Input image")]
    input: String,

    #[clap(long, short, help = "Blur radius")]
    radius: usize,

    #[clap(long, short, help = "Output image")]
    output: String,
}

impl RunnableSubcommand for Median {
    fn run(&self) {
        if !path::parent_exists_and_writable(self.output.as_str()) {
            eprintln!(
                "Error: Output parent directory does not exist or is unwritable: {}",
                path::get_parent(self.output.as_str())
            );
            process::exit(2);
        }

        vprintln!("Opening input file: {}", self.input);
        let mut img = Image::open(&self.input).unwrap();

        vprintln!("Computing median blur of radius {} for band 0", self.radius);
        let r = medianblur::median_blur(img.get_band(0), self.radius);

        vprintln!("Computing median blur of radius {} for band 1", self.radius);
        let g = medianblur::median_blur(img.get_band(1), self.radius);

        vprintln!("Computing median blur of radius {} for band 2", self.radius);
        let b = medianblur::median_blur(img.get_band(2), self.radius);

        img.set_band(&r, 0);
        img.set_band(&g, 1);
        img.set_band(&b, 2);

        img.save(&self.output);
    }
}
