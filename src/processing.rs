use crate::{
    ser,
    constants,
    path,
    vprintln,
    mean,
    solar,
    lunar,
    timestamp,
    ok,
    fpmap,
    parallacticangle,
    enums::Target
};

use sciimg::{
    error,
    enums::ImageMode,
    rgbimage,
    quality
};

use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use std::cmp::Ordering;

const UNKNOWN_ROTATION:f64 = -99999.0;

#[derive(Debug, Clone)]
struct FrameRecord {
    source_file:String,
    frame_id:usize,
    quality_value:f32
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

impl Eq for FrameRecord {
    
}


pub struct HaProcessing {
    pub flat_field:rgbimage::RgbImage,
    pub dark_field:rgbimage::RgbImage,
    pub dark_flat_field:rgbimage::RgbImage,
    pub mask:rgbimage::RgbImage,
    pub width:usize,
    pub height:usize,
    pub buffer:rgbimage::RgbImage,
    pub frame_count:u32,
    pub obj_detect_threshold:f32,
    pub red_scalar:f32,
    pub green_scalar:f32,
    pub blue_scalar:f32,
    pub obs_latitude:f32,
    pub obs_longitude:f32,
    pub min_sigma:f32,
    pub target:Target,

    // Glitch frames tend to score a very high (outlier) sigma on the quality std-dev test. By specifying
    // a maximum sigma, we can exclude those frames that would otherwise be included in the
    // top n% of frames being stacked.
    pub max_sigma:f32,

    // This is a percentage (0 - 100) of the max possible value (65535 for unsigned 16 bit) that the maximum
    // data values will scaled to. This is to prevent undesirable pixel saturation when sharpening in 
    // applications such as RegiStax or ImPPG.
    pub pct_of_max:f32,

    pub number_of_frames: usize,
    pub file_map: fpmap::FpMap
}

impl HaProcessing {

    fn is_ser_file(ser_file_path:&str) -> bool {
        match path::get_extension(ser_file_path) {
            Some("ser") | Some("SER") => true,
            _ => false
        }
    }

    pub fn create_mean_from_ser(ser_file_path:&str) -> error::Result<rgbimage::RgbImage> {
        if ! HaProcessing::is_ser_file(ser_file_path) {
            Err("Not a SER file")
        } else {
            let input_files:Vec<&str> =vec![ser_file_path];
            let mean_stack = mean::compute_mean(&input_files, true).expect("Failed to calculate mean");
            Ok(mean_stack)
        }
    }

