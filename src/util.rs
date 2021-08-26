
use crate::{
    path
};

use std::str::FromStr;

#[macro_export]
macro_rules! is_valid_number {
    ($str_val:ident, $type:ident) => {
        $str_val.parse::<$type>().is_ok()
    }
}

pub fn string_is_valid_num<T:FromStr>(s:&str) -> bool {
    let num = s.parse::<T>();
    num.is_ok()
}

pub fn string_is_valid_f64(s:&str) -> bool {
    string_is_valid_num::<f64>(s)
}

pub fn string_is_valid_f32(s:&str) -> bool {
    string_is_valid_num::<f32>(s)
}

pub fn string_is_valid_i32(s:&str) -> bool {
    string_is_valid_num::<i32>(s)
}

pub fn string_is_valid_usize(s:&str) -> bool {
    string_is_valid_num::<usize>(s)
}

pub fn string_is_valid_u8(s:&str) -> bool {
    string_is_valid_num::<u8>(s)
}

pub fn filename_char_at_pos(filename:&str, pos:usize) -> char {
    let bn = path::basename(&filename);
    bn.chars().nth(pos).unwrap()
}




pub fn stringvec(a:&str, b:&str) -> Vec<String> {
    vec![a.to_owned(), b.to_owned()]
}

pub fn stringvec_b(a:&str, b:String) -> Vec<String> {
    vec![a.to_owned(), b]
}



pub fn append_file_name(input_file:&str, append:&str) -> String {
    let append_with_ext = format!("-{}.png", append);
    replace_image_extension(input_file, append_with_ext.as_str())
    // let append_with_ext = format!("-{}.png", append);
    // let out_file = input_file.replace(".png", append_with_ext.as_str())
    //                          .replace(".PNG", append_with_ext.as_str())
    //                          .replace(".jpg", append_with_ext.as_str())
    //                          .replace(".JPG", append_with_ext.as_str())
    //                          .replace(".tif", append_with_ext.as_str())
    //                          .replace(".TIF", append_with_ext.as_str());
    // String::from(out_file)
}

pub fn replace_image_extension(input_file:&str, append:&str) -> String {
    let out_file = input_file.replace(".png", append)
                             .replace(".PNG", append)
                             .replace(".jpg", append)
                             .replace(".JPG", append)
                             .replace(".tif", append)
                             .replace(".TIF", append);
    String::from(out_file)
}