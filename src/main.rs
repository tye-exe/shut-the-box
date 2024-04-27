use std::io::Write;
use std::ops::Div;
use std::time::SystemTime;
use serde::Serialize;

mod roll;
mod board;
mod playing;

fn main() {
    let before = SystemTime::now();
    playing::compute_weights();

    let duration = SystemTime::now().duration_since(before).unwrap();
    println!("{:?}", duration);
}