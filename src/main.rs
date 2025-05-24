use std::fs;

mod parser;
mod relic;
mod utils;

fn main() {
    // let relics = parser::get_relics();

    // let json = serde_json::to_string_pretty(&relics).unwrap();
    // println!("{}", json);

    let relic_sets = "data/relic_sets.json"; 
    let relics = "data/relics.json"; 
    parser::parse_relic_json(fs::read_to_string(relic_sets).unwrap(), fs::read_to_string(relics).unwrap());
}
