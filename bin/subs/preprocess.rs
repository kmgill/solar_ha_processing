use crate::subs::runnable::RunnableSubcommand;
use rayon::prelude::*;
use sciimg::prelude::*;
use sciimg::{path, quality};
use solhat::enums::Target;
use solhat::processing::HaProcessing;
use solhat::{drizzle, processing, ser};
use std::fs;
use std::process;

pb_create!();

const UNKNOWN_ROTATION: f64 = -99999.0;

#[derive(clap::Args)]
#[clap(author, version, about = "Preprocess and extract frames", long_about = None)]
pub struct PreProcess {
    #[clap(long, short, help = "Input images", multiple_values(true))]
    input_files: Vec<String>,

    #[clap(long, short, help = "Output directory")]
    output: Option<String>,

    #[clap(long, short, help = "Quality estimation sorting")]
    quality: bool,

    #[clap(long, short = 's', help = "Minimum sigma value")]
    minsigma: Option<f32>,

    #[clap(long, short = 'S', help = "Maximum sigma value")]
    maxsigma: Option<f32>,

    #[clap(long, short, help = "Flat frame file")]
    flat: Option<String>,

    #[clap(long, short, help = "Dark frame file")]
    dark: Option<String>,

    #[clap(long, short = 'D', help = "Dark Flat frame file")]
    darkflat: Option<String>,

    #[clap(long, short, help = "Bias frame file")]
    bias: Option<String>,

    #[clap(long, short, help = "Observer latitude", allow_hyphen_values(true))]
    latitude: f32,

    #[clap(
        long,
        short = 'L',
        help = "Observer longitude",
        allow_hyphen_values(true)
    )]
    longitude: f32,

    #[clap(long, short, help = "Object detection threshold")]
    threshold: Option<f32>,

    #[clap(long, short, help = "Crop width")]
    width: Option<usize>,

    #[clap(long, short = 'H', help = "Crop height")]
    height: Option<usize>,

    #[clap(
        long,
        short = 'I',
        help = "Force an initial rotation value",
        allow_hyphen_values(true)
    )]
    rotation: Option<f64>,

    #[clap(long, short = 'T', help = "Target (Moon, Sun)")]
    target: Option<String>,

    #[clap(long, short = 'u', help = "Drizze upscale (1.5, 2.0, 3.0")]
    drizzle: Option<String>,

    #[clap(long, short, help = "Number of frames (default=all)")]
    number_of_frames: Option<usize>,
}

