#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate strum;
extern crate strum_macros;

use clap::Parser;
use rand::seq::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;
use std::str::FromStr; // required by EnumString

mod lifts;
mod static_strings;

use lifts::{generate_assistance_sets, generate_primary_sets, Lift, Week, WorkoutError};
use static_strings::{CORE_EXERCISES, LIMBER_11, WARM_UP};

const DEFAULT_TRAINING_MAX_FILE: &str = "training_max.toml";

#[derive(Deserialize)]
struct TrainingMaxConfig {
    default: HashMap<String, i32>,
}

/*
 * ============================================================
 * CLI parsing types and helpers
 * ============================================================
 */

fn parse_primary_lift(src: &str) -> Result<Lift, String> {
    let lift = Lift::from_str(src).map_err(|_| {
        format!(
            "Invalid primary lift '{src}'. Valid values are: squat/s/bench-press/bench_press/b/bp/deadlift/d/dl/overhead-press/o/ohp/p."
        )
    })?;

    if !Lift::PRIMARY_LIFTS.contains(&lift) {
        return Err(format!(
            "Invalid primary lift '{src}'. Valid values are: squat/s/bench-press/bench_press/b/bp/deadlift/d/dl/overhead-press/o/ohp/p."
        ));
    }

    Ok(lift)
}

fn parse_week(src: &str) -> Result<Week, String> {
    match src {
        "1" => Ok(Week::Week1),
        "2" => Ok(Week::Week2),
        "3" => Ok(Week::Week3),
        "4" => Ok(Week::Week4),
        _ => Err("week must be 1, 2, 3, or 4".to_owned()),
    }
}

fn validate_required_assistance_training_max(
    primary_lift: &Lift,
    training_maxes: &HashMap<Lift, i16>,
) -> Result<(), WorkoutError> {
    let required_lift = match primary_lift {
        Lift::Squat => Lift::PowerClean,
        Lift::Deadlift => Lift::FrontSquat,
        Lift::BenchPress => Lift::InclinePress,
        Lift::OverheadPress => Lift::CloseGripBenchPress,
        _ => return Ok(()),
    };

    training_maxes
        .get(&required_lift)
        .map(|_| ())
        .ok_or(WorkoutError::MissingTrainingMax { lift: required_lift })?;

    Ok(())
}

#[derive(Parser, Debug)]
#[command(
    name = "five-three-one",
    version,
    about = "Generate a 5/3/1 workout plan from your training maxes.",
    long_about = "Generate the primary lift and assistance work for a specific 5/3/1 week.\n\
The program reads lift training maxes from a TOML file that follows:\n\
\n\
[default]\nsquat = 325\nbench_press = 235\ndeadlift = 365\noverhead_press = 170\n\
\n\
By default, it looks for `training_max.toml` in the current working directory.",
)]
struct Cli {
    /// Primary lift for the week that will be done in the 5/3/1 rep pattern.
    /// Examples: `squat`, `s`, `bench-press`, `bench_press`, `b`, `bp`,
    /// `deadlift`, `d`, `dl`, `overhead-press`, `ohp`, `o`, or `p`.
    #[arg(short = 'l', long, value_parser = parse_primary_lift)]
    primary_lift: Lift,

    /// Week number (1-4) in the 5/3/1 cycle for the primary lift.
    #[arg(short = 'n', long, value_parser = parse_week)]
    week: Week,

    /// Include warm-up?
    #[arg(short = 'w', long)]
    warmup: bool,

    /// Include mobility?
    #[arg(short = 'm', long)]
    mobility: bool,

    /// Number of core exercises to include (randomly selected from the built-in list).
    #[arg(default_value = "0", short = 'x', long, value_name = "N")]
    core_exercises: usize,

    /// Path to a TOML config file. Defaults to `training_max.toml` in cwd.
    #[arg(long = "config", value_name = "PATH")]
    config_path: Option<PathBuf>,

    /// Seed for RNG to make assistance/core selection deterministic.
    #[arg(long)]
    seed: Option<u64>,
}

/*
 * ============================================================
 * Config data loading
 * ============================================================
 */

