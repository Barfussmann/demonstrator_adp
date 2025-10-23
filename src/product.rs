use std::{collections::VecDeque, hash::Hash, time::Duration};

use crate::{
    board::Board,
    ligth_point::LigthPoint,
    time_manager::{TimeManager, VirtualInstant},
};
#[cfg(target_arch = "x86_64")]
use macroquad::color::Color;

#[derive(Clone, Hash)]
pub struct Step {
    path: Vec<[i32; 2]>,
    maschine_pos: [i32; 2],
    production_time: Duration,
    is_storage: bool,
}
impl Step {
    pub fn new(
        time_in_seconds: f32,
        maschine_pos: [i32; 2],
        path: Vec<[i32; 2]>,
        storage: bool,
    ) -> Self {
        Self {
            path,
            maschine_pos,
            production_time: Duration::from_secs_f32(time_in_seconds),
            is_storage: storage,
        }
    }
    fn path(&self) -> VecDeque<[i32; 2]> {
        let mut path = VecDeque::from(self.path.clone());
        path.push_back(self.maschine_pos);
        path
    }

    pub fn is_storage(&self) -> bool {
        self.is_storage
    }
    pub fn maschine_pos(&self) -> [i32; 2] {
        self.maschine_pos
    }
}

#[derive(Clone)]
enum State {
    Waiting {
        until: VirtualInstant,
        next_step: Step,
    },
    // Finished { next_step: Step },
    Moving {
        target_wait: Duration,
    },
    WaitingForFreeMaschine {
        next_step: Step,
    },
}

pub struct Product {
    remaining_steps: VecDeque<Step>,
    ligth_point: LigthPoint,
    pub color: Color,
    state: State,
}
impl Product {
    pub fn new(color: Color, mut steps: VecDeque<Step>, time_manager: &TimeManager) -> Self {
        assert!(steps.len() >= 2, "Fertigungsauftag needs atleast 2 steps");
        let step = steps.pop_front().unwrap();

        let path = step.path();

        let ligth_point = LigthPoint::new(path, time_manager.clone());
        Self {
            state: State::Waiting {
                until: time_manager.now() + step.production_time,
                next_step: steps.pop_front().unwrap(),
            },
            remaining_steps: steps,
            ligth_point,
            color,
        }
    }
    pub fn finish(&self, board: &mut Board) {
        board[self.ligth_point.current_i32x2()].in_production -= 1;
    }
    fn waiting_in_storage(&self) -> [f32; 2] {
        self.ligth_point.current()
    }
    pub fn next(&mut self, board: &mut Board, time_manager: &TimeManager) -> Option<[f32; 2]> {
        match &self.state {
            State::Waiting { until, next_step } => {
                if time_manager.now() >= *until {
                    self.state = State::WaitingForFreeMaschine {
                        next_step: next_step.clone(),
                    };
                }
                Some(self.waiting_in_storage())
            }
            State::WaitingForFreeMaschine { next_step } => {
                if board[next_step.maschine_pos].is_full() {
                    return Some(self.waiting_in_storage());
                };
                board[self.ligth_point.current_i32x2()].in_production -= 1;

                self.ligth_point.set_new_target(next_step.path());
                board[next_step.maschine_pos].in_production += 1;
                self.state = State::Moving {
                    target_wait: next_step.production_time,
                };
                Some(self.ligth_point.current())
            }
            State::Moving { target_wait } => {
                if let Some(pos) = self.ligth_point.next() {
                    Some(pos)
                } else {
                    let step = self.remaining_steps.pop_front()?;
                    self.state = State::Waiting {
                        until: time_manager.now() + *target_wait,
                        next_step: step,
                    };
                    Some(self.ligth_point.current())
                }
            }
        }
    }
}
