# five-three-one

Generate a 5/3/1 training template from training max values.

## Usage

```bash
cargo run -- --primary-lift squat --week 1
```

- `--primary-lift` accepts aliases for the main lifts:
  - Squat: `squat`, `s`
  - Bench press: `bench-press`, `bench_press`, `b`, `bp`
  - Deadlift: `deadlift`, `d`, `dl`
  - Overhead press: `overhead-press`, `o`, `ohp`, `p`
- `--week` is `1`, `2`, `3`, or `4`.
- Add `--warmup` or `--mobility` to include those warm-up blocks.
- Add `--core-exercises N` to pick `N` random core exercises.
- Add `--seed` to make assistance and core selection deterministic.
- Add `--config PATH` to load a different TOML config path.

By default, the command looks for `training_max.toml` in the current working directory.

## `training_max.toml`

Create a `training_max.toml` in your project root (or pass `--config PATH`) with this structure:

```toml
[default]
squat = 325
deadlift = 365
bench_press = 235
overhead_press = 170

# Optional assistance lifts
front_squat = 215
good_morning = 215
incline_press = 215
close_grip_bench_press = 215
bulgarian_split_squat = 95
power_clean = 205
```

## Notes

- Primary lifts must all be present.
- All training max values must be positive integers.
