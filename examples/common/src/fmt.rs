// SPDX-License-Identifier: MIT OR Apache-2.0
//! Logging macros for examples — dispatch to `log` crate on all targets.
#![macro_use]
#![allow(unused_macros)]

#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! trace {
    ($s:literal $(, $x:expr)* $(,)?) => {
        $crate::log::trace!($s $(, $x)*)
    };
}

#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! debug {
    ($s:literal $(, $x:expr)* $(,)?) => {
        $crate::log::debug!($s $(, $x)*)
    };
}

#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! info {
    ($s:literal $(, $x:expr)* $(,)?) => {
        $crate::log::info!($s $(, $x)*)
    };
}

#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! warn {
    ($s:literal $(, $x:expr)* $(,)?) => {
        $crate::log::warn!($s $(, $x)*)
    };
}

#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! error {
    ($s:literal $(, $x:expr)* $(,)?) => {
        $crate::log::error!($s $(, $x)*)
    };
}
