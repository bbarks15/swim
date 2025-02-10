use swim_parser::{analyse::Analyse, parser::Parser};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file>", args[0]);
        std::process::exit(1);
    }

    let set_file = std::fs::read_to_string(&args[1]).unwrap();

    let mut parser = Parser::new(&set_file);

    let workout = parser.parse().unwrap();

    let total_distance = workout.total_distance();
    println!("Total distance: {} meters", total_distance);

    let distribution = workout.stroke_distribution();

    // Print distribution
    for (stroke, distance) in distribution {
        println!("{}: {}m", stroke, distance);
    }
}
