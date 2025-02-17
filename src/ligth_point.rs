use macroquad::math::{Vec2, Vec3};

use crate::{COLOR_RADIUS, EPSILON, STEP_SIZE, board::Board};

pub struct LigthPoint {
    current: Vec2,
    start: Vec2,
    target: Vec2,
}

impl LigthPoint {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self {
            current: start,
            start,
            target: end,
        }
    }
    pub fn step(&mut self) {
        let mut dir = self.target - self.current;
        if dir.abs_diff_eq(Vec2::ZERO, EPSILON) {
            return;
        }

        if dir.y.abs() < EPSILON {
            dir.y = 0.;
        } else {
            dir.x = 0.;
        }

        dir = (dir.normalize() * STEP_SIZE).min(dir);
        self.current += dir;

        if self.current.abs_diff_eq(self.target, STEP_SIZE) {
            self.current = self.start;
        }
    }
    pub fn draw(&self, board: &mut Board, color: Vec3) {
        for (pos, led) in board.iter_mut_leds() {
            let coloring_strength = 1. - pos.distance(self.current) / COLOR_RADIUS;

            if coloring_strength >= 0. {
                *led += color * coloring_strength;
            } else {
                continue;
            }
        }
    }
}
