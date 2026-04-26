//! Single source of truth for "should we play sound right now?".

use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone)]
pub struct MuteController {
    inner: Arc<RwLock<MuteState>>,
}

#[derive(Default, Clone, Copy)]
struct MuteState {
    user_muted: bool,
    night_silent_active: bool,
}

impl MuteController {
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(MuteState::default())) }
    }

    pub fn is_muted(&self) -> bool {
        let s = self.inner.read();
        s.user_muted || s.night_silent_active
    }

    pub fn set_user_muted(&self, muted: bool) {
        self.inner.write().user_muted = muted;
    }

    pub fn set_night_silent_active(&self, active: bool) {
        self.inner.write().night_silent_active = active;
    }
}

impl Default for MuteController { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_unmuted() {
        let m = MuteController::new();
        assert!(!m.is_muted());
    }

    #[test]
    fn user_mute_takes_effect() {
        let m = MuteController::new();
        m.set_user_muted(true);
        assert!(m.is_muted());
        m.set_user_muted(false);
        assert!(!m.is_muted());
    }

    #[test]
    fn night_silent_overrides_unmuted() {
        let m = MuteController::new();
        m.set_night_silent_active(true);
        assert!(m.is_muted());
    }

    #[test]
    fn either_source_keeps_muted() {
        let m = MuteController::new();
        m.set_user_muted(true);
        m.set_night_silent_active(true);
        m.set_user_muted(false);
        assert!(m.is_muted()); // night_silent still active
    }
}
