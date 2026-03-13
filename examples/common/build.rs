// SPDX-License-Identifier: MIT OR Apache-2.0
fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("xtensa") {
        println!("cargo:rustc-link-arg=-Tlinkall.x");
    }
}
