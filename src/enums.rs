



// Supported instruments
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Camera {
    Asi174MM,
    Undefined
}



#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CalFileType {
    FlatField,
    InpaintMask,
    Mask
}

