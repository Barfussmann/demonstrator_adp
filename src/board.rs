use macroquad::prelude::state_machine::State;
#[cfg(target_arch = "x86_64")]
use macroquad::prelude::*;
use std::array::from_fn;
use std::collections::VecDeque;
use std::ops::Index;
use std::ops::IndexMut;
use std::time::Duration;

use crate::constants::*;
use crate::product::Product;
use crate::product::Step;
use crate::time_manager::TimeManager;
use crate::time_manager::VirtualInstant;
// use crate::

pub struct Module {
    pub pos: [i32; 2],
    pub in_production: u32,
    pub max_production: u32,
    pub brigthness_x: [[f32; 3]; LEDS_PER_DIR],
    pub brigthness_y: [[f32; 3]; LEDS_PER_DIR],
}

impl Module {
    pub fn new(pos: [i32; 2], color: [f32; 3]) -> Self {
        Self {
            pos,
            in_production: 0,
            max_production: 1,
            brigthness_x: [color; LEDS_PER_DIR],
            brigthness_y: [color; LEDS_PER_DIR],
        }
    }
    pub fn colors(&self, flip: bool) -> Vec<[f32; 3]> {
        let mut pixel_y = self.brigthness_y[0..3].to_vec();
        pixel_y.extend_from_slice(&self.brigthness_y[4..7]);
        let mut pixel_x = self.brigthness_x.to_vec();
        if flip {
            pixel_y.reverse();
            pixel_x.reverse();
        }
        pixel_y.into_iter().chain(pixel_x).collect()
    }
    #[cfg(target_arch = "x86_64")]
    pub fn corner(&self) -> Vec2 {
        vec2(self.pos[0] as f32, self.pos[1] as f32) * PIXEL_PER_MODULE
    }
    #[cfg(target_arch = "x86_64")]
    pub fn center(&self) -> Vec2 {
        self.corner() + Vec2::splat(self.half_width())
    }
    pub fn half_width(&self) -> f32 {
        PIXEL_PER_MODULE / 2.
    }
    #[cfg(target_arch = "x86_64")]
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

    pub fn iter_mut_leds(&mut self) -> impl Iterator<Item = ([f32; 2], &mut [f32; 3])> {
        let corner = [self.pos[0] as f32, self.pos[1] as f32];
        let side_x = [corner[0], corner[1] + 0.5];
        let side_y = [corner[0] + 0.5, corner[1]];

        let x_leds = (self.brigthness_x)
            .iter_mut()
            .enumerate()
            .map(move |(i, color)| {
                let offset = (i as f32 + 0.5) / LEDS_PER_DIR as f32;
                let led_pos = [side_x[0] + offset, side_x[1]];
                (led_pos, color)
            });
        let y_leds = (self.brigthness_y)
            .iter_mut()
            .enumerate()
            .map(move |(i, color)| {
                let offset = (i as f32 + 0.5) / LEDS_PER_DIR as f32;
                let led_pos = [side_y[0], side_y[1] + offset];
                (led_pos, color)
            });

        x_leds.chain(y_leds)
    }

    pub fn is_full(&self) -> bool {
        self.in_production >= self.max_production
    }
}
#[cfg(target_arch = "x86_64")]
fn vec3_to_color(color: [f32; 3], alpha: f32) -> Color {
    Color::new(color[0], color[1], color[2], alpha)
}
#[cfg(target_arch = "x86_64")]
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

