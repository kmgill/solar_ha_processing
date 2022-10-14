use crate::{
    constants,
    drizzle::{self, BilinearDrizzle},
    enums::Target,
    fpmap, lunar, mean, ok, parallacticangle, path, ser, solar, timestamp, vprintln,
};

use rayon::prelude::*;
use sciimg::{error, quality, rgbimage};
use std::cmp::Ordering;

const UNKNOWN_ROTATION: f64 = -99999.0;

#[derive(Debug, Clone)]
pub struct FrameRecord {
    pub source_file: String,
    pub frame_id: usize,
    pub quality_value: f32,
    pub use_frame: bool,
}

impl Ord for FrameRecord {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.quality_value < other.quality_value {
            Ordering::Less
        } else if self.quality_value == other.quality_value {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}

impl PartialOrd for FrameRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FrameRecord {
    fn eq(&self, other: &Self) -> bool {
        self.quality_value == other.quality_value
    }
}

impl Eq for FrameRecord {}

struct ProcessContext {
    pub frame_records: Vec<FrameRecord>,
    pub drizzle_buffer: BilinearDrizzle,
    pub obj_detect_threshold: f32,
    pub obs_latitude: f32,
    pub obs_longitude: f32,
    pub target: Target,
    pub flat_field: rgbimage::RgbImage,
    pub dark_field: rgbimage::RgbImage,
    pub dark_flat_field: rgbimage::RgbImage,
}

pub struct HaProcessing {
    pub flat_field: rgbimage::RgbImage,
    pub dark_field: rgbimage::RgbImage,
    pub dark_flat_field: rgbimage::RgbImage,
    pub mask: rgbimage::RgbImage,
    pub width: usize,
    pub height: usize,
    pub crop_width: usize,
    pub crop_height: usize,
    pub buffer: drizzle::BilinearDrizzle,
    pub frame_count: u32,
    pub obj_detect_threshold: f32,
    pub red_scalar: f32,
    pub green_scalar: f32,
    pub blue_scalar: f32,
    pub obs_latitude: f32,
    pub obs_longitude: f32,
    pub min_sigma: f32,
    pub target: Target,

    // Glitch frames tend to score a very high (outlier) sigma on the quality std-dev test. By specifying
    // a maximum sigma, we can exclude those frames that would otherwise be included in the
    // top n% of frames being stacked.
    pub max_sigma: f32,

    // This is a percentage (0 - 100) of the max possible value (65535 for unsigned 16 bit) that the maximum
    // data values will scaled to. This is to prevent undesirable pixel saturation when sharpening in
    // applications such as RegiStax or ImPPG.
    pub pct_of_max: f32,

    pub number_of_frames: usize,
    pub file_map: fpmap::FpMap,
    pub drizzle_scale: drizzle::Scale,
}

impl HaProcessing {
    pub fn is_ser_file(ser_file_path: &str) -> bool {
        match path::get_extension(ser_file_path) {
            Some("ser") | Some("SER") => true,
            _ => false,
        }
    }

    pub fn create_mean_from_ser(ser_file_path: &str) -> error::Result<rgbimage::RgbImage> {
        if !HaProcessing::is_ser_file(ser_file_path) {
            Err("Not a SER file")
        } else {
            let input_files: Vec<&str> = vec![ser_file_path];
            let mean_stack =
                mean::compute_mean(&input_files, true).expect("Failed to calculate mean");
            Ok(mean_stack)
        }
    }

