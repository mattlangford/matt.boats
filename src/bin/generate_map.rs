use std::env;

use std::fs;
use std::io::{BufRead, BufReader};

fn split_line(mut line: String) -> [f32; 2] {
    let error = format!("Can't parse '{}' into [f32; 2]", line);
    line = line.replace(" ", "");
    let mut res = line.split("\t").map(|s| s.parse::<f32>().expect(&error));
    [res.next().expect(&error), res.next().expect(&error)]
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        println!("Invoke with <input_GSHHS_filename> <output_bin_filename>");
        std::process::exit(1);
    }
    let in_filename = &args[1];
    let in_file = fs::File::open(in_filename).expect(&format!("Unable to read {}", in_filename));
    let reader = BufReader::new(in_file);

    const HEADER_SIZE: usize = 3;
    let data = reader
        .lines()
        .filter_map(|result| result.ok())
        .skip(HEADER_SIZE)
        .flat_map(split_line)
        .collect::<Vec<f32>>();

    let out_filename = &args[2];
    let serialized = bincode::serialize(&data).expect("Unable to serialize matrix data.");
    std::fs::write(out_filename, serialized).expect(&format!(
        "Unable to write data to output file: {}",
        out_filename
    ));
    println!(
        "Read file from '{}' and wrote to '{}' successfully!",
        in_filename, out_filename
    );
}
