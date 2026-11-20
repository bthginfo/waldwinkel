"""Render original music and edited CC0 field recordings.

Run with NumPy and FFmpeg: python art/render_score.py /path/to/sample-cache
Instruments come from VSCO 2 Community Edition, with provenance in docs/audio.md.
"""
from pathlib import Path
from random import Random
from urllib.parse import quote
from urllib.request import Request, urlopen
import re
import subprocess
import sys

import numpy as np

RATE = 24000
OUT = Path(__file__).resolve().parents[1] / "assets" / "audio"
VSCO = "https://raw.githubusercontent.com/sgossner/VSCO-2-CE/440300901dfe9275fd84e0b7763af1f8443ae62e/"
PIANO = (37, 45, 53, 61, 69, 77)
HARP = {45: "A2", 52: "E3", 62: "D4", 69: "A4", 76: "E5"}
RECORDINGS = {
    "axe": "xkeril/sounds/753925/",
    "tree": "ozymandias37/sounds/867699/",
    "rocks-a": "Tim_Verberne/sounds/576107/",
    "rocks-b": "Tim_Verberne/sounds/576103/",
    "birds": "JW_Audio/sounds/817480/",
    "dirt": "dhallcomposer/sounds/696531/",
    "water": "qubodup/sounds/210424/",
}


def source(url):
    with urlopen(Request(url, headers={"User-Agent": "Waldwinkel asset build"}), timeout=40) as response:
        return response.read()


def download(cache, name, url=None):
    path = cache / name
    if not path.exists():
        if url is None:
            page = source("https://freesound.org/people/" + RECORDINGS[path.stem]).decode()
            url = re.search(r'https://cdn.freesound.org/previews/[^" <]+-hq.mp3', page).group()
        path.write_bytes(source(url))
    return path


def decode(path, channels, high=42, low=9800):
    result = subprocess.check_output([
        "ffmpeg", "-v", "error", "-i", str(path), "-ar", str(RATE), "-ac", str(channels),
        "-af", f"highpass=f={high},lowpass=f={low}", "-f", "f32le", "-",
    ])
    return np.frombuffer(result, dtype="<f4").reshape(-1, channels)


def encode(name, audio, peak=None):
    if peak is not None:
        audio = audio * (peak / np.max(np.abs(audio)))
    args = ["-c:a", "libvorbis", "-q:a", "4"] if name.endswith(".ogg") else ["-c:a", "pcm_s16le"]
    subprocess.run([
        "ffmpeg", "-v", "error", "-y", "-f", "f32le", "-ar", str(RATE),
        "-ac", str(audio.shape[1]), "-i", "-", *args, str(OUT / name),
    ], input=audio.astype("<f4").tobytes(), check=True)


def fade(audio, attack, release):
    audio = audio.copy()
    count = round(attack * RATE)
    if count:
        audio[:count] *= np.linspace(0., 1., count, endpoint=False)[:, None]
    count = round(release * RATE)
    if count:
        audio[-count:] *= np.linspace(1., 0., count)[:, None]
    return audio


def sample_note(instrument, target, duration):
    origin = min(instrument, key=lambda key: abs(target-key))
    audio = instrument[origin]
    positions = np.arange(round(duration * RATE)) * 2**((target-origin)/12)
    sample = np.stack([
        np.interp(positions, np.arange(len(audio)), audio[:, channel], right=0.)
        for channel in range(audio.shape[1])
    ], axis=1)
    return fade(sample, .013, min(.43, duration / 2.))


def note(track, instrument, when, pitch, value, duration):
    start = round(when * RATE)
    length = min(duration, (len(track)-start) / RATE)
    pluck = sample_note(instrument, pitch, length)
    track[start:start+len(pluck)] += pluck * value


