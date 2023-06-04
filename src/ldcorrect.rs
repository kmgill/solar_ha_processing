use crate::point::Point;
use anyhow::{anyhow, Result};
use sciimg::matrix::Matrix;
use sciimg::path;
use sciimg::prelude::*;
use sciimg::vector::Vector;
use sciimg::Dn;
use std::process;

trait AvgValueAtRadius {
    fn avg_value_at_radius(&self, radius: f64, band: usize) -> Dn;
}

impl AvgValueAtRadius for Image {
    fn avg_value_at_radius(&self, radius: f64, band: usize) -> Dn {
        let middle_x = self.width / 2;
        let middle_y = self.height / 2;

        let mut ttl: Dn = 0.0;
        let mut cnt = 0;

        for d in 0..360 {
            let mtx = Matrix::rotate((d as f64).to_radians(), Axis::ZAxis);
            let mut pt_vec = Vector::new(radius, 0.0, 0.0);
            pt_vec = mtx.multiply_vector(&pt_vec);
            let pt = Point {
                x: pt_vec.x as f32 + middle_x as f32,
                y: pt_vec.y as f32 + middle_y as f32,
                valid: true,
            };

            if let Some(v) = pt.get_interpolated_color(self.get_band(band)) {
                ttl += v;
                cnt += 1;
            }
        }
        ttl / cnt as Dn
    }
}

pub fn print_radial_intensities(image_file: &String, radius_pixels: usize) {
    if !path::file_exists(image_file) {
        error!("ERROR: File not found: {}", image_file);
        process::exit(1);
    }

    let img = Image::open(image_file).unwrap();

    let middle_x = img.width / 2;
    let middle_y = img.height / 2;

    if middle_x + radius_pixels > img.width {
        error!(
            "ERROR: Radius {} exceeds image bounds. {}",
            radius_pixels, img.width
        );
        process::exit(3);
    }

    for r_u in 0..radius_pixels {
        let r = r_u as f64;

        let mut ttl: Dn = 0.0;
        let mut cnt = 0;

        let c = r * r * std::f64::consts::PI;

        for d in (0..360).step_by((360.0 / c.floor()) as usize) {
            let mtx = Matrix::rotate((d as f64).to_radians(), Axis::ZAxis);
            let mut pt_vec = Vector::new(r, 0.0, 0.0);
            pt_vec = mtx.multiply_vector(&pt_vec);
            let pt = Point {
                x: pt_vec.x as f32 + middle_x as f32,
                y: pt_vec.y as f32 + middle_y as f32,
                valid: true,
            };

            if let Some(v) = pt.get_interpolated_color(img.get_band(0)) {
                ttl += v;
                cnt += 1;
            }
        }
        let value_at_r = ttl / cnt as Dn;

        println!("{},{}", r, value_at_r);
    }
}

pub fn limb_darkening_correction(
    image_file: &String,
    output_file: &String,
    radius_pixels: usize,
    ld_coefficients: &Vec<f64>,
    composite_gradient_margin: f64,
    inverted_chromosphere: bool,
) -> Result<()> {
    if !path::file_exists(image_file) {
        error!("ERROR: File not found: {}", image_file);
        process::exit(1);
    }

    if !path::parent_exists_and_writable(output_file) {
        error!("ERROR: Output directory not found or is not writable");
        process::exit(2);
    }

    vprintln!("Opening input file: {}", image_file);
    let img = Image::open(image_file).unwrap();

    match limb_darkening_correction_on_image(
        &img,
        radius_pixels,
        ld_coefficients,
        composite_gradient_margin,
        inverted_chromosphere,
    ) {
        Ok(corrected_output) => {
            vprintln!("Writing corrected image to {}", output_file);
            corrected_output
                .save(output_file)
                .expect("Failed to save image");
            Ok(())
        }
        Err(why) => Err(why),
    }
}

