use crossterm::style::Color;
use progrs_bar::Bar;

fn main() {
    println!("HP: {}", Bar::new(50, 1000).generate_string(22, Color::Red));
}
