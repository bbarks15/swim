use crate::ast::*;

use std::collections::HashMap;

pub trait Analyse {
    fn total_distance(&self) -> u32;
    fn stroke_distribution(&self) -> HashMap<String, u32>;
}

impl Analyse for Workout {
    fn total_distance(&self) -> u32 {
        self.sets.iter().map(|set| set.total_distance()).sum()
    }

    fn stroke_distribution(&self) -> HashMap<String, u32> {
        let mut distribution = HashMap::new();
        for set in &self.sets {
            let set_dist = set.stroke_distribution();
            for (stroke, distance) in set_dist {
                *distribution.entry(stroke).or_insert(0) += distance;
            }
        }
        distribution
    }
}

impl Analyse for Set {
    fn total_distance(&self) -> u32 {
        match self {
            Set::Repetition { count, set } => count * set.total_distance(),
            Set::Block { sets } => sets.iter().map(|set| set.total_distance()).sum(),
            Set::Statement(stmt) => stmt.total_distance(),
        }
    }

    fn stroke_distribution(&self) -> HashMap<String, u32> {
        match self {
            Set::Repetition { count, set } => {
                let mut dist = set.stroke_distribution();
                for distance in dist.values_mut() {
                    *distance *= count;
                }
                dist
            }
            Set::Block { sets } => {
                let mut distribution = HashMap::new();
                for set in sets {
                    let set_dist = set.stroke_distribution();
                    for (stroke, distance) in set_dist {
                        *distribution.entry(stroke).or_insert(0) += distance;
                    }
                }
                distribution
            }
            Set::Statement(stmt) => stmt.stroke_distribution(),
        }
    }
}

impl Analyse for Statement {
    fn total_distance(&self) -> u32 {
        match self.distance.unit {
            DistanceUnit::Meters => self.distance.value,
            DistanceUnit::Kilometers => self.distance.value * 1000,
        }
    }

    fn stroke_distribution(&self) -> HashMap<String, u32> {
        let mut distribution = HashMap::new();
        let distance = match self.distance.unit {
            DistanceUnit::Meters => self.distance.value,
            DistanceUnit::Kilometers => self.distance.value * 1000,
        };
        distribution.insert(self.stroke.name.clone(), distance);
        distribution
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_simple_distance() {
        let input = "100m fly @ 1:30\n200m free @ 2:00";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        assert_eq!(workout.total_distance(), 300);
    }

    #[test]
    fn test_repetition_distance() {
        let input = "4x50m free @ 60s";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        assert_eq!(workout.total_distance(), 200); // 4 * 50m = 200m
    }

    #[test]
    fn test_complex_distance() {
        let input = "\
            1x100m fly @ 1:30\n\
            50m fly @ 60s\n\
            4x {\n\
              25m choice (easy) @ 60s\n\
              12x50m free @ 60s\n\
            }";

        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        // 100m + 50m + 4 * (25m + 12 * 50m)
        // = 150m + 4 * (25m + 600m)
        // = 150m + 4 * 625m
        // = 150m + 2500m
        // = 2650m
        assert_eq!(workout.total_distance(), 2650);
    }

    #[test]
    fn test_kilometer_distance() {
        let input = "1km free @ 15:00\n500m fly @ 8:00";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        assert_eq!(workout.total_distance(), 1500); // 1km + 500m = 1500m
    }

    #[test]
    fn test_nested_repetition_distance() {
        let input = "3x { 2x100m free @ 1:30 }";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        assert_eq!(workout.total_distance(), 600); // 3 * (2 * 100m) = 600m
    }

    #[test]
    fn test_mixed_units_distance() {
        let input = "2x { 1km free @ 8:00\n100m fly @ 2:00 }";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        // 2 * (1000m + 100m) = 2 * 1100m = 2200m
        assert_eq!(workout.total_distance(), 2200);
    }


    #[test]
    fn test_simple_stroke_distribution() {
        let input = "100m fly @ 1:30\n200m free @ 2:00";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        let distribution = workout.stroke_distribution();
        assert_eq!(distribution.get("fly"), Some(&100));
        assert_eq!(distribution.get("free"), Some(&200));
    }

    #[test]
    fn test_repetition_stroke_distribution() {
        let input = "4x50m free @ 60s";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        let distribution = workout.stroke_distribution();
        assert_eq!(distribution.get("free"), Some(&200)); // 4 * 50m = 200m
    }

    #[test]
    fn test_complex_stroke_distribution() {
        let input = "\
            1x100m fly @ 1:30\n\
            50m fly @ 60s\n\
            4x {\n\
              25m choice (easy) @ 60s\n\
              12x50m free @ 60s\n\
            }";

        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        let distribution = workout.stroke_distribution();
        assert_eq!(distribution.get("fly"), Some(&150)); // 100m + 50m
        assert_eq!(distribution.get("choice"), Some(&100)); // 4 * 25m
        assert_eq!(distribution.get("free"), Some(&2400)); // 4 * 12 * 50m
    }

    #[test]
    fn test_kilometer_stroke_distribution() {
        let input = "1km free @ 15:00\n500m fly @ 8:00";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        let distribution = workout.stroke_distribution();
        assert_eq!(distribution.get("free"), Some(&1000)); // 1km = 1000m
        assert_eq!(distribution.get("fly"), Some(&500)); // 500m
    }
}
