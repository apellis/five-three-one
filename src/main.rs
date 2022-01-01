use rand::Rng;
use rand::seq::SliceRandom;
use std::fmt;
use structopt::StructOpt;

/* TODOs:
 *
 * 1. unit tests
 * 2. choice of weight for all assistance exercises
 * 3. verbose mode for exercise details
 * 4. data structure for particular exercises, so this can be factored into
 *    a library (suitable for, e.g., API integration) and a CLI that uses it
 */ 

/*
 * ============================================================
 * Static data: fixed exercises and sets of exercises
 * ============================================================
 */

/// Warm-up routine never changes
static WARM_UP: &str = "Warm up:
  5min jump rope, jog, row, or bike
  2x15 box jumps";

/// Limber 11, done after warm-up and before main exercises, never changes
static LIMBER_11: &str = "Limber 11:
  1. foam roll IT band, 15s/leg
  2. foam roll adductor, 15s/leg
  3. lacrosse ball glutes and piriformis, 30s/leg
  4. bent-knee iron cross, 10x/side
  5. rollover into V-sit, 10x
  6. rocking frog, 10x
  7. fire hydrant circles, 10x/direction/leg
  8. mountain climbers, 10x/leg
  9. Cossack squats, 10x/side
  10. seated piriforis stretch, 30s/side
  11. rear-foot-elevated hip flexor stretch, 10x/side";

/// One or more core exercises are randomly sampled from this list
static CORE_EXERCISES: [&str; 10] = [
    "ab-mat sit-up, 3x10",
    "bird dog, 3x10/side",
    "windshield wipers, 3x10/side",
    "kayaker, 3x10/side",
    "power point 3x30s/side",
    "bridge 3x10s/side",
    "gymnast L-sit, 3x10s",
    "side plank, 3x10/side",
    "Turkish get-up, 3x5",
    "band torso twist, 3x10/side",
];

static POWER_CLEAN_TO_SQUAT_RATIO: f32 = 0.75;
static POWER_SNATCH_TO_SQUAT_RATIO: f32 = 0.6;
static FRONT_SQUAT_TO_SQUAT_RATIO: f32 = 0.85;
static OVERHEAD_SQUAT_TO_SQUAT_RATIO: f32 = 0.6;

static INDENT: &str = "  ";

/* 
 * ============================================================
 * CLI parsing types and helpers
 * ============================================================
 */

#[derive(Debug)]
enum PrimaryLift {
    Squat,
    BenchPress,
    Deadlift,
    Press,
}

impl fmt::Display for PrimaryLift {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: &str = match self {
            PrimaryLift::Squat => "squat",
            PrimaryLift::BenchPress => "bench press",
            PrimaryLift::Deadlift => "deadlift",
            PrimaryLift::Press => "press"
        };
        write!(f, "{}", s)
    }
}

fn parse_primary_lift(src: &str) -> Result<PrimaryLift, String> {
    match src {
        "squat" | "s" => Ok(PrimaryLift::Squat),
        "bench-press" | "bp" => Ok(PrimaryLift::BenchPress),
        "deadlift" | "d" | "dl" => Ok(PrimaryLift::Deadlift),
        "press" | "p" | "ohp" => Ok(PrimaryLift::Press),
        _ => Err("Invalid lift: ".to_owned() + src)
    }
}

#[derive(Debug)]
enum WorkoutLocation {
    Home,
    Gym,
}

fn parse_workout_location(src: &str) -> Result<WorkoutLocation, &str> {
    match src {
        "home" | "h" => Ok(WorkoutLocation::Home),
        "gym" | "g" => Ok(WorkoutLocation::Gym),
        _ => Err(src)
    }
}

#[derive(Debug)]
enum Week {
    Week1,
    Week2,
    Week3,
    Week4,
}

fn parse_week(src: &str) -> Result<Week, &str> {
    match src {
        "1" => Ok(Week::Week1),
        "2" => Ok(Week::Week2),
        "3" => Ok(Week::Week3),
        "4" => Ok(Week::Week4),
        _ => Err(src)
    }
}

