use rand::Rng;
use std::collections::HashMap;
use std::fmt;
use strum_macros::EnumString;

#[derive(Debug, PartialEq, Eq)]
pub enum Week {
    Week1,
    Week2,
    Week3,
    Week4,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, EnumString)]
pub enum Lift {
    /* Primary */
    #[strum(serialize = "squat", serialize = "s")]
    Squat,
    #[strum(serialize = "bench_press", serialize = "b", serialize = "bp")]
    BenchPress,
    #[strum(serialize = "deadlift", serialize = "d", serialize = "dl")]
    Deadlift,
    #[strum(
        serialize = "overhead_press",
        serialize = "o",
        serialize = "p",
        serialize = "ohp"
    )]
    OverheadPress,

    /* Major assistance */
    // squat-like
    #[strum(serialize = "front_squat", serialize = "fs")]
    FrontSquat,
    #[strum(serialize = "overhead_squat", serialize = "os", serialize = "ohs")]
    OverheadSquat,
    #[strum(serialize = "bulgarian_split_squat", serialize = "bss")]
    BulgarianSplitSquat,
    // deadlift-like
    #[strum(serialize = "good_morning", serialize = "gm")]
    GoodMorning,
    #[strum(serialize = "straight_leg_deadlift", serialize = "sldl")]
    StraightLegDeadlift,
    #[strum(serialize = "romanian_deadlift", serialize = "rdl")]
    RomanianDeadlift,
    #[strum(serialize = "rack_deadlift", serialize = "radl")]
    RackDeadlift,
    #[strum(serialize = "power_clean", serialize = "pc")]
    PowerClean,
    #[strum(serialize = "power_snatch", serialize = "ps")]
    PowerSnatch,
    // bench press-like
    #[strum(serialize = "close_grip_bench_press", serialize = "cgbp")]
    CloseGripBenchPress,
    // overhead press-like
    #[strum(serialize = "incline_press", serialize = "ip")]
    InclinePress,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WorkoutError {
    Config(String),
    MissingTrainingMax { lift: Lift },
}

impl fmt::Display for WorkoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkoutError::Config(message) => write!(f, "{}", message),
            WorkoutError::MissingTrainingMax { lift } => {
                write!(f, "Missing training max for {}", lift)
            }
        }
    }
}

impl std::error::Error for WorkoutError {}

impl fmt::Display for Lift {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: &str = match self {
            Lift::Squat => "squat",
            Lift::BenchPress => "bench press",
            Lift::Deadlift => "deadlift",
            Lift::OverheadPress => "overhead press",
            Lift::FrontSquat => "front squat",
            Lift::OverheadSquat => "overhead squat",
            Lift::PowerClean => "power clean",
            Lift::PowerSnatch => "power snatch",
            Lift::BulgarianSplitSquat => "bulgarian split squat",
            Lift::GoodMorning => "good morning",
            Lift::StraightLegDeadlift => "straight leg deadlift",
            Lift::RomanianDeadlift => "romanian deadlift",
            Lift::RackDeadlift => "rack deadlift",
            Lift::CloseGripBenchPress => "close grip bench press",
            Lift::InclinePress => "incline press",
        };
        write!(f, "{}", s)
    }
}

/// A block of identical sets for a lift
pub struct SetGroup {
    lift: Lift,
    weight: i16,
    sets: i8,
    reps: i8,
    amrap: bool,
}

impl fmt::Display for SetGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // e.g. "squat 315 "
        let mut s = self.lift.to_string() + " " + &self.weight.to_string() + " ";

        // "...3"?
        if self.sets > 1 {
            s += &self.sets.to_string();
        }

        // "...x5"
        s += "x";
        s += &self.reps.to_string();

        // "...+"?
        if self.amrap {
            s += "+";
        }

        write!(f, "{}", s)
    }
}

/// Scales integer weight by floating point multiplier and converts back to integer weight.
pub fn scale(weight: i16, scale: f32) -> i16 {
    return (weight as f32 * scale).round() as i16;
}

