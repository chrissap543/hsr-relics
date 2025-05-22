use std::{thread, time::Duration};

use image::RgbaImage;
use utils::find_template_coords;
use xcap::image::{GenericImageView, ImageReader};

mod utils;

fn main() {
    if utils::focus_window("Honkai: Star Rail") {
        println!("Found game window!");
    } else {
        eprintln!("Could not find game window!");
        panic!();
    }
    let window_coords = find_coords();
    println!("Found coords: {:?}", window_coords);
    let _ = get_relic(window_coords);
}

fn find_coords() -> (u32, u32) {
    thread::sleep(Duration::from_millis(500));
    let screenshot = utils::screenshot_window();

    // find corner of window
    let topbar = ImageReader::open("data/inventoryposition.png")
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8();
    find_template_coords(&screenshot, &topbar)
}

fn get_relic(coords: (u32, u32)) -> RgbaImage {
    let screenshot = utils::screenshot_window(); 

    let width = 600; 
    let height = 625; 
    let relic = screenshot.view(coords.0 + utils::WINDOW_LENGTH - width, coords.1, width, height);
    relic.to_image().save("target/relics/relic.png").unwrap(); 
    relic.to_image()
}