pub struct Scenario {
    pub starting_steps: [VecDeque<Step>; 2],
    pub disturbance_steps: [VecDeque<Step>; 2],
    pub starting_time: VirtualInstant,
    pub pre_duration: Duration,
    pub disturbance_duration: Duration,
    pub state: ScenarioState,
}
impl Scenario {
    fn starting_scenario() -> Scenario {
        Self {
            starting_steps: [STEPS_TOP_NORMAL.clone(), STEPS_BOTTOM_NORMAL.clone()],
            disturbance_steps: [STEPS_TOP_NORMAL.clone(), STEPS_BOTTOM_NORMAL.clone()],
            starting_time: VirtualInstant::zero(),
            pre_duration: Duration::from_secs(1_000_000),
            disturbance_duration: Duration::from_secs(1_000_000),
            state: ScenarioState::Start,
        }
    }
    fn current_steps(&self) -> [VecDeque<Step>; 2] {
        match self.state {
            ScenarioState::Start => self.starting_steps.clone(),
            ScenarioState::Disturbtion => self.disturbance_steps.clone(),
            ScenarioState::End => self.starting_steps.clone(),
        }
    }
    fn current_duration(&self) -> Duration {
        match self.state {
            ScenarioState::Start => self.pre_duration,
            ScenarioState::Disturbtion => self.disturbance_duration,
            ScenarioState::End => Duration::from_secs(1_000_000),
        }
    }
    fn update(&mut self, time: &TimeManager) {
        let elapsed = (time.now() - self.starting_time).inner();
        if elapsed >= self.current_duration() {
            self.starting_time = time.now();
            self.state = match self.state {
                ScenarioState::Start => ScenarioState::Disturbtion,
                ScenarioState::Disturbtion => ScenarioState::End,
                ScenarioState::End => ScenarioState::End,
            };
            println!("Went to state: {:?}", self.state);
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum ScenarioState {
    Start,
    Disturbtion,
    End,
}

pub struct Board {
    pub modules: [[Module; X_NUM_MODULES]; Y_NUM_MODULES],
    current_scenario: Scenario,
    pub time_manager: TimeManager,
    products: Vec<Product>,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn set_scenario(&mut self, scenario: Scenario) {
        self.current_scenario = scenario;
        self.time_manager.reset();
        self.products = Vec::new();
    }
    pub fn colors(&self) -> Vec<[f32; 3]> {
        let mut colors = Vec::new();

        for x in 0..X_NUM_MODULES {
            for y in 0..Y_NUM_MODULES {
                let module = &self.modules[y][x];
                let flip = x % 2 == 1;
                colors.extend(module.colors(flip));
            }
        }

        colors
    }
    pub fn set_storage(&mut self, steps: VecDeque<Step>) {
        for step in &steps {
            if step.is_storage() {
                self[step.maschine_pos()].max_production = MAX_PRODUCT_IN_STORAGE;
            }
        }
    }
    pub fn inbounds(&self, pos: [i32; 2]) -> bool {
        pos[0] >= 0 && pos[1] >= 0 && pos[0] < X_NUM_MODULES as i32 && pos[1] < Y_NUM_MODULES as i32
    }
    pub fn new() -> Self {
        Self {
            modules: from_fn(|y| from_fn(|x| Module::new([x as i32, y as i32], LED_OFF_COLOR))),
            time_manager: TimeManager::new(),
            current_scenario: Scenario::starting_scenario(),
            products: Vec::new(),
        }
    }
    pub fn iter_mut_leds(&mut self) -> impl Iterator<Item = ([f32; 2], &mut [f32; 3])> {
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
    #[cfg(target_arch = "x86_64")]
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

    pub fn update(&mut self) {
        self.time_manager.update();
        self.current_scenario.update(&self.time_manager);

        let current_steps = self.current_scenario.current_steps();
        let top_spawning_pos = current_steps[0][0].maschine_pos();
        let bottom_spawning_pos = current_steps[1][0].maschine_pos();

        // we have to move the check before the in_production += 1 to allow double spawning
        let is_bottom_full = self[bottom_spawning_pos].is_full();
        if !self[top_spawning_pos].is_full() {
            self.products.push(Product::new(
                [0.00, 0.89, 0.19],
                current_steps[0].clone(),
                &self.time_manager,
            ));
            self[top_spawning_pos].in_production += 1;
        }
        if !is_bottom_full {
            self.products.push(Product::new(
                [0.90, 0.16, 0.22],
                current_steps[1].clone(),
                &self.time_manager,
            ));
            self[bottom_spawning_pos].in_production += 1;
        }
        let mut products = std::mem::take(&mut self.products);
        products.retain_mut(|product: &mut Product| {
            let Some(light_point_pos) = product.next(self) else {
                product.finish(self);
                return false;
            };
            self.draw_light_point(light_point_pos, product.color);
            true
        });
        self.products = products;
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
