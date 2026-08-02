mod wm;

fn main() {
    println!("[nwwm] hello!");

    let wm = wm::WindowManager::new();
    wm.run();
}
