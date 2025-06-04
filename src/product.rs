use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use macroquad::{
    color::Color,
    math::{IVec2, Vec2, Vec3},
};
use rand::{prelude::*, rng};

use crate::{board::Board, ligth_point::LigthPoint, module::ModuleType};

#[derive(Clone)]
pub struct Step {
    pos: Vec<IVec2>,
    production_time: Duration,
}
impl Step {
    pub fn new(time_in_seconds: f32, pos: Vec<IVec2>) -> Self {
        Self {
            pos,
            production_time: Duration::from_secs_f32(time_in_seconds),
        }
    }
}

#[derive(Clone)]
enum State {
    Waiting { until: Instant, next_step: Step },
    // Finished { next_step: Step },
    Moving { target_wait: Duration },
    WaitingForFreeMaschine { next_step: Step },
}

pub struct Product {
    remaining_steps: VecDeque<Step>,
    ligth_point: LigthPoint,
    wait_till: Option<Instant>,
    pub color: Color,
    state: State,
}
impl Product {
    pub fn new(color: Color, mut steps: VecDeque<Step>, board: &Board) -> Self {
        assert!(steps.len() >= 2, "Fertigungsauftag needs atleast 2 steps");
        let start_pos = *steps.pop_front().unwrap().pos.first().unwrap();
        let target = steps.pop_front().unwrap();
        let target_pos = *target.pos.first().unwrap();

        let path = board.find_path(start_pos, target_pos);

        let ligth_point = LigthPoint::new(path);
        Self {
            remaining_steps: steps,
            ligth_point,
            wait_till: None,
            color,
            state: State::Waiting {
                until: Instant::now() + target.production_time,
                next_step: target,
            },
        }
    }
    fn waiting_in_storage(&self) -> Vec2 {
        self.ligth_point.current()
    }
    pub fn next(&mut self, board: &mut Board) -> Option<Vec2> {
        match &self.state {
            State::Waiting { until, next_step } => {
                if Instant::now() > *until {
                    self.state = State::WaitingForFreeMaschine {
                        next_step: next_step.clone(),
                    };
                }
                Some(self.waiting_in_storage())
            }
            State::WaitingForFreeMaschine { next_step } => {
                let Some(&target) = next_step.pos.iter().find(|pos| !board[**pos].is_full()) else {
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
                        until: Instant::now() + *target_wait,
                        next_step: step,
                    };
                    Some(self.ligth_point.current())
                }
            }
        }

        // if let Some(wait_till) = self.wait_till {
        //     if Instant::now() < wait_till {
        //         return Some(self.ligth_point.current());
        //     } else {
        //         self.wait_till = None;
        //         board[self.ligth_point.current().as_ivec2()].in_production -= 1;
        //         board[self.ligth_point.target()].in_production += 1;

        //         // board[self.ligth_point.target()].in_production -= 1;
        //     }
        // }

        // if let Some(pos) = self.ligth_point.next() {
        //     Some(pos)
        // } else {
        //     if self.remaining_steps.is_empty() {
        //         return None;
        //     }

        //     let next_step = self.remaining_steps.pop_front().unwrap();

        //     let possible_maschines = next_step
        //         .pos
        //         .iter()
        //         .filter(|pos| !board[**pos].is_full())
        //         .collect::<Vec<_>>();

        //     let Some(&&target) = possible_maschines.choose(&mut rng()) else {
        //         println!("full");
        //         self.remaining_steps.push_front(next_step);
        //         return Some(self.ligth_point.current());
        //     };

        //     self.ligth_point.set_new_target(target, board);

        //     // self.wait_till = Some(Instant::now() + self.next_production_time);
        //     // self.next_production_time = next_step.production_time;

        //     // Some(self.ligth_point.current())
        //     // self.next(board)
        // }
    }
}