impl RunnableSubcommand for PreProcess {
    fn run(&self) {
        pb_set_print!();
        let min_sigma = self.minsigma.unwrap_or(0.0);
        let max_sigma = self.maxsigma.unwrap_or(100000.0);
        let target = match &self.target {
            Some(t) => match Target::from(t) {
                Some(t) => t,
                None => {
                    eprintln!("Error: Unrecognized target value: {}", t);
                    process::exit(1);
                }
            },
            None => Target::Sun,
        };

        let obj_detect_threshold = self.threshold.unwrap_or(40.0);
        let initial_rotation = self.rotation.unwrap_or(0.0);
        let obs_latitude = self.latitude;
        let obs_longitude = self.longitude;

        let crop_width = self.width.unwrap_or(0);
        let crop_height = self.height.unwrap_or(0);

        let drizzle_scale = match &self.drizzle {
            Some(s) => match s.as_str() {
                "1.0" => drizzle::Scale::Scale1_0,
                "1.5" => drizzle::Scale::Scale1_5,
                "2.0" => drizzle::Scale::Scale2_0,
                "3.0" => drizzle::Scale::Scale3_0,
                _ => {
                    error!(
                        "Invalid drizze scale: {}. Valid options: 1.0, 1.5, 2.0, 3.0",
                        s
                    );
                    process::exit(1);
                }
            },
            None => drizzle::Scale::Scale1_0,
        };

        let flat_frame = match &self.flat {
            Some(f) => {
                if !path::file_exists(f) {
                    error!("Error: Flat file not found: {}", f);
                }

                match path::get_extension(f).unwrap().to_uppercase().as_str() {
                    "SER" => Some(processing::HaProcessing::create_mean_from_ser(f).unwrap()),
                    _ => Some(Image::open_str(f).unwrap()),
                }
            }
            None => None,
        };

        let dark_frame = match &self.dark {
            Some(f) => {
                if !path::file_exists(f) {
                    error!("Error: Dark file not found: {}", f);
                }

                match path::get_extension(f).unwrap().to_uppercase().as_str() {
                    "SER" => Some(processing::HaProcessing::create_mean_from_ser(f).unwrap()),
                    _ => Some(Image::open_str(f).unwrap()),
                }
            }
            None => None,
        };

        let dark_flat_frame = match &self.darkflat {
            Some(f) => {
                if !path::file_exists(f) {
                    error!("Error: Dark Flat file not found: {}", f);
                }

                match path::get_extension(f).unwrap().to_uppercase().as_str() {
                    "SER" => Some(processing::HaProcessing::create_mean_from_ser(f).unwrap()),
                    _ => Some(Image::open_str(f).unwrap()),
                }
            }
            None => None,
        };

        let bias_frame = match &self.dark {
            Some(f) => {
                if !path::file_exists(f) {
                    error!("Error: Bias file not found: {}", f);
                }

                match path::get_extension(f).unwrap().to_uppercase().as_str() {
                    "SER" => Some(processing::HaProcessing::create_mean_from_ser(f).unwrap()),
                    _ => Some(Image::open_str(f).unwrap()),
                }
            }
            None => None,
        };

        // let input_files: Vec<&str> = self.input_files.iter().map(|s| s.as_str()).collect();

        self.input_files.iter().for_each(|ser_file_path| {
            if !path::file_exists(ser_file_path) {
                error!("Error: Specified file not found: {}", ser_file_path);
                process::exit(2);
            }

            let mut output_directory = match &self.output {
                Some(o) => o.clone(),
                None => path::get_parent(ser_file_path),
            };

            if self.input_files.len() > 1 {
                let bn = path::basename(ser_file_path);
                let out_file_base = bn.replace(".ser", "").replace(".SER", "");
                output_directory = format!("{}/{}", &output_directory, &out_file_base);
                if !path::file_exists(output_directory.as_str()) {
                    let err = format!("Failed to create output directory {}", &output_directory);
                    fs::create_dir(&output_directory).unwrap_or_else(|_| panic!("{}", err));
                }
            }

            let ser_file = ser::SerFile::load_ser(ser_file_path).expect("Unable to load SER file");
            ser_file.validate();

            let num_frames = if let Some(nf) = self.number_of_frames {
                if nf <= ser_file.frame_count {
                    nf
                } else {
                    ser_file.frame_count
                }
            } else {
                ser_file.frame_count
            };

            pb_set_length!(num_frames);
            pb_zero!();
            (0..num_frames).into_par_iter().for_each(|i| {
                let mut frame = ser_file.get_frame(i).expect("Failed extracting frame");

                frame
                    .buffer
                    .calibrate2(&flat_frame, &dark_frame, &dark_flat_frame, &bias_frame);

                let sd = quality::get_quality_estimation(&frame.buffer);
                if sd.is_nan() {
                    warn!("Frame quality is NaN!");
                    process::exit(2);
                }
                info!("Quality of frame measured as {}", sd);
                if sd < min_sigma || sd > max_sigma {
                    warn!("Frame #{} is outside of sigma range ({})", i, sd);
                    return;
                }

                let offset = frame
                    .buffer
                    .calc_center_of_mass_offset(obj_detect_threshold, 0);

                ///////////// Rotation:
                let (rotation, alt, az) = HaProcessing::get_rotation_for_time(
                    &frame.timestamp,
                    target,
                    obs_latitude,
                    obs_longitude,
                );
                let start_rot = if initial_rotation == UNKNOWN_ROTATION {
                    rotation
                } else {
                    initial_rotation
                };
                let mut do_rotation = initial_rotation - rotation;
                info!(
                    "Rotation for frame is {} for az/alt {},{} at time {:?}",
                    rotation, az, alt, &frame.timestamp
                );
                info!(
                    "Initial rotation was {}, effective rotation is {}",
                    start_rot, do_rotation
                );
                do_rotation = do_rotation.to_radians();

                let mut drizzle_buffer = drizzle::BilinearDrizzle::new(
                    ser_file.image_width,
                    ser_file.image_height,
                    drizzle_scale,
                    3,
                );

                match drizzle_buffer.add_with_transform(&frame.buffer, offset, do_rotation) {
                    Ok(_) => {}
                    Err(why) => {
                        error!("Error drizzling frame: {}", why);
                    }
                }

                let mut calibrated_buffer = drizzle_buffer
                    .get_finalized()
                    .expect("Failed to finalize drizzle buffer"); // Even though we're not actually drizzling anything

                if crop_height > 0 && crop_width > 0 {
                    let x = (ser_file.image_width - crop_width) / 2;
                    let y = (ser_file.image_width - crop_height) / 2;
                    calibrated_buffer.crop(x, y, crop_width, crop_height);
                }

                let new_extension = match self.quality {
                    true => {
                        format!("_{}_{:0width$}.png", (sd * 10000.0) as u32, i, width = 5)
                    }
                    false => format!("_{:0width$}.png", i, width = 5),
                };

                let new_output_parent =
                    format!("{}/{}", output_directory, path::basename(ser_file_path));
                let frame_output_path = new_output_parent
                    .replace(".ser", &new_extension)
                    .replace(".SER", &new_extension);

                info!("Frame #{} Output: {}", i, frame_output_path);

                if !path::parent_exists_and_writable(&frame_output_path) {
                    error!("Error: Output file path cannot be found or is unwritable");
                    process::exit(3);
                }

                calibrated_buffer
                    .save(&frame_output_path)
                    .expect("Failed to save image");

                pb_inc!();
            });
        });
        pb_done!();
    }
}
