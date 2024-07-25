#![allow(non_snake_case)]

use std::time::SystemTime;
use std::os::raw::c_void;
use std::path::Path;

use gl;
extern crate glfw;

use image;
use image::GenericImage;
use image::DynamicImage::*;


#[allow(dead_code)]
pub fn elapsed(start_time: &SystemTime) -> String {
    let elapsed = start_time.elapsed().unwrap();
    format!("{}s {:.*}ms", elapsed.as_secs(), 1, elapsed.subsec_nanos() as f64 / 1_000_000.0)
}

pub fn loadTexture(path: &str) -> u32 {
    let mut textureID = 0;
    unsafe {
        gl::GenTextures(1, &mut textureID);
        let img = image::open(&Path::new(path)).expect("Texture failed to load");
        let format = match img {
            ImageLuma8(_) => gl::RED,
            ImageLumaA8(_) => gl::RG,
            ImageRgb8(_) => gl::RGB,
            ImageRgba8(_) => gl::RGBA,
        };

        let data = img.raw_pixels();

        gl::BindTexture(gl::TEXTURE_2D, textureID);
        gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, img.width() as i32, img.height() as i32,
        0, format, gl::UNSIGNED_BYTE, &data[0] as *const u8 as *const c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    }
    textureID
}


pub fn char_to_glfw_key(c: char) -> Option<glfw::Key> {
    match c.to_ascii_uppercase() {
        'A' => Some(glfw::Key::A),
        'B' => Some(glfw::Key::B),
        'C' => Some(glfw::Key::C),
        'D' => Some(glfw::Key::D),
        'E' => Some(glfw::Key::E),
        'F' => Some(glfw::Key::F),
        'G' => Some(glfw::Key::G),
        'H' => Some(glfw::Key::H),
        'I' => Some(glfw::Key::I),
        'J' => Some(glfw::Key::J),
        'K' => Some(glfw::Key::K),
        'L' => Some(glfw::Key::L),
        'M' => Some(glfw::Key::M),
        'N' => Some(glfw::Key::N),
        'O' => Some(glfw::Key::O),
        'P' => Some(glfw::Key::P),
        'Q' => Some(glfw::Key::Q),
        'R' => Some(glfw::Key::R),
        'S' => Some(glfw::Key::S),
        'T' => Some(glfw::Key::T),
        'U' => Some(glfw::Key::U),
        'V' => Some(glfw::Key::V),
        'W' => Some(glfw::Key::W),
        'X' => Some(glfw::Key::X),
        'Y' => Some(glfw::Key::Y),
        'Z' => Some(glfw::Key::Z),
        '0' => Some(glfw::Key::Num0),
        '1' => Some(glfw::Key::Num1),
        '2' => Some(glfw::Key::Num2),
        '3' => Some(glfw::Key::Num3),
        '4' => Some(glfw::Key::Num4),
        '5' => Some(glfw::Key::Num5),
        '6' => Some(glfw::Key::Num6),
        '7' => Some(glfw::Key::Num7),
        '8' => Some(glfw::Key::Num8),
        '9' => Some(glfw::Key::Num9),
        _ => None,
    }
}
