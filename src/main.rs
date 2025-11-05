#![allow(clippy::needless_range_loop, unused)]

use std::{
    cell::LazyCell,
    collections::VecDeque,
    sync::{LazyLock, OnceLock},
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

#[cfg(not(target_arch = "x86_64"))]
use blinkt::{Blinkt, BlinktSpi};
use board::Board;
#[cfg(target_arch = "x86_64")]
use board::color_to_vec3;
use constants::*;
use ligth_point::LigthPoint;
#[cfg(target_arch = "x86_64")]
use macroquad::prelude::*;
use product::{Product, Step};
use time_manager::TimeManager;

use crate::{
    board::{MachineStateChange, Scenario},
    product::ProductPlan,
    time_manager::VirtualInstant,
};

mod board;
mod constants;
mod ligth_point;
mod module;
mod product;
mod time_manager;

// Material Fluesse Programmieren
#[cfg(target_arch = "x86_64")]
#[macroquad::main("Board")]
async fn main() {
    main_inner().await;
}
#[cfg(not(target_arch = "x86_64"))]
fn main() {
    let future = std::pin::pin!(main_inner());
    let mut context = Context::from_waker(Waker::noop());
    match future.poll(&mut context) {
        Poll::Pending => {}
        Poll::Ready(()) => {}
    }
}

async fn main_inner() {
    #[cfg(target_arch = "x86_64")]
    board::Board::set_screen_size();
    let mut board = Board::new();

    #[cfg(not(target_arch = "x86_64"))]
    let mut blinkt = Blinkt::with_spi(
        BlinktSpi::with_settings(
            blinkt::spi::Bus::Spi1,
            blinkt::spi::SlaveSelect::Ss0,
            1_000_000,
            blinkt::spi::Mode::Mode0,
        )
        .unwrap(),
        X_NUM_MODULES * Y_NUM_MODULES * (7 + 6),
    );

    let steps_bottom_from_top = ProductPlan::new(
        vec![
            Step::new(1.0, [0, 1], vec![[0, 1]], true),
            Step::new(1.0, [1, 0], vec![[0, 0]], true),
            Step::new(2.5, [2, 1], vec![[1, 1]], false),
            Step::new(1.0, [1, 3], vec![[2, 2], [1, 2]], true),
            Step::new(5.0, [2, 3], vec![[1, 2], [2, 2]], false),
            Step::new(5.0, [3, 3], vec![[2, 2], [3, 2]], false),
            Step::new(1.0, [4, 3], vec![[3, 2], [4, 2]], true),
            Step::new(1.0, [5, 2], vec![[5, 3]], false),
        ],
        constants::RED,
    );
    board.set_storage(STEPS_TOP_NORMAL.clone());
    board.set_storage(STEPS_BOTTOM_NORMAL.clone());

    let normal = Scenario::starting_scenario();
    let bottom_supply_difficulty = Scenario {
        name: "Supplyer ausfall oben".to_string(),
        starting_steps: vec![STEPS_TOP_NORMAL.clone(), STEPS_BOTTOM_NORMAL.clone()],
        disturbance_steps: vec![STEPS_TOP_NORMAL.clone(), steps_bottom_from_top.clone()],
        pre_duration: Duration::from_secs(10),
        starting_time: board.time_manager.now(),
        disturbance_duration: Duration::from_secs(56),
        state: board::ScenarioState::Start,
        machine_state_changes: vec![],
    };
    let maintenance = Scenario {
        name: "Wartung Oben".to_string(),
        starting_steps: vec![STEPS_TOP_MAINTAINANCE.clone(), STEPS_BOTTOM_NORMAL.clone()],
        disturbance_steps: vec![STEPS_TOP_MAINTAINANCE.clone(), STEPS_BOTTOM_NORMAL.clone()],
        pre_duration: Duration::from_secs(25),
        starting_time: board.time_manager.now(),
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
    };
    board.set_scenario(bottom_supply_difficulty.clone());

    loop {
        let start_time = Instant::now();

        #[cfg(target_arch = "x86_64")]
        {
            // Handle keyboard input for time control
            handle_time_controls(&mut board.time_manager);
            clear_background(GRAY);
        }

        board.reset(LED_OFF_COLOR);

        board.update();

        #[cfg(not(target_arch = "x86_64"))]
        for (pixel, mut color) in blinkt.iter_mut().zip(board.colors()) {
            pixel.set_rgbb(
                (color.red * 255.0) as u8,
                (color.green * 255.0) as u8,
                (color.blue * 255.0) as u8,
                0.1,
            );
        }
        #[cfg(not(target_arch = "x86_64"))]
        blinkt.show().unwrap();

        #[cfg(target_arch = "x86_64")]
        {
            for key in get_keys_pressed() {
                match key {
                    KeyCode::Key7 => board.set_scenario(normal.clone()),
                    KeyCode::Key8 => board.set_scenario(bottom_supply_difficulty.clone()),
                    KeyCode::Key9 => board.set_scenario(maintenance.clone()),
                    _ => {}
                }
            }
            board.draw();
            // Draw speed indicator
            draw_speed_indicator(&board.time_manager, vec2(10.0, 10.0));
            next_frame().await
        }

        #[cfg(not(target_arch = "x86_64"))]
        std::thread::sleep(
            (start_time + Duration::from_secs(1) / 100).saturating_duration_since(Instant::now()),
        );
    }
}

#[cfg(target_arch = "x86_64")]
/// Handle keyboard input for time control
fn handle_time_controls(time_manager: &mut TimeManager) {
    for key in get_keys_pressed() {
        match key {
            KeyCode::Key1 => time_manager.set_speed(0.25), // Quarter speed
            KeyCode::Key2 => time_manager.set_speed(0.5),  // Half speed
            KeyCode::Key3 => time_manager.set_speed(1.0),  // Normal speed
            KeyCode::Key4 => time_manager.set_speed(2.0),  // Double speed
            KeyCode::Key5 => time_manager.set_speed(4.0),  // Quadruple speed
            KeyCode::Key6 => time_manager.set_speed(8.0),  // 8x speed
            KeyCode::Up => {
                // fine speed adjustment
                let current_speed = time_manager.speed();
                time_manager.set_speed((current_speed * 1.25).min(10.0));
            }
            KeyCode::Down => {
                let current_speed = time_manager.speed();
                time_manager.set_speed((current_speed * 0.8).max(0.1));
            }
            _ => {}
        }
    }
}

#[cfg(target_arch = "x86_64")]
/// Draw speed indicator and controls help
fn draw_speed_indicator(time_manager: &TimeManager, position: Vec2) {
    let speed = time_manager.speed();
    // Status display
    draw_text(
        &format!("Speed: {speed:.2}x"),
        position.x,
        position.y,
        24.0,
        GREEN,
    );

    // Virtual time display (formatted)
    let virtual_time_text = format!("Virtual Time: {}", time_manager.format_time());
    draw_text(
        &virtual_time_text,
        position.x,
        position.y + 25.0,
        20.0,
        LIGHTGRAY,
    );

    // Controls help
    let help_text = &[
        "Controls:",
        "1-6: Set speed (0.25x - 8x)",
        "↑/↓: Fine adjust speed",
        "Space: Pause/Resume",
        "R: Reset time",
    ];

    for (i, line) in help_text.iter().enumerate() {
        draw_text(
            line,
            position.x,
            position.y + 60.0 + i as f32 * 18.0,
            16.0,
            GRAY,
        );
    }
}