fn load_training_maxes_from_file(path: &Path) -> Result<HashMap<Lift, i16>, WorkoutError> {
    let source = path.to_string_lossy().into_owned();
    let contents = std::fs::read_to_string(path)
        .map_err(|err| WorkoutError::Config(format!("Unable to read {}: {}", source, err)))?;
    parse_training_maxes_from_str(&contents, &source)
}

fn parse_training_maxes_from_str(
    contents: &str,
    source: &str,
) -> Result<HashMap<Lift, i16>, WorkoutError> {
    let cfg: TrainingMaxConfig = toml::from_str(contents)
        .map_err(|err| WorkoutError::Config(format!("Unable to parse {} as TOML: {}", source, err)))?;

    let mut ret = HashMap::new();
    for (lift_name, raw_weight) in cfg.default.iter() {
        let lift = Lift::from_str(lift_name).map_err(|_| {
            WorkoutError::Config(format!(
                "Unknown lift '{}' in training max file {}",
                lift_name, source
            ))
        })?;

        if *raw_weight <= 0 {
            return Err(WorkoutError::Config(format!(
                "Training max for '{}' in {} must be a positive integer, got {}",
                lift_name, source, raw_weight
            )));
        }

        let weight =
            i16::try_from(*raw_weight).map_err(|_| WorkoutError::Config(format!(
                "Training max for '{}' in {} is out of range: {}",
                lift_name, source, raw_weight
            )))?;

        ret.insert(lift, weight);
    }

    let missing_primary_lifts: Vec<String> = Lift::PRIMARY_LIFTS
        .iter()
        .filter(|lift| !ret.contains_key(lift))
        .map(|lift| lift.to_string())
        .collect();

    if !missing_primary_lifts.is_empty() {
        return Err(WorkoutError::Config(format!(
            "Missing required primary lift training max(es) in {}: {}",
            source,
            missing_primary_lifts.join(", ")
        )));
    }

    Ok(ret)
}

/*
 * ============================================================
 * Display helpers
 * ============================================================
 */

fn print_header(text: &str) {
    println!("{}\n====================", text);
}

fn print_spacer() {
    println!("\n");
}

