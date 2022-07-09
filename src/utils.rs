use cfg_if::cfg_if;
use image::{EncodableLayout, ImageEncoder, RgbaImage};
use image::codecs::png::PngEncoder;
use rusttype::{Font, point, Scale};
use worker::*;

cfg_if! {
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

pub fn image_response(img: RgbaImage) -> Result<Response> {
    let mut out = vec![];
    let enc = PngEncoder::new(&mut out);
    enc.write_image(
        img.as_bytes(),
        img.width(),
        img.height(),
        image::ColorType::Rgba8,
    ).expect("failed to encode image");

    Response::from_bytes(out.to_vec())
        .map(|mut res| {
            res.headers_mut()
                .set("Content-Type", "image/png")
                .expect("failed to set headers");
            res
        })
}

pub fn string_width(font: &Font, text: &String, scale: Scale) -> f32 {
    font.layout(&*text, scale, point(0.0, 0.0))
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .last()
        .unwrap_or(0.0)
}

pub fn handle_error(error: Error, message: &str, status: u16) -> Result<Response> {
    console_error!("{}", error.to_string());
    Response::error(message, status)
}
