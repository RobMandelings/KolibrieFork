/*
 * Copyright © 2025 Volodymyr Khadzhaia
 * Copyright © 2025 Pieter Bonte
 * KU Leuven — Stream Intelligence Lab, Belgium
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt::Debug;
use std::hash::Hash;
pub mod s2r_tests;
pub mod reporting;
pub mod sparql_window;
pub mod window;

mod test_logging;

/// Tick is a dimension that explains what triggers the report evaluations.
/// Possible ticks are time-driven, tuple-driven, or batch-driven.
#[derive(Clone, Debug)]
pub enum Tick {
    TimeDriven,
    TupleDriven,
    BatchDriven,
}

impl Default for Tick {
    fn default() -> Self {
        Tick::TimeDriven
    }
}

