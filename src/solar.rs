use crate::timestamp;

pub mod util {

    // Decimal Degrees = degrees + (minutes/60) + (seconds/3600)
    pub fn hms_to_dd(degrees: f64, minutes: f64, seconds: f64) -> f64 {
        degrees + (minutes / 60.0) + (seconds / 3600.0)
    }
}

pub fn position_from_lat_lon_and_time(lat: f64, lon: f64, ts: &timestamp::TimeStamp) -> (f64, f64) {
    let unixtime = ts.to_unix_timestamp();

    info!("Time {:?} converted to unix timestamp {}", ts, unixtime);
    let pos = sun::pos(unixtime * 1000, lat, lon);

    (pos.altitude.to_degrees(), pos.azimuth.to_degrees())
}
