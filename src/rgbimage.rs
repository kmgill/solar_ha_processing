
use crate::{
    imagebuffer::ImageBuffer, 
    constants, 
    vprintln, 
    path, 
    error, 
    decompanding, 
    enums, 
    flatfield, 
    inpaint,
    ok,
    debayer,
    noise,
    hotpixel,
    metadata::*,
    util
};

use image::{
    open, 
    DynamicImage, 
    Rgba
};

// A simple image raster buffer.
#[derive(Debug, Clone)]
pub struct RgbImage {
    _red: ImageBuffer,
    _green: ImageBuffer,
    _blue: ImageBuffer,
    pub width: usize,
    pub height: usize,
    instrument: enums::Instrument,
    mode: enums::ImageMode,
    empty: bool,
    metadata: Option<Metadata>
}

#[allow(dead_code)]
impl RgbImage {
    pub fn new(width:usize, height:usize, instrument:enums::Instrument) -> error::Result<RgbImage> {
        let red = ImageBuffer::new(width, height).unwrap();
        let green = ImageBuffer::new(width, height).unwrap();
        let blue = ImageBuffer::new(width, height).unwrap();

        Ok(RgbImage{
            _red:red,
            _green:green,
            _blue:blue,
            width,
            height,
            instrument,
            mode:enums::ImageMode::U8BIT,
            empty:false,
            metadata:None
        })
    }

    pub fn new_empty() -> error::Result<RgbImage> {
        Ok(RgbImage{
            _red:ImageBuffer::new_empty().unwrap(),
            _green:ImageBuffer::new_empty().unwrap(),
            _blue:ImageBuffer::new_empty().unwrap(),
            width:0,
            height:0,
            instrument:enums::Instrument::None,
            mode:enums::ImageMode::U8BIT,
            empty:true,
            metadata:None
        })
    }

    pub fn open_str(file_path:&str, instrument:enums::Instrument) -> error::Result<RgbImage> {
        RgbImage::open(String::from(file_path), instrument)
    }

    pub fn open(file_path:String, instrument:enums::Instrument) -> error::Result<RgbImage> {
        if !path::file_exists(file_path.as_str()) {
            return Err(constants::status::FILE_NOT_FOUND);
        }

        vprintln!("Loading image from {}", file_path);
        let image_data = open(&file_path).unwrap().into_rgb8();
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;
        vprintln!("Input image dimensions: {:?}", image_data.dimensions());

        let mut rgbimage = RgbImage::new(width, height, instrument).unwrap();

        for y in 0..height {
            for x in 0..width {
                let pixel = image_data.get_pixel(x as u32, y as u32);
                let red = pixel[0] as f32;
                let green = pixel[1] as f32;
                let blue = pixel[2] as f32;
                rgbimage.put(x, y, red, green, blue).unwrap();
            }
        }

        let metadata_file = util::replace_image_extension(&file_path.as_str(), "-metadata.json");
        vprintln!("Checking for metadata file at {}", metadata_file);
        if path::file_exists(metadata_file.as_str()) {
            vprintln!("Metadata file exists for loaded image: {}", metadata_file);
            rgbimage.metadata = match load_image_metadata(&metadata_file) {
                Err(why) => panic!("couldn't open {}", why),
                Ok(md) => Some(md)
            };
        }

        Ok(rgbimage)
    }

    pub fn new_from_buffers_rgb(red:&ImageBuffer, green:&ImageBuffer, blue:&ImageBuffer, instrument:enums::Instrument, mode:enums::ImageMode) -> error::Result<RgbImage> {
        Ok(RgbImage{
            _red:red.clone(),
            _green:green.clone(),
            _blue:blue.clone(),
            width:red.width,
            height:red.height,
            instrument,
            mode,
            empty:false,
            metadata:None
        })
    }

