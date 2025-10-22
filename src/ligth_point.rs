use std::collections::VecDeque;

use macroquad::math::{IVec2, Vec2};

use crate::{
    constants::{EPSILON, STEP_SIZE},
    time_manager::TimeManager,
};

const HALVE: Vec2 = Vec2::new(0.5, 0.5);
#[derive(Clone)]
pub struct LigthPoint {
    current: Vec2,
    target: Vec2,
    remaining_path: VecDeque<IVec2>,
    time_manager: TimeManager,
}

impl Iterator for LigthPoint {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        let mut dir = self.target - self.current;

        if dir.abs_diff_eq(Vec2::ZERO, EPSILON) {
            self.target = self.remaining_path.pop_front()?.as_vec2() + HALVE;
        }

        if dir.y.abs() < EPSILON {
            dir.y = 0.;
        } else {
            dir.x = 0.;
        }
        self.current += dir.clamp_length_max(STEP_SIZE * self.time_manager.last_virtual_delta());

        Some(self.current)
    }
}

impl LigthPoint {
    pub fn new(mut path: VecDeque<IVec2>, time_manager: TimeManager) -> Self {
        Self {
            current: path.pop_front().unwrap().as_vec2() + HALVE,
            target: path.pop_front().unwrap().as_vec2() + HALVE,
            remaining_path: path,
            time_manager,
        }
    }
    pub fn current(&self) -> Vec2 {
        self.current
    }
    pub fn target(&self) -> IVec2 {
        *self.remaining_path.back().unwrap()
    }
    pub fn set_new_target(&mut self, path: VecDeque<IVec2>) {
        self.remaining_path = path;

        self.target = self.remaining_path.pop_front().unwrap().as_vec2() + HALVE;
    }
}
