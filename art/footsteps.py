"""Mix short, quiet walking contacts from the project's existing CC0 clips."""
from pathlib import Path
import wave
import numpy as np

AUDIO = Path(__file__).resolve().parents[1] / "assets" / "audio"
RATE = 24000

def resample(values, count):
    return np.interp(np.linspace(0, len(values) - 1, count), np.arange(len(values)), values)

def source(name):
    with wave.open(str(AUDIO / f"{name}.wav"), "rb") as file:
        clip = np.frombuffer(file.readframes(file.getnframes()), dtype="<i2").astype(float)
        clip = clip.reshape(-1, file.getnchannels()).mean(axis=1)
    return clip / max(1., np.max(np.abs(clip)))

def soft_filter(signal, width):
    return np.convolve(signal, np.full(width, 1. / width), mode="same")

def contact(surface, input_file, decay, scrape, seed):
    rng = np.random.default_rng(seed)
    clip = source(input_file)
    for take, duration in enumerate((.117, .126)):
        count = round(RATE * duration)
        t = np.arange(count) / RATE
        offset = int(len(clip) * (.07 + .12 * take))
        short = resample(clip[offset:offset + int(len(clip) * .49)], count)
        texture = rng.normal(0., 1., count)
        brushes = soft_filter(texture, 9) - soft_filter(texture, 39)
        contact_peak = .070 * np.exp(-t * (75. + 11. * take)) * np.sin(t * 2. * np.pi * (72. + 3. * take))
        envelope = np.minimum(1., t * 320.) * np.exp(-t * decay) * np.minimum(1., (duration - t) * 67.)
        audio = envelope * (.19 * soft_filter(short, 13) + scrape * brushes + contact_peak)
        name = f"step_{surface}" + ("_alt" if take else "")
        with wave.open(str(AUDIO / f"{name}.wav"), "wb") as file:
            file.setparams((1, 2, RATE, 0, "NONE", ""))
            file.writeframes(np.round(audio * 32767.).astype("<i2").tobytes())

if __name__ == "__main__":
    for args in [
        ("grass", "soil_till", 35., .27, 3202),
        ("wood", "axe_cut_alt", 54., .08, 6091),
        ("stone", "stone_break_alt", 48., .11, 7293),
        ("sand", "soil_sow", 36., .32, 3621),
        ("snow", "soil_fertilize", 34., .34, 1064),
    ]:
        contact(*args)