    pub fn new_with_size(width:usize, height:usize, instrument:enums::Instrument, mode:enums::ImageMode) -> error::Result<RgbImage> {
        Ok(RgbImage{
            _red:ImageBuffer::new(width, height).unwrap(),
            _green:ImageBuffer::new(width, height).unwrap(),
            _blue:ImageBuffer::new(width, height).unwrap(),
            width:width,
            height:height,
            instrument,
            mode,
            empty:false,
            metadata:None
        })
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn get_metadata(&self) -> Option<Metadata> {
        self.metadata.clone()
    }

    pub fn has_metadata(&self) -> bool {
        self.metadata.is_some()
    }

    pub fn set_instrument(&mut self, instrument:enums::Instrument) {
        self.instrument = instrument;
    }

    pub fn get_mode(&self) -> enums::ImageMode {
        self.mode
    }

    pub fn get_instrument(&self) -> enums::Instrument {
        self.instrument
    }

    pub fn divide_from_each(&mut self, other:&ImageBuffer) -> error::Result<&str> {
        if self.width != other.width || self.height != other.height {
            return Err(constants::status::ARRAY_SIZE_MISMATCH);
        }

        self._red = self._red.divide(&other).unwrap();
        self._green = self._green.divide(&other).unwrap();
        self._blue = self._blue.divide(&other).unwrap();

        ok!()
    }

    pub fn add_to_each(&mut self, other:&ImageBuffer) -> error::Result<&str> {
        if self.width != other.width || self.height != other.height {
            return Err(constants::status::ARRAY_SIZE_MISMATCH);
        }

        self._red = self._red.add(&other).unwrap();
        self._green = self._green.add(&other).unwrap();
        self._blue = self._blue.add(&other).unwrap();

        ok!()
    }

    pub fn add(&mut self, other:&RgbImage) -> error::Result<&str> {
        if self.width != other.width || self.height != other.height {
            return Err(constants::status::ARRAY_SIZE_MISMATCH);
        }

        self._red = self._red.add(&other._red).unwrap();
        self._green = self._green.add(&other._green).unwrap();
        self._blue = self._blue.add(&other._blue).unwrap();

        ok!()
    }

    pub fn put(&mut self, x:usize, y:usize, r:f32, g:f32, b:f32) -> error::Result<&str>{
        if x < self.width && y < self.height {
            self._red.put(x, y, r)?;
            self._green.put(x, y, g)?;
            self._blue.put(x, y, b)?;
            ok!()
        } else {
            Err(constants::status::INVALID_PIXEL_COORDINATES)
        }
    }

    pub fn paste(&mut self, src:&RgbImage, tl_x:usize, tl_y:usize) -> error::Result<&str> {

        self._red = self._red.paste(&src._red, tl_x, tl_y).unwrap();
        self._green = self._green.paste(&src._green, tl_x, tl_y).unwrap();
        self._blue = self._blue.paste(&src._blue, tl_x, tl_y).unwrap();

        ok!()
    }

    pub fn apply_mask(&mut self, mask:&ImageBuffer) {
        self._red.set_mask(mask);
        self._green.set_mask(mask);
        self._blue.set_mask(mask);
    }

    pub fn clear_mask(&mut self) {
        self._red.clear_mask();
        self._green.clear_mask();
        self._blue.clear_mask();
    }

    pub fn copy_mask_from(&mut self, src:&ImageBuffer) {
        src.copy_mask_to(&mut self._red);
        src.copy_mask_to(&mut self._green);
        src.copy_mask_to(&mut self._blue);
    }

    pub fn red(&self) -> &ImageBuffer {
        &self._red
    }

    pub fn green(&self) -> &ImageBuffer {
        &self._green
    }

    pub fn blue(&self) -> &ImageBuffer {
        &self._blue
    }

    fn apply_flat_on_channel(buffer:&ImageBuffer, flat_buffer:&ImageBuffer) -> error::Result<ImageBuffer> {
        let mean_flat = flat_buffer.mean();
        let corrected = buffer.scale(mean_flat).unwrap().divide(&flat_buffer).unwrap();
        Ok(corrected)
    }

    fn apply_flat(&mut self, flat:RgbImage) -> error::Result<&str> {

        self._red = RgbImage::apply_flat_on_channel(&self._red, &flat.red()).unwrap();
        self._green = RgbImage::apply_flat_on_channel(&self._green, &flat.green()).unwrap();
        self._blue = RgbImage::apply_flat_on_channel(&self._blue, &flat.blue()).unwrap();

        if let Some(ref mut md) = self.metadata {
            md.flatfield = true;
        }

        ok!()
    }

    pub fn flatfield(&mut self) -> error::Result<&str> {

        let mut flat = flatfield::load_flat(self.instrument).unwrap();

        // These instrument-specific crops don't really belong in here.
        if self.instrument == enums::Instrument::MslMAHLI && flat.width == 1632 && flat.height == 1200 {
            flat.crop(32, 16, 1584, 1184).unwrap();
        } 
        
        if self.instrument == enums::Instrument::MslMastcamRight {

            if self.width == 1328 && self.height == 1184 {
                //x160, y16
                flat.crop(160, 16, 1328, 1184).unwrap();
            } else if self.width == 848 && self.height == 848 {
                //x400, y192
                flat.crop(400, 192, 848, 848).unwrap();
            }

            if self.mode == enums::ImageMode::U8BIT {
                flat.normalize_to_12bit_with_max(255.0).unwrap();
                flat.compand().unwrap();
            }

        }

        if self.instrument == enums::Instrument::MslMastcamLeft {

            if self.width == 1328 && self.height == 1184 { //9
                flat.crop(160, 16, 1328, 1184).unwrap();
            }  else if self.width == 1152 && self.height == 432 {
                flat.crop(305, 385, 1152, 432).unwrap();
            }

            if self.mode == enums::ImageMode::U8BIT {
                flat.normalize_to_12bit_with_max(255.0).unwrap();
                flat.compand().unwrap();
            }
        }
        

        // Crop the flatfield image if it's larger than the input image. 
        // Sizes need to match
        if flat.width > self.width {
            let x = (flat.width - self.width) / 2;
            let y = (flat.height - self.height) / 2;
            vprintln!("Cropping flat with x/y/width/height: {},{} {}x{}", x, y, self.width, self.height);
            flat.crop(x, y, self.width, self.height).unwrap();
        }

        // if inpaint::inpaint_supported_for_instrument(self.instrument) {
        //     flat.apply_inpaint_fix().unwrap();
        // } else {
        //     vprintln!("No inpaint available for flatfield image on {:?}", self.instrument);
        // }
        self.apply_flat(flat).unwrap();
        ok!()
    }

    pub fn compand(&mut self) -> error::Result<&str> {
        decompanding::compand_buffer(&mut self._red, self.instrument).unwrap();
        decompanding::compand_buffer(&mut self._green, self.instrument).unwrap();
        decompanding::compand_buffer(&mut self._blue, self.instrument).unwrap();
        self.mode = enums::ImageMode::U8BIT;

        if let Some(ref mut md) = self.metadata {
            md.decompand = false;
        }
        ok!()
    }

    pub fn decompand(&mut self) -> error::Result<&str> {
        decompanding::decompand_buffer(&mut self._red, self.instrument).unwrap();
        decompanding::decompand_buffer(&mut self._green, self.instrument).unwrap();
        decompanding::decompand_buffer(&mut self._blue, self.instrument).unwrap();
        self.mode = enums::ImageMode::U12BIT;

        if let Some(ref mut md) = self.metadata {
            md.decompand = true;
        }
        ok!()
    }

    pub fn debayer(&mut self) -> error::Result<&str> {
        let debayered = debayer::debayer(&self._red).unwrap();

        self._red = debayered.red().clone();
        self._green = debayered.green().clone();
        self._blue = debayered.blue().clone();

        if let Some(ref mut md) = self.metadata {
            md.debayer = true;
        }

        ok!()
    }


    pub fn reduce_color_noise(&mut self, amount:i32) -> error::Result<&str> {

        let result = noise::color_noise_reduction(&mut self.clone(), amount).unwrap();
        self._red = result.red().clone();
        self._green = result.green().clone();
        self._blue = result.blue().clone();
        ok!()
    }

    pub fn apply_weight(&mut self, r_scalar:f32, g_scalar:f32, b_scalar:f32) -> error::Result<&str> {

        self._red = self._red.scale(r_scalar).unwrap();
        self._green = self._green.scale(g_scalar).unwrap();
        self._blue = self._blue.scale(b_scalar).unwrap();

        if let Some(ref mut md) = self.metadata {
            md.radiometric = true;
        }
        ok!()
    }

    pub fn hot_pixel_correction(&mut self, window_size:i32, threshold:f32) -> error::Result<&str> {

        self._red = hotpixel::hot_pixel_detection(&self._red, window_size, threshold).unwrap().buffer;
        self._green = hotpixel::hot_pixel_detection(&self._green, window_size, threshold).unwrap().buffer;
        self._blue = hotpixel::hot_pixel_detection(&self._blue, window_size, threshold).unwrap().buffer;
        ok!()
    }

    pub fn crop(&mut self, x:usize, y:usize, width:usize, height:usize) -> error::Result<&str> {
        self._red = self._red.get_subframe(x, y, width, height).unwrap();
        self._green = self._green.get_subframe(x, y, width, height).unwrap();
        self._blue = self._blue.get_subframe(x, y, width, height).unwrap();
        self.width = width;
        self.height = height;

        if let Some(ref mut md) = self.metadata {
            md.cropped = true;
        }

        ok!()
    }

    fn is_pixel_grayscale(&self, x:usize, y:usize) -> bool {
        let r = self._red.get(x, y).unwrap();
        let g = self._green.get(x, y).unwrap();
        let b = self._blue.get(x, y).unwrap();

        //consider comparing them within some margin of error: `(g - b).abs() < error_margin`
        r == g && g == b
    }

    // This makes some assumptions and isn't perfect.
    pub fn is_grayscale(&self) -> bool {

        let tl = self.is_pixel_grayscale(30, 30);
        let bl = self.is_pixel_grayscale(30, self.height - 30);
        let tr = self.is_pixel_grayscale(self.width - 30, 30);
        let br = self.is_pixel_grayscale(self.width - 30, self.height - 30);

        let mid_x = self.width / 2;
        let mid_y = self.height / 2;

        let mtl = self.is_pixel_grayscale(mid_x - 20, mid_y - 20);
        let mbl = self.is_pixel_grayscale(mid_x - 20, mid_y + 20);
        let mtr = self.is_pixel_grayscale(mid_x + 20, mid_y - 20);
        let mbr = self.is_pixel_grayscale(mid_x + 20, mid_y + 20);

        tl && bl && tr && br && mtl && mbl && mtr && mbr
    }

    pub fn apply_inpaint_fix(&mut self) -> error::Result<&str> {
        let fixed = inpaint::apply_inpaint_to_buffer(&self).unwrap();

        let mut new_r = fixed.red().clone();
        self._red.copy_mask_to(&mut new_r);

        let mut new_g = fixed.green().clone();
        self._green.copy_mask_to(&mut new_g);

        let mut new_b = fixed.blue().clone();
        self._blue.copy_mask_to(&mut new_b);

        self._red = new_r;
        self._green = new_g;
        self._blue = new_b;

        if let Some(ref mut md) = self.metadata {
            md.inpaint = true;
        }

        ok!()
    }

    pub fn normalize_to_8bit_with_max(&mut self, max:f32) -> error::Result<&str> {
        self._red = self._red.normalize_force_minmax(0.0, 255.0, 0.0, max).unwrap();
        self._green = self._green.normalize_force_minmax(0.0, 255.0, 0.0, max).unwrap();
        self._blue = self._blue.normalize_force_minmax(0.0, 255.0, 0.0, max).unwrap();
        self.mode = enums::ImageMode::U8BIT;
        ok!()
    }

    pub fn normalize_to_12bit_with_max(&mut self, max:f32) -> error::Result<&str> {
        self._red = self._red.normalize_force_minmax(0.0, decompanding::get_max_for_instrument(self.instrument) as f32, 0.0, max).unwrap();
        self._green = self._green.normalize_force_minmax(0.0, decompanding::get_max_for_instrument(self.instrument) as f32, 0.0, max).unwrap();
        self._blue = self._blue.normalize_force_minmax(0.0, decompanding::get_max_for_instrument(self.instrument) as f32, 0.0, max).unwrap();
        self.mode = enums::ImageMode::U12BIT;
        ok!()
    }

    pub fn normalize_to_16bit_with_max(&mut self, max:f32) -> error::Result<&str> {
        self._red = self._red.normalize_force_minmax(0.0, 65535.0, 0.0, max).unwrap();
        self._green = self._green.normalize_force_minmax(0.0, 65535.0, 0.0, max).unwrap();
        self._blue = self._blue.normalize_force_minmax(0.0, 65535.0, 0.0, max).unwrap();
        self.mode = enums::ImageMode::U16BIT;
        ok!()
    }

    pub fn normalize_to_12bit(&mut self) -> error::Result<&str> {

        let r_mnmx = self._red.get_min_max().unwrap();
        let g_mnmx = self._green.get_min_max().unwrap();
        let b_mnmx = self._blue.get_min_max().unwrap();

        let mut mx = if r_mnmx.max > g_mnmx.max { r_mnmx.max} else { g_mnmx.max };
        mx = if mx > b_mnmx.max { mx } else { b_mnmx.max };

        self.normalize_to_12bit_with_max(mx).unwrap();

        ok!()
    }

    pub fn normalize_to_8bit(&mut self) -> error::Result<&str> {

        let r_mnmx = self._red.get_min_max().unwrap();
        let g_mnmx = self._green.get_min_max().unwrap();
        let b_mnmx = self._blue.get_min_max().unwrap();

        let mut mx = if r_mnmx.max > g_mnmx.max { r_mnmx.max} else { g_mnmx.max };
        mx = if mx > b_mnmx.max { mx } else { b_mnmx.max };

        self.normalize_to_8bit_with_max(mx).unwrap();

        ok!()
    }

    pub fn normalize_to_16bit(&mut self) -> error::Result<&str> {

        let r_mnmx = self._red.get_min_max().unwrap();
        let g_mnmx = self._green.get_min_max().unwrap();
        let b_mnmx = self._blue.get_min_max().unwrap();

        let mut mx = if r_mnmx.max > g_mnmx.max { r_mnmx.max} else { g_mnmx.max };
        mx = if mx > b_mnmx.max { mx } else { b_mnmx.max };

        self.normalize_to_16bit_with_max(mx).unwrap();

        ok!()
    }

    pub fn normalize_16bit_to_8bit(&mut self) -> error::Result<&str> {
        self.normalize_to_8bit_with_max(65535.0).unwrap();
        ok!()
    }

    pub fn normalize_8bit_to_16bit(&mut self) -> error::Result<&str> {
        self.normalize_to_16bit_with_max(255.0).unwrap();
        ok!()
    }

    fn save_16bit(&self, to_file:&str) -> error::Result<&str> {
        let mut out_img = DynamicImage::new_rgba16(self.width as u32, self.height as u32).into_rgba16();

        for y in 0..self.height {
            for x in 0..self.width {
                let r = self._red.get(x, y).unwrap().round() as u16;
                let g = self._green.get(x, y).unwrap().round() as u16;
                let b = self._blue.get(x, y).unwrap().round() as u16;
                let a = if self._red.get_mask_at_point(x, y).unwrap() { 65535 } else { 0 };
                out_img.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
            }
        }

        vprintln!("Writing image buffer to file at {}", to_file);
        if path::parent_exists_and_writable(&to_file) {
            out_img.save(to_file).unwrap();

            match &self.metadata {
                Some(md) => {
                    vprintln!("Writing metadata file...");
                    util::save_image_json(to_file, &md, false).unwrap();
                },
                None => {}
            };

            vprintln!("File saved.");
            ok!()
        } else {
            eprintln!("Parent does not exist or cannot be written: {}", path::get_parent(to_file));
            Err(constants::status::PARENT_NOT_EXISTS_OR_UNWRITABLE)
        }
    }

    fn save_8bit(&self, to_file:&str) -> error::Result<&str> {
        let mut out_img = DynamicImage::new_rgba8(self.width as u32, self.height as u32).into_rgba8();

        for y in 0..self.height {
            for x in 0..self.width {
                let r = self._red.get(x, y).unwrap().round() as u8;
                let g = self._green.get(x, y).unwrap().round() as u8;
                let b = self._blue.get(x, y).unwrap().round() as u8;
                let a = if self._red.get_mask_at_point(x, y).unwrap() { 255 } else { 0 };
                out_img.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
            }
        }

        vprintln!("Writing image buffer to file at {}", to_file);
        if path::parent_exists_and_writable(&to_file) {
            out_img.save(to_file).unwrap();

            match &self.metadata {
                Some(md) => {
                    util::save_image_json(to_file, &md, false).unwrap();
                },
                None => {}
            };

            vprintln!("File saved.");
            ok!()
        } else {
            eprintln!("Parent does not exist or cannot be written: {}", path::get_parent(to_file));
            Err(constants::status::PARENT_NOT_EXISTS_OR_UNWRITABLE)
        }
    }

    pub fn save(&self, to_file:&str) -> error::Result<&str> {
        match self.mode {
            enums::ImageMode::U8BIT => {
                self.save_8bit(to_file)
            },
            _ => {
                self.save_16bit(to_file)
            }
        }
    }
}