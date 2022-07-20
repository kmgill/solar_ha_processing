
use crate::subs::runnable::RunnableSubcommand;

use solar_ha_processing::{
    path,
    processing,
    enums::Target
};

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Process a full observation", long_about = None)]
pub struct Process {
    #[clap(long, short,  help = "Input ser files", multiple_values(true))]
    input_files: Vec<String>,

    #[clap(long, short, help = "Output image")]
    output: String,

    #[clap(long, short,  help = "Flat frame file")]
    flat: Option<String>,

    #[clap(long, short,  help = "Dark frame file")]
    dark: Option<String>,

    #[clap(long, short='D',  help = "dark Flat frame file")]
    darkflat: Option<String>,

    #[clap(long, short, help = "Crop width")]
    width: Option<usize>,

    #[clap(long, short='H', help = "Crop height")]
    height: Option<usize>,

    #[clap(long, short, help = "Red weight")]
    redweight: Option<f32>,

    #[clap(long, short, help = "Green weight")]
    greenweight: Option<f32>,

    #[clap(long, short, help = "Blue weight")]
    blueweight: Option<f32>,

    #[clap(long, short, help = "Observer latitude", allow_hyphen_values(true))]
    latitude: f32,

    #[clap(long, short='L', help = "Observer longitude", allow_hyphen_values(true))]
    longitude: f32,

    #[clap(long, short, help = "Object detection threshold")]
    threshold: Option<f32>,

    #[clap(long, short, help = "Image mask")]
    mask: Option<String>,

    #[clap(long, short, help = "Quality limit (top % frames)")]
    quality: Option<u8>,

    #[clap(long, short='s', help = "Minimum sigma value")]
    minsigma: Option<f32>,

    #[clap(long, short='S', help = "Maximum sigma value")]
    maxsigma: Option<f32>,

    #[clap(long, short='I', help = "Force an initial rotation value", allow_hyphen_values(true))]
    rotation: Option<f64>,

    #[clap(long, short='P', help = "Scale maximum value to percentage max possible (0-100)")]
    percentofmax: Option<f32>,

    #[clap(long, short, help = "Number of frames (default=all)")]
    number_of_frames: Option<usize>,

    #[clap(long, short='T', help = "Target (Moon, Sun)")]
    target: Option<String>,

    #[clap(long, help = "Disable parallactic rotation")]
    norot: bool,

}   


impl RunnableSubcommand for Process {
    fn run(&self) {

        if ! path::parent_exists_and_writable(&self.output) {
            eprintln!("Error: Output parent directory does not exist or is unwritable: {}", path::get_parent(&self.output));
            process::exit(2);
        }

        let target = match &self.target {
            Some(t) => {
                match Target::from(&t) {
                    Some(t) => t,
                    None => {
                        eprintln!("Error: Unrecognized target value: {}", t);
                        process::exit(1);
                    }
                }
            },
            None => Target::Sun
        };

        let obj_detect_threshold = match self.threshold {
            Some(t) => t,
            None => 40.0
        };

        let crop_width = match self.width {
            Some(w) => w,
            None => 0
        };

        let crop_height = match self.height {
            Some(h) => h,
            None => 0
        };

        if crop_width == 0 && crop_height > 0 || crop_width > 0 && crop_height == 0 {
            eprintln!("Error: Both width and height need to be specified if any are");
            process::exit(1);
        }

        let flat_frame = match &self.flat {
            Some(f) => {
                if ! path::file_exists(&f) {
                    eprintln!("Error: Flat file not found: {}", f);
                }
                f.clone()
            },
            None => String::from("")
        };

        let dark_frame = match &self.dark {
            Some(f) => {
                if ! path::file_exists(&f) {
                    eprintln!("Error: Dark file not found: {}", f);
                }
                f.clone()
            },
            None => String::from("")
        };

        let dark_flat_frame = match &self.darkflat {
            Some(f) => {
                if ! path::file_exists(&f) {
                    eprintln!("Error: Dark flat file not found: {}", f);
                }
                f.clone()
            },
            None => String::from("")
        };

        let mask_file = match &self.mask {
            Some(f) => {
                if ! path::file_exists(&f) {
                    eprintln!("Error: Mask file not found: {}", f);
                }
                f.clone()
            },
            None => String::from("")
        };

        let red_scalar = match self.redweight {
            Some(w) => w,
            None => 1.0
        };

        let green_scalar = match self.greenweight {
            Some(w) => w,
            None => 1.0
        };

        let blue_scalar = match self.blueweight {
            Some(w) => w,
            None => 1.0
        };

        let max_sigma = match self.maxsigma {
            Some(m) => m,
            None => 100000.0
        };

        let min_sigma = match self.minsigma {
            Some(m) => m,
            None => 0.0
        };

        let initial_rotation = self.rotation;
        let obs_latitude = self.latitude;
        let obs_longitude = self.longitude;

        let limit_top_pct = match self.quality {
            Some(p) => {
                if p <= 0 {
                    panic!("Error: Quality limit percentage cannot be zero or below");
                } else if p > 100 {
                    panic!("Error: Quality limit percentage cannot exceed 100%");
                } else {
                    p
                }
            },
            None => 100
        };

        let number_of_frames = match self.number_of_frames {
            Some(n) => n,
            None => 10000000
        };

        let pct_of_max = match self.percentofmax {
            Some(p) => {
                if p <= 0.0 {
                    panic!("Error: Percentage cannot be zero or below");
                } else if p > 100.0 {
                    panic!("Error: Percentage cannot exceed 100%");
                } else {
                    p
                }
            },
            None => 100.0
        };

        let enable_rotation = !self.norot;

        let input_files : Vec<&str> = self.input_files.iter().map(|s| s.as_str()).collect();

        let mut ha_processing = processing::HaProcessing::init_new(&flat_frame, 
                                                                                    &dark_frame, 
                                                                                    &dark_flat_frame,
                                                                                    &mask_file,
                                                                                    crop_width, 
                                                                                    crop_height, 
                                                                                    obj_detect_threshold, 
                                                                                    red_scalar, 
                                                                                    green_scalar, 
                                                                                    blue_scalar,
                                                                                    obs_latitude,
                                                                                    obs_longitude,
                                                                                    min_sigma,
                                                                                    max_sigma,
                                                                                    pct_of_max,
                                                                                    number_of_frames,
                                                                                    target).expect("Failed to create processing context");
        ha_processing.process_ser_files(&input_files, limit_top_pct, enable_rotation, initial_rotation);
        ha_processing.finalize(&self.output).expect("Failed to finalize buffer");
        
    }
}