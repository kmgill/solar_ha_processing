use sciimg::prelude::*;
use sciimg::Dn;

pub fn threshtest(frame: &Image, threshold: Dn) -> ImageBuffer {
    info!(
        "Creating test visualization buffer of size {}x{}",
        frame.width, frame.height
    );
    let mut out_img = ImageBuffer::new_with_fill(frame.width, frame.height, 0.0).unwrap();

    info!("Checking threshold value {} across first frame", threshold);
    for y in 0..frame.height {
        for x in 0..frame.width {
            let mut v = 0.0;
            for b in 0..frame.num_bands() {
                v += frame.get_band(b).get(x, y);
            }
            v /= frame.num_bands() as Dn;
            if v > threshold {
                out_img.put(x, y, 65535.0);
            } else {
                out_img.put(x, y, frame.get_band(0).get(x, y));
            }
        }
    }
    out_img.normalize_mut(0.0, 255.0);
    out_img
}