#[derive(StructOpt, Debug)]
struct Cli {
    /// Primary lift for the week that will be done in the 5/3/1 rep pattern;
    /// valid values are "squat", "s", "bench-press", "bp", "deadlift", "dl",
    /// "press", and "p"
    #[structopt(short, long, parse(try_from_str = parse_primary_lift))]
    primary_lift: PrimaryLift,

    /// Training max for primary lift this week; should initially be 90% of 1RM
    /// and increase by 5lb/cycle (press, bench press) or  10lb/cycle (squat,
    /// deadlift) thereafter
    #[structopt(short, long)]
    training_max: i16,

    /// Week number (1-4) in the 5/3/1 cycle for the primary lift
    #[structopt(short, long, parse(try_from_str = parse_week))]
    week: Week,

    /// Will this workout be at the gym or at home?
    #[structopt(short, long, parse(try_from_str = parse_workout_location), default_value = "home")]
    location: WorkoutLocation,

    /// Squat training max, needed on deadlift day to set a weight for front
    /// and overhead squats
    #[structopt(short, long, default_value = "0")]
    squat_training_max: i16,
}

/* 
 * ============================================================
 * Dynamic exercise text generation
 * ============================================================
 */

/// Returns a vector containing the chosen number of core exercises, randomly
/// chosen from CORE_EXERCISES.
fn pick_core_exercises(n: usize) -> Vec<&'static str> {
    return CORE_EXERCISES
        .choose_multiple(&mut rand::thread_rng(), n)
        .cloned()
        .collect();
}

/// Returns a string describing one main exercise set, e.g. "squat 225 x5".
fn set_str(
    lift_name: &str,
    training_max: i16,
    multiplier: f32,
    sets: i8,
    reps: i8,
    amrap: bool
) -> String {
    let weight = scale(training_max, multiplier);
    let mut ret = lift_name.to_owned() + 
        " " + &weight.to_string() + 
        " ";
    if sets > 1 {
        ret += &sets.to_string();
    }
    ret += "x";
    ret += &reps.to_string();
    if amrap {
        ret += "+";
    }
    ret
}

/// Scales integer weight by floating point multiplier and converts back to ingeteger weight.
fn scale(weight: i16, scale: f32) -> i16 {
    return (weight as f32 * scale).round() as i16;
}

/// Returns list of assistance exercises for the chosen lift and location.
fn generate_assistance_exercises(
    primary_lift: &PrimaryLift,
    location: &WorkoutLocation,
    squat_training_max: i16,  // only needed for squat assistance
) -> Vec<String> {
    let mut rng = rand::thread_rng();

    return match primary_lift {
        &PrimaryLift::Squat => {
            let mut n: u8 = rng.gen();
            let first = if n % 2 == 0
                { set_str("power clean", scale(squat_training_max, POWER_CLEAN_TO_SQUAT_RATIO), 0.5, 5, 3, false) }
                else { set_str("power snatch", scale(squat_training_max, POWER_SNATCH_TO_SQUAT_RATIO), 0.5, 5, 3, false) };
            let second = match location {
                &WorkoutLocation::Home => "good mornings 5x12".to_owned(),
                &WorkoutLocation::Gym => {
                    n = rng.gen();
                    if n % 2 == 0 { "leg curl 5x10".to_owned() } else { "dumbbell lunges 5x10 (per side, walking)".to_owned() }
                }
            };
            vec![first, second]
        },
        &PrimaryLift::BenchPress => match location {
            &WorkoutLocation::Home => vec![
                "hanging leg raise 5x15".to_owned(),
                "Pendlay row 5x10".to_owned()
            ],
            &WorkoutLocation::Gym => vec![
                "dumbbell chest press 5x15".to_owned(),
                "dumbbell row 5x10".to_owned()
            ]
        },
        &PrimaryLift::Deadlift => match location {
            &WorkoutLocation::Home => {
                let n: u8 = rng.gen();
                vec![
                    if n % 2 == 0
                        { set_str("front squat", scale(squat_training_max, FRONT_SQUAT_TO_SQUAT_RATIO), 0.5, 5, 10, false) }
                        else { set_str("overhead squat", scale(squat_training_max, OVERHEAD_SQUAT_TO_SQUAT_RATIO), 0.5, 5, 10, false) },
                    "pull-ups (or variation)".to_owned()
                ]
            },
            &WorkoutLocation::Gym => vec![
                "back extension 5x12".to_owned(),
                "dumbbell shrug 5x10".to_owned()
            ]
        },
        &PrimaryLift::Press =>  match location {
            &WorkoutLocation::Home => vec![
                "super push-up 5x15".to_owned(),
                "chin-up 5x10".to_owned()
            ],
            &WorkoutLocation::Gym => vec![
                "dip 5x15".to_owned(),
                "chin-up 5x10".to_owned()
            ],
        }
    };
}

