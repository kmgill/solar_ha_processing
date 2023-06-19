use sciimg::prelude::*;
use sciimg::Dn;

#[derive(Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub valid: bool,
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

    pub fn get_interpolated_color(&self, buffer: &ImageBuffer) -> Option<Dn> {
        if self.x < 0.0 || self.y < 0.0 {
            None
        } else {
            // If the rounded-up value of an x or y point exceeds the buffer dimensions, use the floor
            let x_cl = if self.x_cl() < buffer.width {
                self.x_cl()
            } else {
                self.x_fl()
            };
            let y_cl = if self.y_cl() < buffer.height {
                self.y_cl()
            } else {
                self.y_fl()
            };

            if x_cl >= buffer.width
                || y_cl >= buffer.height
                || self.x_fl() >= buffer.width
                || self.y_fl() >= buffer.height
            {
                return None;
            }

            let v00 = buffer.get(self.x_fl(), self.y_fl());
            let v01 = buffer.get(x_cl, self.y_fl());
            let v10 = buffer.get(self.x_fl(), y_cl);
            let v11 = buffer.get(x_cl, y_cl);

            let yd = self.y_frac();
            let xd = self.x_frac();

            // Bilinear interpolation
            let v0 = v10 * yd + v00 * (1.0 - yd);
            let v1 = v11 * yd + v01 * (1.0 - yd);
            let v = v1 * xd + v0 * (1.0 - xd);

            Some(v)
        }
    }
}
