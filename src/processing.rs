use crate::{
    drizzle::{self, BilinearDrizzle},
    enums::Target,
    fpmap, lunar, mean, parallacticangle, ser, solar, timestamp,
};

use anyhow::{anyhow, Result};
use rayon::prelude::*;
use sciimg::imagebuffer::Offset;
use sciimg::{image, imagerot, max, min, path, quality};
use std::cmp::Ordering;
use std::fmt;

const UNKNOWN_ROTATION: f64 = -99999.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStep {
    QualityEstimation,
    Calibration,
    Finalize,
}

#[derive(Debug, Default, Clone)]
pub struct ProcessReport {
    pub total_frames: usize,
    pub num_frames_used: usize,
    pub min_sigma: f32,
    pub max_sigma: f32,
    pub num_frames_discarded: usize,
    pub num_frames_discarded_min_sigma: usize,
    pub num_frames_discarded_max_sigma: usize,
    pub num_frames_discarded_top_percentage: usize,
    pub initial_rotation: f32,
    pub quality_values: Vec<f32>,
}

impl ProcessReport {
    fn check_sigma(&mut self, s: f32) {
        self.min_sigma = min!(self.min_sigma, s);
        self.max_sigma = max!(self.max_sigma, s);
    }
    pub fn push_sigma(&mut self, s: f32) {
        self.quality_values.push(s);
        self.check_sigma(s);
    }
    pub fn check_total_discarded(&mut self) {
        self.num_frames_discarded = self.num_frames_discarded_max_sigma
            + self.num_frames_discarded_min_sigma
            + self.num_frames_discarded_top_percentage;
    }
}

impl fmt::Display for ProcessReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut text = format!("Total Frames: {}\n", self.total_frames);
        text += format!("Num Frames Used: {}\n", self.num_frames_used).as_ref();
        text += format!("Num Frames Discarded: {}\n", self.num_frames_discarded).as_ref();
        text += format!(
            "\tDue to low sigma: {}\n",
            self.num_frames_discarded_min_sigma
        )
        .as_ref();
        text += format!(
            "\tDue to high sigma: {}\n",
            self.num_frames_discarded_max_sigma
        )
        .as_ref();

        text += format!(
            "\tDue to top pct limitation: {}\n",
            self.num_frames_discarded_top_percentage
        )
        .as_ref();
        text += format!("Maximum Sigma Encountered: {}\n", self.max_sigma).as_ref();
        text += format!("Minimum Sigma Encountered: {}\n", self.min_sigma).as_ref();
        text += format!("Initial Parallatic Rotation: {}\n", self.initial_rotation).as_ref();
        write!(f, "{}", text)
    }
}

#[derive(Debug, Clone)]
pub struct FrameRecord {
    pub source_file: String,
    pub frame_id: usize,
    pub quality_value: f32,
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

trait CenterOfMass {
    fn calc_center_of_mass_offset_with_rotation(
        &self,
        threshold: f32,
        rotation: f32,
        band: usize,
    ) -> Offset;
}

impl CenterOfMass for image::Image {
    fn calc_center_of_mass_offset_with_rotation(
        &self,
        threshold: f32,
        rotation: f32,
        band: usize,
    ) -> Offset {
        let rotated = imagerot::rotate(self.get_band(band), rotation).unwrap();
        rotated.calc_center_of_mass_offset(threshold)
    }
}

struct ProcessContext {
    pub frame_records: Vec<FrameRecord>,
    pub drizzle_buffer: BilinearDrizzle,
    pub obj_detect_threshold: f32,
    pub obs_latitude: f32,
    pub obs_longitude: f32,
    pub target: Target,
    pub flat_field: Option<image::Image>,
    pub dark_field: Option<image::Image>,
    pub dark_flat_field: Option<image::Image>,
    pub bias_field: Option<image::Image>,
}

pub struct HaProcessing {
    pub flat_field: Option<image::Image>,
    pub dark_field: Option<image::Image>,
    pub dark_flat_field: Option<image::Image>,
    pub bias_field: Option<image::Image>,
    pub mask: image::Image,
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
    pub process_report: ProcessReport,
}

impl HaProcessing {
    pub fn is_ser_file(ser_file_path: &str) -> bool {
        matches!(
            path::get_extension(ser_file_path),
            Some("ser") | Some("SER")
        )
    }