    pub fn init_new(
        input_files: &Vec<&str>,
        flat_path: &str,
        dark_path: &str,
        dark_flat_path: &str,
        mask_file: &str,
        crop_width: usize,
        crop_height: usize,
        obj_detect_threshold: f32,
        red_scalar: f32,
        green_scalar: f32,
        blue_scalar: f32,
        obs_latitude: f32,
        obs_longitude: f32,
        min_sigma: f32,
        max_sigma: f32,
        pct_of_max: f32,
        number_of_frames: usize,
        target: Target,
        drizzle_scale: drizzle::Scale,
    ) -> error::Result<HaProcessing> {
        let flat = match flat_path.len() {
            0 => rgbimage::RgbImage::new_empty().unwrap(),
            _ => {
                if !path::file_exists(flat_path) {
                    panic!("File not found: {}", flat_path);
                }

                if HaProcessing::is_ser_file(flat_path) {
                    HaProcessing::create_mean_from_ser(flat_path).unwrap()
                } else {
                    rgbimage::RgbImage::open_str(flat_path).unwrap()
                }
            }
        };

        let dark = match dark_path.len() {
            0 => rgbimage::RgbImage::new_empty().unwrap(),
            _ => {
                if !path::file_exists(dark_path) {
                    panic!("File not found: {}", dark_path);
                }

                if HaProcessing::is_ser_file(dark_path) {
                    HaProcessing::create_mean_from_ser(dark_path).unwrap()
                } else {
                    rgbimage::RgbImage::open_str(dark_path).unwrap()
                }
            }
        };

        let darkflat = match dark_flat_path.len() {
            0 => rgbimage::RgbImage::new_empty().unwrap(),
            _ => {
                if !path::file_exists(dark_flat_path) {
                    panic!("File not found: {}", dark_flat_path);
                }

                if HaProcessing::is_ser_file(dark_flat_path) {
                    HaProcessing::create_mean_from_ser(dark_flat_path).unwrap()
                } else {
                    rgbimage::RgbImage::open_str(dark_flat_path).unwrap()
                }
            }
        };

        let mask = match mask_file.len() {
            0 => rgbimage::RgbImage::new_empty().unwrap(),
            _ => {
                if !path::file_exists(mask_file) {
                    panic!("File not found: {}", mask_file);
                }
                rgbimage::RgbImage::open_str(mask_file).unwrap()
            }
        };

        let ser1 = ser::SerFile::load_ser(input_files[0]).unwrap();

        let drizzle_buffer =
            drizzle::BilinearDrizzle::new(ser1.image_width, ser1.image_height, drizzle_scale, 3);

        Ok(HaProcessing {
            flat_field: flat,
            dark_field: dark,
            dark_flat_field: darkflat,
            mask: mask,
            width: ser1.image_width,
            height: ser1.image_height,
            crop_width: crop_width,
            crop_height: crop_height,
            buffer: drizzle_buffer,
            frame_count: 0,
            obj_detect_threshold: obj_detect_threshold,
            red_scalar: red_scalar,
            green_scalar: green_scalar,
            blue_scalar: blue_scalar,
            obs_latitude: obs_latitude,
            obs_longitude: obs_longitude,
            min_sigma: min_sigma,
            max_sigma: max_sigma,
            pct_of_max: pct_of_max,
            number_of_frames: number_of_frames,
            target: target,
            file_map: fpmap::FpMap::new(),
            drizzle_scale: drizzle_scale,
        })
    }

    pub fn get_rotation_for_time(
        ts: &timestamp::TimeStamp,
        target: Target,
        obs_latitude: f32,
        obs_longitude: f32,
    ) -> (f64, f64, f64) {
        let (alt, az) = match target {
            Target::Moon => {
                vprintln!("Calculating position for Moon");
                lunar::position_from_lat_lon_and_time(
                    obs_latitude as f64,
                    obs_longitude as f64,
                    &ts,
                )
            }
            Target::Sun => {
                vprintln!("Calculating position for Sun");
                solar::position_from_lat_lon_and_time(
                    obs_latitude as f64,
                    obs_longitude as f64,
                    &ts,
                )
            }
        };

        let rotation = parallacticangle::from_lat_azimuth_altitude(obs_latitude as f64, az, alt);

        (rotation, alt, az)
    }

    pub fn process_frame(&self, buffer: &mut rgbimage::RgbImage) {
        buffer.calibrate(&self.flat_field, &self.dark_field, &self.dark_flat_field);
    }