    pub fn init_new(flat_path:&str, 
                    dark_path:&str, 
                    dark_flat_path:&str,
                    mask_file:&str,
                    crop_width:usize, 
                    crop_height:usize, 
                    obj_detect_threshold:f32, 
                    red_scalar:f32, 
                    green_scalar:f32, 
                    blue_scalar:f32,
                    obs_latitude:f32,
                    obs_longitude:f32,
                    min_sigma:f32,
                    max_sigma:f32,
                    pct_of_max:f32,
                    number_of_frames:usize,
                    target:Target) -> error::Result<HaProcessing> {
        let flat = match flat_path.len() {
            0 => rgbimage::RgbImage::new_empty().unwrap(),
            _ => {
                if ! path::file_exists(flat_path) {
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
                if ! path::file_exists(dark_path) {
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
                if ! path::file_exists(dark_flat_path) {
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
                if ! path::file_exists(mask_file) {
                    panic!("File not found: {}", mask_file);
                }
                rgbimage::RgbImage::open_str(mask_file).unwrap()
            }
        };

        Ok(
            HaProcessing {
                flat_field:flat,
                dark_field:dark,
                dark_flat_field:darkflat,
                mask:mask,
                width:crop_width,
                height:crop_height,
                buffer:rgbimage::RgbImage::new_with_bands(crop_width, crop_height, 3, ImageMode::U8BIT).unwrap(),
                frame_count:0,
                obj_detect_threshold:obj_detect_threshold,
                red_scalar:red_scalar,
                green_scalar:green_scalar,
                blue_scalar:blue_scalar,
                obs_latitude:obs_latitude,
                obs_longitude:obs_longitude,
                min_sigma:min_sigma,
                max_sigma:max_sigma,
                pct_of_max:pct_of_max,
                number_of_frames:number_of_frames,
                target:target,
                file_map:fpmap::FpMap::new()
            }
        )
    }


    pub fn get_rotation_for_time(&self, ts:&timestamp::TimeStamp) -> (f64, f64, f64) {

        let (alt, az) = match self.target {
            Target::Moon => {
                vprintln!("Calculating position for Moon");
                lunar::position_from_lat_lon_and_time(self.obs_latitude as f64, self.obs_longitude as f64, &ts)
            },
            Target::Sun => {
                vprintln!("Calculating position for Sun");
                solar::position_from_lat_lon_and_time(self.obs_latitude as f64, self.obs_longitude as f64, &ts)
            }
        };

        let rotation = parallacticangle::from_lat_azimuth_altitude(self.obs_latitude as f64, az, alt);

        (rotation, alt, az)
    }

    pub fn process_frame(&self, buffer:&mut rgbimage::RgbImage, ts:&timestamp::TimeStamp, initial_rotation:f64, enable_rotation:bool) {

        buffer.calibrate(&self.flat_field, &self.dark_field, &self.dark_flat_field);

        let com = buffer.calc_center_of_mass_offset(self.obj_detect_threshold, 0);
        buffer.shift(com.h, com.v);
        
        if self.width > 0 && self.height > 0 {

            let x = (buffer.width - self.width) / 2;
            let y = (buffer.height - self.height) / 2;

            buffer.crop(x, y, self.width, self.height);
        }

        if enable_rotation {
            let (rotation, alt, az) = self.get_rotation_for_time(&ts);
            
            let start_rot = if initial_rotation == UNKNOWN_ROTATION { rotation } else { initial_rotation };

            let do_rotation = initial_rotation - rotation;

            vprintln!("Rotation for frame is {} for az/alt {},{} at time {:?}", rotation, az, alt, ts);
            vprintln!("Initial rotation was {}, effective rotation is {}", start_rot, do_rotation);

            buffer.rotate(do_rotation.to_radians() as f32);
        }
    }

    pub fn finalize(&mut self, out_path:&str) -> error::Result<&str> {

        if self.frame_count > 0 {
            for band in 0..self.buffer.num_bands() {
                self.buffer.apply_weight_on_band(1.0 / self.frame_count as f32, band);
            }

            let (stackmin, stackmax) = self.buffer.get_min_max_all_channel();
            vprintln!("    Stack Min/Max : {}, {} ({} images)", stackmin, stackmax, self.frame_count);

            if ! self.mask.is_empty() {
                self.buffer.apply_mask(&self.mask.get_band(0));
            }

            let mut rgb = match self.buffer.num_bands() {
                1 => {
                    rgbimage::RgbImage::new_from_buffers_rgb(&self.buffer.get_band(0), &self.buffer.get_band(0), &self.buffer.get_band(0), self.buffer.get_mode()).unwrap()
                },
                3 => self.buffer.clone(),
                _ => panic!("Unsupported number of bands")
            };
            rgb.apply_weight_on_band(self.red_scalar, 0);
            rgb.apply_weight_on_band(self.green_scalar, 1);
            rgb.apply_weight_on_band(self.blue_scalar, 2);

            
            if rgb.get_mode() == ImageMode::U8BIT {
                let (_, maxval) = rgb.get_min_max_all_channel();
                rgb.normalize_to_16bit_with_max(maxval / (self.pct_of_max / 100.0));
            }

            rgb.save(out_path);

            ok!()
        } else {
            Err("No frames processed, not saving an empty buffer")
        }

    }


    fn get_rotation_of_single_frame(&self, frame_records:&Vec<FrameRecord>) -> f64 {
        let frame_record = &frame_records[0];
        let ser_file = ser::SerFile::load_ser(frame_record.source_file.as_str()).expect("Unable to load SER file");
        let frame_buffer = ser_file.get_frame(frame_record.frame_id).unwrap();
        let (rotation, _alt, _az) = self.get_rotation_for_time(&frame_buffer.timestamp);
        rotation
    }

    fn process_frame_records(&mut self, frame_records:&Vec<FrameRecord>, enable_rotation:bool) {

        let mut self_buffer = self.buffer.clone();
        let buffer_mtx = Arc::new(Mutex::new(&mut self_buffer));

        // We'll ignore this if we aren't doing rotation
        let initial_rotation = self.get_rotation_of_single_frame(&frame_records);

        frame_records.par_iter().for_each(|fr| {

            // Using get_dont_open so we can keep it as unmutable. Should we want to go to lazy loading of
            // the ser files (though they'd already be loaded in the quality estimation stage), we'd
            // have to find a way to use just get()
            match self.file_map.get_dont_open(&fr.source_file) {
                None => panic!("SER file does not exist in file map. Not good, Kevin. Not good."),
                Some(ser_file) => {
                    
                    let mut frame_buffer = ser_file.get_frame(fr.frame_id).unwrap();
                    self.process_frame(&mut frame_buffer.buffer, &frame_buffer.timestamp, initial_rotation, enable_rotation);
                    
                    //self.buffer.add(&frame_buffer.buffer);
                    // This is a bottleneck to parallelization. 
                    buffer_mtx.lock().unwrap().add(&frame_buffer.buffer);
                }
            };

        });

        self.buffer = self_buffer.clone();
        self.frame_count += frame_records.len() as u32;
    }

    fn determine_quality_in_ser(&self, ser_file:&ser::SerFile) -> Vec<FrameRecord>{
        let mut frame_records: Vec<FrameRecord> = vec!();

        let frame_records_mtx = Arc::new(Mutex::new(&mut frame_records));

        let frame_count = if ser_file.frame_count > self.number_of_frames { self.number_of_frames } else { ser_file.frame_count };

        (0..frame_count).into_par_iter().for_each(|i| {
            let frame_buffer = ser_file.get_frame(i).unwrap();
            let qual = quality::get_quality_estimation(&frame_buffer.buffer);
            vprintln!("Quality value of frame {} is {}", ser_file.source_file, qual);
            if qual >= self.min_sigma && qual <= self.max_sigma {
                let fr = FrameRecord{
                    source_file:ser_file.source_file.to_string(),
                    frame_id:i,
                    quality_value:qual
                };
                frame_records_mtx.lock().unwrap().push(fr);
            } else {
                vprintln!("Frame #{} in file {} falls out of sigma range ({}) and will be excluded", i, ser_file.source_file, qual);
            }
        });

        frame_records
    }

    fn determine_quality_across_sers(&self) -> Vec<FrameRecord>{
        let mut frame_records: Vec<FrameRecord> = vec!();

        let frame_records_mtx = Arc::new(Mutex::new(&mut frame_records));

        self.file_map.get_map().par_iter().for_each(|item| {
            let (_pth, sf) = item;
            let mut list = self.determine_quality_in_ser(sf);
            frame_records_mtx.lock().unwrap().append(&mut list);
        });

        frame_records.sort(); // Sorts in ascending order
        frame_records.reverse();
        frame_records
    }


    pub fn init_ser_file_map(&mut self, ser_files:&Vec<&str>) {

        for sf in ser_files.iter() {
            if ! path::file_exists(sf) {
                panic!("File not found: {}", sf);
            }

            self.file_map.open(&sf.to_string()).unwrap();
        }

    }

    pub fn process_ser_files(&mut self, ser_files:&Vec<&str>, limit_top_pct:u8, enable_rotation:bool) {

        if limit_top_pct > 100 {
            panic!("Invalid percentage: Exceeds 100%: {}", limit_top_pct);
        }

        // We actually have the option to do lazy loading here. That'd be fine, but for now we'll open them up
        // beforehard. 
        self.init_ser_file_map(ser_files);

        let frame_records: Vec<FrameRecord> = self.determine_quality_across_sers();

        let max_frame = ((limit_top_pct as f32 / 100.0) * frame_records.len() as f32).round() as usize;

        let limited_frame_records: Vec<FrameRecord> = frame_records[0..max_frame].to_vec();

        self.process_frame_records(&limited_frame_records, enable_rotation);

        vprintln!("Total frames considered: {}", frame_records.len());
        vprintln!("Limited to top {}% of frames", limit_top_pct);
        vprintln!("Processed with {} frames", limited_frame_records.len());
    }

}

