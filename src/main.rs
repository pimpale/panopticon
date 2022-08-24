use chrono::Local;
use clap::Parser;
use screenshots::Screen;
use std::{fs, thread, time};

#[derive(Parser, Clone)]
struct Opts {
    #[clap(long)]
    dir: String,
    #[clap(long)]
    interval: u64,
}

fn screenshot_all(dir: String) {
    let screens = Screen::all().unwrap();
    for screen in screens {
        let image = screen.capture().unwrap();
        let buffer = image.buffer();
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            format!("{}/{}_{}.png", dir, Local::now(), screen.display_info.id),
            &buffer,
        )
        .unwrap();
    }
}

fn main() {
    let Opts { dir, interval } = Opts::parse();

    loop {
        let dir_clone = dir.clone();
        thread::spawn(|| {
            screenshot_all(dir_clone);
        });

        thread::sleep(time::Duration::from_secs(interval));
    }
}
