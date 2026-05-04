// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::cmp::Ordering;

/// Persisted state for bubble sort algorithm.
#[derive(Clone, Copy, Debug)]
pub struct BubbleState {
    /// Current direction of the pass.
    direction: Direction,

    /// Current index within the pass.
    index: usize,
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    LeftToRight,
    RightToLeft,
}

/// Perform a single step of the bubble sort algorithm.
///
/// Arguments
/// ---------
///
/// - `values`: The slice of values to sort. This will be modified in place.
/// - `width`: The number of elements in `values`.
/// - `ordering`: The desired ordering (Less for ascending, Greater for descending).
/// - `state`: The persisted state of the bubble sort algorithm. This will be modified in place.
///
/// Returns
/// -------
///
/// `true` if the sorting is complete, `false` if more steps are needed.
pub fn step_bubble(values: &mut [usize], width: usize, ordering: Ordering, state: &mut BubbleState) -> bool {
    let i = match state.direction {
        Direction::LeftToRight => state.index,
        Direction::RightToLeft => width - 2 - state.index,
    };

    let a = values[i];
    let b = values[i + 1];

    if a.cmp(&b) == ordering {
        values[i] = b;
        values[i + 1] = a;
    }

    state.index += 1;

    if state.index >= width - 1 {
        // Reach the end of the pass; reset for next pass
        state.index = 0;
        state.direction = state.direction.opposite();

        if values.windows(2).all(|w| w[0].cmp(&w[1]) != ordering) {
            // Sorting complete
            return true;
        }
    }

    false
}

impl BubbleState {
    pub fn new() -> Self {
        Self {
            direction: Direction::LeftToRight,
            index: 0,
        }
    }
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Direction::LeftToRight => Direction::RightToLeft,
            Direction::RightToLeft => Direction::LeftToRight,
        }
    }
}
