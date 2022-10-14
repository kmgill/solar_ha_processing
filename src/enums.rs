// Supported instruments
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Camera {
    Asi174MM,
    Undefined,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CalFileType {
    FlatField,
    InpaintMask,
    Mask,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Target {
    Sun,
    Moon,
}

impl Target {
    pub fn from(s: &str) -> Option<Target> {
        match s.to_uppercase().as_str() {
            "MOON" => Some(Target::Moon),
            "SUN" => Some(Target::Sun),
            _ => None,
        }
    }
}
