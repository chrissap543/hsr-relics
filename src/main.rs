
mod parser;
mod relic;
mod utils;

fn main() {
    let relics = parser::get_relics();

    let json = serde_json::to_string_pretty(&relics).unwrap();
    println!("{}", json);
}
