//! Global keyboard listener via CGEventTap. Requires Accessibility permission.

use std::thread;

use core_foundation::base::{CFTypeRef, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::runloop::{
    kCFRunLoopCommonModes, CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRun,
};
use core_foundation::string::CFString;
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType,
};
use crossbeam_channel::{unbounded, Receiver, Sender};

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    static kAXTrustedCheckOptionPrompt: CFTypeRef;
    fn AXIsProcessTrustedWithOptions(options: CFTypeRef) -> bool;
}

/// Returns true if this process has Accessibility permission. If `prompt` is true
/// and permission is not granted, macOS shows the system permission prompt and
/// adds the app to the Accessibility list — required for CGEventTap to succeed.
pub fn ensure_accessibility(prompt: bool) -> bool {
    unsafe {
        let key = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt as _);
        let val = CFBoolean::from(prompt);
        let dict = CFDictionary::from_CFType_pairs(&[(key, val)]);
        AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef() as CFTypeRef)
    }
}

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
    /// Returns Err if CGEventTap creation fails (typically when Accessibility
    /// permission has not been granted to this binary). Caller should retry.
    pub fn start() -> Result<Self, String> {
        let (tx, rx) = unbounded::<KeyEvent>();
        // Sync channel of capacity 0 means the sender blocks until receiver
        // accepts — a one-shot handshake that guarantees we know the
        // CGEventTap creation result before start() returns.
        let (init_tx, init_rx) = std::sync::mpsc::sync_channel::<Result<(), String>>(0);

        thread::Builder::new()
            .name("bubblekeys-keys".into())
            .spawn(move || run_tap(tx, init_tx))
            .map_err(|e| format!("spawn key thread: {e}"))?;

        match init_rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(Ok(())) => Ok(Self { rx }),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(format!("listener init handshake timeout: {e}")),
        }
    }
}

fn run_tap(
    tx: Sender<KeyEvent>,
    init_tx: std::sync::mpsc::SyncSender<Result<(), String>>,
) {
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
            let _ = init_tx.send(Err(
                "CGEventTap creation failed (Accessibility permission missing or revoked)".into(),
            ));
            return;
        }
    };

    let _ = init_tx.send(Ok(()));

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
