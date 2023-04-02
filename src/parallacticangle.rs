// Based on calculations found here:
// https://www.petermeadows.com/html/parallactic.html

// f = observer latitude
// d = declination of the sun
// z = zenith distance (altitude = 90 - z)
// A = altitude
// a = azimuth angle of the Sun
// h = parallactic angle

// sin(h) = sin(A).cos(f)/cos(d)
pub fn from_az_dec_and_lat(a: f64, d: f64, f: f64) -> f64 {
    let mut pa = (a.to_radians().sin() * f.to_radians().cos() / d.to_radians().cos())
        .asin()
        .to_degrees();
    if a < 180.0 {
        pa *= -1.0;
    }
    pa
}

// cos(h) = (sin(f) - sin(d ).cos(z))/(cos(d).sin(z))
pub fn from_lat_dec_and_zenith(f: f64, d: f64, z: f64, a: f64) -> f64 {
    let mut pa = ((f.to_radians().sin() - d.to_radians().sin() * z.to_radians().cos())
        / (d.to_radians().cos() * z.to_radians().sin()))
    .acos()
    .to_degrees();
    if a < 180.0 {
        pa *= -1.0;
    }
    pa
}

// cos(h) = (sin(f).sin(z) - cos(f).cos(z).cos(A))/cos(d)
pub fn from_lat_zenith_azimuth_dec(f: f64, z: f64, a: f64, d: f64) -> f64 {
    let mut pa = ((f.to_radians().sin() * z.to_radians().sin()
        - f.to_radians().cos() * z.to_radians().cos() * a.to_radians().cos())
        / d.to_radians().cos())
    .acos()
    .to_degrees();
    if a < 180.0 {
        pa *= -1.0;
    }
    pa
}

pub fn from_lat_azimuth_altitude(f: f64, a: f64, al: f64) -> f64 {
    // sin(dec) = sin(lat) * sin(alt) + cos(lat) * cos(alt) * cos(az)
    // sin(d) = sin(f) * sin(A) + cos(f) * cos(A) * cos(a)
    let d = f.to_radians().sin() * al.to_radians().sin()
        + f.to_radians().cos() * al.to_radians().cos() * a.to_radians().cos();
    let z = 90.0 - al;
    let mut pa = ((f.to_radians().sin() * z.to_radians().sin()
        - f.to_radians().cos() * z.to_radians().cos() * a.to_radians().cos())
        / d.to_radians().cos())
    .acos()
    .to_degrees();
    if a < 180.0 {
        pa *= -1.0;
    }
    pa
}
