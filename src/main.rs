use std::{thread, time::Duration};

use image::RgbaImage;
use relic::*;
use utils::find_template_coords;
use xcap::image::{GenericImageView, ImageReader};

use ocrs::{ImageSource, OcrEngine, OcrEngineParams, TextLine};
use rten::Model;

mod relic;
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

    let detection_model = Model::load_file("data/text-detection.rten").unwrap();
    let recognition_model = Model::load_file("data/text-recognition.rten").unwrap();

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })
    .unwrap();

    let relic_img = get_relic(window_coords);
    println!("Found relic!");
    let relic = parse_text(&engine, &relic_img);
    println!("Relic: {:?}", relic);
    let mut all_relics = vec![relic.clone()];

    // do all of them
    let mut prev_relic = relic.clone(); 
    loop {
        println!("Move to next relic"); 
        thread::sleep(Duration::from_secs(3));
        let relic_img_loop = get_relic(window_coords);
        let relic = parse_text(&engine, &relic_img_loop);

        if relic == prev_relic {
            println!("Found dupe!");
            break;
        } else {
            prev_relic = relic.clone();
            all_relics.push(relic.clone());
        }
    }

    println!("{:?}", all_relics);
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
    let height = 525;
    let relic = screenshot.view(
        coords.0 + utils::WINDOW_LENGTH - width,
        coords.1 + 100,
        width,
        height,
    );
    relic.to_image().save("target/relics/relic.png").unwrap();
    relic.to_image()
}

fn parse_text(engine: &OcrEngine, relic: &RgbaImage) -> Relic {
    let height = 250;
    let chop_left = 75;
    let bottom_margin = 50;
    let vertical_offset = 15;

    let full_width = relic.width();
    let full_height = relic.height();
    let stat_area_height = full_height - height - bottom_margin - vertical_offset;
    let line_height = stat_area_height / 5; // Since we expect 4 lines

    let top = relic.view(0, 0, relic.width(), height);
    // Crop the entire bottom portion (the stat area)
    let bot = relic
        .view(
            chop_left,
            height + vertical_offset,
            full_width - chop_left,
            stat_area_height,
        )
        .to_image();

    // Now slice into 5 horizontal lines
    let bot_imgs: Vec<RgbaImage> = (0..5)
        .map(|i| {
            bot.view(0, i * line_height, bot.width(), line_height)
                .to_image()
        })
        .collect();
    // bot_imgs.iter().enumerate().for_each(|(i, img)| {
    //     let filename = format!("target/relics/stat_line_{}.png", i);
    //     let _ = img.save(&filename);
    // });

    let top_parsed = get_text(engine, &top.to_image());
    let bot_parsed = bot_imgs
        .iter()
        .map(|img| get_text(engine, &img)[0].clone())
        .collect();
    parse_relic(top_parsed, bot_parsed)
}

fn get_text(engine: &OcrEngine, img: &RgbaImage) -> Vec<ocrs::TextLine> {
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

fn parse_relic(top: Vec<TextLine>, bot: Vec<TextLine>) -> Relic {
    // print!("top: ");
    // top.iter().for_each(|t| println!("{}", t));
    // print!("bot: ");
    // bot.iter().for_each(|t| println!("{}", t));

    let name = top[0].to_string();
    let slot = top
        .iter()
        .filter_map(|s| parse_slot(&s.to_string()))
        .next()
        .expect("No valid slot found in input!");

    let mut stats_iter = bot.iter().filter_map(|t| parse_stat(&t.to_string()));
    let mainstat = stats_iter
        .next()
        .expect("Failed to parse any valid mainstat");
    let substats: Vec<Stat> = stats_iter.collect();

    Relic::new(name, Set::POET, slot, mainstat, substats)
}

fn parse_slot(s: &str) -> Option<Slot> {
    match s {
        "Body" => Some(Slot::BODY),
        "Head" => Some(Slot::HEAD),
        "Hands" => Some(Slot::HANDS),
        "Feet" => Some(Slot::FEET),
        "Link Rope" => Some(Slot::ROPE),
        "Planar Sphere" => Some(Slot::SPHERE),
        _ => None,
    }
}

pub fn parse_stat(s: &str) -> Option<Stat> {
    let normalized = s.trim().replace("Boost", "").to_lowercase();
    let parts: Vec<&str> = normalized.split_whitespace().collect();

    if parts.len() < 2 {
        return None;
    }

    let value_str = parts.last()?.trim_end_matches('%');
    let value: f32 = value_str.parse().ok()?;

    let stat_key = parts[..parts.len() - 1].join(" ");

    match stat_key.as_str() {
        "hp" => {
            if s.contains('%') {
                Some(Stat::HPP(value))
            } else {
                Some(Stat::HP(value as u32))
            }
        }
        "atk" => {
            if s.contains('%') {
                Some(Stat::ATKP(value))
            } else {
                Some(Stat::ATK(value as u32))
            }
        }
        "def" => {
            if s.contains('%') {
                Some(Stat::DEFP(value))
            } else {
                Some(Stat::DEF(value as u32))
            }
        }
        "spd" => Some(Stat::SPD(value as u32)),
        "break effect" => Some(Stat::BE(value)),
        "effect hit rate" => Some(Stat::EHR(value)),
        "effect res" => Some(Stat::ERR(value)),
        "outgoing healing" => Some(Stat::OHB(value)),
        "physical dmg" => Some(Stat::PHYS(value)),
        "fire dmg" => Some(Stat::FIRE(value)),
        "ice dmg" => Some(Stat::ICE(value)),
        "wind dmg" => Some(Stat::WIND(value)),
        "lightning dmg" => Some(Stat::LIGHTNING(value)),
        "quantum dmg" => Some(Stat::QUANTUM(value)),
        "imaginary dmg" => Some(Stat::IMAGINARY(value)),
        "crit rate" => Some(Stat::CR(value)),
        "crit dmg" => Some(Stat::CD(value)),
        _ => {
            println!("stat none: {}", s);
            None
        }
    }
}
