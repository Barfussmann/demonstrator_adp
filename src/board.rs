use glam::{Vec2, vec2};
#[cfg(target_arch = "x86_64")]
use macroquad::prelude::{Color, GREEN, draw_circle, draw_text, request_new_screen_size};
use palette::Srgb;
use std::array::from_fn;
use std::ops::Index;
use std::ops::IndexMut;
use std::time::Duration;

use crate::constants::*;
use crate::product::Product;
use crate::product::ProductPlan;
use crate::time_manager::TimeManager;
use crate::time_manager::VirtualInstant;
// use crate::

#[derive(Clone, Debug)]
pub enum ModuleState {
    Functional,
    Maintaining,
    Broken,
}

pub struct Module {
    pub pos: [i32; 2],
    pub in_production: u32,
    pub in_storage: u32,
    pub max_production: u32,
    pub brightness_x: [Srgb; LEDS_PER_DIR],
    pub brightness_y: [Srgb; LEDS_PER_DIR],
    pub state: ModuleState,
}

impl Module {
    pub fn new(pos: [i32; 2], color: Srgb) -> Self {
        Self {
            pos,
            in_production: 0,
            in_storage: 0,
            max_production: 1,
            brightness_x: [color; LEDS_PER_DIR],
            brightness_y: [color; LEDS_PER_DIR],
            state: ModuleState::Functional,
        }
    }
    pub fn colors(&self, flip: bool) -> Vec<Srgb> {
        let mut pixel_y = self.brightness_y.to_vec();
        let mut pixel_x = self.brightness_x[0..3].to_vec();
        pixel_x.extend_from_slice(&self.brightness_x[4..7]);
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
    pub fn draw_on_screen(&self) {
        let center = self.center();

        let text = format!("{}", self.in_production.clamp(0, 100));
        draw_text(&text, center.x + 20., center.y + 20., 40., GREEN);

        draw_led_strip(
            center - vec2(self.half_width(), 0.),
            center + vec2(self.half_width(), 0.),
            self.brightness_x,
        );
        draw_led_strip(
            center - vec2(0., self.half_width()),
            center + vec2(0., self.half_width()),
            self.brightness_y,
        );
    }
    pub fn reset(&mut self) {
        self.in_production = 0;
        self.state = ModuleState::Functional;
    }
    pub fn set_all_colors(&mut self, color: Srgb) {
        self.brightness_x = [color; LEDS_PER_DIR];
        self.brightness_y = [color; LEDS_PER_DIR];
    }

    pub fn iter_mut_leds(&mut self) -> impl Iterator<Item = ([f32; 2], &mut Srgb)> {
        let corner = [self.pos[0] as f32, self.pos[1] as f32];
        let side_x = [corner[0], corner[1] + 0.5];
        let side_y = [corner[0] + 0.5, corner[1]];

        let x_leds = (self.brightness_x)
            .iter_mut()
            .enumerate()
            .map(move |(i, color)| {
                let offset = (i as f32 + 0.5) / LEDS_PER_DIR as f32;
                let led_pos = [side_x[0] + offset, side_x[1]];
                (led_pos, color)
            });
        let y_leds = (self.brightness_y)
            .iter_mut()
            .enumerate()
            .map(move |(i, color)| {
                let offset = (i as f32 + 0.5) / LEDS_PER_DIR as f32;
                let led_pos = [side_y[0], side_y[1] + offset];
                (led_pos, color)
            });

        x_leds.chain(y_leds)
    }

    pub fn can_receiv_product(&self) -> bool {
        self.in_production < self.max_production && matches!(self.state, ModuleState::Functional)
    }

    pub fn draw(&mut self) {
        if self.is_storage() {
            self.draw_as_storage();
            return;
        }

        let color = match self.state {
            ModuleState::Functional => return,
            ModuleState::Maintaining => YELLOW,
            ModuleState::Broken => RED,
        };

        self.brightness_x[1..LEDS_PER_DIR - 1].fill(color);
        self.brightness_y[1..LEDS_PER_DIR - 1].fill(color);
    }
    fn is_storage(&self) -> bool {
        self.max_production > 1
    }
    pub fn draw_as_storage(&mut self) {
        let middle = LEDS_PER_DIR / 2;

        let middle_color = self.brightness_x[middle];

        for in_storage in 0..=self.in_storage {
            match in_storage {
                0 => {}
                1 => {} // middle is already lit
                2 => {
                    self.brightness_x[middle - 1] = middle_color;
                }
                3 => {
                    self.brightness_y[middle - 1] = middle_color;
                }
                4 => {
                    self.brightness_x[middle + 1] = middle_color;
                }
                5 => {
                    self.brightness_y[middle + 1] = middle_color;
                }
                _ => {
                    unreachable!("There are more in production then assumed")
                }
            }
        }
        self.in_storage = 0;
    }
}
#[cfg(target_arch = "x86_64")]
fn vec3_to_color(color: Srgb, alpha: f32) -> Color {
    Color::new(color.red, color.green, color.blue, alpha)
}
#[cfg(target_arch = "x86_64")]
pub fn color_to_vec3(color: Color) -> Srgb {
    Srgb::new(color.r, color.g, color.b)
}

#[cfg(target_arch = "x86_64")]
pub fn draw_led_strip(start: Vec2, end: Vec2, colors: [Srgb; LEDS_PER_DIR]) {
    let shift_per_led = (end - start) / colors.len() as f32;
    let radius = shift_per_led.length() / 2.;

    let mut pos = start + shift_per_led / 2.;
    for color in colors {
        let alpha = if color == LED_OFF_COLOR { 0.1 } else { 1.0 };

        draw_circle(pos.x, pos.y, radius, vec3_to_color(color, alpha));
        pos += shift_per_led;
    }
}

#[derive(Clone)]
pub struct MachineStateChange {
    time: Duration,
    state: ModuleState,
    pos: [i32; 2],
}
impl MachineStateChange {
    pub fn new(time: Duration, state: ModuleState, pos: [i32; 2]) -> Self {
        Self { time, state, pos }
    }
}

#[derive(Clone)]
pub struct Scenario {
    pub name: String,
    pub starting_steps: Vec<ProductPlan>,
    pub disturbance_steps: Vec<ProductPlan>,
    pub starting_time: VirtualInstant,
    pub pre_duration: Duration,
    pub disturbance_duration: Duration,
    pub state: ScenarioState,
    pub machine_state_changes: Vec<MachineStateChange>,
}
impl Scenario {
    pub fn starting_scenario() -> Scenario {
        Self {
            name: "Starting Scenario".to_string(),
            starting_steps: vec![STEPS_TOP_NORMAL.clone(), STEPS_BOTTOM_NORMAL.clone()],
            disturbance_steps: vec![STEPS_TOP_NORMAL.clone(), STEPS_BOTTOM_NORMAL.clone()],
            starting_time: VirtualInstant::zero(),
            pre_duration: Duration::from_secs(1_000_000),
            disturbance_duration: Duration::from_secs(1_000_000),
            state: ScenarioState::Start,
            machine_state_changes: Vec::new(),
        }
    }
    fn current_steps(&self) -> Vec<ProductPlan> {
        match self.state {
            ScenarioState::Start => self.starting_steps.clone(),
            ScenarioState::Disturbtion => self.disturbance_steps.clone(),
            ScenarioState::End => self.starting_steps.clone(),
        }
    }
    fn current_state_durration(&self) -> Duration {
        match self.state {
            ScenarioState::Start => self.pre_duration,
            ScenarioState::Disturbtion => self.disturbance_duration,
            ScenarioState::End => Duration::from_secs(1_000_000),
        }
    }
    #[must_use]
    fn update(&mut self, time: &TimeManager) -> Vec<MachineStateChange> {
        let elapsed = (time.now() - self.starting_time).inner();
        if elapsed >= self.current_state_durration() {
            self.starting_time = time.now();
            self.state = match self.state {
                ScenarioState::Start => ScenarioState::Disturbtion,
                ScenarioState::Disturbtion => ScenarioState::End,
                ScenarioState::End => ScenarioState::End,
            };
            println!("Went to state: {:?}", self.state);
        }
        let mut activated_machine_states = Vec::new();
        let mut machine_state_changes = std::mem::take(&mut self.machine_state_changes);
        machine_state_changes.retain(|machine_state_change| {
            let is_active = machine_state_change.time < time.now().inner();
            if is_active {
                activated_machine_states.push(machine_state_change.clone());
            }
            !is_active
        });
        self.machine_state_changes = machine_state_changes;
        activated_machine_states
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
        println!("Starting Scenario: {}", &scenario.name);
        for module in self.modules.as_flattened_mut() {
            module.reset();
        }
        self.current_scenario = scenario;
        self.time_manager.reset();
        self.products = Vec::new();
    }
    pub fn colors(&self) -> Vec<Srgb> {
        let mut colors = Vec::new();

        for x in 0..X_NUM_MODULES {
            let flip = x % 2 == 1;

            let iter = match flip {
                true => (0..Y_NUM_MODULES).rev().collect::<Vec<_>>(),
                false => (0..Y_NUM_MODULES).collect::<Vec<_>>(),
            };

            for y in iter {
                let module = &self.modules[y][x];
                colors.extend(module.colors(flip));
            }
        }

        colors
    }
    pub fn set_storage(&mut self, product_plan: ProductPlan) {
        for step in &product_plan.steps {
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
    pub fn iter_mut_leds(&mut self) -> impl Iterator<Item = ([f32; 2], &mut Srgb)> {
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
    pub fn draw_on_screen(&self) {
        for module in self.modules.as_flattened() {
            module.draw_on_screen();
        }
    }
    pub fn draw_modules(&mut self) {
        for module in self.modules.as_flattened_mut() {
            module.draw();
        }
    }
    pub fn reset(&mut self, color: Srgb) {
        for module in self.modules.as_flattened_mut() {
            module.set_all_colors(color);
        }
    }
    pub fn draw_light_point(&mut self, pos: [f32; 2], color: Srgb) {
        for (led_pos, led) in self.iter_mut_leds() {
            let diff = [led_pos[0] - pos[0], led_pos[1] - pos[1]];
            let distance = diff[0].hypot(diff[1]);
            let coloring_strength = (1. - distance / COLOR_RADIUS) * COLOR_STRENGTH;

            if coloring_strength >= 0. {
                *led += color * coloring_strength;
            } else {
                continue;
            }
        }
    }

    pub fn update(&mut self) {
        self.time_manager.update();
        let activated_machine_states = self.current_scenario.update(&self.time_manager);
        for machine_state in activated_machine_states {
            println!("Activated Statechange at: {:?}", machine_state.pos);
            println!(
                "From {:?} to {:?}",
                self[machine_state.pos].state, machine_state.state
            );
            self[machine_state.pos].state = machine_state.state;
        }

        let current_steps = self.current_scenario.current_steps();

        let mut new_products = Vec::new();
        for product in current_steps {
            let starting_maschine = product.steps[0].maschine_pos();
            if self[starting_maschine].can_receiv_product() {
                new_products.push((
                    Product::new(product.color, product.steps, &self.time_manager),
                    starting_maschine,
                ))
            }
        }
        for (product, starting_pos) in new_products {
            self[starting_pos].in_production += 1;
            self.products.push(product);
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
