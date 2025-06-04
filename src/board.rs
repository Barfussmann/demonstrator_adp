use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::ops::Index;
use std::ops::IndexMut;

use ::rand::rng;
use ::rand::seq::IndexedRandom;
use ::rand::seq::IteratorRandom;
use macroquad::prelude::*;

use crate::COLOR_RADIUS;
use crate::LED_OFF_COLOR;
use crate::ligth_point::LigthPoint;
use crate::module::BOARD_MUDLES;
use crate::module::ModuleType;

use super::LEDS_PER_DIR;
use super::PIXEL_PER_MODULE;
use super::X_NUM_MODULES;
use super::Y_NUM_MODULES;

#[derive(Debug, Clone, Copy)]
pub struct Module {
    pub module_type: ModuleType,
    pub pos: IVec2,
    pub in_production: u32,
    pub brigthness_x: [Vec3; LEDS_PER_DIR],
    pub brigthness_y: [Vec3; LEDS_PER_DIR],
}

impl Module {
    pub fn new(pos: IVec2, color: Vec3, module_type: ModuleType) -> Self {
        Self {
            module_type,
            pos,
            in_production: 0,
            brigthness_x: [color; LEDS_PER_DIR],
            brigthness_y: [color; LEDS_PER_DIR],
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
    pub fn reset(&mut self, color: Vec3) {
        self.brigthness_x = [color; LEDS_PER_DIR];
        self.brigthness_y = [color; LEDS_PER_DIR];
    }

    pub fn iter_mut_leds(&mut self) -> impl Iterator<Item = (Vec2, &mut Vec3)> {
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
        if self.module_type == ModuleType::Lager
            || self.module_type == ModuleType::Anlieferung
            || self.module_type == ModuleType::Versand
            || self.module_type == ModuleType::Kunde
        {
            false
        } else {
            if self.pos == ivec2(4, 0) && is_key_down(KeyCode::Q) {
                return true;
            }

            self.in_production > 0
        }
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
    pub modules: [[Module; X_NUM_MODULES]; Y_NUM_MODULES],
}

impl Board {
    pub fn find_path(&self, start: IVec2, target: IVec2) -> VecDeque<IVec2> {
        if start == target {
            return VecDeque::from([target]);
        }
        let mut queue = VecDeque::from([start]);
        let mut visited = HashSet::from([start]);
        let mut path = HashMap::new();

        let mut first_step = true;

        while let Some(current) = queue.pop_front() {
            for neighbor in self.neighbors(current) {
                if self[neighbor].module_type != ModuleType::Weg && neighbor != target {
                    continue;
                }
                if neighbor == target && first_step {
                    continue;
                }
                if visited.contains(&neighbor) {
                    continue;
                }
                queue.push_back(neighbor);
                visited.insert(neighbor);
                path.insert(neighbor, current);

                if neighbor == target {
                    return self.reconstruct_path(start, target, path);
                }
            }
            first_step = false;
        }
        println!("No Path found");
        VecDeque::new()
    }
    fn reconstruct_path(
        &self,
        start: IVec2,
        target: IVec2,
        path: HashMap<IVec2, IVec2>,
    ) -> VecDeque<IVec2> {
        let mut current = target;
        let mut full_path = VecDeque::from([target]);
        while let Some(&parent) = path.get(&current) {
            full_path.push_front(parent);
            current = parent;
        }
        full_path
    }
    fn neighbors(&self, pos: IVec2) -> Vec<IVec2> {
        const OFFSETS: [IVec2; 4] = [ivec2(-1, 0), ivec2(1, 0), ivec2(0, -1), ivec2(0, 1)];
        OFFSETS
            .map(|offset| pos + offset)
            .into_iter()
            .filter(|neighbor| self.inbounds(*neighbor))
            .collect()
    }
    pub fn inbounds(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < X_NUM_MODULES as i32 && pos.y < Y_NUM_MODULES as i32
    }
    pub fn new() -> Self {
        let mut modules = [[Module::new(IVec2::ZERO, LED_OFF_COLOR, ModuleType::Anlieferung);
            Y_NUM_MODULES]; X_NUM_MODULES];

        for y in 0..Y_NUM_MODULES {
            for x in 0..X_NUM_MODULES {
                let module_type = BOARD_MUDLES[y][x];
                let module = Module::new(ivec2(x as i32, y as i32), LED_OFF_COLOR, module_type);
                modules[y][x] = module;
            }
        }
        Self { modules }
    }
    pub fn filter_modules(&self, module_type: ModuleType) -> impl Iterator<Item = &Module> {
        self.modules
            .as_flattened()
            .iter()
            .filter(move |module| module.module_type == module_type)
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
    pub fn draw_ligth_point(&mut self, pos: Vec2, color: Vec3) {
        for (led_pos, led) in self.iter_mut_leds() {
            let coloring_strength = 1. - led_pos.distance(pos) / COLOR_RADIUS;

            if coloring_strength >= 0. {
                *led += color * coloring_strength;
            } else {
                continue;
            }
        }
    }
}

impl Index<IVec2> for Board {
    type Output = Module;

    fn index(&self, index: IVec2) -> &Self::Output {
        &self.modules[index.y as usize][index.x as usize]
    }
}
impl IndexMut<IVec2> for Board {
    fn index_mut(&mut self, index: IVec2) -> &mut Self::Output {
        &mut self.modules[index.y as usize][index.x as usize]
    }
}
