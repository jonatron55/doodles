// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::cmp::Ordering;

pub struct BubbleState {
    direction: bool,
    index: usize,
}

pub fn step_bubble(
    actual: &mut [usize],
    width: usize,
    ordering: Ordering,
    state: &mut BubbleState,
) -> bool {
    let i = if state.direction {
        state.index
    } else {
        width - 2 - state.index
    };

    let a = actual[i];
    let b = actual[i + 1];

    if a.cmp(&b) == ordering {
        actual[i] = b;
        actual[i + 1] = a;
    }

    state.index += 1;

    if state.index >= width - 1 {
        state.index = 0;
        state.direction = !state.direction;

        if actual.windows(2).all(|w| w[0].cmp(&w[1]) != ordering) {
            return true;
        }
    }

    false
}

impl BubbleState {
    pub fn new() -> Self {
        Self {
            direction: true,
            index: 0,
        }
    }
}
