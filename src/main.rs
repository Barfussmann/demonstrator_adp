#![allow(clippy::needless_range_loop, unused)]
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use board::{Board, color_to_vec3};
use ligth_point::LigthPoint;
use macroquad::prelude::*;
use module::ModuleType;
use product::{Product, Step};

// const MOUDLES =

const X_NUM_MODULES: usize = 10;
const Y_NUM_MODULES: usize = 10;
const DRAW_SCALE: f32 = 1.0;
const PIXEL_PER_MODULE: f32 = DRAW_SCALE * 100.;
const LEDS_PER_DIR: usize = 14;
const STEP_SIZE: f32 = 0.1;
const COLOR_RADIUS: f32 = 0.1;
const EPSILON: f32 = 1e-4;

const LED_OFF_COLOR: Vec3 = Vec3::new(0.0, 0.0, 0.0);

const PIKTOGRAM_PATH: &str = "./assets/demonstrator_piktogramme.png";

mod board;
mod ligth_point;
mod module;
mod product;

// Material Fluesse Programmieren
//

#[macroquad::main("Board")]
async fn main() {
    board::Board::set_screen_size();
    let mut board = Board::new();

    let piktogram_image = load_image(PIKTOGRAM_PATH).await.unwrap();

    let gpu_piktogram = Texture2D::from_image(&piktogram_image);

    let steps_red = VecDeque::from([
        Step::new(0.5, vec![ivec2(0, 0)]),
        Step::new(0.5, vec![ivec2(2, 0)]),
        Step::new(2.0, vec![ivec2(3, 0)]),
        Step::new(2.0, vec![ivec2(4, 0), ivec2(4, 2)]),
        Step::new(2.0, vec![ivec2(5, 1)]),
        Step::new(0.5, vec![ivec2(6, 0)]),
        Step::new(0.5, vec![ivec2(7, 1)]),
        Step::new(1.5, vec![ivec2(8, 0), ivec2(9, 1), ivec2(9, 2)]),
        Step::new(2.0, vec![ivec2(8, 3)]),
        Step::new(0.5, vec![ivec2(9, 5)]),
    ]);
    let steps_green = VecDeque::from([
        Step::new(0.5, vec![ivec2(0, 5)]),
        Step::new(2.0, vec![ivec2(1, 4)]),
        // Step::new(1.0, vec![ivec2(2, 7), ivec2(3, 6)]),
        Step::new(1.0, vec![ivec2(2, 6), ivec2(3, 6)]),
        Step::new(2.0, vec![ivec2(4, 4)]),
        Step::new(0.5, vec![ivec2(5, 6)]),
        Step::new(2.0, vec![ivec2(7, 6)]),
        Step::new(0.5, vec![ivec2(9, 5)]),
    ]);
    let mut products: Vec<Product> = Vec::new();
    let mut last_product = Instant::now();
    loop {
        clear_background(BLACK);

        let params = DrawTextureParams {
            dest_size: Some(
                vec2(PIXEL_PER_MODULE, PIXEL_PER_MODULE)
                    * vec2(X_NUM_MODULES as f32, Y_NUM_MODULES as f32),
            ),
            ..Default::default()
        };

        draw_texture_ex(&gpu_piktogram, 0., 0., WHITE, params);

        board.reset(LED_OFF_COLOR);

        products.retain_mut(|product| {
            let Some(ligth_point_pos) = product.next(&mut board) else {
                return false;
            };
            board.draw_ligth_point(ligth_point_pos, color_to_vec3(product.color));
            true
        });
        if last_product.elapsed() > Duration::from_millis(1000) {
            products.push(Product::new(RED, steps_red.clone(), &board));
            products.push(Product::new(GREEN, steps_green.clone(), &board));
            last_product = Instant::now();
        }

        board.draw();

        next_frame().await
    }
}
