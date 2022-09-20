use sciimg::prelude::*;
use sciimg::path;
use sciimg::vector::Vector;
use sciimg::matrix::Matrix;
use sciimg::Dn;
use crate::point::Point;
use crate::vprintln;

use std::process;

static LIMB_EXTENSION_MARGIN:usize = 3;

trait AvgValueAtRadius {
    fn avg_value_at_radius(&self, radius:f64) -> Dn;
}

impl AvgValueAtRadius for RgbImage {
    fn avg_value_at_radius(&self, radius:f64) -> Dn {
        let middle_x = self.width / 2;
        let middle_y = self.height / 2;

        let mut ttl:Dn = 0.0;
        let mut cnt = 0;

        for d in 0..360 {
            
            let mtx = Matrix::rotate((d as f64).to_radians(), Axis::ZAxis);
            let mut pt_vec = Vector::new(
                                                radius as f64, 
                                                0.0, 
                                                0.0);
            pt_vec = mtx.multiply_vector(&pt_vec);
            let pt = Point { 
                x: pt_vec.x as f32 + middle_x as f32, 
                y: pt_vec.y as f32 + middle_y as f32, 
                valid: true 
            };

            match pt.get_interpolated_color(self.get_band(0)) {
                Some(v) => {
                    ttl += v;
                    cnt += 1;
                },
                None => {}
            };

        }
        let value_at_r = ttl / cnt as Dn;
        value_at_r
    }
}

pub fn print_radial_intensities(image_file:&String, radius_pixels:usize) {

    if ! path::file_exists(image_file) {
        eprintln!("ERROR: File not found: {}", image_file);
        process::exit(1);
    }

    let img = RgbImage::open16(image_file).unwrap();

    let middle_x = img.width / 2;
    let middle_y = img.height / 2;

    if middle_x + radius_pixels > img.width {
        eprintln!("ERROR: Radius {} exceeds image bounds. {}", radius_pixels, img.width);
        process::exit(3);
    }

    for r_u in 0..radius_pixels {
        let r = r_u as f64;

        let mut ttl:Dn = 0.0;
        let mut cnt = 0;

        let c = r*r*std::f64::consts::PI;

        for d in (0..360).step_by((360.0 / c.floor()) as usize) {
            
            let mtx = Matrix::rotate((d as f64).to_radians(), Axis::ZAxis);
            let mut pt_vec = Vector::new(
                                                r, 
                                                0.0, 
                                                0.0);
            pt_vec = mtx.multiply_vector(&pt_vec);
            let pt = Point { 
                x: pt_vec.x as f32 + middle_x as f32, 
                y: pt_vec.y as f32 + middle_y as f32, 
                valid: true 
            };

            match pt.get_interpolated_color(img.get_band(0)) {
                Some(v) => {
                    ttl += v;
                    cnt += 1;
                },
                None => {}
            };

        }
        let value_at_r = ttl / cnt as Dn;

        println!("{},{}", r, value_at_r);

    }
}


pub fn limb_darkening_correction(image_file:&String, output_file:&String, radius_pixels:usize, ld_coefficient:f64) {
    if ! path::file_exists(image_file) {
        eprintln!("ERROR: File not found: {}", image_file);
        process::exit(1);
    }

    if ! path::parent_exists_and_writable(output_file) {
        eprintln!("ERROR: Output directory not found or is not writable");
        process::exit(2);
    }

    vprintln!("Opening input file: {}", image_file);
    let img = RgbImage::open16(image_file).unwrap();

    let corrected_output = limb_darkening_correction_on_image(&img, radius_pixels, ld_coefficient);

    vprintln!("Writing corrected image to {}", output_file);
    corrected_output.save(output_file); 
}  


pub fn limb_darkening_correction_on_image(img:&RgbImage, radius_pixels:usize, ld_coefficient:f64) -> RgbImage {

    vprintln!("Generating output buffer of size {}x{}", img.width, img.height);
    let mut corrected_output = ImageBuffer::new(img.width, img.height).unwrap();

    let middle_x = img.width / 2;
    let middle_y = img.height / 2;
    if middle_x + radius_pixels > img.width {
        eprintln!("ERROR: Radius {} exceeds image bounds. {}", radius_pixels, img.width);
        process::exit(3);
    }

    vprintln!("Computing radial averages...");
    let mut radial_averages:Vec<Dn> = vec!();
    for r in 0..radius_pixels+LIMB_EXTENSION_MARGIN {
        radial_averages.push(img.avg_value_at_radius(r as f64));
    }
    
    // Pixel value at the center
    // Taking the average of intensities out to radius 'n' to 
    // compensate for filaments or spicules
    let center_value_radius = 10;
    let mut i_c = 0.0;
    for r in 0..center_value_radius {
        i_c += radial_averages[r] as f64;
    }
    i_c /= center_value_radius as f64;
    
    let mid_vec = Vector::new(middle_x as f64, middle_y as f64, 0.0);

    let a = radius_pixels as f64;

    vprintln!("Computed limb darkening coefficient: {}", (i_c as Dn - radial_averages[radial_averages.len() - 1 - LIMB_EXTENSION_MARGIN]) / i_c as Dn);

    // Compute radial coefficient if caller passed in zero
    let u = if ld_coefficient == 0.0 {
        (i_c - radial_averages[radial_averages.len() - 1 - LIMB_EXTENSION_MARGIN]as f64) / i_c
    } else {
        ld_coefficient
    };
    
    vprintln!("Computing pixel corrections");
    for y in 0..img.height {
        for x in 0..img.width {

            let p = Vector::new(x as f64, y as f64, 0.0);

            // Radial distance from the center of the disc at the pixel
            let r = mid_vec.distance_to(&p);

            // Observed value of the pixel
            let i = img.get_band(0).get(x, y).unwrap();

            if r > radius_pixels as f64 {

                // If the pixel is outside of the solar radius, we just use
                // the uncorrected pixel value
                corrected_output.put(x, y, i);

            } else {
                
                let model_intensity = i_c * (1.0 - u * (1.0 - ( (a*a - r*r) / (a*a)  ).sqrt()));
                let corrected_u = (i_c - model_intensity) / i_c;
                let corrected = i_c * (corrected_u * (1.0 - ( (a*a - r*r) / (a*a)  ).sqrt())) + i as f64;

                corrected_output.put(x, y, corrected as Dn);
            }
        }
    }

    RgbImage::new_from_buffers_rgb(&corrected_output, &corrected_output, &corrected_output, ImageMode::U16BIT).unwrap()

}