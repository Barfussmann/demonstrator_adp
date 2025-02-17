use macroquad::prelude::*;

use crate::LED_OFF_COLOR;

use super::LEDS_PER_DIR;
use super::PIXEL_PER_MODULE;
use super::X_NUM_MODULES;
use super::Y_NUM_MODULES;

#[derive(Debug, Clone, Copy)]
pub struct Module {
    pub pos: IVec2,
    pub colors_x: [Vec3; LEDS_PER_DIR],
    pub colors_y: [Vec3; LEDS_PER_DIR],
}

impl Module {
    pub fn new(pos: IVec2, color: Vec3) -> Self {
        Self {
            pos,
            colors_x: [color; LEDS_PER_DIR],
            colors_y: [color; LEDS_PER_DIR],
        }
    }
    pub fn corner(&self) -> Vec2 {
        self.pos.as_vec2() * PIXEL_PER_MODULE
    }
    pub fn center(&self) -> Vec2 {
        self.corner() + Vec2::splat(self.half_width())
    }
    pub fn half_width(&self) -> f32 {
        PIXEL_PER_MODULE / 2.
    }
    pub fn draw(&self) {
        let center = self.center();

        draw_led_strip(
            center - vec2(self.half_width(), 0.),
            center + vec2(self.half_width(), 0.),
            self.colors_x,
        );
        draw_led_strip(
            center - vec2(0., self.half_width()),
            center + vec2(0., self.half_width()),
            self.colors_y,
        );
    }
    pub fn reset(&mut self, color: Vec3) {
        self.colors_x = [color; LEDS_PER_DIR];
        self.colors_y = [color; LEDS_PER_DIR];
    }

    pub fn iter_mut_leds(&mut self) -> impl Iterator<Item = (Vec2, &mut Vec3)> {
        let corner = self.pos.as_vec2();
        let side_x = vec2(corner.x, corner.y + 0.5);
        let side_y = vec2(corner.x + 0.5, corner.y);

        let x_leds = (self.colors_x)
            .iter_mut()
            .enumerate()
            .map(move |(i, color)| {
                let offset = (i as f32 + 0.5) / LEDS_PER_DIR as f32;
                let led_pos = side_x + vec2(offset, 0.);
                (led_pos, color)
            });
        let y_leds = (self.colors_y)
            .iter_mut()
            .enumerate()
            .map(move |(i, color)| {
                let offset = (i as f32 + 0.5) / LEDS_PER_DIR as f32;
                let led_pos = side_y + vec2(0., offset);
                (led_pos, color)
            });

        x_leds.chain(y_leds)
    }
}
fn vec3_to_color(color: Vec3, alpha: f32) -> Color {
    Color::new(color.x, color.y, color.z, alpha)
}
pub fn color_to_vec3(color: Color) -> Vec3 {
    Vec3::new(color.r, color.g, color.b)
}

pub fn draw_led_strip(start: Vec2, end: Vec2, colors: [Vec3; LEDS_PER_DIR]) {
    let shift_per_led = (end - start) / colors.len() as f32;
    let radius = shift_per_led.length() / 2.;

    let mut pos = start + shift_per_led / 2.;
    for color in colors {
        let alpha = if color == LED_OFF_COLOR { 0.1 } else { 1.0 };

        draw_circle(pos.x, pos.y, radius, vec3_to_color(color, alpha));
        pos += shift_per_led;
    }
}

pub struct Board {
    pub modules: [[Module; Y_NUM_MODULES]; X_NUM_MODULES],
}

impl Board {
    pub fn new() -> Self {
        let mut modules = [[Module::new(IVec2::ZERO, LED_OFF_COLOR); Y_NUM_MODULES]; X_NUM_MODULES];

        for y in 0..Y_NUM_MODULES {
            for x in 0..X_NUM_MODULES {
                let module = Module::new(ivec2(x as i32, y as i32), LED_OFF_COLOR);
                modules[x][y] = module;
            }
        }
        Self { modules }
    }
    pub fn iter_mut_leds(&mut self) -> impl Iterator<Item = (Vec2, &mut Vec3)> {
        self.modules
            .as_flattened_mut()
            .iter_mut()
            .flat_map(|module| module.iter_mut_leds())
    }
    pub fn set_screen_size() {
        request_new_screen_size(
            X_NUM_MODULES as f32 * PIXEL_PER_MODULE,
            Y_NUM_MODULES as f32 * PIXEL_PER_MODULE,
        );
    }
    pub fn draw(&self) {
        for module in self.modules.as_flattened() {
            module.draw();
        }
    }
    pub fn reset(&mut self, color: Vec3) {
        for module in self.modules.as_flattened_mut() {
            module.reset(color);
        }
    }
}
