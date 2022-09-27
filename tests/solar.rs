

use solhat::{
    solar::util,
    parallacticangle
};


// https://stackoverflow.com/questions/30856285/assert-eq-with-floating-point-numbers-and-delta
macro_rules! assert_delta {
    ($x:expr, $y:expr, $d:expr) => {
        if !($x - $y < $d || $y - $x < $d) { panic!(); }
    }
}

#[test]
fn test_parallactic_angle_1() {
    let lat = 34.0522;
    //let lon = -118.2437;

    let dec = util::hms_to_dd(10.0, 28.0, 45.0);
    let alt = 63.29;
    let azi = 148.73;

    do_test_parallactic_angle_methods(lat, dec, alt, azi, -25.94);
}

#[test]
fn test_parallactic_angle_2() {
    let lat = 34.0522;
    //let lon = -118.2437;

    let dec = util::hms_to_dd(10.0, 27.0, 58.0);
    let alt = 66.41;
    let azi = 179.45;

    do_test_parallactic_angle_methods(lat, dec, alt, azi, -0.46);
}

#[test]
fn test_parallactic_angle_3() {
    let lat = 34.0522;
    //let lon = -118.2437;

    let dec = util::hms_to_dd(-20.0, 55.0, 31.0);
    let alt = 18.25;
    let azi = 227.41;

    do_test_parallactic_angle_methods(lat, dec, alt, azi, 40.77);
}

#[test]
fn test_parallactic_angle_4() {
    let lat = -34.0522;
    //let lon = -118.2437;

    let dec = util::hms_to_dd(-20.0, 55.0, 31.0);
    let alt = 45.49;
    let azi = 274.16;

    do_test_parallactic_angle_methods(lat, dec, alt, azi, 117.79);
}

fn do_test_parallactic_angle_methods(lat:f64, dec:f64, alt:f64, azi:f64, expected:f64) {
    let zenith = 90.0 - alt;

    assert_delta!(parallacticangle::from_az_dec_and_lat(azi, dec, lat), expected, 0.01);
    assert_delta!(parallacticangle::from_lat_dec_and_zenith(lat, dec, zenith, azi), expected, 0.01);
    assert_delta!(parallacticangle::from_lat_zenith_azimuth_dec(lat, zenith, azi, dec), expected, 0.01);
    assert_delta!(parallacticangle::from_lat_azimuth_altitude(lat, azi, alt), expected, 0.01);
}