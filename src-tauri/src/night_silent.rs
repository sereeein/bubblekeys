//! Pure time-window check + a background thread that ticks every minute
//! to update MuteController.night_silent_active per Settings.

use crate::mute_controller::MuteController;
use crate::settings_store::Settings;
use chrono::Timelike;
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[derive(Clone, Copy)]
pub struct Window {
    pub start: (u8, u8),
    pub end: (u8, u8),
}

pub fn parse_hhmm(s: &str) -> Option<(u8, u8)> {
    let (h, m) = s.split_once(':')?;
    Some((h.parse().ok()?, m.parse().ok()?))
}

pub fn in_window(window: Window, now_hm: (u8, u8)) -> bool {
    let to_min = |(h, m): (u8, u8)| h as u16 * 60 + m as u16;
    let s = to_min(window.start);
    let e = to_min(window.end);
    let n = to_min(now_hm);
    if s == e {
        false
    } else if s < e {
        n >= s && n < e
    } else {
        n >= s || n < e
    }
}

pub fn spawn(settings: Arc<RwLock<Settings>>, mute: MuteController) {
    std::thread::Builder::new()
        .name("night-silent".into())
        .spawn(move || loop {
            tick(&settings, &mute);
            std::thread::sleep(Duration::from_secs(60));
        })
        .expect("spawn night-silent thread");
}

fn tick(settings: &Arc<RwLock<Settings>>, mute: &MuteController) {
    let s = settings.read().unwrap();
    if !s.night_silent.enabled {
        mute.set_night_silent_active(false);
        return;
    }
    let now = chrono::Local::now();
    let now_hm = (now.hour() as u8, now.minute() as u8);
    let win = Window {
        start: parse_hhmm(&s.night_silent.start).unwrap_or((22, 0)),
        end: parse_hhmm(&s.night_silent.end).unwrap_or((7, 0)),
    };
    mute.set_night_silent_active(in_window(win, now_hm));
}

#[cfg(test)]
mod tests {
    use super::*;
    fn w(s: &str, e: &str) -> Window {
        Window {
            start: parse_hhmm(s).unwrap(),
            end: parse_hhmm(e).unwrap(),
        }
    }

    #[test]
    fn same_day_window() {
        assert!(in_window(w("09:00", "17:00"), (12, 0)));
        assert!(!in_window(w("09:00", "17:00"), (8, 0)));
        assert!(!in_window(w("09:00", "17:00"), (17, 0)));
    }

    #[test]
    fn midnight_wrap() {
        let win = w("22:00", "07:00");
        assert!(in_window(win, (23, 0)));
        assert!(in_window(win, (3, 0)));
        assert!(in_window(win, (6, 59)));
        assert!(!in_window(win, (7, 0)));
        assert!(!in_window(win, (12, 0)));
    }

    #[test]
    fn equal_start_end_is_off() {
        let win = w("12:00", "12:00");
        assert!(!in_window(win, (12, 0)));
    }
}
