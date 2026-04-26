//! Global keyboard listener via CGEventTap. Requires Accessibility permission.

use std::thread;

use core_foundation::base::TCFType;
use core_foundation::runloop::{
    kCFRunLoopCommonModes, CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRun,
};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType,
};
use crossbeam_channel::{unbounded, Receiver, Sender};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEventKind { Down, Up }

#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
    pub keycode: u16,
    pub kind: KeyEventKind,
}

pub trait KeyListener: Send + Sync {
    fn events(&self) -> Receiver<KeyEvent>;
}

pub struct MacKeyListener {
    rx: Receiver<KeyEvent>,
}

impl MacKeyListener {
    /// Spawns a thread running the event tap on its own CFRunLoop.
    /// Returns Err only if Accessibility permission is missing AND the OS refuses to create the tap.
    pub fn start() -> Result<Self, String> {
        let (tx, rx) = unbounded::<KeyEvent>();

        thread::Builder::new()
            .name("bubblekeys-keys".into())
            .spawn(move || run_tap(tx))
            .map_err(|e| format!("spawn key thread: {e}"))?;

        Ok(Self { rx })
    }
}

fn run_tap(tx: Sender<KeyEvent>) {
    let events = vec![CGEventType::KeyDown, CGEventType::KeyUp];
    let tap = match CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::ListenOnly,
        events,
        move |_proxy, etype, event| {
            let keycode = event.get_integer_value_field(
                core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE,
            ) as u16;
            let kind = match etype {
                CGEventType::KeyDown => KeyEventKind::Down,
                CGEventType::KeyUp   => KeyEventKind::Up,
                _ => return None,
            };
            let _ = tx.send(KeyEvent { keycode, kind });
            None
        },
    ) {
        Ok(t) => t,
        Err(()) => {
            log::error!("key listener: failed to create event tap (Accessibility permission missing?)");
            return;
        }
    };

    unsafe {
        let loop_source = tap.mach_port.create_runloop_source(0).expect("loop source");
        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_loop, loop_source.as_concrete_TypeRef(), kCFRunLoopCommonModes);
        tap.enable();
        CFRunLoopRun();
    }
}

impl KeyListener for MacKeyListener {
    fn events(&self) -> Receiver<KeyEvent> { self.rx.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_event_constructs() {
        let e = KeyEvent { keycode: 0, kind: KeyEventKind::Down };
        assert_eq!(e.kind, KeyEventKind::Down);
    }
}
