
mod parser;
mod relic;
mod utils;
mod optimizer;

use std::{env, fs::File, io::{BufReader, Write}};

use optimizer::optimize;

use crate::relic::Relic;
use crate::optimizer::types::Goal; 

fn main() {
    let args: Vec<String> = env::args().collect(); 

    if args[1] == "collect" {
        let mut output = File::create(&args[2]).expect("Failed to create file."); 
        let relics = parser::get_relics();

        let json = serde_json::to_string_pretty(&relics).unwrap();
        if let Err(e) = output.write_all(json.as_bytes()) {
            eprintln!("Error writing json to file: {}",  e);
        }
    } else {
        let input_relics = File::open(&args[2]).expect("Failed to read file."); 
        let reader = BufReader::new(input_relics);

        let relics: Vec<Relic> = serde_json::from_reader(reader).expect("Failed to parse input relics."); 

        // parse goals
        let goals: Vec<Goal> = vec![]; 

        optimize(&relics, &goals);
    }

}