/// Primary lift set generator
pub fn generate_primary_sets(
    lift: &Lift,
    week: &Week,
    training_maxes: &HashMap<Lift, i16>,
) -> Result<Vec<String>, WorkoutError> {
    let mut ret = vec![];
    let training_max = *training_maxes
        .get(lift)
        .ok_or(WorkoutError::MissingTrainingMax { lift: *lift })?;

    let make_set_str = |scalar: f32, sets: i8, reps: i8, amrap: bool| -> String {
        SetGroup {
            lift: *lift,
            weight: scale(training_max, scalar),
            sets,
            reps,
            amrap,
        }
        .to_string()
    };

    // warm-up sets
    // no warm-up needed for deload week
    if week != &Week::Week4 {
        ret.push(make_set_str(0.4, 1, 5, false));
        ret.push(make_set_str(0.5, 1, 5, false));
        // for week 1, the 60% warm-up is too close to the first working set at
        // 65% to be helpful
        if week != &Week::Week1 {
            ret.push(make_set_str(0.6, 1, 3, false));
        }
    }

    // working sets
    match week {
        Week::Week1 => {
            ret.push(make_set_str(0.65, 1, 5, false));
            ret.push(make_set_str(0.75, 1, 5, false));
            ret.push(make_set_str(0.85, 1, 5, true));
        }
        Week::Week2 => {
            ret.push(make_set_str(0.7, 1, 3, false));
            ret.push(make_set_str(0.8, 1, 3, false));
            ret.push(make_set_str(0.9, 1, 3, true));
        }
        Week::Week3 => {
            ret.push(make_set_str(0.75, 1, 5, false));
            ret.push(make_set_str(0.85, 1, 3, false));
            ret.push(make_set_str(0.95, 1, 1, true));
        }
        Week::Week4 => {
            ret.push(make_set_str(0.4, 1, 5, false));
            ret.push(make_set_str(0.5, 1, 5, false));
            ret.push(make_set_str(0.6, 1, 5, false));
        }
    }

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::HashMap;

    fn baseline_training_maxes() -> HashMap<Lift, i16> {
        let mut map = HashMap::new();
        map.insert(Lift::Squat, 325);
        map.insert(Lift::BenchPress, 235);
        map.insert(Lift::Deadlift, 365);
        map.insert(Lift::OverheadPress, 170);
        map.insert(Lift::PowerClean, 205);
        map.insert(Lift::FrontSquat, 215);
        map.insert(Lift::InclinePress, 215);
        map.insert(Lift::CloseGripBenchPress, 215);
        map
    }

    #[test]
    fn generates_expected_week_one_primary_sets() {
        let training_maxes = baseline_training_maxes();
        let sets = generate_primary_sets(&Lift::Squat, &Week::Week1, &training_maxes).unwrap();
        assert_eq!(
            sets,
            vec![
                "squat 130 x5",
                "squat 163 x5",
                "squat 211 x5",
                "squat 244 x5",
                "squat 276 x5+",
            ]
        );
    }

    #[test]
    fn assistance_sets_require_training_max() {
        let mut training_maxes = HashMap::new();
        training_maxes.insert(Lift::Squat, 325);
        let mut rng = StdRng::seed_from_u64(1);
        let err = generate_assistance_sets(&Lift::Squat, &Week::Week1, &training_maxes, &mut rng)
            .unwrap_err();
        assert_eq!(
            err,
            WorkoutError::MissingTrainingMax {
                lift: Lift::PowerClean
            }
        );
    }

    #[test]
    fn assistance_sets_are_deterministic_with_seed() {
        let training_maxes = baseline_training_maxes();
        let mut rng_a = StdRng::seed_from_u64(42);
        let mut rng_b = StdRng::seed_from_u64(42);
        let sets_a =
            generate_assistance_sets(&Lift::BenchPress, &Week::Week2, &training_maxes, &mut rng_a)
                .unwrap();
        let sets_b =
            generate_assistance_sets(&Lift::BenchPress, &Week::Week2, &training_maxes, &mut rng_b)
                .unwrap();
        assert_eq!(sets_a, sets_b);
    }
}

