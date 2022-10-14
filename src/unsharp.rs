use crate::guassianblur::guassian_blur_nband;
use crate::vprintln;
use sciimg::error;
use sciimg::imagebuffer::ImageBuffer;
use sciimg::rgbimage::RgbImage;

pub fn unsharp_mask_nbands(
    buffers: &Vec<ImageBuffer>,
    sigma: f32,
    amount: f32,
) -> error::Result<Vec<ImageBuffer>> {
    match guassian_blur_nband(&buffers, sigma) {
        Ok(blurred) => {
            let mut out_buffers: Vec<ImageBuffer> = vec![];
            for b in 0..blurred.len() {
                out_buffers.push(
                    buffers[b]
                        .add(
                            &buffers[b]
                                .subtract(&blurred[b])
                                .unwrap()
                                .scale(amount)
                                .unwrap(),
                        )
                        .unwrap(),
                );
            }

            Ok(out_buffers)
        }
        Err(why) => Err(why),
    }
}

pub trait RgbImageUnsharpMask {
    fn unsharp_mask(&mut self, sigma: f32, amount: f32);
}

impl RgbImageUnsharpMask for RgbImage {
    fn unsharp_mask(&mut self, sigma: f32, amount: f32) {
        let mut buffers = vec![];
        for b in 0..self.num_bands() {
            buffers.push(self.get_band(b).to_owned());
        }

        match unsharp_mask_nbands(&buffers, sigma, amount) {
            Ok(buffers) => {
                for b in 0..buffers.len() {
                    self.set_band(&buffers[b], b);
                }
            }
            Err(why) => vprintln!("{}", why),
        };
    }
}
