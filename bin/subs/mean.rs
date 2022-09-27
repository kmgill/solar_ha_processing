
use crate::subs::runnable::RunnableSubcommand;

use solhat::{
    path,
    vprintln,
    mean
};

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Compute mean of images", long_about = None)]
pub struct Mean {
    #[clap(long, short,  help = "Input ser file", multiple_values(false))]
    input_file: String,

    #[clap(long, short, help = "Output image")]
    output: String,
}   


impl RunnableSubcommand for Mean {
    fn run(&self) {
        if ! path::parent_exists_and_writable(&self.output.as_str()) {
            eprintln!("Error: Output parent directory does not exist or is unwritable: {}", path::get_parent(&self.output.as_str()));
            process::exit(2);
        }

        let input_files = vec![self.input_file.as_str()];

        let mean_stack = mean::compute_mean(&input_files, true).expect("Failed to calculate mean");

        vprintln!("Saving stack buffer to {}", self.output);
        mean_stack.save(&self.output);
    }
}