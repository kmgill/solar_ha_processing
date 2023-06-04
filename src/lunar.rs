use crate::timestamp;

use astral;

pub fn position_from_lat_lon_and_time(lat: f64, lon: f64, ts: &timestamp::TimeStamp) -> (f64, f64) {
    let unixtime = ts.to_unix_timestamp() as f64;
    info!("Time {:?} converted to unix timestamp {}", ts, unixtime);
    let jd = (unixtime * 1000.0) / astral::util::MILLLISECONDS_IN_DAY - 0.5 + astral::util::J1970;
    let pos = astral::moon::getMoonPosition(jd, lat, lon);

    //(alt, az)
    (pos.alt, pos.az + 90.0)
}
