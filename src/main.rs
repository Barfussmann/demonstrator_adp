#![allow(clippy::needless_range_loop, unused)]

use std::{collections::VecDeque, time::Duration};

use board::{Board, color_to_vec3};
use constants::*;
use ligth_point::LigthPoint;
use macroquad::prelude::*;
use product::{Product, Step};
use time_manager::TimeManager;

mod board;
mod constants;
mod ligth_point;
mod module;
mod product;
mod time_manager;

// Material Fluesse Programmieren

#[macroquad::main("Board")]
async fn main() {
    board::Board::set_screen_size();
    let mut board = Board::new();

    let piktogram_image = load_image(PIKTOGRAM_PATH).await.unwrap();
    let gpu_piktogram = Texture2D::from_image(&piktogram_image);

    let steps_top = VecDeque::from([
        Step::new(1.0, ivec2(0, 1), vec![ivec2(0, 1)], false),
        Step::new(1.0, ivec2(1, 0), vec![ivec2(0, 0)], true),
        Step::new(5.0, ivec2(2, 1), vec![ivec2(1, 1)], false),
        Step::new(1.0, ivec2(3, 0), vec![ivec2(2, 0)], true),
        Step::new(5.0, ivec2(4, 1), vec![ivec2(3, 1)], false),
        Step::new(5.0, ivec2(5, 0), vec![ivec2(4, 0)], false),
        Step::new(5.0, ivec2(5, 2), vec![ivec2(5, 1)], false),
    ]);
    let steps_bottom = VecDeque::from([
        Step::new(0.0, ivec2(0, 2), vec![ivec2(0, 2)], false),
        Step::new(1.0, ivec2(1, 3), vec![ivec2(0, 3)], true),
        Step::new(5.0, ivec2(2, 3), vec![ivec2(1, 2), ivec2(2, 2)], false),
        Step::new(5.0, ivec2(3, 3), vec![ivec2(2, 2), ivec2(3, 2)], false),
        Step::new(1.0, ivec2(4, 3), vec![ivec2(3, 2), ivec2(4, 2)], true),
        Step::new(1.0, ivec2(5, 2), vec![ivec2(5, 3)], false),
    ]);
    board.set_storage(steps_bottom.clone());
    board.set_storage(steps_top.clone());

    // Initialize the time manager
    let mut time_manager = TimeManager::new();

    let mut products = Vec::new();
    let mut last_product = time_manager.now();
    // let mut product_spawn_timer =
    //     time_manager.create_repeating_timer(VirtualTime::from_millis(1000));
    loop {
        // Update the time manager
        time_manager.update();

        // Handle keyboard input for time control
        handle_time_controls(&mut time_manager);

        clear_background(GRAY);

        board.reset(LED_OFF_COLOR);

        products.retain_mut(|product: &mut Product| {
            let Some(ligth_point_pos) = product.next(&mut board, &time_manager) else {
                product.finish(&mut board);
                return false;
            };
            board.draw_ligth_point(ligth_point_pos, color_to_vec3(product.color));
            true
        });

        // Check if it's time to spawn new products using virtual time
        if last_product + Duration::from_millis(3000) < time_manager.now() {
            products.push(Product::new(GREEN, steps_top.clone(), &time_manager));
            products.push(Product::new(RED, steps_bottom.clone(), &time_manager));
            last_product = time_manager.now();
        }

        board.draw();

        // Draw speed indicator
        draw_speed_indicator(&time_manager, vec2(10.0, 700.0));

        next_frame().await
    }
}

/// Handle keyboard input for time control
fn handle_time_controls(time_manager: &mut TimeManager) {
    // Speed controls
    if is_key_pressed(KeyCode::Key1) {
        time_manager.set_speed(0.25); // Quarter speed
    }
    if is_key_pressed(KeyCode::Key2) {
        time_manager.set_speed(0.5); // Half speed
    }
    if is_key_pressed(KeyCode::Key3) {
        time_manager.set_speed(1.0); // Normal speed
    }
    if is_key_pressed(KeyCode::Key4) {
        time_manager.set_speed(2.0); // Double speed
    }
    if is_key_pressed(KeyCode::Key5) {
        time_manager.set_speed(4.0); // Quadruple speed
    }
    if is_key_pressed(KeyCode::Key6) {
        time_manager.set_speed(8.0); // 8x speed
    }

    // Fine speed adjustment
    if is_key_pressed(KeyCode::Up) {
        let current_speed = time_manager.speed();
        time_manager.set_speed((current_speed * 1.25).min(10.0));
    }
    if is_key_pressed(KeyCode::Down) {
        let current_speed = time_manager.speed();
        time_manager.set_speed((current_speed * 0.8).max(0.1));
    }
}

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
