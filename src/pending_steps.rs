/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/steps
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use discoid::discoid::DiscoidBuffer;

use crate::{Step, TickId};

pub struct PendingStepInfo<T> {
    pub step: Step<T>,
    pub tick_id: TickId,
}

pub struct PendingSteps<T> {
    steps: DiscoidBuffer<PendingStepInfo<T>>,
    front_tick_id: TickId,
    capacity: usize,
}

impl<T> PendingSteps<T> {
    pub fn new(window_size: usize, tick_id: TickId) -> Self {
        Self {
            steps: DiscoidBuffer::new(window_size),
            front_tick_id: tick_id,
            capacity: window_size,
        }
    }

    pub fn set(&mut self, tick_id: TickId, step: Step<T>) -> Result<(), String> {
        let index_in_discoid = tick_id.value() - self.front_tick_id.value();
        if index_in_discoid >= self.capacity as u32 { // self.steps.capacity()
            return Err("pending_steps: out of scope".to_string());
        }

        self.steps.set_at_index(index_in_discoid as usize, PendingStepInfo::<T> {
            step,
            tick_id,
        });
        Ok(())
    }

    pub fn discard_up_to(&mut self, tick_id: TickId) {
        let count_in_discoid = tick_id - self.front_tick_id;
        if count_in_discoid < 0 {
            return;
        }
        self.steps.discard_front(count_in_discoid as usize);
    }

    pub fn is_empty(&self) -> bool {
        self.steps.get_at_index(0).is_none()
    }

    pub fn pop(&mut self) -> &PendingStepInfo<T> {
        let value = self.steps.get_at_index(0).unwrap();
        //self.steps.discard_front(1);
        value
    }

    pub fn front_tick_id(&self) -> Option<TickId> {
        self.steps.get_at_index(0).map(|info| info.tick_id)
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
        let mut steps = PendingSteps::<GameInput>::new(32, TickId(10));
        let first_tick_id = TickId(12);
        steps.set(first_tick_id, Custom(GameInput::MoveHorizontal(-2))).expect("this should work");
        assert_eq!(steps.front_tick_id(), None);
        assert_eq!(steps.is_empty(), true);
        steps.set(first_tick_id - 2, Custom(GameInput::Jumping(false))).expect("this should work");
        assert_eq!(steps.is_empty(), false);
        let first_jumping_step = steps.pop();
        assert_eq!(first_jumping_step.tick_id, first_tick_id - 2);
        assert_eq!(steps.front_tick_id().unwrap().value(), 10);
        steps.discard_up_to(first_tick_id);
        assert_eq!(steps.is_empty(), false);
        steps.discard_up_to(first_tick_id + 1);
        assert_eq!(steps.is_empty(), true);
    }
}