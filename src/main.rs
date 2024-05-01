use std::{collections::HashMap, time::Duration};

use chumsky::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
enum Set {
    Repetition {
        number: u32,
        set: Box<Set>,
    },
    Statement {
        distance: u32,
        stroke: String,
        interval: u32,
    },
    Block(Vec<Set>),
}

impl Set {
    fn total_distance(&self) -> u32 {
        match self {
            Set::Repetition { number, set } => number * set.total_distance(),
            Set::Statement { distance, .. } => *distance,
            Set::Block(blocks) => blocks.iter().map(|block| block.total_distance()).sum(),
        }
    }

    fn total_duration(&self) -> u32 {
        match self {
            Set::Repetition { number, set } => set.total_duration().checked_mul(*number).unwrap(),
            Set::Statement { interval, .. } => *interval,
            Set::Block(blocks) => blocks.iter().map(|block| block.total_duration()).sum(),
        }
    }

    fn distribution(&self) -> HashMap<&str, u32> {
        let mut dist = HashMap::new();
        distribution_helper(self, &mut dist);
        dist
    }
}

fn distribution_helper<'a>(set: &'a Set, strokes: &mut HashMap<&'a str, u32>) {
    match set {
        Set::Repetition { number, set } => {
            for _ in 0..*number {
                distribution_helper(set, strokes);
            }
        }
        Set::Statement {
            distance, stroke, ..
        } => {
            let count = strokes.entry(stroke).or_insert(0);
            *count += distance;
        }
        Set::Block(blocks) => {
            for block in blocks {
                distribution_helper(block, strokes);
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum Stroke {
    Free,
    Back,
    Breast,
    Fly,
    IM,
    Kick,
    Drill,
    Choice,
}

#[allow(clippy::let_and_return)]
fn parser() -> impl Parser<char, Set, Error = Simple<char>> {
    let distance = text::int(10).then_ignore(just('m'));

    let stroke = filter(|&x| char::is_alphabetic(x) || x.is_ascii_punctuation())
        .repeated()
        .at_least(1)
        .collect::<String>();

    let interval = just("@").padded().ignore_then(
        text::int(10)
            .then_ignore(just('s'))
            .map(|x| x.parse::<u32>().unwrap())
            .or(text::int(10)
                .then_ignore(just(':'))
                .then(text::digits(10))
                .map(|(x, y)| {
                    let minutes: u32 = x.parse().unwrap();
                    let seconds: u32 = y.parse().unwrap();
                    minutes * 60 + seconds
                })),
    );

    let statement = distance
        .then(stroke.padded())
        .then(interval.padded())
        .map(|((distance, stroke), interval)| Set::Statement {
            stroke,
            distance: distance.parse().unwrap(),
            interval,
        })
        .padded();

    let repetition = recursive(|rep| {
        let block = rep
            .or(statement)
            .repeated()
            .map(Set::Block)
            .delimited_by(just("{"), just("}"))
            .padded();

        let repetition = text::int(10)
            .then_ignore(just('x').padded())
            .then(block.or(statement))
            .map(|(reps, set)| Set::Repetition {
                number: reps.parse().unwrap(),
                set: Box::new(set),
            });

        repetition
    });

    repetition.or(statement).repeated().map(Set::Block)
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    match parser().parse(src) {
        Ok(set) => {
            println!("{:#?}", set);
            println!("TOTAL DISTANCE = {}m", set.total_distance());
            println!("DISTRIBUTION = {:#?}", set.distribution());
            println!("DURATION = {:#?}", set.total_duration());
            let serialized = serde_json::to_string(&set).unwrap();
            println!("serialized = {}", serialized);
        }
        Err(parse_errs) => parse_errs
            .into_iter()
            .for_each(|e| println!("Parse error: {}", e)),
    }
}
