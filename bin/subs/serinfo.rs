use crate::subs::runnable::RunnableSubcommand;

use solhat::{path, ser};

#[derive(clap::Args)]
#[clap(author, version, about = "SER file details", long_about = None)]
pub struct SerInfo {
    #[clap(long, short, help = "Input ser file", multiple_values(false))]
    input_file: String,
}

impl RunnableSubcommand for SerInfo {
    fn run(&self) {
        if path::file_exists(self.input_file.as_str()) {
            let ser_file =
                ser::SerFile::load_ser(&self.input_file).expect("Unable to load SER file");
            ser_file.validate();

            ser_file.print_header_details();
        }
    }
}
