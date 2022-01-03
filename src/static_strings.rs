/// Warm-up routine never changes
pub static WARM_UP: [&str; 2] = [
    "5min jump rope, jog, row, or bike",
    "2x15 box jumps",
];

/// Limber 11, done after warm-up and before main exercises, never changes
pub static LIMBER_11: [&str; 11] = [
    "1. foam roll IT band, 15x/leg",
    "2. foam roll adductor, 15x/leg",
    "3. lacrosse ball glutes and piriformis, 60s/leg",
    "4. bent-knee iron cross, 10x/side",
    "5. rollover into V-sit, 10x",
    "6. rocking frog, 10x",
    "7. fire hydrant circles, 10x/direction/leg",
    "8. mountain climbers, 10x/leg",
    "9. Cossack squats, 10x/side",
    "10. seated piriforis stretch, 30s/side",
    "11. rear-foot-elevated hip flexor stretch, 10x/side",
];

/// One or more core exercises are randomly sampled from this list
pub static CORE_EXERCISES: [&str; 10] = [
    "ab-mat sit-up, 3x10",
    "bird dog, 3x10/side",
    "windshield wipers, 3x10/side",
    "kayaker, 3x10/side",
    "power point 3x30s/side",
    "bridge 3x10s/side",
    "gymnast L-sit, 3x10s",
    "side plank, 3x10/side",
    "Turkish get-up, 3x3/side",
    "band torso twist, 3x10/side",
];
