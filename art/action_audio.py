"""Layer dry Foley and soft resonances for cards and the three workstations."""

from array import array
from math import exp, pi, sin
from random import Random
import sys
import wave

from build_audio import OUT, RATE, low, save


def source(name):
    with wave.open(str(OUT / (name + ".wav")), "rb") as wav:
        pcm = array("h", wav.readframes(wav.getnframes()))
        if sys.byteorder != "little":
            pcm.byteswap()
        channels = wav.getnchannels()
        return [pcm[min(int(i*wav.getframerate()/RATE)*channels, len(pcm)-1)]/32768
                for i in range(int(wav.getnframes()*RATE/wav.getframerate()))]


def foley(name, seconds, clips, resonances, seed, texture=.08):
    frames = round(RATE*seconds)
    rng = Random(seed)
    grit = [rng.uniform(-1., 1.) for _ in range(frames)]
    soft = low(grit, .19)
    signal = [0.]*frames
    for at, gain, speed, clip in clips:
        take = source(clip)
        offset = int(at*RATE)
        for i in range(min(frames-offset, int(len(take)/speed))):
            signal[offset+i] += gain*take[int(i*speed)]*min(i/64, 1.)
    for i in range(frames):
        t = i/RATE
        attack = min(t/.025, 1.)
        air = .78*soft[i] + .22*(grit[i]-soft[i])
        signal[i] += texture*air*attack*exp(-t/(seconds*.32))
        for when, tone, gain, fall in resonances:
            u = t-when
            if u > 0:
                struck = (sin(2*pi*tone*u) + .26*sin(2*pi*tone*2.416*u))
                signal[i] += gain*struck*(1-exp(-u*280))*exp(-u*fall)
        signal[i] *= min((frames-i)/360, 1.)
    save(name,signal)


if __name__ == "__main__":
    foley("card_slide",.24,[(.012,.20,1.31,"step_snow")],[],432,.31)
    foley("turn_page",.35,[(0.,.19,1.14,"step_snow"),(.096,.14,1.64,"step_snow")],[],813,.18)
    foley("card_bloom",.52,[(0.,.19,1.14,"garden_water")],
          [(.045,392,.12,12),(.08,523,.09,11),(.15,659,.075,9)],571)
    foley("kitchen_chop",.28,[(.005,.36,1.85,"axe_cut"),(.112,.21,1.91,"axe_cut_alt")],
          [(0.,271,.11,56),(.11,295,.08,56)],521)
    foley("kitchen_stir",.40,[(0.,.33,1.45,"garden_water")],
          [(.085,614,.08,45),(.197,712,.09,38)],689,.15)
    foley("kitchen_jar",.31,[(0.,.17,1.58,"stone_break_alt")],
          [(0.,861,.14,34),(.07,1179,.058,29)],242)
    foley("kitchen_mill",.36,[(0.,.28,1.39,"stone_break"),(.105,.16,.94,"step_wood")],
          [(0.,176,.12,54)],951,.24)
    foley("kitchen_done",.68,[(0.,.16,1.06,"craft_done")],
          [(.06,349,.11,9),(.12,440,.09,9),(.21,523,.09,8)],631,.04)
