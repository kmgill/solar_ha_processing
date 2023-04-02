use crate::subs::runnable::RunnableSubcommand;

use sciimg::vector::Vector;
use sciimg::{enums::ImageMode, imagebuffer, prelude::ImageBuffer};
use solhat::{medianblur, path, vprintln};

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Create composite", long_about = None)]
pub struct Composite {
    #[clap(long, short, help = "Input image")]
    input: String,

    #[clap(long, short, help = "Chromosphere radius, in pixels")]
    radius: f32,

    #[clap(long, short, help = "Output image")]
    output: String,
}

fn invert_image_buffer(image: &ImageBuffer) -> ImageBuffer {
    let mut inverted = image.clone();

    for y in 0..image.height {
        for x in 0..image.width {
            inverted.put(x, y, 65535.0 - inverted.get(x, y).unwrap());
        }
    }

    inverted
}

fn overlay_image_buffers(bottom: &ImageBuffer, top: &ImageBuffer) -> ImageBuffer {
    if bottom.width != top.width || bottom.height != top.height {
        panic!("Incompatible images");
    }
    let mut overlaid = bottom.clone();

    for y in 0..bottom.height {
        for x in 0..top.width {
            let a = bottom.get(x, y).unwrap();
            let b = top.get(x, y).unwrap();
            let f = if a < 0.5 {
                2.0 * a * b
            } else {
                1.0 - 2.0 * (1.0 - a) * (1.0 - b)
            };
            overlaid.put(x, y, f);
        }
    }
    overlaid
}

impl RunnableSubcommand for Composite {
    fn run(&self) {
        if !path::parent_exists_and_writable(self.output.as_str()) {
            eprintln!(
                "Error: Output parent directory does not exist or is unwritable: {}",
                path::get_parent(self.output.as_str())
            );
            process::exit(2);
        }

        if !path::file_exists(self.input.as_str()) {
            eprintln!("Error: Input file not found: {}", self.input);
            process::exit(2);
        }

        // Open input image
        vprintln!("Opening image at {}", self.input);
        let orig_image =
            imagebuffer::ImageBuffer::from_file(&self.input).expect("Error: failed to load file");

        // Create a median-blurred image, normalized to between 0 and 1
        vprintln!("Applying median blur filter to overlay layer");
        let blurred = medianblur::median_blur(&orig_image, 70)
            .normalize(0.0, 1.0)
            .unwrap();

        // Create an inverted-chromosphere copy, normalize to between 0 and 1
        vprintln!("Inverting chromosphere layer");
        let inverted = invert_image_buffer(&orig_image)
            .normalize(0.0, 1.0)
            .unwrap();

        // Overlay the blurred image over the inverted image
        vprintln!("Overlaying blurred image to chromosphere layer");
        let overlaid = overlay_image_buffers(&inverted, &blurred);

        // Normalize to within 16bit boundaries
        vprintln!("Normalizing to 16 bit boundaries");
        let normalized = overlaid.normalize(0.0, 65535.0).unwrap();

        vprintln!("Compositing chromosphere and prominence layers");
        let middle_x = orig_image.width / 2;
        let middle_y = orig_image.height / 2;

        let mid_vec = Vector::new(middle_x as f64, middle_y as f64, 0.0);

        let border_blur_margin = 5.0;
        let mut composited = orig_image.clone();
        for y in 0..orig_image.height {
            for x in 0..orig_image.width {
                let p = Vector::new(x as f64, y as f64, 0.0);

                // Radial distance from the center of the disc at the pixel
                let r = mid_vec.distance_to(&p);

                if r <= self.radius as f64 {
                    let a = composited.get(x, y).unwrap();
                    let b = normalized.get(x, y).unwrap();

                    let f = if self.radius as f64 - r <= border_blur_margin {
                        let frac = ((self.radius as f64 - r) / border_blur_margin) as f32;
                        b * frac + (1.0 - frac) * a
                    } else {
                        b
                    };

                    composited.put(x, y, f);
                }
            }
        }

        // Save resulting image to disk
        vprintln!("Saving image to {}", self.output);
        composited.save(self.output.as_str(), ImageMode::U16BIT);
    }
}