    pub fn create_mean_from_ser(ser_file_path: &str) -> Result<image::Image> {
        if !HaProcessing::is_ser_file(ser_file_path) {
            Err(anyhow!("Not a SER file"))
        } else {
            let input_files: Vec<&str> = vec![ser_file_path];
            let mean_stack =
                mean::compute_mean(&input_files, true).expect("Failed to calculate mean");
            Ok(mean_stack)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn init_new(
        input_files: &[&str],
        flat_path: &str,
        dark_path: &str,
        dark_flat_path: &str,
        bias_path: &str,
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
    ) -> Result<HaProcessing> {
        let flat = match flat_path.len() {
            0 => None,
            _ => {
                if !path::file_exists(flat_path) {
                    panic!("File not found: {}", flat_path);
                }

                if HaProcessing::is_ser_file(flat_path) {
                    Some(HaProcessing::create_mean_from_ser(flat_path).unwrap())
                } else {
                    Some(image::Image::open_str(flat_path).unwrap())
                }
            }
        };

        let dark = match dark_path.len() {
            0 => None,
            _ => {
                if !path::file_exists(dark_path) {
                    panic!("File not found: {}", dark_path);
                }

                if HaProcessing::is_ser_file(dark_path) {
                    Some(HaProcessing::create_mean_from_ser(dark_path).unwrap())
                } else {
                    Some(image::Image::open_str(dark_path).unwrap())
                }
            }
        };

        let darkflat = match dark_flat_path.len() {
            0 => None,
            _ => {
                if !path::file_exists(dark_flat_path) {
                    panic!("File not found: {}", dark_flat_path);
                }

                if HaProcessing::is_ser_file(dark_flat_path) {
                    Some(HaProcessing::create_mean_from_ser(dark_flat_path).unwrap())
                } else {
                    Some(image::Image::open_str(dark_flat_path).unwrap())
                }
            }
        };

        let bias = match bias_path.len() {
            0 => None,
            _ => {
                if !path::file_exists(dark_flat_path) {
                    panic!("File not found: {}", dark_flat_path);
                }

                if HaProcessing::is_ser_file(dark_flat_path) {
                    Some(HaProcessing::create_mean_from_ser(dark_flat_path).unwrap())
                } else {
                    Some(image::Image::open_str(dark_flat_path).unwrap())
                }
            }
        };

        let mask = match mask_file.len() {
            0 => image::Image::new_empty().unwrap(),
            _ => {
                if !path::file_exists(mask_file) {
                    panic!("File not found: {}", mask_file);
                }
                image::Image::open_str(mask_file).unwrap()
            }
        };

        let ser1 = ser::SerFile::load_ser(input_files[0]).unwrap();

        let drizzle_buffer =
            drizzle::BilinearDrizzle::new(ser1.image_width, ser1.image_height, drizzle_scale, 3);

        Ok(HaProcessing {
            flat_field: flat,
            dark_field: dark,
            dark_flat_field: darkflat,
            bias_field: bias,
            mask,
            width: ser1.image_width,
            height: ser1.image_height,
            crop_width,
            crop_height,
            buffer: drizzle_buffer,
            frame_count: 0,
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
            target,
            file_map: fpmap::FpMap::new(),
            drizzle_scale,
            process_report: ProcessReport::default(),
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
                info!("Calculating position for Moon");
                lunar::position_from_lat_lon_and_time(obs_latitude as f64, obs_longitude as f64, ts)
            }
            Target::Sun => {
                info!("Calculating position for Sun");
                solar::position_from_lat_lon_and_time(obs_latitude as f64, obs_longitude as f64, ts)
            }
        };

        let rotation = parallacticangle::from_lat_azimuth_altitude(obs_latitude as f64, az, alt);

        (rotation, alt, az)
    }

    pub fn process_frame(&self, buffer: &mut image::Image) {
        buffer.calibrate2(
            &self.flat_field,
            &self.dark_field,
            &self.dark_flat_field,
            &self.bias_field,
        );
    }

    pub fn finalize(&mut self, out_path: &str) -> Result<()> {
        if self.frame_count > 0 {
            let mut final_buffer = self.buffer.get_finalized().unwrap();

            // for band in 0..self.buffer.num_bands() {
            //     self.buffer.apply_weight_on_band(1.0 / self.frame_count as f32, band);
            // }

            if self.crop_height > 0 && self.crop_width > 0 {
                let crop_width =
                    (self.crop_width as f32 * self.drizzle_scale.value()).round() as usize;
                let crop_height =
                    (self.crop_height as f32 * self.drizzle_scale.value()).round() as usize;
                let x = (final_buffer.width - crop_width) / 2;
                let y = (final_buffer.height - crop_height) / 2;
                final_buffer.crop(x, y, crop_width, crop_height);
            }

            let (stackmin, stackmax) = final_buffer.get_min_max_all_channel();
            info!(
                "    Stack Min/Max : {}, {} ({} images)",
                stackmin, stackmax, self.frame_count
            );

            if !self.mask.is_empty() {
                for i in 0..final_buffer.num_bands() {
                    final_buffer.apply_mask_to_band(self.mask.get_band(0), i)
                }
            }

            final_buffer.apply_weight_on_band(self.red_scalar, 0);
            final_buffer.apply_weight_on_band(self.green_scalar, 1);
            final_buffer.apply_weight_on_band(self.blue_scalar, 2);

            //if final_buffer.get_mode() == ImageMode::U8BIT {
            let (_, maxval) = final_buffer.get_min_max_all_channel();
            final_buffer.normalize_to_16bit_with_max(maxval / (self.pct_of_max / 100.0));
            //}

            info!(
                "Final image size: {}, {}",
                final_buffer.width, final_buffer.height
            );
            final_buffer.save(out_path).expect("Failed to save image");

            Ok(())
        } else {
            Err(anyhow!("No frames processed, not saving an empty buffer"))
        }
    }

    fn get_rotation_of_single_frame(
        frame_records: &[FrameRecord],
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
        frame_records: &[FrameRecord],
        enable_rotation: bool,
        initial_rotation: Option<f64>,
    ) {
        // We'll ignore this if we aren't doing rotation
        let initial_rotation = match initial_rotation {
            Some(r) => r,
            None => HaProcessing::get_rotation_of_single_frame(
                frame_records,
                self.target,
                self.obs_latitude,
                self.obs_longitude,
            ),
        };
        self.process_report.initial_rotation = initial_rotation as f32;

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
                bias_field: self.bias_field.clone(),
            })
            .collect::<Vec<ProcessContext>>();

        let drizzles = contexts
            .into_par_iter()
            .map(|mut context| {
                let mut file_map = fpmap::FpMap::new();

                for frame_record in context.frame_records {
                    match file_map.get(&frame_record.source_file) {
                        None => panic!(
                            "SER file does not exist in file map. Not good, Kevin. Not good."
                        ),
                        Some(ser_file) => {
                            let mut frame_buffer =
                                ser_file.get_frame(frame_record.frame_id).unwrap();

                            frame_buffer.buffer.calibrate2(
                                &context.flat_field,
                                &context.dark_field,
                                &context.dark_flat_field,
                                &context.bias_field,
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
                                info!(
                                    "Rotation for frame is {} for az/alt {},{} at time {:?}",
                                    rotation, az, alt, &frame_buffer.timestamp
                                );
                                info!(
                                    "Initial rotation was {}, effective rotation is {}",
                                    start_rot, do_rotation
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
                                    error!("Error drizzling frame: {}", why);
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
                info!(
                    "Quality value of frame {} is {}",
                    ser_file.source_file, qual
                );

                FrameRecord {
                    source_file: ser_file.source_file.to_string(),
                    frame_id: i,
                    quality_value: qual,
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
                self.determine_quality_in_ser(sf)
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

    pub fn init_ser_file_map(&mut self, ser_files: &[&str]) {
        ser_files.iter().for_each(|sf| {
            if !path::file_exists(sf) {
                panic!("File not found: {}", sf);
            }

            self.file_map.open(&sf.to_string()).unwrap();
        });
    }

    pub fn process_ser_files<F: Fn(ProcessStep, usize), C: Fn(ProcessStep)>(
        &mut self,
        ser_files: &[&str],
        limit_top_pct: u8,
        enable_rotation: bool,
        initial_rotation: Option<f64>,
        _on_frame_checked: F,
        _on_step_completed: C,
    ) {
        if limit_top_pct > 100 {
            panic!("Invalid percentage: Exceeds 100%: {}", limit_top_pct);
        }

        // We actually have the option to do lazy loading here. That'd be fine, but for now we'll open them up
        // beforehard.
        self.init_ser_file_map(ser_files);

        self.file_map.map.iter().for_each(|(_, m)| {
            self.process_report.total_frames += m.frame_count;
        });
        info!(
            "Total frames considered: {}",
            self.process_report.total_frames
        );

        self.process_report.min_sigma = std::f32::MAX;
        self.process_report.max_sigma = std::f32::MIN;
        let frame_records: Vec<FrameRecord> = self
            .determine_quality_across_sers()
            .iter()
            .map(|fr| fr.to_owned())
            .filter(|fr| {
                self.process_report.push_sigma(fr.quality_value);
                if fr.quality_value < self.min_sigma {
                    self.process_report.num_frames_discarded_min_sigma += 1
                } else if fr.quality_value > self.max_sigma {
                    self.process_report.num_frames_discarded_max_sigma += 1
                }
                fr.quality_value >= self.min_sigma && fr.quality_value <= self.max_sigma
            })
            .collect();

        let max_frame =
            ((limit_top_pct as f32 / 100.0) * frame_records.len() as f32).round() as usize;

        let limited_frame_records: Vec<FrameRecord> = frame_records[0..max_frame].to_vec();

        self.process_report.num_frames_discarded_top_percentage =
            frame_records.len() - limited_frame_records.len();
        info!(
            "Number of frames discarded while limiting to top {}%: {}",
            limit_top_pct, self.process_report.num_frames_discarded_top_percentage
        );
        self.process_report.num_frames_used = limited_frame_records.len();

        self.process_report.check_total_discarded();

        self.process_frame_records(&limited_frame_records, enable_rotation, initial_rotation);

        info!("Total frames considered: {}", frame_records.len());
        info!("Limited to top {}% of frames", limit_top_pct);
        info!("Processed with {} frames", limited_frame_records.len());
    }
}
