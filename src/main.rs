use chrono::{DateTime, Local};
use clap::Parser;
use screenshots::Screen;
use std::{fs, thread, time};
use user_idle::UserIdle;

#[derive(Parser, Clone)]
#[clap(name = "Retroactive Time Tracker")]
#[clap(author = "Govind Pimpale <gpimpale29@gmail.com>")]
#[clap(version = "0.1")]
#[clap(about = "Takes periodic screenshots", long_about = None)]
struct Opts {
    #[clap(long, short, help = "Directory to store screenshots in")]
    dir: String,
    #[clap(
        long,
        short,
        default_value = "300",
        help = "Interval in seconds between consecutive screenshots"
    )]
    interval: u64,
    #[clap(long, short, help = "Don't check whether the user is afk or not")]
    no_afk: bool,
    #[clap(
        long,
        short,
        default_value = "300",
        help = "Duration in seconds of no mouse or keyboard activity after which the user will be considered AFK"
    )]
    afk_threshold: u64,
}

fn screenshot_all(base_dir: String, time: DateTime<Local>, afk: bool) {
    let screens = Screen::all().unwrap();
    for screen in screens {
        let image = screen.capture().unwrap();
        let buffer = image.buffer();
        let dir = format!("{}/{}", base_dir, time.format("%Y-%m-%d"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            format!(
                "{}/{}{}{}.png",
                dir,
                time.format("%H:%M:%S"),
                format!("_screen-{}", screen.display_info.id),
                if afk { "_AFK" } else { "" }
            ),
            &buffer,
        )
        .unwrap();
    }
}

fn main() {
    let Opts {
        dir,
        interval,
        no_afk,
        afk_threshold,
    } = Opts::parse();

    loop {
        let afk = if no_afk {
            false
        } else {
            UserIdle::get_time().unwrap().as_seconds() > afk_threshold
        };

        let dir = dir.clone();
        let time = Local::now();
        thread::spawn(move || {
            screenshot_all(dir, time, afk);
        });

        thread::sleep(time::Duration::from_secs(interval));
    }
}
