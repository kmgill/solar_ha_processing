use solhat::print;
mod subs;
use subs::runnable::RunnableSubcommand;
use subs::*;

// use std::ffi::OsString;
// use std::path::PathBuf;

extern crate wild;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "solha")]
#[clap(about = "Solar Imaging Calibration and Processing", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: SolHa,

    #[clap(long, short, help = "Verbose output")]
    verbose: bool,
}

#[derive(Subcommand)]
enum SolHa {
    Add(add::Add),
    Composite(composite::Composite),
    ExtractFrame(extractframe::ExtractFrame),
    Extract(extract::Extract),
    FrameStats(framestats::FrameStats),
    Mean(mean::Mean),
    Process(process::Process),
    SerInfo(serinfo::SerInfo),
    Subtract(subtract::Subtract),
    LdCorrect(ldcorrect::LdCorrect),
    ThreshTest(threshtest::ThreshTest),
    Median(median::Median),
}

fn main() {
    let args = Cli::parse_from(wild::args());

    if args.verbose {
        print::set_verbose(true);
    }

    match args.command {
        SolHa::Add(args) => {
            args.run();
        }
        SolHa::Composite(args) => {
            args.run();
        }
        SolHa::ExtractFrame(args) => {
            args.run();
        }
        SolHa::Extract(args) => {
            args.run();
        }
        SolHa::FrameStats(args) => {
            args.run();
        }
        SolHa::Mean(args) => {
            args.run();
        }
        SolHa::Process(args) => {
            args.run();
        }
        SolHa::SerInfo(args) => {
            args.run();
        }
        SolHa::Subtract(args) => {
            args.run();
        }
        SolHa::LdCorrect(args) => {
            args.run();
        }
        SolHa::ThreshTest(args) => {
            args.run();
        }
        SolHa::Median(args) => {
            args.run();
        }
    };
}
