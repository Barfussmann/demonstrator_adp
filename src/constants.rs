use macroquad::math::Vec3;

pub const X_NUM_MODULES: usize = 10;
pub const Y_NUM_MODULES: usize = 10;
pub const DRAW_SCALE: f32 = 1.0;
pub const PIXEL_PER_MODULE: f32 = DRAW_SCALE * 100.;
pub const LEDS_PER_DIR: usize = 14;
pub const STEP_SIZE: f32 = 10.;
pub const COLOR_RADIUS: f32 = 0.1;
pub const EPSILON: f32 = 1e-4;

pub const LED_OFF_COLOR: Vec3 = Vec3::new(0.0, 0.0, 0.0);

pub const PIKTOGRAM_PATH: &str = "./assets/demonstrator_piktogramme.png";
