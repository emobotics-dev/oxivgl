// SPDX-License-Identifier: MIT OR Apache-2.0
// -Tlinkall.x emitted by oxivgl/build.rs for xtensa targets.
//
// The m5stack-core identity mark is NOT emitted here: `app_desc!()` expands in
// the crate that owns the example (oxivgl), and `cargo:rustc-env` reaches only
// the crate whose script emitted it. See oxivgl's own build.rs.
fn main() {}
