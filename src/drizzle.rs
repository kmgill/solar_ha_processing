

use sciimg::prelude::*;
use sciimg::Dn;
use sciimg::imagebuffer::Offset;
use sciimg::vector::Vector;
use sciimg::matrix::Matrix;
use crate::vprintln;

fn round_f64(v:f64) -> f64 {
    (v * 100000.0).round() / 100000.0
}

/// Supported drizzle scalings
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Scale {
    Scale1_0, // No upscaling
    Scale1_5,
    Scale2_0,
    Scale3_0
}

impl Scale {
    pub fn value(&self) -> f32 {
        match *self {
            Scale::Scale1_0 => 1.0,
            Scale::Scale1_5 => 1.5,
            Scale::Scale2_0 => 2.0,
            Scale::Scale3_0 => 3.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct BilinearDrizzle {
    in_width:usize,
    in_height:usize,
    out_width:usize,
    out_height:usize,
    buffer:RgbImage,
    frame_add_count:usize
}

#[derive(Debug)]
struct Point {
    x:f32,
    y:f32,
    valid:bool
}

impl Point {
    pub fn x_frac(&self) -> f32 {
        self.x - self.x.floor()
    }

    pub fn x_fl(&self) -> usize {
        self.x.floor() as usize
    }

    pub fn x_cl(&self) -> usize {
        self.x.ceil() as usize
    }

    pub fn y_frac(&self) -> f32 {
        self.y - self.y.floor()
    }

    pub fn y_fl(&self) -> usize {
        self.y.floor() as usize
    }

    pub fn y_cl(&self) -> usize {
        self.y.ceil() as usize
    }
    

}


impl BilinearDrizzle {

    pub fn new(in_width:usize, in_height:usize, scale:Scale, num_bands:usize) -> BilinearDrizzle {
        let out_width = (in_width as f32 * scale.value()).ceil() as usize;
        let out_height = (in_height as f32 * scale.value()).ceil() as usize;
        BilinearDrizzle { 
            in_width: in_width, 
            in_height: in_height, 
            out_width: out_width, 
            out_height: out_height, 
            frame_add_count: 0,
            buffer:RgbImage::new_with_bands(out_width, out_height, num_bands, ImageMode::U16BIT).expect("Failed to allocate rgbimage")
        }
    }



    /// Convert an x/y point on the drizzle buffer to the respective point on the input buffer
    fn buffer_point_to_input_point(&self, out_x:usize, out_y:usize) -> Point {
        if out_x < self.out_width  && out_y < self.out_height {
            let x = round_f64((out_x as f64 / self.out_width as f64) * self.in_width as f64);
            let y = round_f64((out_y as f64 / self.out_height as f64) * self.in_height as f64);

            Point {
                x:x as f32,
                y:y as f32,
                valid:(x < self.in_width as f64 && y < self.in_height as f64)
            }
        } else {
            Point {
                x:-1.0,
                y:-1.0,
                valid:false
            }
        }
    }

    fn get_interpolated_color(&self, pt:&Point, buffer:&ImageBuffer) -> Option<Dn> {
        // If the rounded-up value of an x or y point exceeds the buffer dimensions, use the floor
        let x_cl = if pt.x_cl() < buffer.width { pt.x_cl() } else { pt.x_fl() };
        let y_cl = if pt.y_cl() < buffer.height { pt.y_cl() } else { pt.y_fl() };

        if x_cl >= buffer.width || y_cl >= buffer.height || pt.x_fl() >= buffer.width || pt.y_fl() >= buffer.height {
            return None;
        }


        let v00 = buffer.get(pt.x_fl(), pt.y_fl()).unwrap();
        let v01 = buffer.get(x_cl, pt.y_fl()).unwrap();
        let v10 = buffer.get(pt.x_fl(), y_cl).unwrap();
        let v11 = buffer.get(x_cl, y_cl).unwrap();

        let yd = pt.y_frac();
        let xd = pt.x_frac();


        // Bilinear interpolation
        let v0 = v10 * yd + v00 * (1.0 - yd);
        let v1 = v11 * yd + v01 * (1.0 - yd);
        let v = v1 * xd + v0 * (1.0 - xd);
        
        Some(v)



    }

    /// Adds an image that has already been translated and rotated but not upscaled.
    pub fn add(&mut self, other:&RgbImage) -> error::Result<&'static str>{
        if other.width != self.in_width || other.height != self.in_height {
            return Err("Input image does not match expected input dimensions");
        }

        for y in 0..self.out_height {
            for x in 0..self.out_width {

                let in_pt = self.buffer_point_to_input_point(x, y);

                if in_pt.valid {

                    
                    for band in 0..other.num_bands() {
                        match self.get_interpolated_color(&in_pt, &other.get_band(band)) {
                            Some(v) => {
                                self.buffer.put(x, 
                                                y, 
                                                v + self.buffer.get_band(0).get(x, y).unwrap(), 
                                                band);

                                // If we're running as a 3 band drizzle buffer and the user passed in a single-band frame
                                if other.num_bands() == 1 {
                                    self.buffer.put(x, y, v + self.buffer.get_band(1).get(x, y).unwrap(), 1);
                                    self.buffer.put(x, y, v + self.buffer.get_band(2).get(x, y).unwrap(), 2);
                                }
                            },
                            None => {}
                        };
                        
                        
                        
                    }

                }

            }
        }
        
        self.frame_add_count += 1;

        Ok("ok")
    }

    // Adds the image. Each pixel point will be transformed by the offset and rotation. Rotation is relative to 
    // the center of mass.
    pub fn add_with_transform(&mut self, other:&RgbImage, offset:Offset, rotation:f64) -> error::Result<&'static str>{
        vprintln!("Adding drizzle frame of offset {:?} and rotation {}", offset, rotation.to_degrees());


        //let mut mtx = Matrix::identity();
        let mtx = Matrix::rotate(rotation, Axis::ZAxis);

        for y in 0..self.out_height {
            for x in 0..self.out_width {
                let mut in_pt = self.buffer_point_to_input_point(x, y);

                let mut pt_vec = Vector::new(
                                in_pt.x as f64 - (other.width / 2) as f64, 
                                in_pt.y as f64 - (other.height / 2) as f64, 
                                0.0);

                pt_vec = mtx.multiply_vector(&pt_vec);
                
                in_pt.x = pt_vec.x as f32 + (other.width / 2) as f32;
                in_pt.y = pt_vec.y as f32 + (other.height / 2) as f32;

                in_pt.x -= offset.h;
                in_pt.y -= offset.v;                

                for band in 0..other.num_bands() {
                    match self.get_interpolated_color(&in_pt, &other.get_band(band)) {
                        Some(v) => {
                            self.buffer.put(x, 
                                            y, 
                                            v + self.buffer.get_band(band).get(x, y).unwrap(), 
                                            band);

                            // If we're running as a 3 band drizzle buffer and the user passed in a single-band frame
                            if other.num_bands() == 1 {
                                self.buffer.put(x, y, v + self.buffer.get_band(1).get(x, y).unwrap(), 1);
                                self.buffer.put(x, y, v + self.buffer.get_band(2).get(x, y).unwrap(), 2);
                            }
                        },
                        None => {}
                    };
                }
            }
        }
        self.frame_add_count += 1;
        Ok("ok")
    }

    pub fn get_finalized(&self) -> error::Result<RgbImage> {

        if self.frame_add_count == 0 {
            Err("No frames have been added, cannot divide mean by zero")
        } else {
            let mut final_buffer = self.buffer.clone();
            for band in 0..final_buffer.num_bands() {
                final_buffer.apply_weight_on_band(1.0 / self.frame_add_count as f32, band);
            }
            Ok(final_buffer)
        }
    }

}