use std::collections::VecDeque;

use crate::{
    constants::{EPSILON, STEP_SIZE},
    time_manager::TimeManager,
};

#[derive(Clone)]
pub struct LigthPoint {
    current: [f32; 2],
    target: [f32; 2],
    remaining_path: VecDeque<[i32; 2]>,
    time_manager: TimeManager,
}

impl Iterator for LigthPoint {
    type Item = [f32; 2];

    fn next(&mut self) -> Option<Self::Item> {
        let mut dir = [
            self.target[0] - self.current[0],
            self.target[1] - self.current[1],
        ];

        if dir[0].abs() < EPSILON && dir[1].abs() < EPSILON {
            let path = self.remaining_path.pop_front()?;

            self.target[0] = path[0] as f32 + 0.5;
            self.target[1] = path[1] as f32 + 0.5;
        }

        if dir[1].abs() < EPSILON {
            dir[1] = 0.;
        } else {
            dir[0] = 0.;
        }
        let length = dir[0].hypot(dir[1]);
        let new_length = length.clamp(0.0, STEP_SIZE * self.time_manager.last_virtual_delta());
        let mult = new_length / length;
        if mult.is_finite() {
            self.current[0] += dir[0] * mult;
            self.current[1] += dir[1] * mult;
        }

        Some(self.current)
    }
}

impl LigthPoint {
    pub fn new(mut path: VecDeque<[i32; 2]>, time_manager: TimeManager) -> Self {
        let current = path.pop_front().unwrap();
        let target = path.pop_front().unwrap();

        Self {
            current: [current[0] as f32 + 0.5, current[1] as f32 + 0.5],
            target: [target[0] as f32 + 0.5, target[1] as f32 + 0.5],
            remaining_path: path,
            time_manager,
        }
    }
    pub fn current_i32x2(&self) -> [i32; 2] {
        [self.current[0] as i32, self.current[1] as i32]
    }
    pub fn current(&self) -> [f32; 2] {
        self.current
    }
    pub fn target(&self) -> [i32; 2] {
        *self.remaining_path.back().unwrap()
    }
    pub fn set_new_target(&mut self, path: VecDeque<[i32; 2]>) {
        self.remaining_path = path;

        let next_target = self.remaining_path.pop_front().unwrap();
        self.target = [next_target[0] as f32 + 0.5, next_target[1] as f32 + 0.5];
    }
}
