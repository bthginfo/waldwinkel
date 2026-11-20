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


waves()
rain()
step("step_grass",[(109,.34),(55,.31)],37.,.44,863)
step("step_wood",[(213,.57),(391,.23),(88,.28)],31.,.16,281)
step("step_snow",[(142,.16)],28.,.67,639)
step("axe_cut",[(154,.77),(332,.20),(94,.24)],28.,.82,803)
step("stone_break",[(428,.42),(782,.31),(1390,.23),(128,.37)],24.,.97,981)
