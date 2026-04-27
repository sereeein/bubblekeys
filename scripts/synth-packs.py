#!/usr/bin/env python3
"""
Synthesize bundled sound packs for v0.2.2.

Outputs (under out/synth-packs/):
  cherry-black.wav       cherry-black.ogg
  cherry-silver.wav      cherry-silver.ogg
  cherry-red-silent.wav  cherry-red-silent.ogg
  cherry-purple.wav      cherry-purple.ogg
  cherry-white.wav       cherry-white.ogg
  PREVIEW.md             (links for VSCode click-to-play)

Acoustic model: each "key press" is a short (~120 ms) impulse made of
  - filtered noise transient(s)  -> the contact / impact "snap"
  - damped sine resonances       -> housing body color
  - amplitude envelope           -> attack / decay shape

Each switch derives from this primitive, with parameters tuned to its
character (linear vs tactile vs clicky, light vs heavy, silent etc.)
"""

from __future__ import annotations

import shutil
import struct
import subprocess
import sys
import wave
from pathlib import Path

import numpy as np

SR = 44100  # sample rate
OUT_DIR = Path(__file__).resolve().parent.parent / "out" / "synth-packs"

rng = np.random.default_rng(seed=2026_04_28)


# ---------------------------------------------------------------------------
# Primitives
# ---------------------------------------------------------------------------

def env_ad(n: int, attack_ms: float, decay_ms: float, peak: float = 1.0) -> np.ndarray:
    """Attack-decay envelope: linear ramp up, exponential decay."""
    out = np.zeros(n, dtype=np.float64)
    a = max(1, int(SR * attack_ms / 1000))
    d = max(1, int(SR * decay_ms / 1000))
    a = min(a, n)
    out[:a] = np.linspace(0.0, peak, a)
    end = min(n, a + d)
    if end > a:
        # exponential -ln(0.001) ~ 6.9 reaches ~ -60 dB at decay_ms
        out[a:end] = peak * np.exp(-np.linspace(0.0, 6.9, end - a))
    return out


def bandpass_noise(n: int, low_hz: float, high_hz: float) -> np.ndarray:
    """Pink-ish band-limited noise via FFT brick-wall."""
    noise = rng.standard_normal(n)
    spec = np.fft.rfft(noise)
    freqs = np.fft.rfftfreq(n, 1.0 / SR)
    mask = (freqs >= low_hz) & (freqs <= high_hz)
    spec *= mask
    return np.fft.irfft(spec, n)


def lowpass(x: np.ndarray, cutoff_hz: float) -> np.ndarray:
    spec = np.fft.rfft(x)
    freqs = np.fft.rfftfreq(len(x), 1.0 / SR)
    spec[freqs > cutoff_hz] = 0
    return np.fft.irfft(spec, len(x))


def highpass(x: np.ndarray, cutoff_hz: float) -> np.ndarray:
    spec = np.fft.rfft(x)
    freqs = np.fft.rfftfreq(len(x), 1.0 / SR)
    spec[freqs < cutoff_hz] = 0
    return np.fft.irfft(spec, len(x))


def damped_sine(n: int, freq_hz: float, decay_ms: float, amp: float = 1.0) -> np.ndarray:
    t = np.arange(n) / SR
    decay = np.exp(-t * (6.9 * 1000.0 / decay_ms))
    return amp * decay * np.sin(2 * np.pi * freq_hz * t)


def offset(x: np.ndarray, delay_ms: float, total_n: int) -> np.ndarray:
    """Place x starting at delay_ms inside a zero buffer of length total_n."""
    out = np.zeros(total_n, dtype=np.float64)
    start = int(SR * delay_ms / 1000)
    end = min(total_n, start + len(x))
    out[start:end] = x[: end - start]
    return out


def normalize_peak(x: np.ndarray, target_peak: float) -> np.ndarray:
    peak = float(np.max(np.abs(x)))
    if peak < 1e-9:
        return x
    return x * (target_peak / peak)


# ---------------------------------------------------------------------------
# Switch synth functions
# ---------------------------------------------------------------------------

def synth_black() -> np.ndarray:
    """Linear, heavy spring. Single bottom-out, low body, slightly muffled."""
    n = int(SR * 0.13)  # ~130 ms total
    # Slightly slower attack (heavier spring -> later peak)
    env = env_ad(n, attack_ms=4, decay_ms=70)
    transient = bandpass_noise(n, 200, 3500) * 0.85
    body = (
        damped_sine(n, freq_hz=110, decay_ms=55, amp=0.35)
        + damped_sine(n, freq_hz=180, decay_ms=40, amp=0.20)
        + damped_sine(n, freq_hz=320, decay_ms=25, amp=0.10)
    )
    sig = (transient + body) * env
    sig = lowpass(sig, 5000)  # darker, "thocky"
    return normalize_peak(sig, 0.55)


def synth_silver() -> np.ndarray:
    """Linear, speed switch. Short travel, fast and snappy."""
    n = int(SR * 0.09)  # ~90 ms — shorter
    env = env_ad(n, attack_ms=1.5, decay_ms=35)
    transient = bandpass_noise(n, 800, 8000) * 1.0
    body = (
        damped_sine(n, freq_hz=2200, decay_ms=18, amp=0.30)
        + damped_sine(n, freq_hz=3500, decay_ms=12, amp=0.20)
    )
    sig = (transient + body) * env
    sig = highpass(sig, 400)  # crisp, no low rumble
    return normalize_peak(sig, 0.70)


