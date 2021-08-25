

pub mod util {

    // Decimal Degrees = degrees + (minutes/60) + (seconds/3600)
    pub fn hms_to_dd(degrees:f64, minutes:f64, seconds:f64) -> f64 {
        degrees + (minutes / 60.0) + (seconds / 3600.0)
    }

    pub fn radians(d:f64) -> f64 {
        d * std::f64::consts::PI / 180.0
    }
    
    pub fn degrees(r:f64) -> f64 {
        r * 180.0 / std::f64::consts::PI
    }
}  


// Based on calculations found here:
// https://www.petermeadows.com/html/parallactic.html
pub mod parallactic_angle {

    use crate::astro::util;

    // f = observer latitude
    // d = declination of the sun
    // z = zenith distance (altitude = 90 - z)
    // a = azimuth angle of the Sun
    // h = parallactic angle

    // sin(h) = sin(A).cos(f)/cos(d)
    pub fn from_az_dec_and_lat(a:f64, d:f64, f:f64) -> f64 {
        let mut pa = util::degrees((util::radians(a).sin() * util::radians(f).cos() / util::radians(d).cos()).asin());
        if a < 180.0 {
            pa = pa * -1.0;
        }
        pa
    }

    // cos(h) = (sin(f) - sin(d ).cos(z))/(cos(d).sin(z))
    pub fn from_lat_dec_and_zenith(f:f64, d:f64, z:f64, a:f64) -> f64 {
        let mut pa = util::degrees(((util::radians(f).sin() - util::radians(d).sin() * util::radians(z).cos()) / (util::radians(d).cos() * util::radians(z).sin())).acos());
        if a < 180.0 {
            pa = pa * -1.0;
        }
        pa
    }

    // cos(h) = (sin(f).sin(z) - cos(f).cos(z).cos(A))/cos(d)
    pub fn from_lat_zenith_azimuth_dec(f:f64, z:f64, a:f64, d:f64) -> f64 {
        let mut pa = util::degrees(((util::radians(f).sin() * util::radians(z).sin() - util::radians(f).cos() * util::radians(z).cos() * util::radians(a).cos()) / util::radians(d).cos()).acos());
        if a < 180.0 {
            pa = pa * -1.0;
        }
        pa
    }
}