/// Returns main and assistance exercises. We do these together because on squat
/// day, main lifts are sandwiched between assistance lifts, while on other days,
/// all assistance work follows main exercises.
fn main_and_assistance_exercises(
    primary_lift: &PrimaryLift,
    training_max: i16,
    week: Week,
    location: &WorkoutLocation,
    squat_training_max: i16
) -> String {
    // Get weight and reps strings (e.g. "175 x3") for main lift
    let primary_sets: [String; 3] = match week {
        Week::Week1 => [
            set_str(&primary_lift.to_string(), training_max, 0.65, 1, 5, false),
            set_str(&primary_lift.to_string(), training_max, 0.75, 1, 5, false),
            set_str(&primary_lift.to_string(), training_max, 0.85, 1, 5, true)
        ],
        Week::Week2 => [
            set_str(&primary_lift.to_string(), training_max, 0.7, 1, 3, false),
            set_str(&primary_lift.to_string(), training_max, 0.8, 1, 3, false),
            set_str(&primary_lift.to_string(), training_max, 0.9, 1, 3, true)
        ],
        Week::Week3 => [
            set_str(&primary_lift.to_string(), training_max, 0.75, 1, 5, false),
            set_str(&primary_lift.to_string(), training_max, 0.85, 1, 3, false),
            set_str(&primary_lift.to_string(), training_max, 0.95, 1, 1, true)
        ],
        Week::Week4 => [
            set_str(&primary_lift.to_string(), training_max, 0.4, 1, 5, false),
            set_str(&primary_lift.to_string(), training_max, 0.5, 1, 5, false),
            set_str(&primary_lift.to_string(), training_max, 0.6, 1, 5, false)
        ]
    };

    let assistance_exercises = generate_assistance_exercises(
        primary_lift, location,
        match primary_lift {
            PrimaryLift::Squat => training_max,
            _ => squat_training_max,
        });

    let assistance_header = "Assistance:\n";

    let mut ret = "".to_owned();

    // If it's squat day, the first assistance exercise (power clean or power snatch)
    // goes before the main lifts
    if matches!(primary_lift, PrimaryLift::Squat) {
        ret = ret + assistance_header + INDENT + &assistance_exercises[0] + "\n\n";
    };

    // Main lifts
    ret = ret + "Main lifts:\n" +
        INDENT + &primary_sets[0] + "\n" + 
        INDENT + &primary_sets[1] + "\n" + 
        INDENT + &primary_sets[2] + "\n\n";

    // Assistance work (possibly less power clean or power snatch)
    ret = ret + assistance_header + INDENT;
    if !matches!(primary_lift, PrimaryLift::Squat) {
        ret = ret + &assistance_exercises[0] + "\n" + INDENT;
    }
    ret = ret + &assistance_exercises[1];

    return ret;
}

/* 
 * ============================================================
 * Main
 * ============================================================
 */

fn main() {
    let args = Cli::from_args();

    println!("{}", WARM_UP);
    println!();

    println!("{}", LIMBER_11);
    println!();

    println!("{}", main_and_assistance_exercises(
        &args.primary_lift, args.training_max, args.week, &args.location,
        args.squat_training_max));
    println!();

    let core_exercises: Vec<&str> = pick_core_exercises(2);
    println!("Core:\n  {}\n  {}", core_exercises[0], core_exercises[1]);

    println!();
}
