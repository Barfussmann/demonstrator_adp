#![allow(clippy::needless_range_loop, unused)]
use board::{Board, color_to_vec3};
use ligth_point::LigthPoint;
use macroquad::prelude::*;

// Material Fluesse Programmieren
// Bilder Von Praesi ziehen und einfuegen

// const MOUDLES =

const X_NUM_MODULES: usize = 10;
const Y_NUM_MODULES: usize = 10;
const DRAW_SCALE: f32 = 1.0;
const PIXEL_PER_MODULE: f32 = DRAW_SCALE * 100.;
const LEDS_PER_DIR: usize = 14;
const STEP_SIZE: f32 = 0.03;
const COLOR_RADIUS: f32 = 0.1;
const EPSILON: f32 = 1e-4;

const LED_OFF_COLOR: Vec3 = Vec3::new(0.0, 0.0, 0.0);

const PIKTOGRAM_PATH: &str = "./assets/demonstrator_piktogramme.png";

mod board;
mod ligth_point;
mod module;

#[macroquad::main("Board")]
async fn main() {
    board::Board::set_screen_size();
    let mut board = Board::new();

    let piktogram_image = load_image(PIKTOGRAM_PATH).await.unwrap();

    dbg!(piktogram_image.height);
    dbg!(piktogram_image.width);

    let gpu_piktogram = Texture2D::from_image(&piktogram_image);

    let start = vec2(0.5, 0.5);
    let end = vec2(9.5, 7.5);

    let mut ligth_points = Vec::new();
    let mut i = 0;
    loop {
        clear_background(BLACK);
        i += 1;
        if i % 100 == 0 {
            ligth_points.push(LigthPoint::new(start, end))
        }

        let params = DrawTextureParams {
            dest_size: Some(
                vec2(PIXEL_PER_MODULE, PIXEL_PER_MODULE)
                    * vec2(X_NUM_MODULES as f32, Y_NUM_MODULES as f32),
            ),
            ..Default::default()
        };

        draw_texture_ex(&gpu_piktogram, 0., 0., WHITE, params);

        board.reset(LED_OFF_COLOR);

        for light_point in &mut ligth_points {
            light_point.draw(&mut board, color_to_vec3(GREEN));
            light_point.step();
        }

        board.draw();

        next_frame().await
    }
}
