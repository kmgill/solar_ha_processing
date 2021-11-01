use crate::{
    ser,
    constants,
    path,
    vprintln,
    mean,
    solar,
    imagerot,
    timestamp,
    quality,
    ok
};

use sciimg::{
    imagebuffer,
    error,
    enums::ImageMode,
    rgbimage
};

use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use std::cmp::Ordering;

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
    pub flat_field:imagebuffer::ImageBuffer,
    pub dark_field:imagebuffer::ImageBuffer,
    pub mask:imagebuffer::ImageBuffer,
    pub width:usize,
    pub height:usize,
    pub buffer:imagebuffer::ImageBuffer,
    pub frame_count:u32,
    pub obj_detect_threshold:f32,
    pub red_scalar:f32,
    pub green_scalar:f32,
    pub blue_scalar:f32,
    pub obs_latitude:f32,
    pub obs_longitude:f32,
    pub min_sigma:f32,

    // Glitch frames tend to score a very high (outlier) sigma on the quality std-dev test. By specifying
    // a maximum sigma, we can exclude those frames that would otherwise be included in the
    // top n% of frames being stacked.
    pub max_sigma:f32,

    // This is a percentage (0 - 100) of the max possible value (65535 for unsigned 16 bit) that the maximum
    // data values will scaled to. This is to prevent undesirable pixel saturation when sharpening in 
    // applications such as RegiStax or ImPPG.
    pub pct_of_max:f32
}

impl HaProcessing {

    fn is_ser_file(ser_file_path:&str) -> bool {
        match path::get_extension(ser_file_path) {
            Some("ser") | Some("SER") => true,
            _ => false
        }
    }

    pub fn create_mean_from_ser(ser_file_path:&str) -> error::Result<imagebuffer::ImageBuffer> {
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
                    pct_of_max:f32) -> error::Result<HaProcessing> {
        let flat = match flat_path.len() {
            0 => imagebuffer::ImageBuffer::new_empty().unwrap(),
            _ => {
                if ! path::file_exists(flat_path) {
                    panic!("File not found: {}", flat_path);
                }

                if HaProcessing::is_ser_file(flat_path) {
                    HaProcessing::create_mean_from_ser(flat_path).unwrap()
                } else {
                    imagebuffer::ImageBuffer::from_file(flat_path).unwrap()
                }
                
            }
        };
    
        let dark = match dark_path.len() {
            0 => imagebuffer::ImageBuffer::new_empty().unwrap(),
            _ => {
                if ! path::file_exists(dark_path) {
                    panic!("File not found: {}", dark_path);
                }

                if HaProcessing::is_ser_file(dark_path) {
                    HaProcessing::create_mean_from_ser(dark_path).unwrap()
                } else {
                    imagebuffer::ImageBuffer::from_file(dark_path).unwrap()
                }
            }
        };

        let mask = match mask_file.len() {
            0 => imagebuffer::ImageBuffer::new_empty().unwrap(),
            _ => {
                if ! path::file_exists(mask_file) {
                    panic!("File not found: {}", mask_file);
                }
                imagebuffer::ImageBuffer::from_file(mask_file).unwrap()
            }
        };

        Ok(
            HaProcessing {
                flat_field:flat,
                dark_field:dark,
                mask:mask,
                width:crop_width,
                height:crop_height,
                buffer:imagebuffer::ImageBuffer::new_as_mode(crop_width, crop_height, ImageMode::U8BIT).unwrap(),
                frame_count:0,
                obj_detect_threshold:obj_detect_threshold,
                red_scalar:red_scalar,
                green_scalar:green_scalar,
                blue_scalar:blue_scalar,
                obs_latitude:obs_latitude,
                obs_longitude:obs_longitude,
                min_sigma:min_sigma,
                max_sigma:max_sigma,
                pct_of_max:pct_of_max
            }
        )
    }

    pub fn apply_dark_flat_on_buffer(flat_field:&imagebuffer::ImageBuffer, dark_field:&imagebuffer::ImageBuffer, buffer:&imagebuffer::ImageBuffer) -> error::Result<imagebuffer::ImageBuffer> {

        let mut frame_buffer = buffer.clone();
        if ! flat_field.is_empty() && ! dark_field.is_empty() {
            let darkflat = flat_field.subtract(&dark_field).unwrap();
            let mean_flat = darkflat.mean();
            let frame_minus_dark = frame_buffer.subtract(&dark_field).unwrap();
            frame_buffer = frame_minus_dark.scale(mean_flat).unwrap().divide(&flat_field).unwrap();
        } else if ! flat_field.is_empty() && dark_field.is_empty() {
            let mean_flat = flat_field.mean();
            frame_buffer = frame_buffer.scale(mean_flat).unwrap().divide(&flat_field).unwrap();
        } else if flat_field.is_empty() && ! dark_field.is_empty() {
            frame_buffer = frame_buffer.subtract(&dark_field).unwrap();
        }

        Ok(frame_buffer)
    }

    fn _apply_dark_flat_on_buffer(&self, buffer:&imagebuffer::ImageBuffer) -> error::Result<imagebuffer::ImageBuffer> {
        HaProcessing::apply_dark_flat_on_buffer(&self.flat_field, &self.dark_field, &buffer)
    }


