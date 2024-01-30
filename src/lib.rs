/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/steps
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use std::collections::VecDeque;

use tick_id::TickId;

pub mod pending_steps;

#[derive(Debug, PartialEq, Eq)]
pub enum Step<T> {
    Forced,
    WaitingForReconnect,
    Custom(T),
}

pub struct ParticipantStep<T> {
    pub participant_id: u8,
    pub step: Step<T>,
}

pub struct ParticipantSteps<T> {
    pub steps: Vec<ParticipantStep<T>>,
}

impl<T> Default for ParticipantSteps<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ParticipantStep<T> {
    pub fn new(participant_id: u8, step: Step<T>) -> Self {
        Self {
            participant_id,
            step,
        }
    }
}

impl<T> ParticipantSteps<T> {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn push(&mut self, participant_id: u8, step: Step<T>) {
        self.steps.push(ParticipantStep::new(participant_id, step));
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

pub struct StepInfo<T> {
    pub step: Step<T>,
    pub tick_id: TickId,
}

pub struct Steps<T> {
    steps: VecDeque<StepInfo<T>>,
    expected_read_id: TickId,
    expected_write_id: TickId,
}

impl<T> Default for Steps<T> {
    fn default() -> Self {
        Self::new()
    }
}

const TICK_ID_MAX: u32 = u32::MAX;

impl<T> Steps<T> {
    pub fn new() -> Self {
        Self {
            steps: VecDeque::new(),
            expected_read_id: TickId::new(TICK_ID_MAX),
            expected_write_id: TickId::new(TICK_ID_MAX),
        }
    }
    pub fn new_with_initial_tick(initial_tick_id: TickId) -> Self {
        Self {
            steps: VecDeque::new(),
            expected_read_id: initial_tick_id,
            expected_write_id: initial_tick_id,
        }
    }

    pub fn push(&mut self, step: Step<T>) {
        let info = StepInfo {
            step,
            tick_id: self.expected_write_id,
        };
        self.steps.push_back(info);
        self.expected_write_id += 1;
    }

    pub fn pop(&mut self) -> Option<StepInfo<T>> {
        let info = self.steps.pop_front();
        if let Some(ref step_info) = info {
            assert_eq!(step_info.tick_id, self.expected_read_id);
            self.expected_read_id += 1;
        }
        info
    }

    pub fn pop_up_to(&mut self, tick_id: TickId) {
        while let Some(info) = self.steps.front() {
            if info.tick_id >= tick_id {
                break;
            }

            self.steps.pop_front();
        }
    }

    pub fn pop_count(&mut self, count: usize) {
        if count >= self.steps.len() {
            self.steps.clear();
        } else {
            self.steps.drain(..count);
        }
    }

    pub fn front_tick_id(&self) -> Option<TickId> {
        self.steps.front().map(|step_info| step_info.tick_id)
    }

    pub fn back_tick_id(&self) -> Option<TickId> {
        self.steps.back().map(|step_info| step_info.tick_id)
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::Step::Custom;

    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum GameInput {
        Jumping(bool),
        MoveHorizontal(i32),
    }

    #[test]
    fn add_step() {
        let mut steps = Steps::<GameInput>::new_with_initial_tick(TickId(23));
        steps.push(Custom(GameInput::MoveHorizontal(-2)));
        assert_eq!(steps.len(), 1);
        assert_eq!(steps.front_tick_id().unwrap().value(), 23)
    }

    #[test]
    fn push_and_pop_step() {
        let mut steps = Steps::<GameInput>::new_with_initial_tick(TickId(23));
        steps.push(Custom(GameInput::Jumping(true)));
        steps.push(Custom(GameInput::MoveHorizontal(42)));
        assert_eq!(steps.len(), 2);
        assert_eq!(steps.front_tick_id().unwrap().value(), 23);
        assert_eq!(steps.pop().unwrap().step, Custom(GameInput::Jumping(true)));
        assert_eq!(steps.front_tick_id().unwrap().value(), 24);
    }

    #[test]
    fn push_and_pop_count() {
        let mut steps = Steps::<GameInput>::new_with_initial_tick(TickId(23));
        steps.push(Custom(GameInput::Jumping(true)));
        steps.push(Custom(GameInput::MoveHorizontal(42)));
        assert_eq!(steps.len(), 2);
        steps.pop_count(8);
        assert_eq!(steps.len(), 0);
    }

    #[test]
    fn push_and_pop_up_to_lower() {
        let mut steps = Steps::<GameInput>::new_with_initial_tick(TickId(23));
        steps.push(Custom(GameInput::Jumping(true)));
        steps.push(Custom(GameInput::MoveHorizontal(42)));
        assert_eq!(steps.len(), 2);
        steps.pop_up_to(TickId(1));
        assert_eq!(steps.len(), 2);
    }

    #[test]
    fn push_and_pop_up_to_equal() {
        let mut steps = Steps::<GameInput>::new_with_initial_tick(TickId(23));
        steps.push(Custom(GameInput::Jumping(true)));
        steps.push(Custom(GameInput::MoveHorizontal(42)));
        assert_eq!(steps.len(), 2);
        steps.pop_up_to(TickId::new(24));
        assert_eq!(steps.len(), 1);
    }
}
