mod types;

// imports
use crate::relic::*;
use crate::utils::*;
use types::RelicSetStub;

use image::RgbaImage;
use ocrs::{OcrEngine, OcrEngineParams, TextLine};
use rten::Model;
use std::fs;
use std::{collections::HashMap, thread, time::Duration};
use types::RelicStub;
use xcap::image::{GenericImageView, ImageReader};

const RELIC_SETS_JSON: &str = "data/relic_sets.json";
const RELICS_JSON: &str = "data/relics.json";

pub fn get_relics() -> Vec<Relic> {
    if focus_window("Honkai: Star Rail") {
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

    let name_map = parse_relic_json();

    let relic_img = get_relic_image(window_coords);
    println!("Found relic!");
    println!("Move to next relic: 1");
    let relic = parse_text(&engine, &relic_img, &name_map);
    let mut all_relics = vec![relic.clone()];

    // do all of them
    let mut prev_relic = relic.clone();
    let mut i = 1;
    loop {
        thread::sleep(Duration::from_secs(2));
        let relic_img_loop = get_relic_image(window_coords);
        println!("Move to next relic: {}", i);
        i += 1;
        let relic = parse_text(&engine, &relic_img_loop, &name_map);

        if relic == prev_relic {
            println!("Found dupe!");
            break;
        } else {
            prev_relic = relic.clone();
            all_relics.push(relic.clone());
        }
    }
    all_relics
}

fn find_coords() -> (u32, u32) {
    thread::sleep(Duration::from_millis(500));
    let screenshot = screenshot_window();

    // find corner of window
    let topbar = ImageReader::open("data/inventoryposition.png")
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8();
    find_template_coords(&screenshot, &topbar)
}

fn get_relic_image(coords: (u32, u32)) -> RgbaImage {
    let screenshot = screenshot_window();

    let width = 600;
    let height = 550;
    let relic = screenshot.view(
        coords.0 + WINDOW_LENGTH - width,
        coords.1 + 75,
        width,
        height,
    );
    relic.to_image()
}

fn parse_text(
    engine: &OcrEngine,
    relic: &RgbaImage,
    name_map: &HashMap<Set, Vec<String>>,
) -> Relic {
    let height = 275;
    let chop_left = 75;
    let bottom_margin = 50;
    let vertical_offset = 15;

    let full_width = relic.width();
    let full_height = relic.height();
    let stat_area_height = full_height - height - bottom_margin - vertical_offset;
    let line_height = stat_area_height / 5; // Since we expect 4 lines

    let top = relic.view(0, 0, relic.width(), height);
    // top.to_image().save("target/relics/relic_info.png");

    // Crop the entire bottom portion (the stat area)
    let bot = relic
        .view(
            chop_left,
            height + vertical_offset,
            full_width - chop_left,
            stat_area_height,
        )
        .to_image();
    // bot.save("target/relics/stats.png");

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
    let set_img = relic
        .view(0, full_height - bottom_margin, full_width, bottom_margin)
        .to_image();
    // let _ = set_img.save("target/relics/set.png");
    let set = get_text(engine, &set_img)[0].clone();

    let relic_info = get_text(engine, &top.to_image());
    let stats = bot_imgs
        .iter()
        .map(|img| get_text(engine, &img)[0].clone())
        .collect();
    parse_relic(relic_info, stats, set, name_map)
}

fn parse_relic(
    relic_info: Vec<TextLine>,
    stats: Vec<TextLine>,
    set_text: TextLine,
    name_map: &HashMap<Set, Vec<String>>,
) -> Relic {
    // print!("relic_info: ");
    // relic_info.iter().for_each(|t| println!("{}", t));
    // print!("bot: ");
    // bot.iter().for_each(|t| println!("{}", t));

    let slot = relic_info
        .iter()
        .filter_map(|s| parse_slot(&s.to_string()))
        .next()
        .expect("No valid slot found in input!");

    let mut stats_iter = stats.iter().filter_map(|t| parse_stat(&t.to_string()));
    let mainstat = stats_iter
        .next()
        .expect("Failed to parse any valid mainstat");
    let substats: Vec<Stat> = stats_iter.collect();
    let set = parse_set(&set_text.to_string()).unwrap();
    let name = &name_map[&set][usize::from(slot.clone())];

    Relic::new(name.clone(), set, slot, mainstat, substats)
}

fn parse_slot(s: &str) -> Option<Slot> {
    match s.to_lowercase().as_str() {
        // image parsing
        "body" => Some(Slot::BODY),
        "head" => Some(Slot::HEAD),
        "hands" => Some(Slot::HANDS),
        "feet" => Some(Slot::FEET),
        "link rope" => Some(Slot::ROPE),
        "planar sphere" => Some(Slot::SPHERE),

        // json parsing
        "neck" => Some(Slot::SPHERE),
        "object" => Some(Slot::ROPE),
        "hand" => Some(Slot::HANDS),
        "foot" => Some(Slot::FEET),
        _ => {
            // println!("slot none: {}", s);
            None
        }
    }
}

fn parse_stat(s: &str) -> Option<Stat> {
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
        "effect res" => Some(Stat::EFR(value)),
        "energy regeneration rate" => Some(Stat::ERR(value)),
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

fn parse_set(s: &str) -> Option<Set> {
    let split = s
        .split(|c: char| !c.is_alphanumeric())
        .collect::<Vec<&str>>();
    let first = split[0].to_lowercase();
    match first.as_str() {
        // relic set
        "band" => Some(Set::BAND),
        "champion" => Some(Set::CHAMPION),
        "eagle" => Some(Set::EAGLE),
        "firesmith" => Some(Set::FIRESMITH),
        "genius" => Some(Set::GENIUS),
        "guard" => Some(Set::GUARD),
        "hero" => Some(Set::HERO),
        "hunter" => Some(Set::HUNTER),
        "iron" => Some(Set::IRON),
        "knight" => Some(Set::KNIGHT),
        "longevous" => Some(Set::DISCIPLE),
        "messenger" => Some(Set::MESSENGER),
        "musketeer" => Some(Set::MUSKETEER),
        "passerby" => Some(Set::PASSERBY),
        "pioneer" => Some(Set::PIONEER),
        "poet" => Some(Set::POET),
        "prisoner" => Some(Set::PRISONER),
        "sacerdos" => Some(Set::ORDEAL),
        "scholar" => Some(Set::SCHOLAR),
        "thief" => Some(Set::THIEF),
        "warrior" => Some(Set::GODDESS),
        "wastelander" => Some(Set::WASTELANDER),
        "watchmaker" => Some(Set::WATCHMAKER),
        "wavestrider" => Some(Set::WAVESTRIDER),
        "the" => match split[1].to_lowercase().as_str() {
            "ashblazing" => Some(Set::DUKE),
            "wind" => Some(Set::SOAR),
            "wondrous" => Some(Set::PARK),
            _ => None,
        },
        // planar
        "belobog" => Some(Set::BELOBOG),
        "bone" => Some(Set::BONE),
        "broken" => Some(Set::KEEL),
        "celestial" => Some(Set::CELESTIAL),
        "duran" => Some(Set::DURAN),
        "firmament" => Some(Set::GLAMOTH),
        "fleet" => Some(Set::AGELESS),
        "forge" => Some(Set::FORGE),
        "giant" => Some(Set::TREE),
        "inert" => Some(Set::INERT),
        "izumo" => Some(Set::REALM),
        "lushaka" => Some(Set::SUNKEN),
        "pan" => Some(Set::ENTERPRISE),
        "penacony" => Some(Set::PENACONY),
        "rutilant" => Some(Set::ARENA),
        "sigonia" => Some(Set::DESOLATION),
        "space" => Some(Set::STATION),
        "sprightly" => Some(Set::SPRIGHTLY),
        "talia" => Some(Set::BANDITRY),

        _ => {
            println!("set: {}", s);
            None
        }
    }
}

pub fn parse_relic_json() -> HashMap<Set, Vec<String>> {
    let relic_set_stubs: HashMap<String, RelicSetStub> = serde_json::from_str(
        &fs::read_to_string(RELIC_SETS_JSON).expect("Failed to read relic sets JSON."),
    )
    .expect("Failed to parse JSON");
    let relic_stubs: HashMap<String, RelicStub> = serde_json::from_str(
        &fs::read_to_string(RELICS_JSON).expect("Failed to read relic sets JSON."),
    )
    .expect("Failed to parse JSON");

    let mut set_to_names: HashMap<Set, Vec<String>> = HashMap::new();
    for relic in relic_stubs.values() {
        let set_name = relic_set_stubs
            .get(&relic.set_id)
            .map(|stub| &stub.name)
            .expect("Set ID not found in set stubs");

        let set = parse_set(set_name).expect("Failed to parse set from JSON");
        let slot = parse_slot(&relic.slot).expect("Failed to parse slot from JSON");

        let names = set_to_names
            .entry(set)
            .or_insert_with(|| vec![String::new(); 6]);
        names[usize::from(slot)] = relic.name.clone();
    }

    set_to_names
}