def synth_red_silent() -> np.ndarray:
    """Red linear with silicone dampener: heavily low-passed, quieter."""
    n = int(SR * 0.10)
    env = env_ad(n, attack_ms=3, decay_ms=55)
    transient = bandpass_noise(n, 150, 2500) * 0.6
    body = (
        damped_sine(n, freq_hz=160, decay_ms=40, amp=0.20)
        + damped_sine(n, freq_hz=380, decay_ms=22, amp=0.10)
    )
    sig = (transient + body) * env
    sig = lowpass(sig, 1500)  # soft / dull (silicone ring)
    # Apply secondary -6 dB attenuation on top of normalization
    return normalize_peak(sig, 0.28)


def synth_purple() -> np.ndarray:
    """Tactile, more pronounced than brown. Two distinct events: bump + bottom-out."""
    total = int(SR * 0.16)  # ~160 ms — long enough for both events
    # Tactile bump @ ~0 ms
    n_bump = int(SR * 0.05)
    env_bump = env_ad(n_bump, attack_ms=2, decay_ms=25)
    bump_noise = bandpass_noise(n_bump, 400, 3000) * 0.6
    bump_body = damped_sine(n_bump, freq_hz=600, decay_ms=18, amp=0.30)
    bump = (bump_noise + bump_body) * env_bump
    bump = offset(bump, 0, total)

    # Bottom-out @ ~30 ms
    n_btm = int(SR * 0.10)
    env_btm = env_ad(n_btm, attack_ms=2, decay_ms=55)
    btm_noise = bandpass_noise(n_btm, 200, 4500) * 0.8
    btm_body = (
        damped_sine(n_btm, freq_hz=180, decay_ms=45, amp=0.30)
        + damped_sine(n_btm, freq_hz=420, decay_ms=30, amp=0.20)
        + damped_sine(n_btm, freq_hz=900, decay_ms=18, amp=0.18)  # 2nd formant boost
    )
    btm = (btm_noise + btm_body) * env_btm
    btm = offset(btm, 30, total)

    sig = bump + btm
    sig = lowpass(sig, 6000)
    return normalize_peak(sig, 0.62)


def synth_white() -> np.ndarray:
    """Clicky, lighter than blue. Sharp click + bottom-out, contained tail."""
    total = int(SR * 0.13)  # ~130 ms
    # Sharp click @ ~0 ms
    n_clk = int(SR * 0.025)
    env_clk = env_ad(n_clk, attack_ms=0.5, decay_ms=12)
    clk_noise = bandpass_noise(n_clk, 1500, 6500) * 0.9
    clk = clk_noise * env_clk
    clk = offset(clk, 0, total)

    # Bottom-out @ ~18 ms
    n_btm = int(SR * 0.09)
    env_btm = env_ad(n_btm, attack_ms=1.5, decay_ms=45)
    btm_noise = bandpass_noise(n_btm, 300, 5000) * 0.7
    btm_body = (
        damped_sine(n_btm, freq_hz=220, decay_ms=35, amp=0.25)
        + damped_sine(n_btm, freq_hz=520, decay_ms=22, amp=0.18)
    )
    btm = (btm_noise + btm_body) * env_btm
    btm = offset(btm, 18, total)

    sig = clk + btm
    sig = lowpass(sig, 5500)  # tighter HF tail than blue
    # ~-2 dB vs a hypothetical full-blue
    return normalize_peak(sig, 0.62)


SWITCHES: dict[str, callable] = {
    "cherry-black": synth_black,
    "cherry-silver": synth_silver,
    "cherry-red-silent": synth_red_silent,
    "cherry-purple": synth_purple,
    "cherry-white": synth_white,
}


# ---------------------------------------------------------------------------
# I/O
# ---------------------------------------------------------------------------

def write_wav(path: Path, samples: np.ndarray) -> None:
    clipped = np.clip(samples, -1.0, 1.0)
    pcm16 = (clipped * 32767.0).astype(np.int16)
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes(pcm16.tobytes())


def encode_ogg(wav_path: Path, ogg_path: Path) -> None:
    subprocess.run(
        ["oggenc", "-Q", "-q", "4", "-o", str(ogg_path), str(wav_path)],
        check=True,
    )


def write_preview_md(out_dir: Path, ids: list[str]) -> None:
    lines = ["# v0.2.2 sound pack preview", "", "Click each WAV to play in VSCode:", ""]
    for sid in ids:
        lines.append(f"- [{sid}](./{sid}.wav)")
    lines.append("")
    (out_dir / "PREVIEW.md").write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    if shutil.which("oggenc") is None:
        print("error: `oggenc` not found. Install via `brew install vorbis-tools`.", file=sys.stderr)
        return 2
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    for sid, fn in SWITCHES.items():
        samples = fn()
        wav = OUT_DIR / f"{sid}.wav"
        ogg = OUT_DIR / f"{sid}.ogg"
        write_wav(wav, samples)
        encode_ogg(wav, ogg)
        wav_size = wav.stat().st_size
        ogg_size = ogg.stat().st_size
        print(f"{sid:24s}  wav={wav_size:>6} B   ogg={ogg_size:>5} B")
    write_preview_md(OUT_DIR, list(SWITCHES.keys()))
    print(f"\nWrote {len(SWITCHES)} packs to {OUT_DIR}")
    print(f"Preview list: {OUT_DIR / 'PREVIEW.md'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
