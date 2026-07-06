use std::env;

fn main() {
    let image = env::args().nth(1).filter(|a| !a.starts_with('-'));
    if let Err(e) = rspfdisk_tui::run(image.as_deref()) {
        eprintln!("rspfdisk-tui error: {e}");
        std::process::exit(1);
    }
}
