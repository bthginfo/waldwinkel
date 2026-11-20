#[cfg(not(target_os = "android"))]
fn main() {
    waldwinkel::main();
}

#[cfg(target_os = "android")]
fn main() {}
