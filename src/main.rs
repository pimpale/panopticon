use chrono::{DateTime, Local};
use clap::Parser;
use device_query::{device_state, DeviceEvents, DeviceState};
use screenshots::Screen;
use std::{
    fs,
    sync::{Arc, Mutex},
    thread,
    time::{self, Instant},
};

#[derive(Parser, Clone)]
struct Opts {
    #[clap(long)]
    dir: String,
    #[clap(long)]
    interval: u64,
    #[clap(long)]
    no_afk: bool,
}

fn screenshot_all(dir: String, time: DateTime<Local>) {
    let screens = Screen::all().unwrap();
    for screen in screens {
        let image = screen.capture().unwrap();
        let buffer = image.buffer();
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            format!(
                "{}/{}_screen{}.png",
                dir,
                time.format("%Y-%m-%d_%H:%M:%S"),
                screen.display_info.id
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
        let dir = dir.clone();
        let time = Local::now();
        thread::spawn(move || {
            screenshot_all(dir, time);
        });

        println!("{:?}", last_interaction.lock().unwrap());

        thread::sleep(time::Duration::from_secs(interval));
    }
}
