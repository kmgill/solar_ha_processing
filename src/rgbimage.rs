
use crate::{
    imagebuffer::ImageBuffer, 
    constants, 
    vprintln, 
    path, 
    error, 
    enums, 
    ok,
    debayer,
    hotpixel
};

use image::{
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
    mode: enums::ImageMode,
    empty: bool,
}

#[allow(dead_code)]
impl RgbImage {
    pub fn new(width:usize, height:usize) -> error::Result<RgbImage> {
        let red = ImageBuffer::new(width, height).unwrap();
        let green = ImageBuffer::new(width, height).unwrap();
        let blue = ImageBuffer::new(width, height).unwrap();

        Ok(RgbImage{
            _red:red,
            _green:green,
            _blue:blue,
            width,
            height,
            mode:enums::ImageMode::U8BIT,
            empty:false
        })
    }

    pub fn new_empty() -> error::Result<RgbImage> {
        Ok(RgbImage{
            _red:ImageBuffer::new_empty().unwrap(),
            _green:ImageBuffer::new_empty().unwrap(),
            _blue:ImageBuffer::new_empty().unwrap(),
            width:0,
            height:0,
            mode:enums::ImageMode::U8BIT,
            empty:true
        })
    }


    pub fn new_from_buffers_rgb(red:&ImageBuffer, green:&ImageBuffer, blue:&ImageBuffer, mode:enums::ImageMode) -> error::Result<RgbImage> {
        Ok(RgbImage{
            _red:red.clone(),
            _green:green.clone(),
            _blue:blue.clone(),
            width:red.width,
            height:red.height,
            mode,
            empty:false
        })
    }

    pub fn new_with_size(width:usize, height:usize, mode:enums::ImageMode) -> error::Result<RgbImage> {
        Ok(RgbImage{
            _red:ImageBuffer::new(width, height).unwrap(),
            _green:ImageBuffer::new(width, height).unwrap(),
            _blue:ImageBuffer::new(width, height).unwrap(),
            width:width,
            height:height,
            mode,
            empty:false
        })
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }


    pub fn get_mode(&self) -> enums::ImageMode {
        self.mode
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

    pub fn debayer(&mut self) -> error::Result<&str> {
        let debayered = debayer::debayer(&self._red).unwrap();

        self._red = debayered.red().clone();
        self._green = debayered.green().clone();
        self._blue = debayered.blue().clone();

        ok!()
    }

    pub fn apply_weight(&mut self, r_scalar:f32, g_scalar:f32, b_scalar:f32) -> error::Result<&str> {

        self._red = self._red.scale(r_scalar).unwrap();
        self._green = self._green.scale(g_scalar).unwrap();
        self._blue = self._blue.scale(b_scalar).unwrap();

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



    pub fn normalize_to_8bit_with_max(&mut self, max:f32) -> error::Result<&str> {
        self._red = self._red.normalize_force_minmax(0.0, 255.0, 0.0, max).unwrap();
        self._green = self._green.normalize_force_minmax(0.0, 255.0, 0.0, max).unwrap();
        self._blue = self._blue.normalize_force_minmax(0.0, 255.0, 0.0, max).unwrap();
        self.mode = enums::ImageMode::U8BIT;
        ok!()
    }

    pub fn normalize_to_16bit_with_max(&mut self, max:f32) -> error::Result<&str> {
        self._red = self._red.normalize_force_minmax(0.0, 65535.0, 0.0, max).unwrap();
        self._green = self._green.normalize_force_minmax(0.0, 65535.0, 0.0, max).unwrap();
        self._blue = self._blue.normalize_force_minmax(0.0, 65535.0, 0.0, max).unwrap();
        self.mode = enums::ImageMode::U16BIT;
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