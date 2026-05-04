// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::cmp::Ordering;

/// Persisted state for quicksort algorithm.
#[derive(Clone, Debug)]
pub struct QsortState {
    /// Stack of substates for each recursive step.
    stack: Vec<Substate>,
}

/// Persisted substate for a single quicksort partitioning step.
#[derive(Clone, Debug)]
struct Substate {
    /// Lower bound of the current subslice.
    low: usize,

    /// Upper bound of the current subslice.
    high: usize,

    /// Pivot value for the current partitioning step.
    pivot: usize,

    /// Current index for placing elements less than the pivot.
    i: usize,

    /// Current index for scanning through the subslice.
    j: usize,
}

/// Perform a single step of the quicksort algorithm.
///
/// Arguments
/// ---------
///
/// - `values`: The slice of values to sort. This will be modified in place.
/// - `ordering`: The desired ordering (Less for ascending, Greater for descending).
/// - `state`: The persisted state of the quicksort algorithm. This will be modified in place.
///
/// Returns
/// -------
///
/// If sorting is complete, the function returns `true`. If the function needs more steps, it will return `false` and
/// should be called again with the updated state.
pub fn step_qsort(values: &mut [usize], ordering: Ordering, state: &mut QsortState) -> bool {
    let Some(substate) = state.stack.pop() else {
        // Recursion complete
        return true;
    };

    let Substate { low, high, pivot, i, j } = substate;

    if j > high {
        // Partitioning step complete; place pivot in correct position and recurse
        values.swap(i, high);

        let newhigh = i.saturating_sub(1);
        if low < newhigh {
            state.stack.push(Substate {
                low,
                high: newhigh,
                pivot: values[newhigh],
                i: low,
                j: low,
            });
        }

        let newlow = i + 1;
        if newlow < high {
            state.stack.push(Substate {
                low: newlow,
                high,
                pivot: values[high],
                i: newlow,
                j: newlow,
            });
        }
    } else {
        // Continue partitioning
        if values[j].cmp(&pivot) == ordering {
            state.stack.push(Substate {
                low,
                high,
                pivot,
                i: i + 1,
                j: j + 1,
            });
            values.swap(i, j);
        } else {
            state.stack.push(Substate {
                low,
                high,
                pivot,
                i,
                j: j + 1,
            });
        }
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