/*
 * ============================================================
 * Main
 * ============================================================
 */

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), WorkoutError> {
    let args = Cli::parse();
    let config_path = args
        .config_path
        .as_deref()
        .unwrap_or_else(|| Path::new(DEFAULT_TRAINING_MAX_FILE));

    let training_maxes = load_training_maxes_from_file(config_path)?;
    validate_required_assistance_training_max(&args.primary_lift, &training_maxes)?;

    let mut rng = match args.seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None => StdRng::from_entropy(),
    };

    if args.warmup {
        print_header("Warm-up");
        for &s in WARM_UP.iter() {
            println!("  {}", s);
        }
        print_spacer();
    }

    if args.mobility {
        print_header("Limber 11");
        for &s in LIMBER_11.iter() {
            println!("  {}", s);
        }
        print_spacer();
    }

    print_header("Primary lift");
    let primary_sets = generate_primary_sets(&args.primary_lift, &args.week, &training_maxes)?;
    for s in primary_sets.iter() {
        println!("  {}", s);
    }
    print_spacer();

    print_header("Assistance lifts");
    let assistance_sets = generate_assistance_sets(
        &args.primary_lift,
        &args.week,
        &training_maxes,
        &mut rng,
    )?;
    for s in assistance_sets.iter() {
        println!("  {}", s);
    }
    print_spacer();

    if args.core_exercises > 0 {
        print_header("Core");
        let core_exercises = CORE_EXERCISES.choose_multiple(&mut rng, args.core_exercises);
        for &s in core_exercises {
            println!("  {}", s);
        }
        print_spacer();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use std::collections::HashMap;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_temp_config(contents: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock moved backwards")
            .as_nanos();
        path.push(format!("five-three-one-config-{}.toml", unique));
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn parse_week_supports_only_supported_values() {
        assert_eq!(parse_week("1").unwrap(), Week::Week1);
        assert_eq!(parse_week("2").unwrap(), Week::Week2);
        assert_eq!(parse_week("3").unwrap(), Week::Week3);
        assert_eq!(parse_week("4").unwrap(), Week::Week4);
        assert_eq!(parse_week("0"), Err("week must be 1, 2, 3, or 4".to_owned()));
        assert_eq!(parse_week("5"), Err("week must be 1, 2, 3, or 4".to_owned()));
    }

    #[test]
    fn parse_primary_lift_accepts_supported_aliases() {
        assert_eq!(parse_primary_lift("squat").unwrap(), Lift::Squat);
        assert_eq!(parse_primary_lift("s").unwrap(), Lift::Squat);
        assert_eq!(parse_primary_lift("bench-press").unwrap(), Lift::BenchPress);
        assert_eq!(parse_primary_lift("bench_press").unwrap(), Lift::BenchPress);
        assert_eq!(parse_primary_lift("b").unwrap(), Lift::BenchPress);
        assert_eq!(parse_primary_lift("deadlift").unwrap(), Lift::Deadlift);
        assert_eq!(parse_primary_lift("dl").unwrap(), Lift::Deadlift);
        assert_eq!(parse_primary_lift("overhead-press").unwrap(), Lift::OverheadPress);
        assert_eq!(parse_primary_lift("p").unwrap(), Lift::OverheadPress);
    }

    #[test]
    fn parse_primary_lift_rejects_assistance_lifts() {
        assert!(parse_primary_lift("front_squat").is_err());
        assert!(parse_primary_lift("power_clean").is_err());
        assert!(parse_primary_lift("close_grip_bench_press").is_err());
    }

    #[test]
    fn parse_training_maxes_from_toml_requires_primary_lifts() {
        let config = "[default]
squat = 325
bench_press = 235
deadlift = 365";
        let err = parse_training_maxes_from_str(config, "test-training_max.toml").unwrap_err();
        assert!(err.to_string().contains("Missing required primary lift training max"));
        assert!(err.to_string().contains("overhead press"));
    }

    #[test]
    fn validate_required_assistance_training_max_for_primary_lift() {
        let config = "[default]
squat = 325
bench_press = 235
deadlift = 365
overhead_press = 170";
        let training_maxes = parse_training_maxes_from_str(config, "training_max.toml").unwrap();
        let err = validate_required_assistance_training_max(&Lift::Squat, &training_maxes).unwrap_err();
        assert_eq!(
            err,
            WorkoutError::MissingTrainingMax {
                lift: Lift::PowerClean
            }
        );
    }

    #[test]
    fn parse_training_maxes_rejects_invalid_values_and_unknown_lifts() {
        let missing_primary = "[default]
squat = 325
bench_press = 235
deadlift = 365
overhead_press = 170
fakelift = 200";
        assert!(parse_training_maxes_from_str(missing_primary, "training_max.toml").is_err());

        let negative = "[default]
squat = -325
bench_press = 235
deadlift = 365
overhead_press = 170";
        assert!(parse_training_maxes_from_str(negative, "training_max.toml").is_err());
    }

    #[test]
    fn parse_training_maxes_from_file_uses_provided_path() {
        let config = "[default]
squat = 325
bench_press = 235
deadlift = 365
overhead_press = 170";
        let path = write_temp_config(config);
        let result = parse_training_maxes_from_str(config, "training_max.toml").unwrap();

        let from_disk = load_training_maxes_from_file(&path).unwrap();
        fs::remove_file(&path).unwrap();

        assert_eq!(result, from_disk);
        assert_eq!(from_disk.get(&Lift::Squat), Some(&325));
    }

    #[test]
    fn cli_parses_defaults_and_config_option_is_optional() {
        let args = Cli::parse_from(["five-three-one", "--primary-lift", "squat", "--week", "1"]);
        assert_eq!(args.primary_lift, Lift::Squat);
        assert_eq!(args.week, Week::Week1);
        assert!(args.config_path.is_none());
    }

    #[test]
    fn help_output_documents_training_max_toml_path() {
        let mut cmd = Cli::command();
        let help = cmd.render_long_help().to_string();
        assert!(help.contains("training_max.toml"));
        assert!(help.contains("--config"));
    }

    #[test]
    fn parse_training_maxes_rejects_config_without_primary_keys() {
        let config = "[default]\nsquat = 325
bench_press = 235
front_squat = 215";
        let err = parse_training_maxes_from_str(config, "training_max.toml").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("deadlift"));
        assert!(msg.contains("overhead press"));
        assert!(msg.contains("Missing required primary lift training max"));
    }
}
