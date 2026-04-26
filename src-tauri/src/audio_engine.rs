//! cpal/rodio-based audio engine. Plays decoded samples on demand.

use std::io::Cursor;
use std::sync::Arc;
use std::thread;

use crossbeam_channel::{unbounded, Sender};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

#[derive(Clone, Debug)]
pub struct PlayCommand {
    pub sample: Arc<Vec<u8>>,
    pub volume: f32,
    pub pitch_offset: f32,
}

pub trait AudioEngine: Send + Sync {
    fn play(&self, cmd: PlayCommand);
}

pub struct RodioEngine {
    tx: Sender<PlayCommand>,
}

impl RodioEngine {
    /// Spawns a dedicated audio thread that owns the OutputStream.
    /// rodio's stream isn't Send, so it stays parked on this thread.
    pub fn new() -> Result<Self, String> {
        let (tx, rx) = unbounded::<PlayCommand>();
        thread::Builder::new()
            .name("bubblekeys-audio".into())
            .spawn(move || {
                let (_stream, handle) = match OutputStream::try_default() {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("audio: failed to open default device: {e}");
                        return;
                    }
                };
                let handle = Arc::new(handle);

                while let Ok(cmd) = rx.recv() {
                    spawn_oneshot(handle.clone(), cmd);
                }
            })
            .map_err(|e| format!("spawn audio thread: {e}"))?;

        Ok(Self { tx })
    }
}

fn spawn_oneshot(handle: Arc<OutputStreamHandle>, cmd: PlayCommand) {
    let cursor = Cursor::new((*cmd.sample).clone());
    let decoder = match Decoder::new(cursor) {
        Ok(d) => d,
        Err(e) => {
            log::warn!("audio: decode failed: {e}");
            return;
        }
    };
    // Pitch offset via speed change. ±0.5 semitones ≈ ±3% speed.
    let speed = 2f32.powf(cmd.pitch_offset / 12.0);
    let source = decoder.amplify(cmd.volume).speed(speed);
    let sink = match Sink::try_new(&handle) {
        Ok(s) => s,
        Err(e) => { log::warn!("audio: sink: {e}"); return; }
    };
    sink.append(source);
    sink.detach(); // play asynchronously, dropped when finished
}

impl AudioEngine for RodioEngine {
    fn play(&self, cmd: PlayCommand) {
        if let Err(e) = self.tx.send(cmd) {
            log::warn!("audio: queue full or closed: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_command_constructs_with_zero_pitch() {
        let cmd = PlayCommand {
            sample: Arc::new(vec![0u8; 100]),
            volume: 0.5,
            pitch_offset: 0.0,
        };
        assert_eq!(cmd.volume, 0.5);
    }

    // Real-output tests are skipped in CI (no audio device).
}
