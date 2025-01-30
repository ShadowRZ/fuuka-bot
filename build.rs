fn main() {
    println!("cargo::rustc-check-cfg=cfg(ruma_unstable_exhaustive_types)");
}
