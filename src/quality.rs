
use sciimg::{
    stats,
    imagebuffer,
    enums::ImageMode
};

use fastblur::gaussian_blur;


fn buffer_to_vec(image:&imagebuffer::ImageBuffer) -> Vec<[u8; 3]> {
    let mut data: Vec<[u8; 3]> = Vec::with_capacity(image.width * image.height);
    data.resize(image.width * image.height, [0, 0, 0]);

    for y in 0..image.height {
        for x in 0..image.width {
            let r = image.get(x, y).unwrap() as u8;
            let idx = (y * image.width) + x;
            data[idx][0] = r;
            data[idx][1] = r;
            data[idx][2] = r;
        }
    }
    data
}

fn vec_to_buffer(data: Vec<[u8; 3]>, width:usize, height:usize, mask:&Option<Vec<bool>>) -> imagebuffer::ImageBuffer {
    let mut newbuffer = imagebuffer::ImageBuffer::new_with_mask(width, height, &mask).unwrap();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width) + x;
            newbuffer.put(x, y, data[idx][0] as f32);
        }
    }

    newbuffer
}


fn apply_blur(image:&imagebuffer::ImageBuffer, amount:f32) -> imagebuffer::ImageBuffer {

    // fastblur::guassian_blur expects a 3 channel vector of u8
    let mut data: Vec<[u8; 3]> = buffer_to_vec(&image);
    gaussian_blur(&mut data, image.width, image.height, amount);
    vec_to_buffer(data, image.width, image.height, &image.mask)
}


// A very simple image sharpness quantifier that computes the standard deviation of the difference between
// an image and a blurred copy.
pub fn get_quality_estimation(image:&imagebuffer::ImageBuffer) -> f32 {

    let scaled = match image.mode {
        ImageMode::U8BIT => image.clone(),
        ImageMode::U16BIT => {
            let mm = image.get_min_max().unwrap();
            image.normalize_force_minmax(0.0, 255.0, mm.min, mm.max).unwrap()
        },
        _ => panic!("Unexpected bit depth encountered!")
    };

    let blurred = apply_blur(&scaled, 3.5);
    let diff = blurred.subtract(&scaled).unwrap();
    match stats::std_deviation(&diff.buffer[..]) {
        Some(sd) => sd,
        None => 0.0
    }
}