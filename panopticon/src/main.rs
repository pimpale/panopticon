use chrono::{DateTime, Local};
use clap::{error::ErrorKind, CommandFactory, Parser};
use rand::Rng;
use screenshots::Screen;
use std::{fs, thread, time};
use user_idle::UserIdle;

#[derive(Parser, Clone)]
#[clap(name = "panopticon")]
#[clap(author = "Govind Pimpale <gpimpale29@gmail.com>")]
#[clap(version = "0.1")]
#[clap(about = "Takes periodic screenshots", long_about = None)]
struct Opts {
    /// Target directory to store screenshots in
    dir: String,
    /// Interval in seconds between screenshots
    #[clap(long, short, default_value = "60")]
    interval: f32,
    /// Seconds of jitter to add to the screenshot time. Must be less than or equal to interval.
    #[clap(long, short, default_value = "0")]
    jitter: f32,
    /// Don't check whether the user is afk or not
    #[clap(long, short)]
    no_afk: bool,
    /// Duration in seconds of no mouse or keyboard activity after which the user will be considered AFK
    #[clap(long, short, default_value = "60")]
    afk_threshold: u64,
}

fn screenshot_all(base_dir: String, time: DateTime<Local>, afk: bool) {
    let screens = Screen::all().unwrap();
    for screen in screens {
        let image = screen.capture().unwrap();
        let dir = format!("{}/{}", base_dir, time.format("%Y-%m-%d"));
        fs::create_dir_all(&dir).unwrap();
        image.save(
            format!(
                "{}/{}{}{}.png",
                dir,
                time.format("%H:%M:%S"),
                format!("_screen-{}", screen.display_info.id),
                if afk { "_AFK" } else { "" }
            ),
        )
        .unwrap();
    }
}

fn main() {
    let Opts {
        dir,
        interval,
        jitter,
        no_afk,
        afk_threshold,
    } = Opts::parse();

    if interval <= 0.0 {
        let mut cmd = Opts::command();
        cmd.error(ErrorKind::InvalidValue, "interval must be greater than 0")
            .exit();
    }

    if jitter > interval {
        let mut cmd = Opts::command();
        cmd.error(
            ErrorKind::InvalidValue,
            "jitter must be less than or equal to interval",
        )
        .exit();
    }

    let mut rng = rand::thread_rng();

    loop {
        let delay = jitter * rng.gen::<f32>();
        thread::sleep(time::Duration::from_secs_f32(delay));

        let afk = if no_afk {
            false
        } else {
            UserIdle::get_time().unwrap().as_seconds() > afk_threshold
        };
        screenshot_all(dir.clone(), Local::now(), afk);

        thread::sleep(time::Duration::from_secs_f32(interval - delay));
    }
}
