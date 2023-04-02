use std::fs::File;
use std::io::Read;

//use serde_derive::Deserialize;
use serde::Deserialize;

use crate::path;

// threshold = 20560
// sigma_min = 1.23
// sigma_max = 2.0
// top_percent = 80
// latitude = 34.05
// longitude = -118.4955
// frame_limit = 2500
// crop_width = 1200
// crop_height = 1200
// stretch_max = 90
// red_scalar = 1.0
// green_scalar = 1.0
// blue_scalar = 1.0

//

/*
   Used to parse the default/standard parameters toml file (stuff not expected to change from run-to-run)
*/
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Params {
    crop_width: usize,
    crop_height: usize,
    obj_detect_threshold: f32,
    red_scalar: f32,
    green_scalar: f32,
    blue_scalar: f32,
    obs_latitude: f32,
    obs_longitude: f32,
    min_sigma: f32,
    max_sigma: f32,
    pct_of_max: f32,
    number_of_frames: usize,
}

pub fn load_params_mapping_toml(caldata_toml_path: &String) -> Params {
    if !path::file_exists(caldata_toml_path) {
        panic!("File not found: {}", caldata_toml_path);
    }

    let mut file = match File::open(caldata_toml_path) {
        Err(why) => panic!("couldn't open {}", why),
        Ok(file) => file,
    };

    let mut buf: Vec<u8> = Vec::default();
    file.read_to_end(&mut buf).unwrap();
    let toml = String::from_utf8(buf).unwrap();

    let params: Params = toml::from_str(&toml).unwrap();

    params
}