    pub fn finalize(&mut self, out_path: &str) -> error::Result<&str> {
        if self.frame_count > 0 {
            let mut final_buffer = self.buffer.get_finalized().unwrap();

            // for band in 0..self.buffer.num_bands() {
            //     self.buffer.apply_weight_on_band(1.0 / self.frame_count as f32, band);
            // }

            let crop_width = (self.crop_width as f32 * self.drizzle_scale.value()).round() as usize;
            let crop_height =
                (self.crop_height as f32 * self.drizzle_scale.value()).round() as usize;
            let x = (final_buffer.width - crop_width) / 2;
            let y = (final_buffer.height - crop_height) / 2;
            final_buffer.crop(x, y, crop_width, crop_height);

            let (stackmin, stackmax) = final_buffer.get_min_max_all_channel();
            vprintln!(
                "    Stack Min/Max : {}, {} ({} images)",
                stackmin,
                stackmax,
                self.frame_count
            );

            if !self.mask.is_empty() {
                for i in 0..final_buffer.num_bands() {
                    final_buffer.apply_mask_to_band(&self.mask.get_band(0), i)
                }
            }

            final_buffer.apply_weight_on_band(self.red_scalar, 0);
            final_buffer.apply_weight_on_band(self.green_scalar, 1);
            final_buffer.apply_weight_on_band(self.blue_scalar, 2);

            //if final_buffer.get_mode() == ImageMode::U8BIT {
            let (_, maxval) = final_buffer.get_min_max_all_channel();
            final_buffer.normalize_to_16bit_with_max(maxval / (self.pct_of_max / 100.0));
            //}

            vprintln!(
                "Final image size: {}, {}",
                final_buffer.width,
                final_buffer.height
            );
            final_buffer.save(out_path);

            ok!()
        } else {
            Err("No frames processed, not saving an empty buffer")
        }
    }

    fn get_rotation_of_single_frame(
        frame_records: &Vec<FrameRecord>,
        target: Target,
        obs_latitude: f32,
        obs_longitude: f32,
    ) -> f64 {
        let frame_record = &frame_records[0];
        let ser_file = ser::SerFile::load_ser(frame_record.source_file.as_str())
            .expect("Unable to load SER file");
        let frame_buffer = ser_file.get_frame(frame_record.frame_id).unwrap();
        let (rotation, _alt, _az) = HaProcessing::get_rotation_for_time(
            &frame_buffer.timestamp,
            target,
            obs_latitude,
            obs_longitude,
        );
        rotation
    }

    fn process_frame_records(
        &mut self,
        frame_records: &Vec<FrameRecord>,
        enable_rotation: bool,
        initial_rotation: Option<f64>,
    ) {
        // We'll ignore this if we aren't doing rotation
        let initial_rotation = match initial_rotation {
            Some(r) => r,
            None => HaProcessing::get_rotation_of_single_frame(
                &frame_records,
                self.target,
                self.obs_latitude,
                self.obs_longitude,
            ),
        };

        let num_per_chunk = frame_records.len() / num_cpus::get();

        let contexts = frame_records
            .chunks(num_per_chunk)
            .map(|fr| ProcessContext {
                frame_records: fr.to_vec(),
                drizzle_buffer: self.buffer.clone(),
                obj_detect_threshold: self.obj_detect_threshold,
                obs_latitude: self.obs_latitude,
                obs_longitude: self.obs_longitude,
                target: self.target,
                flat_field: self.flat_field.clone(),
                dark_field: self.dark_field.clone(),
                dark_flat_field: self.dark_flat_field.clone(),
            })
            .collect::<Vec<ProcessContext>>();

        let drizzles = contexts
            .into_par_iter()
            .map(|mut context| {
                let mut file_map = fpmap::FpMap::new();

                for frame_record in context.frame_records {
                    if !frame_record.use_frame {
                        continue;
                    }

                    match file_map.get(&frame_record.source_file) {
                        None => panic!(
                            "SER file does not exist in file map. Not good, Kevin. Not good."
                        ),
                        Some(ser_file) => {
                            let mut frame_buffer =
                                ser_file.get_frame(frame_record.frame_id).unwrap();

                            frame_buffer.buffer.calibrate(
                                &context.flat_field,
                                &context.dark_field,
                                &context.dark_flat_field,
                            );

                            let offset = frame_buffer
                                .buffer
                                .calc_center_of_mass_offset(context.obj_detect_threshold, 0);

                            let rotation = if enable_rotation {
                                let (rotation, alt, az) = HaProcessing::get_rotation_for_time(
                                    &frame_buffer.timestamp,
                                    context.target,
                                    context.obs_latitude,
                                    context.obs_longitude,
                                );
                                let start_rot = if initial_rotation == UNKNOWN_ROTATION {
                                    rotation
                                } else {
                                    initial_rotation
                                };
                                let do_rotation = initial_rotation - rotation;
                                vprintln!(
                                    "Rotation for frame is {} for az/alt {},{} at time {:?}",
                                    rotation,
                                    az,
                                    alt,
                                    &frame_buffer.timestamp
                                );
                                vprintln!(
                                    "Initial rotation was {}, effective rotation is {}",
                                    start_rot,
                                    do_rotation
                                );
                                do_rotation.to_radians()
                            } else {
                                0.0
                            };

                            match context.drizzle_buffer.add_with_transform(
                                &frame_buffer.buffer,
                                offset,
                                rotation,
                            ) {
                                Ok(_) => {}
                                Err(why) => {
                                    eprintln!("Error drizzling frame: {}", why);
                                }
                            }
                        }
                    };
                }

                context.drizzle_buffer
            })
            .collect::<Vec<BilinearDrizzle>>();

        for drizzle_buffer in drizzles {
            self.buffer.add_drizzle(&drizzle_buffer).unwrap();
        }

        self.frame_count += frame_records.len() as u32;
    }

