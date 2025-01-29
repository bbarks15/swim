use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Workout {
    pub sets: Vec<Set>,
}

/// A single set in the workout
#[derive(Debug, Clone, PartialEq)]
pub enum Set {
    /// A repeated set of exercises
    Repetition { count: u32, set: Box<Set> },
    /// A block containing multiple sets
    Block { sets: Vec<Set> },
    /// A single swimming statement
    Statement(Statement),
}

/// A single swimming statement with distance, stroke, and interval
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub distance: Distance,
    pub stroke: Stroke,
    pub interval: Option<Interval>,
}

/// Distance specification
#[derive(Debug, Clone, PartialEq)]
pub struct Distance {
    pub value: u32,
    pub unit: DistanceUnit,
}

/// Distance units
#[derive(Debug, Clone, PartialEq)]
pub enum DistanceUnit {
    Meters,
    Kilometers,
}

/// Stroke specification with optional modifiers
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub name: String,
    pub modifiers: Vec<String>,
}

/// Interval timing
#[derive(Debug, Clone, PartialEq)]
pub enum Interval {
    /// Simple seconds interval (e.g., @30s)
    Seconds(u32),
    /// Minutes and seconds interval (e.g., @1:30)
    MinutesSeconds { minutes: u32, seconds: u32 },
}

impl fmt::Display for Workout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for set in &self.sets {
            writeln!(f, "{}", set)?;
        }
        Ok(())
    }
}

impl fmt::Display for Set {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Set::Repetition { count, set } => write!(f, "{}x {}", count, set),
            Set::Block { sets } => {
                writeln!(f, "{{")?;
                for set in sets {
                    writeln!(f, "    {}", set)?;
                }
                write!(f, "}}")
            }
            Set::Statement(stmt) => write!(f, "{}", stmt),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.distance, self.stroke)?;
        if let Some(interval) = &self.interval {
            write!(f, " {}", interval)?;
        }
        Ok(())
    }
}

impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.value, self.unit)
    }
}

impl fmt::Display for DistanceUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DistanceUnit::Meters => write!(f, "m"),
            DistanceUnit::Kilometers => write!(f, "km"),
        }
    }
}

impl fmt::Display for Stroke {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.modifiers.is_empty() {
            write!(f, "(")?;
            for (i, modifier) in self.modifiers.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", modifier)?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Interval::Seconds(secs) => write!(f, "@{}s", secs),
            Interval::MinutesSeconds { minutes, seconds } => {
                write!(f, "@{}:{:02}", minutes, seconds)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_construction() {
        // Create a sample workout: 4x { 100m freestyle @1:30, 50m butterfly (drill) @45s }
        let workout = Workout {
            sets: vec![Set::Repetition {
                count: 4,
                set: Box::new(Set::Block {
                    sets: vec![
                        Set::Statement(Statement {
                            distance: Distance {
                                value: 100,
                                unit: DistanceUnit::Meters,
                            },
                            stroke: Stroke {
                                name: "freestyle".to_string(),
                                modifiers: vec![],
                            },
                            interval: Some(Interval::MinutesSeconds {
                                minutes: 1,
                                seconds: 30,
                            }),
                        }),
                        Set::Statement(Statement {
                            distance: Distance {
                                value: 50,
                                unit: DistanceUnit::Meters,
                            },
                            stroke: Stroke {
                                name: "butterfly".to_string(),
                                modifiers: vec!["drill".to_string()],
                            },
                            interval: Some(Interval::Seconds(45)),
                        }),
                    ],
                }),
            }],
        };

        // Test Display implementation
        let output = workout.to_string();
        assert!(output.contains("4x"));
        assert!(output.contains("100m"));
        assert!(output.contains("freestyle"));
        assert!(output.contains("@1:30"));
        assert!(output.contains("50m"));
        assert!(output.contains("butterfly(drill)"));
        assert!(output.contains("@45s"));
    }
}
