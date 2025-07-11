use std::{
    collections::VecDeque,
    hash::{DefaultHasher, Hash, Hasher},
    time::Duration,
};

use crate::{
    board::Board,
    ligth_point::LigthPoint,
    time_manager::{TimeManager, VirtualInstant},
};
use macroquad::{
    color::Color,
    math::{IVec2, Vec2},
};

struct Maschine {
    possible_products: Vec<ProductHash>,
    possible_input_warehouses: Vec<IVec2>,
}

#[derive(Clone, Hash)]
pub struct Step {
    maschine_pos: Vec<IVec2>,
    storage_pos: IVec2,
    production_time: Duration,
}
impl Step {
    pub fn new(time_in_seconds: f32, maschine_pos: Vec<IVec2>, storage_pos: IVec2) -> Self {
        Self {
            maschine_pos,
            storage_pos,
            production_time: Duration::from_secs_f32(time_in_seconds),
        }
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

#[derive(Clone, Copy, PartialEq, Eq)]
struct ProductHash {
    hash: u64,
}

pub struct Product {
    remaining_steps: VecDeque<Step>,
    ligth_point: LigthPoint,
    pub color: Color,
    state: State,
    product_hash: ProductHash,
}
impl Product {
    pub fn new(
        color: Color,
        mut steps: VecDeque<Step>,
        board: &Board,
        time_manager: &TimeManager,
    ) -> Self {
        assert!(steps.len() >= 2, "Fertigungsauftag needs atleast 2 steps");
        let start_pos = *steps.pop_front().unwrap().maschine_pos.first().unwrap();
        let target = steps.pop_front().unwrap();
        let target_pos = *target.maschine_pos.first().unwrap();

        let path = board.find_path(start_pos, target_pos);

        let ligth_point = LigthPoint::new(path, time_manager.clone());
        let mut hasher = DefaultHasher::new();
        steps.hash(&mut hasher);
        let product_hash = ProductHash {
            hash: hasher.finish(),
        };
        Self {
            remaining_steps: steps,
            ligth_point,
            color,
            state: State::Waiting {
                until: time_manager.now() + target.production_time,
                next_step: target,
            },
            product_hash,
        }
    }
    fn waiting_in_storage(&self) -> Vec2 {
        self.ligth_point.current()
    }
    pub fn next(&mut self, board: &mut Board, time_manager: &TimeManager) -> Option<Vec2> {
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
                let Some(&target) = next_step
                    .maschine_pos
                    .iter()
                    .find(|pos| !board[**pos].is_full())
                else {
                    return Some(self.waiting_in_storage());
                };
                board[self.ligth_point.current().as_ivec2()].in_production -= 1;

                self.ligth_point.set_new_target(target, board);
                board[target].in_production += 1;
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
