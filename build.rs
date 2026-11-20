fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        // Controller events need their source and axes before Winit converts the native queue
        // into keyboard and touch events. Ordinary input follows Winit's existing path.
        println!("cargo:rustc-link-arg=-Wl,--wrap=AInputQueue_getEvent");
    }
}
