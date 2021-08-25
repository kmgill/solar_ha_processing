
use crate::{
    ser,
    path,
    vprintln,
    imagebuffer,
    constants,
    enums,
    error
};

// Computes a simple mean stack of frames across a list of ser files.
pub fn compute_mean(ser_files:&Vec<&str>, _skip_glitch_frames:bool) -> error::Result<imagebuffer::ImageBuffer> {

    let mut mean_buffer = imagebuffer::ImageBuffer::new_empty().unwrap();

    let mut cnt = 0;

    for ser_file_path in ser_files {
        if ! path::file_exists(ser_file_path) {
            return Err("File not found");
        }

        let ser_file = ser::SerFile::load_ser(ser_file_path).expect("Failed to load SER file");

        if mean_buffer.is_empty() {
            mean_buffer = imagebuffer::ImageBuffer::new_as_mode(
                                                            ser_file.image_width, 
                                                            ser_file.image_height,
                                                            match ser_file.pixel_depth {
                                                                8 => enums::ImageMode::U8BIT,
                                                                _ => enums::ImageMode::U16BIT
                                                            }
                                                        ).expect("Failed to allocate image buffer");
        }

        for i in 0..ser_file.frame_count {
            // TODO: Remove
            // if i >= 10 {
            //     break;
            // }

            let frame = ser_file.get_frame(i).expect("Failed to load image frame");
            let framemm = frame.buffer.get_min_max().unwrap();
            vprintln!("    Frame Min/Max : {}, {}", framemm.min, framemm.max);

            // TODO: Add glitch frame detection

            mean_buffer = mean_buffer.add(&frame.buffer).expect("ImageBuffer addition error");
            cnt = cnt + 1;
        }
    }

    if cnt > 0 {
        mean_buffer = mean_buffer.scale(1.0 / cnt as f32).unwrap();
        let stackmm = mean_buffer.get_min_max().unwrap();
        vprintln!("    Stack Min/Max : {}, {} ({} images)", stackmm.min, stackmm.max, cnt);
        Ok(mean_buffer)
    } else {
        eprintln!("No files used");
        Err("No files used")
    }
}