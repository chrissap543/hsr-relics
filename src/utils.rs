use image::{DynamicImage, RgbaImage};
use ocrs::{ImageSource, OcrEngine};
use std::borrow::Cow;
use std::ffi::CString;
use template_matching::{find_extremes, match_template, Image as TMImage, MatchTemplateMethod};
use windows::{core::PCSTR, Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};
use xcap::Monitor;

pub const WINDOW_LENGTH: u32 = 1920;
pub const WINDOW_HEIGHT: u32 = 1080;

pub fn screenshot_window() -> RgbaImage {
    let monitor = Monitor::all().unwrap()[0].clone();
    monitor.capture_image().unwrap()
}
pub fn focus_window(title: &str) -> bool {
    let title_c = CString::new(title).unwrap();
    unsafe {
        let hwnd = FindWindowA(None, PCSTR(title_c.as_ptr() as _)).unwrap();
        if hwnd == HWND(std::ptr::null_mut()) {
            eprintln!("Window not found");
            return false;
        }
        let _ = ShowWindow(hwnd, SW_RESTORE); // Restore if minimized
        let _ = SetForegroundWindow(hwnd); // Bring to front
        true
    }
}

fn convert_to_f32_luma(img: &image::GrayImage) -> TMImage {
    let (width, height) = img.dimensions();
    let buffer = img.pixels().map(|p| p[0] as f32).collect::<Vec<f32>>();
    TMImage::new(Cow::Owned(buffer), width, height)
}

pub fn find_template_coords(full: &RgbaImage, template: &RgbaImage) -> (u32, u32) {
    let full_img = DynamicImage::ImageRgba8(full.clone()).to_luma8();
    let template_img = DynamicImage::ImageRgba8(template.clone()).to_luma8();

    let full_f32 = convert_to_f32_luma(&full_img);
    let template_f32 = convert_to_f32_luma(&template_img);

    let result = match_template(
        full_f32,
        template_f32,
        MatchTemplateMethod::SumOfSquaredDifferences,
    );
    let extremes = find_extremes(&result);

    (
        extremes.min_value_location.0 as u32,
        extremes.min_value_location.1 as u32,
    )
}

pub fn get_text(engine: &OcrEngine, img: &RgbaImage) -> Vec<ocrs::TextLine> {
    let img_source = ImageSource::from_bytes(img.as_raw(), img.dimensions()).unwrap();
    let ocr_input = engine.prepare_input(img_source).unwrap();

    let word_rects = engine.detect_words(&ocr_input).unwrap();
    // Group words into lines. Each line is represented by a list of word
    // bounding boxes.
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

    // Recognize the characters in each line.
    let line_texts = engine.recognize_text(&ocr_input, &line_rects).unwrap();

    line_texts
        .iter()
        .flatten()
        .filter(|l| l.to_string().len() > 1)
        .map(|x| x.clone())
        .collect()
}
