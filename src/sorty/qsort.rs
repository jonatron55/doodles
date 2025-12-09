// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::cmp::Ordering;

pub struct QsortState {
    stack: Vec<Substate>,
}

struct Substate {
    low: usize,
    high: usize,
    pivot: usize,
    i: usize,
    j: usize,
}

pub fn step_qsort(actual: &mut [usize], ordering: Ordering, state: &mut QsortState) -> bool {
    let Some(substate) = state.stack.pop() else {
        return true;
    };

    let Substate {
        low,
        high,
        pivot,
        i,
        j,
    } = substate;

    if j > high {
        actual.swap(i, high);
        {
            let high = i.saturating_sub(1);
            if low < high {
                state.stack.push(Substate {
                    low,
                    high,
                    pivot: actual[high],
                    i: low,
                    j: low,
                });
            }
        }
        {
            let low = i + 1;
            if low < high {
                state.stack.push(Substate {
                    low,
                    high,
                    pivot: actual[high],
                    i: low,
                    j: low,
                });
            }
        }

        return false;
    }

    if actual[j].cmp(&pivot) == ordering {
        state.stack.push(Substate {
            low,
            high,
            pivot,
            i: i + 1,
            j: j + 1,
        });
        actual.swap(i, j);
    } else {
        state.stack.push(Substate {
            low,
            high,
            pivot,
            i,
            j: j + 1,
        });
    }

    false
}

impl QsortState {
    pub fn new(actual: &[usize]) -> Self {
        let mut stack = Vec::new();
        stack.push(Substate {
            low: 0,
            high: actual.len() - 1,
            pivot: actual[actual.len() - 1],
            i: 0,
            j: 0,
        });
        QsortState { stack }
    }
}
