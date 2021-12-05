use structopt::StructOpt;

#[derive(Debug)]
enum PrimaryLift {
    Squat,
    BenchPress,
    Deadlift,
    Press,
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

fn parse_week(src: &str) -> Result<i8, &str> {
    match src {
        "1" => Ok(1),
        "2" => Ok(2),
        "3" => Ok(3),
        "4" => Ok(4),
        _ => Err(src)
    }
}

#[derive(StructOpt, Debug)]
struct Cli {
    /// Primary lift for the week that will be done in the 5/3/1 rep pattern;
    /// valid values are "squat", "s", "bench-press", "bp", "deadlift", "dl",
    /// "press", and "p"
    #[structopt(parse(try_from_str = parse_primary_lift))]
    primary_lift: PrimaryLift,

    /// Training max for primary lift this week; should initially be 90% of 1RM
    /// and increase by 5lb/cycle (press, bench press) or  10lb/cycle (squat,
    /// deadlift) thereafter
    training_max: i16,

    /// Week number (1-4) in the 5/3/1 cycle for the primary lift
    #[structopt(parse(try_from_str = parse_week))]
    week: i8,

    /// Will this workout be at the gym or at home?
    #[structopt(parse(try_from_str = parse_workout_location), default_value = "home")]
    location: WorkoutLocation,
}

fn main() {
    let args = Cli::from_args();
    println!("{:?}", args);
}
