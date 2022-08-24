use chrono::{DateTime, Local};
use clap::Parser;
use device_query::{DeviceEvents, DeviceState};
use screenshots::Screen;
use std::{
    fs,
    sync::{Arc, Mutex},
    thread,
    time::{self, Instant},
};

#[derive(Parser, Clone)]
#[clap(name = "Retroactive Time Tracker")]
#[clap(author = "Govind Pimpale <gpimpale29@gmail.com>")]
#[clap(version = "0.1")]
#[clap(about = "Takes periodic screenshots", long_about = None)]
struct Opts {
    #[clap(long, short, help="Directory to store screenshots in")]
    dir: String,
    #[clap(long, short, default_value = "300", help="Interval in seconds between consecutive screenshots")]
    interval: u64,
    #[clap(long, short, help="Don't check whether the user is afk or not.")]
    no_afk: bool,
    #[clap(long, short, default_value = "300", help="Duration in seconds of no mouse or keyboard activity after which the user will be considered AFK")]
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
                if afk { "_[AFK]" } else { "" }
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

    // last interaction with computer
    let last_interaction = Arc::new(Mutex::new(Instant::now()));

    // if not no afk, keep tabs on when the last interaction was
    if !no_afk {
        let last_interaction_clone = last_interaction.clone();
        thread::spawn(move || {
            let device_state = DeviceState::new();

            // every time we recieve an event, set the last interaction to now
            let mm_li_clone = last_interaction_clone.clone();
            let _mm_grd = device_state.on_mouse_move(move |_| {
                let mut m = mm_li_clone.lock().expect("could not lock mutex");
                *m = Instant::now();
            });
            let mu_li_clone = last_interaction_clone.clone();
            let _mu_grd = device_state.on_mouse_up(move |_| {
                let mut m = mu_li_clone.lock().expect("could not lock mutex");
                *m = Instant::now();
            });
            let md_li_clone = last_interaction_clone.clone();
            let _md_grd = device_state.on_mouse_down(move |_| {
                let mut m = md_li_clone.lock().expect("could not lock mutex");
                *m = Instant::now();
            });
            let kd_li_clone = last_interaction_clone.clone();
            let _kd_guard = device_state.on_key_down(move |_| {
                let mut m = kd_li_clone.lock().expect("could not lock mutex");
                *m = Instant::now();
            });
            let ku_li_clone = last_interaction_clone.clone();
            let _ku_guard = device_state.on_key_up(move |_| {
                let mut m = ku_li_clone.lock().expect("could not lock mutex");
                *m = Instant::now();
            });

            // need to sleep forever to make sure guards dont get dropped
            thread::sleep(time::Duration::MAX);
        });
    }

    loop {
        let afk = if no_afk {
            false
        } else {
            last_interaction.lock().unwrap().elapsed().as_secs() > afk_threshold
        };

        let dir = dir.clone();
        let time = Local::now();
        thread::spawn(move || {
            screenshot_all(dir, time, afk);
        });

        thread::sleep(time::Duration::from_secs(interval));
    }
}