pub fn limb_darkening_correction_on_image(
    img: &Image,
    radius_pixels: usize,
    ld_coefficients: &Vec<f64>,
    composite_gradient_margin: f64,
    inverted_chromosphere: bool,
) -> Result<Image> {
    info!(
        "Generating output buffer of size {}x{}",
        img.width, img.height
    );
    //let mut corrected_output = ImageBuffer::new(img.width, img.height).unwrap();

    if ld_coefficients.is_empty()
        || (ld_coefficients.len() > 1 && ld_coefficients.len() != img.num_bands())
    {
        error!("Invalid number of limb darkening coefficients");
        return Err(anyhow!("Invalid number of limb darkening coefficients"));
    }

    let mut corrected_output =
        Image::new_with_bands(img.width, img.height, img.num_bands(), img.get_mode()).unwrap();

    let middle_x = img.width / 2;
    let middle_y = img.height / 2;
    if middle_x + radius_pixels > img.width {
        error!(
            "ERROR: Radius {} exceeds image bounds. {}",
            radius_pixels, img.width
        );
        return Err(anyhow!("Radius exceeds image bounds"));
    }

    let mid_vec = Vector::new(middle_x as f64, middle_y as f64, 0.0);
    let a = radius_pixels as f64;

    let mut center_intensities = vec![0.0, 0.0, 0.0];
    let mut coefficients = if ld_coefficients.len() == 1 {
        let mut ld_coefficients_tmp = ld_coefficients.clone();
        while ld_coefficients_tmp.len() < img.num_bands() {
            ld_coefficients_tmp.push(0.0);
        }
        ld_coefficients_tmp
    } else {
        ld_coefficients.clone()
    };

    // Pixel value at the center
    // Taking the average of intensities out to radius 'n' to
    // compensate for filaments or spicules
    let center_value_radius = 10;

    for band in 0..img.num_bands() {
        for r in 0..center_value_radius {
            center_intensities[band] += img.avg_value_at_radius(r as f64, band) as f64;
        }
        center_intensities[band] /= center_value_radius as f64;
        info!(
            "Center intensity for band #{}: {}",
            band, center_intensities[band]
        );

        let limb_intensity = img.avg_value_at_radius(radius_pixels as f64, band) as Dn;
        info!("Limb intensity for band #{}: {}", band, limb_intensity);
        info!(
            "Computed limb darkening coefficient: {}",
            ((center_intensities[band] as Dn - limb_intensity) / center_intensities[band] as Dn)
        );

        if coefficients[band] == 0.0 {
            coefficients[band] = ((center_intensities[band] as Dn - limb_intensity)
                / center_intensities[band] as Dn) as f64;
        }
    }

    let (_, data_max) = img.get_min_max_all_channel();
    info!("Chromosphere inversion maximum: {}", data_max);

    info!("Computing pixel corrections");
    for y in 0..img.height {
        for x in 0..img.width {
            let p = Vector::new(x as f64, y as f64, 0.0);

            // Radial distance from the center of the disc at the pixel
            let r = mid_vec.distance_to(&p);

            for band in 0..img.num_bands() {
                // Observed value of the pixel
                let i = img.get_band(band).get(x, y);

                let model_intensity = center_intensities[band]
                    * (1.0 - coefficients[band] * (1.0 - ((a * a - r * r) / (a * a)).sqrt()));
                let corrected_u =
                    (center_intensities[band] - model_intensity) / center_intensities[band];
                let corrected = center_intensities[band]
                    * (corrected_u * (1.0 - ((a * a - r * r) / (a * a)).sqrt()))
                    + i as f64;

                let corrected_adjusted = if inverted_chromosphere {
                    data_max - corrected as Dn
                } else {
                    corrected as Dn
                };

                let final_value = if r > radius_pixels as f64 {
                    // If the pixel is outside of the solar radius, we just use
                    // the uncorrected pixel value
                    i as Dn
                } else if radius_pixels as f64 - r <= composite_gradient_margin {
                    // A linear-interpolated gradiant transition
                    let frac =
                        ((radius_pixels as Dn - r as Dn) / composite_gradient_margin as Dn) as Dn;
                    corrected_adjusted as Dn * frac + (1.0 - frac) * i
                } else {
                    // Using the same coefficient for multiple wavelengths is incorrect.
                    corrected_adjusted as Dn
                };

                corrected_output.put(x, y, final_value, band);
            }
        }
    }

    Ok(corrected_output)
}
