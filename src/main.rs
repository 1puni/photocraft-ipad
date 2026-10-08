#[cfg(target_arch = "wasm32")]
fn main() {
    photocraft_ipad::start();
}
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("Build the browser app with trunk build --release");
}
