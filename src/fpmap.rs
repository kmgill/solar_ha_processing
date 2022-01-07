
use crate::{
    path,
    constants,
    vprintln,
    error,
    ser,
    ok
};

use std::collections::HashMap;

/** file pointer map */
pub struct FpMap {
    map: HashMap<String, ser::SerFile>
}



impl FpMap {

    pub fn new() -> Self {

        FpMap {
            map:HashMap::new()
        }

    }

    pub fn get_map(&self) -> &HashMap<String, ser::SerFile> {
        &self.map
    }

    pub fn contains(&self, path:&String) -> bool {
        self.map.contains_key(path)
    }

    pub fn get_dont_open(&self, path:&String) -> Option<&ser::SerFile> {
        self.map.get(path)
    }

    pub fn get(&mut self, path:&String) -> Option<&ser::SerFile> {
        if ! self.contains(path) {
            match self.open(&path) {
                Ok(_) => {},
                Err(e) => {
                    panic!("Failed to open file: {}", e);
                }
            };
        }

        self.map.get(path)
    }

    pub fn open(&mut self, path:&String) -> error::Result<&str> {

        if self.contains(path) {
            return Err("File already open");
        }

        vprintln!("Opening file in fpmap: {}", path);

        if ! path::file_exists(path) {
            panic!("File not found: {}", path);
        }

        match ser::SerFile::load_ser(&path) {
            Ok(ser_file) => {
                ser_file.validate();
                self.map.insert(path.clone(), ser_file);
                ok!()
            },
            Err(e) => Err(e)
        }     
    }

}