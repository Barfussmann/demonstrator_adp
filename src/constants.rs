use std::{sync::LazyLock, time::Duration};

use palette::Srgb;

use crate::{
    board::{self, MachineStateChange, Scenario},
    product::{ProductPlan, Step},
    time_manager::VirtualInstant,
};

pub const X_NUM_MODULES: usize = 6;
pub const Y_NUM_MODULES: usize = 4;
pub const DRAW_SCALE: f32 = 1.0;
pub const PIXEL_PER_MODULE: f32 = DRAW_SCALE * 100.;
pub const LEDS_PER_DIR: usize = 7;
pub const STEP_SIZE: f32 = 3.;
pub const COLOR_RADIUS: f32 = 0.1;
pub const COLOR_STRENGTH: f32 = 1.0;
pub const EPSILON: f32 = 1e-4;

pub const MAX_PRODUCT_IN_STORAGE: u32 = 5;

pub const LED_OFF_COLOR: Srgb = Srgb::new(0.0, 0.0, 0.0);

pub const GREEN: Srgb = Srgb::new(0.00, 1.0, 0.0);
pub const YELLOW: Srgb = Srgb::new(1.0, 1.0, 0.0);
pub const RED: Srgb = Srgb::new(1.0, 0.0, 0.0);
pub const BLUE: Srgb = Srgb::new(0.0, 0.5, 1.0);
pub const MAGENTA: Srgb = Srgb::new(1.0, 0.15, 1.0);

pub static STEPS_TOP_NORMAL: LazyLock<ProductPlan> = LazyLock::new(|| {
    ProductPlan::new(
        vec![
            Step::new(1.0, [0, 1], vec![[0, 1]], false),
            Step::new(1.0, [1, 0], vec![[0, 0]], false),
            Step::new(2.5, [2, 1], vec![[1, 1]], false),
            Step::new(1.0, [3, 0], vec![[2, 0]], true),
            Step::new(2.5, [4, 1], vec![[3, 1]], false),
            Step::new(1.0, [3, 0], vec![[4, 0]], false),
            Step::new(5.0, [5, 0], vec![[4, 0]], false),
            Step::new(2.5, [5, 2], vec![[5, 1]], false),
        ],
        GREEN,
    )
});
pub static STEPS_TOP_MAINTAINANCE: LazyLock<ProductPlan> = LazyLock::new(|| {
    ProductPlan::new(
        vec![
            Step::new(1.0, [0, 1], vec![[0, 1]], false),
            Step::new(1.0, [1, 0], vec![[0, 0]], false),
            Step::new(2.5, [2, 1], vec![[1, 1]], false),
            // Step::new(1.0, [3, 0], vec![[2, 0]], true),
            Step::new(2.5, [4, 1], vec![[3, 1]], false),
            Step::new(1.0, [3, 0], vec![[4, 0]], true),
            Step::new(5.0, [5, 0], vec![[4, 0]], false),
            Step::new(2.5, [5, 2], vec![[5, 1]], false),
        ],
        GREEN,
    )
});

pub static STEPS_BOTTOM_NORMAL: LazyLock<ProductPlan> = LazyLock::new(|| {
    ProductPlan::new(
        vec![
            Step::new(1.0, [0, 2], vec![[0, 2]], false),
            Step::new(1.0, [1, 3], vec![[0, 3]], true),
            Step::new(5.0, [2, 3], vec![[1, 2], [2, 2]], false),
            Step::new(5.0, [3, 3], vec![[2, 2], [3, 2]], false),
            Step::new(1.0, [4, 3], vec![[3, 2], [4, 2]], true),
            Step::new(1.0, [5, 2], vec![[5, 3]], false),
        ],
        RED,
    )
});

pub static STEPS_BOTTOM_FROM_TOP: LazyLock<ProductPlan> = LazyLock::new(|| {
    ProductPlan::new(
        vec![
            Step::new(1.0, [0, 1], vec![[0, 1]], true),
            Step::new(1.0, [1, 0], vec![[0, 0]], true),
            Step::new(2.5, [2, 1], vec![[1, 1]], false),
            Step::new(1.0, [1, 3], vec![[1, 1], [1, 2]], true),
            Step::new(5.0, [2, 3], vec![[1, 2], [2, 2]], false),
            Step::new(5.0, [3, 3], vec![[2, 2], [3, 2]], false),
            Step::new(1.0, [4, 3], vec![[3, 2], [4, 2]], true),
            Step::new(1.0, [5, 2], vec![[5, 3]], false),
        ],
        RED,
    )
});

pub static BOTTOM_SUPPLY_DIFFICULTY: LazyLock<Scenario> = LazyLock::new(|| Scenario {
    name: "Supplyer ausfall oben".to_string(),
    starting_steps: vec![STEPS_TOP_NORMAL.clone(), STEPS_BOTTOM_NORMAL.clone()],
    disturbance_steps: vec![STEPS_TOP_NORMAL.clone(), STEPS_BOTTOM_FROM_TOP.clone()],
    pre_duration: Duration::from_secs(10),
    starting_time: VirtualInstant::zero(),
    disturbance_duration: Duration::from_secs(56),
    state: board::ScenarioState::Start,
    machine_state_changes: vec![],
});
pub static MAINTENANCE: LazyLock<Scenario> = LazyLock::new(|| Scenario {
    name: "Wartung Oben".to_string(),
    starting_steps: vec![STEPS_TOP_MAINTAINANCE.clone(), STEPS_BOTTOM_NORMAL.clone()],
    disturbance_steps: vec![STEPS_TOP_MAINTAINANCE.clone(), STEPS_BOTTOM_NORMAL.clone()],
    pre_duration: Duration::from_secs(25),
    starting_time: VirtualInstant::zero(),
    disturbance_duration: Duration::from_secs(20),
    state: board::ScenarioState::Start,
    machine_state_changes: vec![
        MachineStateChange::new(
            Duration::from_secs(32),
            board::ModuleState::Maintaining,
            [4, 1],
        ),
        MachineStateChange::new(
            Duration::from_secs(32 + 20),
            board::ModuleState::Functional,
            [4, 1],
        ),
    ],
});
