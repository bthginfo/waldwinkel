"""Synthesize the original Waldwinkel ambience and footstep samples."""
from array import array
from math import exp, pi, sin
from pathlib import Path
from random import Random
import sys
import wave

RATE=24000
OUT=Path(__file__).resolve().parents[1]/"assets"/"audio"
OUT.mkdir(parents=True,exist_ok=True)


def save(name,samples):
    pcm=array("h",(round(max(-.98,min(.98,x))*32767) for x in samples))
    if sys.byteorder!="little":
        pcm.byteswap()
    with wave.open(str(OUT/(name+".wav")),"wb") as sound:
        sound.setparams((1,2,RATE,0,"NONE","not compressed"))
        sound.writeframes(pcm.tobytes())


def low(source,response):
    memory=0.
    result=[]
    for value in source[-2400:]:
        memory+=(value-memory)*response
    for value in source:
        memory+=(value-memory)*response
        result.append(memory)
    return result


def waves():
    season=18
    rng=Random(541)
    noise=[rng.uniform(-1,1) for _ in range(season*RATE)]
    surf=low(noise,.13)
    swell=low(noise,.010)
    def sample(i):
        t=i/RATE
        wash=(.5+.5*sin(2*pi*(t/6.-.28)+.48*sin(2*pi*t/18.)))**2.25
        foam=(.5+.5*sin(2*pi*(t/4.5-.40)))**5
        return surf[i]*(.24+wash*.66)+swell[i]*(.39+foam*2.2)+noise[i]*wash*.035
    save("coast",(sample(i) for i in range(len(noise))))


def rain():
    rng=Random(771)
    frames=12*RATE
    noise=[rng.uniform(-1,1) for _ in range(frames)]
    soft=low(noise,.085)
    signal=[x*.075+y*.38 for x,y in zip(noise,soft)]
    for _ in range(760):
        start=rng.randrange(frames)
        tone=rng.uniform(370,1540)
        strength=rng.uniform(.045,.34)
        for j in range(rng.randrange(80,290)):
            t=j/RATE
            tap=(.49*sin(2*pi*t*tone)+.51*rng.uniform(-1.,1.))*(1-exp(-j/9))*exp(-j/59)
            signal[(start+j)%frames]+=tap*strength
    save("rain_wood",signal)


def step(name,tones,decay,grit,seed):
    rng=Random(seed)
    raw=[rng.uniform(-1,1) for _ in range(int(RATE*.21))]
    dust=low(raw,.25)
    def sample(i):
        t=i/RATE
        attack=1-exp(-t*750)
        grunt=sum(sin(2*pi*t*f*(1.-.31*t))*a for f,a in tones)
        return (.39*grunt+grit*dust[i])*attack*exp(-t*decay) + .13*grit*raw[i]*exp(-abs(t-.06)*75)
    save(name,(sample(i) for i in range(len(raw))))

def score(name, notes, beats, interval, base):
    samples = [0.] * round(RATE * beats * interval)
    for beat, semi, strength in notes:
        frequency = base * 2**(semi / 12)
        start = round(RATE * beat * interval)
        tail = min(round(RATE * interval * 3.8), len(samples) - start)
        for i in range(tail):
            t = i / RATE
            fade = min(1., (tail-i) / (RATE*.1))
            attack = (1-exp(-t*46)) * exp(-t*2.9)
            pluck = .78*sin(2*pi*t*frequency) + .18*sin(2*pi*t*frequency*2.01)
            overtone = .045*sin(2*pi*t*frequency*3.96)*exp(-t*7.4)
            samples[start+i] += strength * fade * attack * (pluck + overtone)
    save(name, samples)


def forest():
    rng = Random(283)
    count = RATE * 12
    breeze = low([rng.uniform(-1, 1) for _ in range(count)], .026)
    signal = [x * .44 * (.68 + .32*sin(2*pi*i/count)) for i,x in enumerate(breeze)]
    for offset, pitches in [(1.2, (1680, 2220)), (5.8, (1960, 1510, 1950)), (9.7, (1760, 2080))]:
        for j, pitch in enumerate(pitches):
            for i in range(round(RATE*.16)):
                t = i/RATE
                phase = 2*pi*(pitch*t+940*t*t)
                signal[round(RATE*(offset+j*.22))+i] += .07 * sin(phase) * sin(pi*t/.16)**2
    save("fernwind", signal)


waves()
rain()
forest()
score("farm_theme", [(0,-12,.27),(0,0,.24),(2,7,.22),(3,9,.26),(5,4,.24),
                      (7,-8,.20),(8,2,.23),(10,7,.25),(11,4,.18),(13,0,.24),
                      (16,-12,.23),(16,4,.25),(18,7,.22),(19,12,.24),(21,9,.20),
                      (24,-5,.25),(24,7,.25),(26,2,.21),(28,0,.23)],
      32,.86,261.626)
score("harbor_theme", [(0,-12,.25),(0,4,.22),(2,7,.20),(4,11,.25),(5,12,.28),
                        (8,-7,.19),(8,11,.22),(10,7,.22),(11,4,.24),(14,2,.22),
                        (16,-12,.22),(16,7,.26),(18,11,.23),(19,16,.24),(21,12,.22),
                        (24,-5,.21),(24,9,.23),(26,7,.22),(28,4,.21),(30,0,.26)],
      35,.76,293.665)
score("harvest_sound", [(0,0,.49),(1,4,.37),(2,7,.35)], 5,.075,440.)
score("card_shield", [(0,-12,.47),(1,-5,.35)], 4,.075,293.665)
step("card_hit",[(129,.58),(319,.21),(742,.12)],25.,.64,194)
step("step_grass",[(109,.34),(55,.31)],37.,.44,863)
step("step_wood",[(213,.57),(391,.23),(88,.28)],31.,.16,281)
step("step_snow",[(142,.16)],28.,.67,639)
step("axe_cut",[(154,.77),(332,.20),(94,.24)],28.,.82,803)
step("stone_break",[(428,.42),(782,.31),(1390,.23),(128,.37)],24.,.97,981)