    pub fn process_frame(&self, buffer:&imagebuffer::ImageBuffer, ts:&timestamp::TimeStamp) -> imagebuffer::ImageBuffer {
        let mut frame_buffer = self._apply_dark_flat_on_buffer(&buffer).unwrap();

        let com = frame_buffer.calc_center_of_mass_offset(self.obj_detect_threshold);
        frame_buffer = frame_buffer.shift(com.h, com.v).unwrap();
        
        let (alt, az) = solar::position::position_from_lat_lon_and_time(self.obs_latitude as f64, self.obs_longitude as f64, &ts);
        let rotation = solar::parallactic_angle::from_lat_azimuth_altitude(self.obs_latitude as f64, az, alt);
        
        if self.width > 0 && self.height > 0 {
            frame_buffer = frame_buffer.crop(self.width, self.height).unwrap();
        }


        vprintln!("Rotation for frame is {} for az/alt {},{} at time {:?}", rotation, az, alt, ts);
        frame_buffer = imagerot::rotate(&frame_buffer, -1.0 * rotation.to_radians() as f32).expect("Error rotating image");

        frame_buffer
    }

    pub fn finalize(&self, out_path:&str) -> error::Result<&str> {

        if self.frame_count > 0 {
            let mean_buffer = self.buffer.scale(1.0 / self.frame_count as f32).unwrap();
            let stackmm = mean_buffer.get_min_max().unwrap();
            vprintln!("    Stack Min/Max : {}, {} ({} images)", stackmm.min, stackmm.max, self.frame_count);


            // if ! self.mask.is_empty() {
            //     rgb.apply_mask(&self.mask);
            //}

            let buffer2 = match self.mask.is_empty() {
                false => {
                    let mm = self.mask.get_min_max().unwrap();
                    let sc = self.mask.scale(1.0 / mm.max).unwrap();
                    mean_buffer.multiply(&sc).unwrap()
                },
                true => mean_buffer
            };

            let mut rgb = rgbimage::RgbImage::new_from_buffers_rgb(&buffer2, &buffer2, &buffer2, ImageMode::U8BIT).unwrap();
            rgb.apply_weight_on_band(self.red_scalar, 0);
            rgb.apply_weight_on_band(self.green_scalar, 1);
            rgb.apply_weight_on_band(self.blue_scalar, 2);


            if rgb.get_mode() == ImageMode::U8BIT {
                rgb.normalize_to_16bit_with_max(self.pct_of_max / 100.0);
            }

            rgb.save(out_path);

            ok!()
        } else {
            Err("No frames processed, not saving an empty buffer")
        }

    }

    // fn add_frame(&mut self, buffer:&imagebuffer::ImageBuffer) {
    //     self.buffer = self.buffer.add(&buffer).unwrap();
    //     self.frame_count += 1;
    // }

    fn process_frame_records(&mut self, frame_records:&Vec<FrameRecord>) {

        let mut self_buffer = self.buffer.clone();
        let buffer_mtx = Arc::new(Mutex::new(&mut self_buffer));

        frame_records.par_iter().for_each(|fr| {

            let ser_file = ser::SerFile::load_ser(fr.source_file.as_str()).expect("Unable to load SER file");
            let frame_buffer = ser_file.get_frame(fr.frame_id).unwrap();
            let frame = self.process_frame(&frame_buffer.buffer, &frame_buffer.timestamp);

            // This is a bottleneck to parallelization. 
            buffer_mtx.lock().unwrap().add_mut(&frame);

        });

        self.buffer = self_buffer.clone();
        self.frame_count += frame_records.len() as u32;
    }

    fn determine_quality_in_ser(&self, ser_file_path:&str) -> Vec<FrameRecord>{
        if ! path::file_exists(ser_file_path) {
            panic!("File not found: {}", ser_file_path);
        }
    
        let ser_file = ser::SerFile::load_ser(ser_file_path).expect("Unable to load SER file");
        ser_file.validate();

        let mut frame_records: Vec<FrameRecord> = vec!();

        let frame_records_mtx = Arc::new(Mutex::new(&mut frame_records));

        (0..ser_file.frame_count).into_par_iter().for_each(|i| {
            let frame_buffer = ser_file.get_frame(i).unwrap();
            let qual = quality::get_quality_estimation(&frame_buffer.buffer);
            
            if qual >= self.min_sigma && qual <= self.max_sigma {
                let fr = FrameRecord{
                    source_file:ser_file_path.to_string(),
                    frame_id:i,
                    quality_value:qual
                };
                frame_records_mtx.lock().unwrap().push(fr);
            } else {
                vprintln!("Frame #{} in file {} falls out of sigma range ({}) and will be excluded", i, ser_file_path, qual);
            }
        });

        frame_records
    }

    fn determine_quality_across_sers(&self, ser_files:&Vec<&str>) -> Vec<FrameRecord>{
        let mut frame_records: Vec<FrameRecord> = vec!();

        let frame_records_mtx = Arc::new(Mutex::new(&mut frame_records));

        ser_files.par_iter().for_each(|sf| {
            let mut list = self.determine_quality_in_ser(&sf);
            frame_records_mtx.lock().unwrap().append(&mut list);
        });

        frame_records.sort(); // Sorts in ascending order
        frame_records.reverse();
        frame_records
    }

    pub fn process_ser_files(&mut self, ser_files:&Vec<&str>, limit_top_pct:u8) {

        if limit_top_pct > 100 {
            panic!("Invalid percentage: Exceeds 100%: {}", limit_top_pct);
        }

        let frame_records: Vec<FrameRecord> = self.determine_quality_across_sers(&ser_files);

        let max_frame = ((limit_top_pct as f32 / 100.0) * frame_records.len() as f32).round() as usize;

        let limited_frame_records: Vec<FrameRecord> = frame_records[0..max_frame].to_vec();

        self.process_frame_records(&limited_frame_records);

        vprintln!("Total frames considered: {}", frame_records.len());
        vprintln!("Limited to top {}% of frames", limit_top_pct);
        vprintln!("Processed with {} frames", limited_frame_records.len());
    }

}

