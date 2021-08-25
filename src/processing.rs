use crate::{
    ser,
    constants,
    path,
    vprintln,
    imagebuffer,
    error,
    enums,
    mean,
    solar,
    imagerot,
    timestamp,
    rgbimage,
    ok
};




pub struct HaProcessing {
    pub flat_field:imagebuffer::ImageBuffer,
    pub dark_field:imagebuffer::ImageBuffer,
    pub width:usize,
    pub height:usize,
    pub buffer:imagebuffer::ImageBuffer,
    pub frame_count:u32,
    pub obj_detect_threshold:f32,
    pub red_scalar:f32,
    pub green_scalar:f32,
    pub blue_scalar:f32,
    pub obs_latitude:f32,
    pub obs_longitude:f32
}

impl HaProcessing {

    fn is_ser_file(ser_file_path:&str) -> bool {
        match path::get_extension(ser_file_path) {
            Some("ser") | Some("SER") => true,
            _ => false
        }
    }

    fn create_mean_from_ser(ser_file_path:&str) -> error::Result<imagebuffer::ImageBuffer> {
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
                    crop_width:usize, 
                    crop_height:usize, 
                    obj_detect_threshold:f32, 
                    red_scalar:f32, 
                    green_scalar:f32, 
                    blue_scalar:f32,
                    obs_latitude:f32,
                    obs_longitude:f32) -> error::Result<HaProcessing> {
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
    
        Ok(
            HaProcessing {
                flat_field:flat,
                dark_field:dark,
                width:crop_width,
                height:crop_height,
                buffer:imagebuffer::ImageBuffer::new_as_mode(crop_width, crop_height, enums::ImageMode::U8BIT).unwrap(),
                frame_count:0,
                obj_detect_threshold:obj_detect_threshold,
                red_scalar:red_scalar,
                green_scalar:green_scalar,
                blue_scalar:blue_scalar,
                obs_latitude:obs_latitude,
                obs_longitude:obs_longitude
            }
        )
    }

    fn apply_dark_flat_on_buffer(&self, buffer:&imagebuffer::ImageBuffer) -> error::Result<imagebuffer::ImageBuffer> {

        let mut frame_buffer = buffer.clone();
        if ! self.flat_field.is_empty() && ! self.dark_field.is_empty() {
            let darkflat = self.flat_field.subtract(&self.dark_field).unwrap();
            let mean_flat = darkflat.mean();
            let frame_minus_dark = frame_buffer.subtract(&self.dark_field).unwrap();
            frame_buffer = frame_minus_dark.scale(mean_flat).unwrap().divide(&self.flat_field).unwrap();
        } else if ! self.flat_field.is_empty() && self.dark_field.is_empty() {
            let mean_flat = self.flat_field.mean();
            frame_buffer = frame_buffer.scale(mean_flat).unwrap().divide(&self.flat_field).unwrap();
        } else if self.flat_field.is_empty() && ! self.dark_field.is_empty() {
            frame_buffer = frame_buffer.subtract(&self.dark_field).unwrap();
        }

        Ok(frame_buffer)
    }


    pub fn add_frame(&mut self, buffer:&imagebuffer::ImageBuffer, ts:&timestamp::TimeStamp) {

        let mut frame_buffer = buffer.clone();

        frame_buffer = self.apply_dark_flat_on_buffer(&frame_buffer).unwrap();


        let com = frame_buffer.calc_center_of_mass_offset(40.0).unwrap();
        frame_buffer = frame_buffer.shift(com.h, com.v).unwrap();
        
        
        let (alt, az) = solar::position::position_from_lat_lon_and_time(self.obs_latitude as f64, self.obs_longitude as f64, &ts);
        let rotation = solar::parallactic_angle::from_lat_azimuth_altitude(self.obs_latitude as f64, az, alt);
        

        if self.width > 0 && self.height > 0 {
            frame_buffer = frame_buffer.crop(self.width, self.height).unwrap();
        }

        vprintln!("Rotation for frame is {} for az/alt {},{} at time {:?}", rotation, az, alt, ts);
        frame_buffer = imagerot::rotate(&frame_buffer, -1.0 * rotation.to_radians() as f32).expect("Error rotating image");

        self.buffer = self.buffer.add(&frame_buffer).unwrap();
        self.frame_count += 1;
    }

    pub fn finalize(&self, out_path:&str) -> error::Result<&str> {

        if self.frame_count > 0 {
            let mut mean_buffer = self.buffer.scale(1.0 / self.frame_count as f32).unwrap();
            let stackmm = mean_buffer.get_min_max().unwrap();
            vprintln!("    Stack Min/Max : {}, {} ({} images)", stackmm.min, stackmm.max, self.frame_count);

            if mean_buffer.mode == enums::ImageMode::U8BIT {
                let mnmx = mean_buffer.get_min_max().unwrap();
                mean_buffer = mean_buffer.normalize_force_minmax(0.0, 65535.0, 0.0, mnmx.max).unwrap();
                mean_buffer.mode = enums::ImageMode::U16BIT;
            }

            let mut rgb = rgbimage::RgbImage::new_from_buffers_rgb(&mean_buffer, &mean_buffer, &mean_buffer, enums::ImageMode::U16BIT).unwrap();
            rgb.apply_weight(self.red_scalar, self.green_scalar, self.blue_scalar).expect("Error applying channel weights");
            rgb.save(out_path).expect("Error: Error saving output image");

            //mean_buffer.save(out_path).expect("Error: Error saving output image");

            ok!()
        } else {
            Err("No frames processed, not saving an empty buffer")
        }

    }


    pub fn process_ser_file(&mut self, ser_file_path:&str) {

        if ! path::file_exists(ser_file_path) {
            panic!("File not found: {}", ser_file_path);
        }
    
        let ser_file = ser::SerFile::load_ser(ser_file_path).expect("Unable to load SER file");
        ser_file.validate();
    
        for i in 0..ser_file.frame_count {
            // if i >= 3 {
            //     break;
            // }
            let frame_buffer = ser_file.get_frame(i).unwrap();
    
            // TODO: Detect and reject glitch frames
    
            self.add_frame(&frame_buffer.buffer, &frame_buffer.timestamp);
        }
    
    }

    pub fn process_ser_files(&mut self, ser_files:&Vec<&str>) {
        for ser_file_path in ser_files.iter() {
            self.process_ser_file(ser_file_path);
        }
    }

}

