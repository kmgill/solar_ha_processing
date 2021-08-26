
use crate::{
    path, 
    constants, 
    vprintln, 
    error, 
    ok,
    enums
};

extern crate image;
use image::{
    open, 
    DynamicImage, 
    Rgba
};

// A simple image raster buffer.
#[derive(Debug, Clone)]
pub struct ImageBuffer {
    pub buffer: Vec<f32>,
    pub width: usize,
    pub height: usize,
    empty: bool,
    pub mask: Option<Vec<bool>>,
    pub mode: enums::ImageMode
}

pub struct Offset {
    pub h: i32,
    pub v: i32,
}

pub struct MinMax {
    pub min: f32,
    pub max: f32,
}

// Implements a center crop
fn crop_array<T:Copy>(arr:&[T], from_width:usize, from_height:usize, to_width:usize, to_height:usize) -> Vec<T> {
    let mut new_arr : Vec<T> = Vec::with_capacity(to_width * to_height);
 
    for y in 0..to_height {
        for x in 0..to_width {
    
            let from_x = ((from_width - to_width) / 2) + x;
            let from_y = ((from_height - to_height) / 2) + y;
            let from_idx = from_y * from_width + from_x;

            //let to_idx = y * to_width + x;
            new_arr.push(arr[from_idx]);
        }
    }
    
    new_arr
}

fn subframe_array<T:Copy>(arr:&[T], from_width:usize, _from_height:usize, left_x:usize, top_y:usize, to_width:usize, to_height:usize) -> Vec<T> {
    let mut new_arr : Vec<T> = Vec::with_capacity(to_width * to_height);

    for y in 0..to_height {
        for x in 0..to_width {
            let from_idx = (top_y + y) * from_width + (left_x + x);
            new_arr.push(arr[from_idx]);
        }
    }
    new_arr
}


#[allow(dead_code)]
impl ImageBuffer {

    // Creates a new image buffer of the requested width and height
    pub fn new(width:usize, height:usize) -> error::Result<ImageBuffer> {
        ImageBuffer::new_as_mode(width, height, enums::ImageMode::U16BIT)
    }

    // Creates a new image buffer of the requested width and height
    pub fn new_as_mode(width:usize, height:usize, mode:enums::ImageMode) -> error::Result<ImageBuffer> {
        ImageBuffer::new_with_fill_as_mode(width, height, 0.0, mode)
    }

    // Creates a new image buffer of the requested width and height
    pub fn new_with_fill(width:usize, height:usize, fill_value:f32) -> error::Result<ImageBuffer> {
        ImageBuffer::new_with_fill_as_mode(width, height, fill_value, enums::ImageMode::U16BIT)
    }

    // Creates a new image buffer of the requested width and height
    pub fn new_with_fill_as_mode(width:usize, height:usize, fill_value:f32, mode:enums::ImageMode) -> error::Result<ImageBuffer> {

        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, fill_value);