    fn determine_quality_in_ser(&self, ser_file: &ser::SerFile) -> Vec<FrameRecord> {
        let frame_count = if ser_file.frame_count > self.number_of_frames {
            self.number_of_frames
        } else {
            ser_file.frame_count
        };

        (0..frame_count)
            .into_par_iter()
            .map(|i| {
                let frame_buffer = ser_file.get_frame(i).unwrap();
                let qual = quality::get_quality_estimation(&frame_buffer.buffer);
                vprintln!(
                    "Quality value of frame {} is {}",
                    ser_file.source_file,
                    qual
                );
                FrameRecord {
                    source_file: ser_file.source_file.to_string(),
                    frame_id: i,
                    quality_value: qual,
                    use_frame: qual >= self.min_sigma && qual <= self.max_sigma,
                }
            })
            .collect::<Vec<FrameRecord>>()
    }

    fn determine_quality_across_sers(&self) -> Vec<FrameRecord> {
        let mut frame_records: Vec<FrameRecord> = self
            .file_map
            .get_map()
            .par_iter()
            .map(|item| {
                let (_pth, sf) = item;
                self.determine_quality_in_ser(&sf)
            })
            .collect::<Vec<Vec<FrameRecord>>>()
            .iter()
            .flatten()
            .map(|fr| fr.to_owned())
            .collect::<Vec<FrameRecord>>();

        frame_records.sort(); // Sorts in ascending order
        frame_records.reverse();
        frame_records
    }

    pub fn init_ser_file_map(&mut self, ser_files: &Vec<&str>) {
        for sf in ser_files.iter() {
            if !path::file_exists(sf) {
                panic!("File not found: {}", sf);
            }

            self.file_map.open(&sf.to_string()).unwrap();
        }
    }

    pub fn process_ser_files(
        &mut self,
        ser_files: &Vec<&str>,
        limit_top_pct: u8,
        enable_rotation: bool,
        initial_rotation: Option<f64>,
    ) {
        if limit_top_pct > 100 {
            panic!("Invalid percentage: Exceeds 100%: {}", limit_top_pct);
        }

        // We actually have the option to do lazy loading here. That'd be fine, but for now we'll open them up
        // beforehard.
        self.init_ser_file_map(ser_files);

        let frame_records: Vec<FrameRecord> = self.determine_quality_across_sers();

        let max_frame =
            ((limit_top_pct as f32 / 100.0) * frame_records.len() as f32).round() as usize;

        let limited_frame_records: Vec<FrameRecord> = frame_records[0..max_frame].to_vec();

        self.process_frame_records(&limited_frame_records, enable_rotation, initial_rotation);

        vprintln!("Total frames considered: {}", frame_records.len());
        vprintln!("Limited to top {}% of frames", limit_top_pct);
        vprintln!("Processed with {} frames", limited_frame_records.len());
    }
}
