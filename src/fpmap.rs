use crate::ser;
use anyhow::{anyhow, Result};
use sciimg::path;
use std::collections::HashMap;

/** file pointer map */
pub struct FpMap {
    pub map: HashMap<String, ser::SerFile>,
}

impl Default for FpMap {
    fn default() -> Self {
        Self::new()
    }
}

impl FpMap {
    pub fn new() -> Self {
        FpMap {
            map: HashMap::new(),
        }
    }

    pub fn get_map(&self) -> &HashMap<String, ser::SerFile> {
        &self.map
    }

    pub fn contains(&self, path: &String) -> bool {
        self.map.contains_key(path)
    }

    pub fn get_dont_open(&self, path: &String) -> Option<&ser::SerFile> {
        self.map.get(path)
    }

    pub fn get(&mut self, path: &String) -> Option<&ser::SerFile> {
        if !self.contains(path) {
            match self.open(path) {
                Ok(_) => {}
                Err(e) => {
                    panic!("Failed to open file: {}", e);
                }
            };
        }

        self.map.get(path)
    }

    pub fn open(&mut self, path: &String) -> Result<()> {
        if self.contains(path) {
            return Err(anyhow!("File already open"));
        }

        info!("Opening file in fpmap: {}", path);

        if !path::file_exists(path) {
            panic!("File not found: {}", path);
        }

        match ser::SerFile::load_ser(path) {
            Ok(ser_file) => {
                ser_file.validate();
                self.map.insert(path.clone(), ser_file);
                Ok(())
            }
            Err(e) => Err(anyhow!(e)),
        }
    }
}