        Ok(ImageBuffer{buffer:v,
            width,
            height,
            empty:false,
            mask:None,
            mode: mode
        })
    }

    // Creates a new image buffer of the requested width and height
    pub fn new_with_mask(width:usize, height:usize, mask:&Option<Vec<bool>>) -> error::Result<ImageBuffer> {

        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, 0.0);

        Ok(ImageBuffer{buffer:v,
            width,
            height,
            empty:false,
            mask: if *mask != None { Some(mask.as_ref().unwrap().to_owned()) } else { None },
            mode: enums::ImageMode::U16BIT
        })
    }

    fn new_with_mask_as_mode(width:usize, height:usize, mask:&Option<Vec<bool>>, mode:enums::ImageMode) -> error::Result<ImageBuffer> {

        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, 0.0);

        Ok(ImageBuffer{buffer:v,
            width,
            height,
            empty:false,
            mask: if *mask != None { Some(mask.as_ref().unwrap().to_owned()) } else { None },
            mode: mode
        })
    }

    pub fn new_empty() -> error::Result<ImageBuffer> {
        Ok(ImageBuffer{buffer:Vec::new(),
            width:0,
            height:0,
            empty:true,
            mask:None,
            mode: enums::ImageMode::U16BIT
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec(v:Vec<f32>, width:usize, height:usize) -> error::Result<ImageBuffer> {
        ImageBuffer::from_vec_as_mode(v, width, height, enums::ImageMode::U16BIT)
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_as_mode(v:Vec<f32>, width:usize, height:usize, mode:enums::ImageMode) -> error::Result<ImageBuffer> {

        if v.len() != (width * height) {
            return Err(constants::status::DIMENSIONS_DO_NOT_MATCH_VECTOR_LENGTH);
        }

        Ok(ImageBuffer{buffer:v,
                    width,
                    height,
                    empty:false,
                    mask:None,
                    mode: mode
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_u8(v_u8:Vec<u8>, width:usize, height:usize) -> error::Result<ImageBuffer> {

        if v_u8.len() != (width * height) {
            return Err(constants::status::DIMENSIONS_DO_NOT_MATCH_VECTOR_LENGTH);
        }

        let mut v = vec![0.0_f32; width * height];
        for i in 0..v_u8.len() {
            v[i] = v_u8[i] as f32;
        }

        Ok(ImageBuffer{buffer:v,
                    width,
                    height,
                    empty:false,
                    mask:None,
                    mode: enums::ImageMode::U16BIT
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_u8_with_mask(v_u8:Vec<u8>, width:usize, height:usize, mask:&Option<Vec<bool>>) -> error::Result<ImageBuffer> {

        if v_u8.len() != (width * height) {
            return Err(constants::status::DIMENSIONS_DO_NOT_MATCH_VECTOR_LENGTH);
        }

        let mut v = vec![0.0_f32; width * height];
        for i in 0..v_u8.len() {
            v[i] = v_u8[i] as f32;
        }

        Ok(ImageBuffer{buffer:v,
                    width:width,
                    height:height,
                    empty:false,
                    mask: if *mask != None { Some(mask.as_ref().unwrap().to_owned()) } else { None },
                    mode: enums::ImageMode::U16BIT
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_with_mask(v:Vec<f32>, width:usize, height:usize, mask:&Option<Vec<bool>>) -> error::Result<ImageBuffer> {

        if v.len() != (width * height) {
            return Err(constants::status::DIMENSIONS_DO_NOT_MATCH_VECTOR_LENGTH);
        }

        Ok(ImageBuffer{buffer:v,
                    width,
                    height,
                    empty:false,
                    mask: if *mask != None { Some(mask.as_ref().unwrap().to_owned()) } else { None },
                    mode: enums::ImageMode::U16BIT
        })
    }

    fn new_from_op(v:Vec<f32>, width:usize, height:usize, mask:&Option<Vec<bool>>, mode:enums::ImageMode) -> error::Result<ImageBuffer> {
        if v.len() != (width * height) {
            return Err(constants::status::DIMENSIONS_DO_NOT_MATCH_VECTOR_LENGTH);
        }

        Ok(ImageBuffer{buffer:v,
            width,
            height,
            empty:false,
            mask: if *mask != None { Some(mask.as_ref().unwrap().to_owned()) } else { None },
            mode: mode
        })
    }

    pub fn from_image_u8(image_data:&image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>) -> error::Result<ImageBuffer> {
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;
        vprintln!("Input image dimensions: {:?}", image_data.dimensions());

        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, 0.0);

        for y in 0..height {
            for x in 0..width {
                let pixel = image_data.get_pixel(x as u32, y as u32);
                let value = pixel[0] as f32;
                let idx = y * width + x;
                v[idx] = value;
            }
        }

        ImageBuffer::from_vec_as_mode(v, width, height, enums::ImageMode::U16BIT)
    }

    pub fn from_image_u16(image_data:&image::ImageBuffer<image::Rgba<u16>, std::vec::Vec<u16>>) -> error::Result<ImageBuffer> {
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;
        vprintln!("Input image dimensions: {:?}", image_data.dimensions());

        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, 0.0);

        for y in 0..height {
            for x in 0..width {
                let pixel = image_data.get_pixel(x as u32, y as u32);
                let value = pixel[0] as f32;
                let idx = y * width + x;
                v[idx] = value;
            }
        }

        ImageBuffer::from_vec_as_mode(v, width, height, enums::ImageMode::U16BIT)
    }

    pub fn from_file(file_path:&str) -> error::Result<ImageBuffer> {

        if !path::file_exists(file_path) {
            return Err(constants::status::FILE_NOT_FOUND);
        }

        let image_data = open(file_path).unwrap().into_rgba16();
        ImageBuffer::from_image_u16(&image_data)
    }

    fn buffer_to_mask(&self, buffer:&ImageBuffer) -> error::Result<Vec<bool>> {
        if buffer.width != self.width || buffer.height != self.height {
            return Err(constants::status::ARRAY_SIZE_MISMATCH);
        }

        let mut m : Vec<bool> = Vec::with_capacity(self.buffer.len());
        m.resize(self.buffer.len(), false);

        for i in 0..self.buffer.len() {
            m[i] = buffer.buffer[i] > 0.0;
        }

        Ok(m)
    }

    pub fn set_mask(&mut self, mask:&ImageBuffer) {
        self.mask = Some(self.buffer_to_mask(&mask).unwrap());
    }

    pub fn copy_mask_to(&self, dest:&mut ImageBuffer) {
        dest.mask = self.mask.to_owned();
    }

    pub fn clear_mask(&mut self) {
        self.mask = None;
    }

    fn get_mask_at_index(&self, idx:usize) -> error::Result<bool> {
        match &self.mask {
            Some(b) => {
                if idx >= b.len() {
                    return Err(constants::status::INVALID_PIXEL_COORDINATES);
                }
                Ok(b[idx])
            },
            None => Ok(true)
        }
    }

    pub fn get_mask_at_point(&self, x:usize, y:usize) -> error::Result<bool> {
        match &self.mask {
            Some(b) => {
                if x >= self.width || y >= self.height {
                    return Err(constants::status::INVALID_PIXEL_COORDINATES);
                }
                let msk_idx = self.width * y + x;
                Ok(b[msk_idx])
            },
            None => Ok(true)
        }
    }

    pub fn get_slice(&self, top_y:usize, len:usize) -> error::Result<ImageBuffer> {
        let start_index = top_y * self.width;
        let stop_index = (top_y + len) * self.width;

        let slice = self.buffer[start_index..stop_index].to_vec();

        ImageBuffer::from_vec(slice, self.width, len)
    }

    pub fn get_subframe(&self, left_x:usize, top_y:usize, width:usize, height:usize) -> error::Result<ImageBuffer> {

        let subframed_buffer = subframe_array(&self.buffer, self.width, self.height, left_x, top_y, width, height);
        let subframed_mask = match &self.mask {
            Some(m) => Some(subframe_array(&m, self.width, self.height, left_x, top_y, width, height)),
            None => None,
        };
        ImageBuffer::new_from_op(subframed_buffer, width, height, &subframed_mask, self.mode)
    }

    pub fn get(&self, x:usize, y:usize) -> error::Result<f32> {
        if x < self.width && y < self.height {
            if ! self.get_mask_at_point(x, y).unwrap() {
                return Ok(0.0);
            }
            let index = y * self.width + x;
            Ok(self.buffer[index])
        } else {
            Err(constants::status::INVALID_PIXEL_COORDINATES) // TODO: learn to throw exceptions
        }
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn put_u16(&mut self, x:usize, y:usize, val:u16) -> error::Result<&str> {
        self.put(x, y, val as f32)
    }

    pub fn put(&mut self, x:usize, y:usize, val:f32) -> error::Result<&str>{
        if x < self.width && y < self.height {
            if self.get_mask_at_point(x, y).unwrap() {
                let index = y * self.width + x;
                self.buffer[index] = val;
            }
            ok!()
        } else {
            Err(constants::status::INVALID_PIXEL_COORDINATES)
        }
    }

    // Computes the mean of all pixel values
    pub fn mean(&self) -> f32 {

        let mut total:f32 = 0.0;
        let mut count:f32 = 0.0;

        // It is *soooo* inefficient to keep doing this...
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_mask_at_point(x, y).unwrap() {
                    let pixel_value = self.get(x, y).unwrap();
                    if pixel_value > 0.0 {
                        total += pixel_value;
                        count += 1.0;
                    }
                }   
            }
        }

        if count > 0.0 { // Prevent divide-by-zero on fully-masked images
            total / count
        } else {
            0.0
        }
    }

    pub fn divide(&self, other:&ImageBuffer) -> error::Result<ImageBuffer> {

        if self.width != other.width || self.height != other.height {
            return Err(constants::status::ARRAY_SIZE_MISMATCH);
        }

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let quotient = if other.buffer[i] != 0.0 { self.buffer[i] / other.buffer[i] } else { 0.0 };
                v[i] = quotient;
            }
        }

        ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode)
    }

    pub fn divide_into(&self, divisor:f32) -> error::Result<ImageBuffer> {
        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let quotient = if self.buffer[i] != 0.0 { divisor / self.buffer[i] } else { 0.0 };
                v[i] = quotient;
            }
        }

        ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode)
    }

    pub fn scale(&self, scalar:f32) -> error::Result<ImageBuffer> {
        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let product = self.buffer[i] * scalar;
                v[i] = product;
            }
        }

        ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode)
    }

    pub fn multiply(&self, other:&ImageBuffer) -> error::Result<ImageBuffer> {

        if self.width != other.width || self.height != other.height {
            return Err(constants::status::ARRAY_SIZE_MISMATCH);
        }

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let product = self.buffer[i] * other.buffer[i];
                v[i] = product;
            }
        }

        ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode)
    }


    pub fn add_mut(&mut self, other:&ImageBuffer) {
        if self.width != other.width || self.height != other.height {
            panic!("Array size mismatch");
        }

        for i in 0..self.buffer.len() {
            if self.get_mask_at_index(i).unwrap() {
                self.buffer[i] = self.buffer[i] + other.buffer[i];
            }
        }
    }

    pub fn add(&self, other:&ImageBuffer) -> error::Result<ImageBuffer> {

        if self.width != other.width || self.height != other.height {
            return Err(constants::status::ARRAY_SIZE_MISMATCH);
        }

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let result = self.buffer[i] + other.buffer[i];
                v[i] = result;
            }
        }

        ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode)
    }

    pub fn subtract(&self, other:&ImageBuffer) -> error::Result<ImageBuffer> {

        if self.width != other.width || self.height != other.height {
            return Err(constants::status::ARRAY_SIZE_MISMATCH);
        }

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let difference = self.buffer[i] - other.buffer[i];
                v[i] = difference;
            }
        }

        ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode)
    }


    pub fn shift_to_min_zero(&self) -> error::Result<ImageBuffer> {

        let minmax = self.get_min_max().unwrap();

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let value = self.buffer[i];
                if minmax.min < 0.0 {
                    v[i] = value + minmax.min;
                } else {
                    v[i] = value - minmax.min;
                }
            }
        }

        Ok(ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode).unwrap())
    }

    pub fn normalize_force_minmax(&self, min:f32, max:f32, forced_min:f32, forced_max:f32) -> error::Result<ImageBuffer> {
        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let value = ((self.buffer[i] - forced_min) / (forced_max- forced_min)) * (max - min) + min;
                v[i] = value;
            }
        }

        Ok(ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode).unwrap())
    }

    pub fn normalize(&self, min:f32, max:f32) -> error::Result<ImageBuffer> {
        let minmax = self.get_min_max().unwrap();
        self.normalize_force_minmax(min, max, minmax.min, minmax.max)
    }


    pub fn crop(&self, height:usize, width:usize) -> error::Result<ImageBuffer> {
        let cropped_buffer = crop_array(&self.buffer, self.width, self.height, width, height);

        let cropped_mask = match &self.mask {
            Some(m) => Some(crop_array(&m, self.width, self.height, width, height)),
            None => None,
        };
        ImageBuffer::new_from_op(cropped_buffer, width, height, &cropped_mask, self.mode)
    }

    pub fn calc_center_of_mass_offset(&self, threshold:f32) -> error::Result<Offset> {
        let mut ox: f32 = 0.0;
        let mut oy: f32 = 0.0;
        let mut count: u32 = 0;
    
        for y in 0..self.height {
            for x in 0..self.width {
                let val = self.get(x, y).unwrap();
                if val >= threshold {
                    ox = ox + (x as f32);
                    oy = oy + (y as f32);
                    count = count + 1;
                }   
            }
        }
    
        if count > 0 {
            ox = (self.width as f32 / 2.0) - (ox / (count as f32));
            oy = (self.height as f32 / 2.0) - (oy / (count as f32));
        }
    
        Ok(Offset{h:ox.round() as i32, v:oy.round() as i32})
    }

    pub fn paste(&self, src:&ImageBuffer, tl_x:usize, tl_y:usize) -> error::Result<ImageBuffer> {
        let mut new_buffer = ImageBuffer::new_from_op(self.buffer.clone(), self.width, self.height, &self.mask, self.mode).unwrap();
        for y in 0..src.height {
            for x in 0..src.width {

                let dest_x = tl_x + x;
                let dest_y = tl_y + y;

                match new_buffer.put(dest_x, dest_y, src.get(x, y).unwrap()) {
                    Ok(_) => {},
                    Err(_) => {}
                };

            }
        }
        Ok(new_buffer)
    }


    pub fn shift(&self, horiz:i32, vert:i32) -> error::Result<ImageBuffer> {

        let mut shifted_buffer = ImageBuffer::new_with_mask_as_mode(self.width, self.height, &self.mask, self.mode).unwrap();

        let h = self.height as i32;
        let w = self.width as i32;

        for y in 0..h {
            for x in 0..w {
                if self.get_mask_at_point(x as usize, y as usize).unwrap() {
                    let shift_x = x as i32 + horiz;
                    let shift_y = y as i32 + vert;
                
                    if shift_x >= 0 && shift_y >= 0 && shift_x < w  && shift_y < h {
                        shifted_buffer.put(shift_x as usize, shift_y as usize, self.get(x as usize, y as usize).unwrap()).unwrap();
                    }
                }
            }
        }
        Ok(shifted_buffer)
    }

    pub fn get_min_max(&self) -> error::Result<MinMax> {
        
        let mut mx:f32 = std::f32::MIN;
        let mut mn:f32 = std::f32::MAX;

        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_mask_at_point(x, y).unwrap() {
                    let val = self.get(x, y).unwrap() as f32;
                    mx = if val > mx { val } else { mx };
                    mn = if val < mn { val } else { mn };
                }
            }
        }
        
        Ok(MinMax{min:mn, max:mx})
    }

    pub fn buffer_to_image_8bit(&self) -> image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>> {
        let mut out_img = DynamicImage::new_rgba8(self.width as u32, self.height as u32).into_rgba8();
        
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_mask_at_point(x, y).unwrap() {
                    let val = self.get(x, y).unwrap().round() as u8;
                    let a = if self.get_mask_at_point(x, y).unwrap() { 255 } else { 0 };
                    out_img.put_pixel(x as u32, y as u32, Rgba([val, val, val, a]));
                }
            }
        }

        out_img
    }

    pub fn buffer_to_image_16bit(&self) -> image::ImageBuffer<image::Rgba<u16>, std::vec::Vec<u16>>
    {
        let mut out_img = DynamicImage::new_rgba16(self.width as u32, self.height as u32).into_rgba16();
        
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_mask_at_point(x, y).unwrap() {
                    let val = self.get(x, y).unwrap().round() as u16;
                    let a = if self.get_mask_at_point(x, y).unwrap() { 65535 } else { 0 };
                    out_img.put_pixel(x as u32, y as u32, Rgba([val, val, val, a]));
                }
            }
        }

        out_img
    }

    pub fn save_16bit(&self, to_file:&str) -> error::Result<&str> {
        let out_img = self.buffer_to_image_16bit();

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

    pub fn save_8bit(&self, to_file:&str) -> error::Result<&str> {
        let out_img = self.buffer_to_image_8bit();

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

