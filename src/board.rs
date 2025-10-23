#[cfg(target_arch = "x86_64")]
use macroquad::prelude::*;
use std::array::from_fn;
use std::collections::VecDeque;
use std::ops::Index;
use std::ops::IndexMut;

use crate::constants::*;
use crate::product::Step;

pub struct Module {
    pub pos: IVec2,
    pub in_production: u32,
    pub max_production: u32,
    pub brigthness_x: [[f32; 3]; LEDS_PER_DIR],
    pub brigthness_y: [[f32; 3]; LEDS_PER_DIR],
}

impl Module {
    pub fn new(pos: IVec2, color: [f32; 3]) -> Self {
        Self {
            pos,
            in_production: 0,
            max_production: 1,
            brigthness_x: [color; LEDS_PER_DIR],
            brigthness_y: [color; LEDS_PER_DIR],
        }
    }
    pub fn colors(&self, flip: bool) -> Vec<[f32; 3]> {
        let mut pixel_x = self.brigthness_x[0..3].to_vec();
        pixel_x.extend_from_slice(&self.brigthness_x[4..7]);
        let mut pixel_y = self.brigthness_y.to_vec();
        if flip {
            pixel_x.reverse();
            pixel_y.reverse();
        }
        pixel_x.into_iter().chain(pixel_y).collect()
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

        let text = format!("{}", self.in_production.clamp(0, 100));
        draw_text(&text, center.x + 20., center.y + 20., 40., GREEN);

        draw_led_strip(
            center - vec2(self.half_width(), 0.),
            center + vec2(self.half_width(), 0.),
            self.brigthness_x,
        );
        draw_led_strip(
            center - vec2(0., self.half_width()),
            center + vec2(0., self.half_width()),
            self.brigthness_y,
        );
    }
    pub fn reset(&mut self, color: [f32; 3]) {
        self.brigthness_x = [color; LEDS_PER_DIR];
        self.brigthness_y = [color; LEDS_PER_DIR];
    }

    pub fn iter_mut_leds(&mut self) -> impl Iterator<Item = (Vec2, &mut [f32; 3])> {
        let corner = self.pos.as_vec2();
        let side_x = vec2(corner.x, corner.y + 0.5);
        let side_y = vec2(corner.x + 0.5, corner.y);

        let x_leds = (self.brigthness_x)
            .iter_mut()
            .enumerate()
            .map(move |(i, color)| {
                let offset = (i as f32 + 0.5) / LEDS_PER_DIR as f32;
                let led_pos = side_x + vec2(offset, 0.);
                (led_pos, color)
            });
        let y_leds = (self.brigthness_y)
            .iter_mut()
            .enumerate()
            .map(move |(i, color)| {
                let offset = (i as f32 + 0.5) / LEDS_PER_DIR as f32;
                let led_pos = side_y + vec2(0., offset);
                (led_pos, color)
            });

        x_leds.chain(y_leds)
    }

    pub fn is_full(&self) -> bool {
        self.in_production >= self.max_production
    }
}
fn vec3_to_color(color: [f32; 3], alpha: f32) -> Color {
    Color::new(color[0], color[1], color[2], alpha)
}
pub fn color_to_vec3(color: Color) -> [f32; 3] {
    [color.r, color.g, color.b]
}

#[cfg(target_arch = "x86_64")]
pub fn draw_led_strip(start: Vec2, end: Vec2, colors: [[f32; 3]; LEDS_PER_DIR]) {
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
    pub modules: [[Module; X_NUM_MODULES]; Y_NUM_MODULES],
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn colors(&self) -> Vec<[f32; 3]> {
        self.modules
            .iter()
            .enumerate()
            .flat_map(|(y, row)| row.iter().flat_map(move |module| module.colors(y % 2 == 1)))
            .collect()
    }
    pub fn set_storage(&mut self, steps: VecDeque<Step>) {
        for step in &steps {
            if step.is_storage() {
                self[step.maschine_pos()].max_production = MAX_PRODUCT_IN_STORAGE;
            }
        }
    }
    pub fn inbounds(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < X_NUM_MODULES as i32 && pos.y < Y_NUM_MODULES as i32
    }
    pub fn new() -> Self {
        Self {
            modules: from_fn(|y| {
                from_fn(|x| Module::new(ivec2(x as i32, y as i32), LED_OFF_COLOR))
            }),
        }
    }
    pub fn iter_mut_leds(&mut self) -> impl Iterator<Item = (Vec2, &mut [f32; 3])> {
        self.modules
            .as_flattened_mut()
            .iter_mut()
            .flat_map(|module| module.iter_mut_leds())
    }
    #[cfg(target_arch = "x86_64")]
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
    pub fn reset(&mut self, color: [f32; 3]) {
        for module in self.modules.as_flattened_mut() {
            module.reset(color);
        }
    }
    pub fn draw_light_point(&mut self, pos: [f32; 2], color: [f32; 3]) {
        for (led_pos, led) in self.iter_mut_leds() {
            let diff = [led_pos[0] - pos[0], led_pos[1] - pos[1]];
            let distance = diff[0].hypot(diff[1]);
            let coloring_strength = 1. - distance / COLOR_RADIUS;

            if coloring_strength >= 0. {
                led[0] += color[0] * coloring_strength;
                led[1] += color[1] * coloring_strength;
                led[2] += color[2] * coloring_strength;
            } else {
                continue;
            }
        }
    }
}

impl Index<[i32; 2]> for Board {
    type Output = Module;

    fn index(&self, index: [i32; 2]) -> &Self::Output {
        &self.modules[index[1] as usize][index[0] as usize]
    }
}
impl IndexMut<[i32; 2]> for Board {
    fn index_mut(&mut self, index: [i32; 2]) -> &mut Self::Output {
        &mut self.modules[index[1] as usize][index[0] as usize]
    }
}
