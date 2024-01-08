use find_peaks::PeakFinder;

use std::fs::File;
use std::io::prelude::*;

fn read_file(path: &str) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn main() {
    let data: Vec<f64> = read_file("data/spectrum.dat")
        .expect("File not read!")
        .as_str()
        .split_whitespace()
        .map(|x| x.parse::<f64>().unwrap())
        .collect();

    let mut fp = PeakFinder::new(&data);
    fp.with_min_prominence(200.);
    fp.with_min_height(0.);

    let peaks = fp.find_peaks();
    for p in peaks {
        println!("{} {}", p.middle_position(), p.height.unwrap());
    }
}