def load_instruments(cache):
    instruments = []
    for kind, keys in [
        ("piano", {
            key: f"Keys/Upright Piano/Player_dyn1_rr1_{(key-21)//2:03}.wav" for key in PIANO
        }),
        ("harp", {key: f"Strings/Harp/KSHarp_{note}_mf.wav" for key, note in HARP.items()}),
    ]:
        bank = {}
        for key, location in keys.items():
            raw = decode(download(cache, f"{kind}-{key}.wav", VSCO+quote(location)), 2, 38, 6000)
            onset = np.flatnonzero(np.max(np.abs(raw), axis=1) > 0.004)
            offset = max(int(onset[0]) - RATE//75, 0) if onset.size else 0
            raw = raw[offset:offset+RATE*6]
            bank[key] = raw * (.66 / np.percentile(np.abs(raw[:RATE]), 99.8))
        instruments.append(bank)
    return instruments


def score(piano, harp):
    # A slow D-major motif with an answer in B minor. Bars 21–24 leave room for the landscape.
    length = 96 * RATE
    base, color = [np.zeros((length, 2), np.float32) for _ in range(2)]
    random = Random(124)
    score_bars = [
        (50, 57, (66, 69, 76)), (47, 57, (73, 71)), (43, 54, (69, 66)), (45, 52, (64,)),
        (50, 57, (66, 69, 78)), (42, 57, (76, 73)), (43, 54, (71, 69, 66)), (45, 55, (64,)),
        (47, 54, (66, 73)), (43, 57, (74, 76)), (50, 57, (73, 69)), (45, 52, (66, 64)),
        (47, 54, (66, 71, 73)), (43, 54, (74, 69)), (45, 57, (66, 64)), (50, 57, (62,)),
        (50, 57, (69, 76)), (43, 54, (74, 71)), (45, 52, (69, 64)), (50, 57, (66, 62)),
    ]
    for i, (bass, fifth, melody) in enumerate(score_bars):
        start = i*4.0 + random.uniform(.015, .037)
        note(base, piano, start, bass, .25 + random.random()*.046, 2.9)
        note(base, piano, start + .32, fifth + (12 if i % 6 == 3 else 0), .16, 2.55)
        for j, pitch in enumerate(melody):
            when = start + .65 + j*1.01 + random.uniform(-.025, .043)
            note(base, piano, when, pitch, (.27 - .046*j)*random.uniform(.89, 1.11), 2.25)
        if i % 4 != 3:
            for when, pitch in [(1.57, fifth+12), (3.23, bass+24)]:
                note(color, harp, start+when, pitch, random.uniform(.17,.24), 2.1)
    encode("farm_piano.ogg", base, .67)
    encode("farm_harp.ogg", color, .47)


def take(audio, start, length, attack=.009, release=.14):
    frame = round(start * RATE)
    return fade(audio[frame:frame+round(length*RATE)], attack, release)


def effects(cache, piano, harp):
    sources = {
        name: decode(download(cache, name + ".mp3"), 1)
        for name in RECORDINGS if name != "birds"
    }
    for name, key, start, length, peak in [
        ("axe_cut", "axe", 1.81, .33, .83),
        ("axe_cut_alt", "axe", 7.01, .29, .78),
        ("stone_break", "rocks-a", .204, .55, .68),
        ("stone_break_alt", "rocks-b", .231, .60, .72),
        ("tree_crack", "tree", .197, .60, .64),
        ("tree_fall", "tree", 3.093, .94, .86),
        ("soil_till", "dirt", 4.49, .45, .65),
        ("soil_sow", "dirt", 7.64, .25, .48),
        ("soil_fertilize", "dirt", 16.45, .45, .48),
        ("garden_water", "water", 42.32, .88, .60),
        ("card_hit", "rocks-b", .67, .35, .65),
    ]:
        clip = take(sources[key], start, length, release=min(.18, length*.44))
        encode(name + ".wav", clip, peak)
    for name, notes, instrument, total in [
        ("harvest_sound", [(0., 69, .58), (.135, 74, .38)], harp, .67),
        ("card_shield", [(0., 57, .54), (.043, 62, .49), (.17, 66, .29)], harp, .74),
        ("craft_done", [(0., 54, .57), (.13, 62, .42), (.24, 66, .27)], piano, .82),
        ("skill_learn", [(0., 62, .51), (.12, 66, .55), (.24, 69, .50), (.41, 76, .40)], harp, 1.29),
    ]:
        clip = np.zeros((round(total*RATE), 2), dtype=np.float32)
        for when, pitch, value in notes:
            note(clip, instrument, when, pitch, value, total-when)
        encode(name + ".wav", clip, .65)
    birds = decode(download(cache, "birds.mp3"), 2, high=260, low=7900)
    encode("wild_birds.ogg", take(birds, 36., 66., 1.8, 1.8), .58)


if __name__ == "__main__":
    cache = Path(sys.argv[1])
    cache.mkdir(parents=True, exist_ok=True)
    piano, harp = load_instruments(cache)
    score(piano, harp)
    effects(cache, piano, harp)