/// Simplest strength template (SST) set generator
pub fn generate_assistance_sets(
    primary_lift: &Lift,
    week: &Week,
    training_maxes: &HashMap<Lift, i16>,
    rng: &mut impl Rng,
) -> Result<Vec<String>, WorkoutError> {
    let mut ret = vec![];

    let make_set_str = |lift: Lift, scalar: f32, sets: i8, reps: i8| -> Result<String, WorkoutError> {
        let training_max = *training_maxes
            .get(&lift)
            .ok_or(WorkoutError::MissingTrainingMax { lift })?;
        Ok(SetGroup {
            lift,
            weight: scale(training_max, scalar),
            sets,
            reps,
            amrap: false,
        }
        .to_string())
    };

    // big assistance
    let big_assistance_lift = match primary_lift {
        Lift::Squat => Lift::PowerClean,
        Lift::Deadlift => Lift::FrontSquat,
        Lift::BenchPress => Lift::InclinePress,
        Lift::OverheadPress => Lift::CloseGripBenchPress,
        _ => {
            return Err(WorkoutError::Config(format!(
                "Unsupported primary lift {}",
                primary_lift
            )))
        }
    };
    match (big_assistance_lift, week) {
        (Lift::PowerClean, Week::Week4) => {
            ret.push(make_set_str(big_assistance_lift, 0.5, 1, 3)?);
            ret.push(make_set_str(big_assistance_lift, 0.6, 1, 3)?);
            ret.push(make_set_str(big_assistance_lift, 0.7, 1, 3)?);
        }
        (Lift::PowerClean, _) => {
            ret.push(make_set_str(big_assistance_lift, 0.65, 1, 3)?);
            ret.push(make_set_str(big_assistance_lift, 0.75, 1, 3)?);
            ret.push(make_set_str(big_assistance_lift, 0.85, 1, 3)?);
        }
        (_, Week::Week1) => {
            ret.push(make_set_str(big_assistance_lift, 0.5, 1, 10)?);
            ret.push(make_set_str(big_assistance_lift, 0.6, 1, 10)?);
            ret.push(make_set_str(big_assistance_lift, 0.7, 1, 10)?);
        }
        (_, Week::Week2) => {
            ret.push(make_set_str(big_assistance_lift, 0.6, 1, 8)?);
            ret.push(make_set_str(big_assistance_lift, 0.7, 1, 8)?);
            ret.push(make_set_str(big_assistance_lift, 0.8, 1, 6)?);
        }
        (_, Week::Week3) => {
            ret.push(make_set_str(big_assistance_lift, 0.65, 1, 5)?);
            ret.push(make_set_str(big_assistance_lift, 0.75, 1, 5)?);
            ret.push(make_set_str(big_assistance_lift, 0.85, 1, 5)?);
        }
        (_, Week::Week4) => {
            ret.push(make_set_str(big_assistance_lift, 0.4, 1, 5)?);
            ret.push(make_set_str(big_assistance_lift, 0.5, 1, 5)?);
            ret.push(make_set_str(big_assistance_lift, 0.6, 1, 5)?);
        }
    }

    // small assistance
    match primary_lift {
        Lift::Squat => {
            ret.push("RDLs, up to 225, 3x10".to_owned());
            let coin: bool = rng.gen();
            ret.push(if coin {
                "chin-ups, 2x10".to_owned()
            } else {
                "pull-ups, 2x10".to_owned()
            });
        }
        Lift::Deadlift => {
            ret.push("overhead squat, 3x10".to_owned());
        }
        Lift::BenchPress => {
            let coin: bool = rng.gen();
            ret.push(if coin {
                "chin-ups, 3x10".to_owned()
            } else {
                "pull-ups, 3x10".to_owned()
            });
        }
        Lift::OverheadPress => {
            let coin: bool = rng.gen();
            ret.push(if coin {
                "barbell 21s x3".to_owned()
            } else {
                "Kroc row, 3x20".to_owned()
            });
        }
        _ => {
            return Err(WorkoutError::Config(format!(
                "Unsupported primary lift {}",
                primary_lift
            )));
        }
    }

    Ok(ret)
}
