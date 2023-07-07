#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate strum;
extern crate strum_macros;

use configparser::ini::Ini;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::str::FromStr; // required by EnumString
use structopt::StructOpt;

mod lifts;
mod static_strings;

use lifts::{generate_assistance_sets, generate_primary_sets, Lift, Week};
use static_strings::{CORE_EXERCISES, LIMBER_11, WARM_UP};

/*
 * ============================================================
 * CLI parsing types and helpers
 * ============================================================
 */

fn parse_primary_lift(src: &str) -> Result<Lift, String> {
    match src {
        "squat" | "s" => Ok(Lift::Squat),
        "bench-press" | "b" | "bp" => Ok(Lift::BenchPress),
        "deadlift" | "d" | "dl" => Ok(Lift::Deadlift),
        "overhead-press" | "p" | "o" | "ohp" => Ok(Lift::OverheadPress),
        _ => Err("Invalid primary lift: ".to_owned() + src),
    }
}

fn parse_week(src: &str) -> Result<Week, &str> {
    match src {
        "1" => Ok(Week::Week1),
        "2" => Ok(Week::Week2),
        "3" => Ok(Week::Week3),
        "4" => Ok(Week::Week4),
        _ => Err(src),
    }
}

#[derive(StructOpt, Debug)]
struct Cli {
    /// Primary lift for the week that will be done in the 5/3/1 rep pattern;
    /// valid values are "squat", "s", "bench-press", "bp", "deadlift", "d",
    /// "dl", "overhead-press", "ohp", and "p"
    #[structopt(short, long, parse(try_from_str = parse_primary_lift))]
    primary_lift: Lift,

    /// Week number (1-4) in the 5/3/1 cycle for the primary lift
    #[structopt(short, long, parse(try_from_str = parse_week))]
    week: Week,

    /// Include warm-up?
    #[structopt(short, long)]
    warmup: bool,

    /// Include mobility?
    #[structopt(short, long)]
    mobility: bool,

    /// Number of core exercises (default: 0)
    #[structopt(default_value = "0", short, long)]
    core_exercises: usize,
}

/*
 * ============================================================
 * Config data loading
 * ============================================================
 */

fn load_training_maxes_from_file(filename: &str) -> HashMap<Lift, i16> {
    let mut config = Ini::new();
    let all_settings = config.load(filename).unwrap();
    let training_max_settings = all_settings
        .get("default") // we don't use a section heading
        .unwrap();

    let mut ret: HashMap<Lift, i16> = HashMap::new();
    for lift_name in training_max_settings.keys() {
        ret.insert(
            Lift::from_str(&lift_name).unwrap(),
            config.getint("default", &lift_name).unwrap().unwrap() as i16,
        );
    }

    ret
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
    let args = Cli::from_args();

    let training_maxes = load_training_maxes_from_file("training_max.ini");

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
    let primary_sets = generate_primary_sets(&args.primary_lift, &args.week, &training_maxes);
    for s in primary_sets.iter() {
        println!("  {}", s);
    }
    print_spacer();

    print_header("Assistance lifts");
    let assistance_sets = generate_assistance_sets(&args.primary_lift, &args.week, &training_maxes);
    for s in assistance_sets.iter() {
        println!("  {}", s);
    }
    print_spacer();

    if args.core_exercises > 0 {
        print_header("Core");
        let core_exercises =
            CORE_EXERCISES.choose_multiple(&mut rand::thread_rng(), args.core_exercises);
        for &s in core_exercises {
            println!("  {}", s);
        }
        print_spacer();
    }
}
