"""The relief and the three entrances of Waldwinkel's continuous island."""
import math
import random


SHOULDERS = [
    (-121., -4., 53., 51.), (-73., -10., 58., 52.),
    (-27., -11., 61., 55.), (31., -6., 63., 44.), (87., -10., 62., 54.),
    (-54., 35., 48., 40.), (-25., 63., 42., 49.),
    (12., 74., 63., 48.), (26., 103., 55., 39.),
]

# Preserve the small tidal bays, the farm coast and the sheltered water at the fisher pier.
INLETS = [(63., 42., 14., 13.), (76., 49., 17., 15.), (91., 57., 20., 16.)]

PEAKS = [
    (-157., -47., 18., 18., 16.), (-140., -58., 24., 14., 19.),
    (-112., -42., 15., 18., 12.), (-63., -44., 23., 19., 24.),
    (-42., -50., 19., 13., 27.), (-26., -38., 19., 16., 18.),
    (12., -36., 28., 17., 24.), (36., -32., 24., 16., 19.),
    (109., -43., 28., 19., 20.), (139., -51., 25., 18., 27.),
    (168., -47., 23., 18., 21.), (187., -19., 13., 21., 18.),
    (-57., 25., 15., 13., 8.), (-41., 39., 17., 16., 10.),
    (-29., 57., 17., 13., 8.), (-26., 96., 19., 16., 12.),
    (59., 101., 19., 16., 12.), (73., 133., 14., 19., 9.),
    (12., 175., 23., 11., 10.), (48., 173., 22., 11., 11.),
]


def smooth(value):
    value=max(0.,min(value,1.))
    return value*value*(3.-2.*value)


def divide(x, z):
    return 76.15+1.13*math.sin((x-9.5)*.079)+.41*math.sin((x-9.5)*.153)


def bridge(x, z):
    if 8.13 < x < 10.87 and 72.70 < z < 79.25:
        return .225
    return None


def height(x, z, base, shore):
    if shore < -1.95:
        return base
    # Each saddle remains in the original road alignment.
    mountains = 0.
    for axis, along, center, pass_z, peak, width in [
        (x,z,78.,9.,27.,15.0), (x,z,-87.,-14.,22.,14.0),
    ]:
        if abs(axis-center) < width+4.0:
            crest = center + 2.7*math.sin((along-pass_z)*.095)
            side = max(0.,1.-abs(axis-crest)/width)
            notch = smooth((abs(along-pass_z)-2.55)/4.75)
            ridges = .76+.15*math.sin(along*.227)+.11*math.sin(along*.471)
            mountains += peak*side**1.32*notch*ridges
    for a,b,w,d,h in PEAKS:
        if abs(x-a)<w and abs(z-b)<d:
            dist = math.hypot((x-a)/w,(z-b)/d)
            if dist<1.:
                face = .92+.17*math.sin(x*.28+z*.17)*math.sin(z*.33-x*.16)
                mountains = max(mountains,h*(1.-dist)**1.21*face)
    if 69.0 < z < 83.0:
        river = abs(z-divide(x, z))
        watercourse=1.-smooth((river-1.18)/1.25)
        base -= 1.31*watercourse
        mountains *= 1.-watercourse
        # A softer shoulder above the river makes the bridge visible from the southern paths.
        if abs(x-9.5)>6.0 and 2.6<river<6.2:
            mountains += .89*(1.-abs(river-4.4)/1.8)
    return base + mountains*smooth((shore+.70)/5.0)


def crest_blocks(x, z, height):
    width=3.40 if height>=2.1 else 1.28
    return ((abs(x-78.)<width and z<70. and abs(z-9.)>3.0)
            or (abs(x+87.)<width and abs(z+14.)>3.0)
            or (73.0<z<79.3 and abs(z-divide(x,z))<.85 and bridge(x,z) is None))


def models(api):
    Model, land = api.Model, api.landscape
    rng = random.Random(15801)
    stone = Model("Insel · Hanggeröll und Felsfüße","island_relief")
    woods = Model("Insel · Wälder an den Pässen","island_relief")
    river = Model("Insel · Steinbrücke über den Weidebach","island_relief")
    banks = Model("Insel · Ufergras am Weidebach","island_relief",casts=False)
    # The sloped bridge approaches sit at the height of the collision deck.
    for i in range(24):
        z=72.72+i*.278
        river.box(9.50,z,.154,3.16,.267,.142,["bc986f","c8a47c","ae8c6b","d1af84"][i%4],.023)
        if i%5==0:
            for x in [7.95,11.05]:
                river.pole((x,z,-1.32),(x,z,.73),.144,"967858",7)
                river.pole((x,z,.731),(x,z,.793),.153,"ceab79",7)
    for x in [7.96,11.04]:
        for y in [.409,.652]:
            river.pole((x,72.72,y),(x,79.16,y),.051,"b5946f",6)
        river.obstacle_rail(x,72.70,x,79.25,.095)
    for side in [-1,1]:
        for j in range(12):
            x=9.5+side*(2.22+j*.64)
            z=divide(x,76)+side*.65
            for k in range(3):
                wx=x+rng.uniform(-.26,.26)
                wz=z+side*(1.24+k*.35)
                y=land.height(wx,wz)
                if y>-.37:
                    api.grass(banks,wx,wz,y,.22+rng.random()*.19,"99ab68",4)
    # Mark the three saddles with visible stone shoulders and outlying stands of fir.
    for cx,cz,cxscale,czscale,count in [
        (-91.,-26.,13.,12.,42),(-86.,.7,15.,14.,51),
        (74.,-8.,14.,15.,53),(84.,27.,15.,12.,42),
        (-8.,68.,20.,7.,49),(26.,66.,15.,9.,42),
        (-42.,51.,16.,18.,49),(38.,97.,19.,11.,44),
        (-33.,-38.,26.,11.,48),(27.,-28.,23.,11.,44),
    ]:
        placed=[]
        for i in range(count):
            x,z=cx+rng.uniform(-cxscale,cxscale),cz+rng.uniform(-czscale,czscale)
            y=land.height(x,z)
            if y<-.16 or land.road_distance(x,z)<2.0:
                continue
            if any(math.hypot(x-a,z-b)<2.20 for a,b in placed):
                continue
            placed.append((x,z))
            pressure=max(abs(y-land.height(x+.55,z)), abs(y-land.height(x,z+.55)))
            if pressure<.29 and y < 10.5 and i%3==0:
                s=rng.uniform(.61,1.22)
                api.pine(woods,x,z,y,s,i)
            else:
                s=rng.uniform(.43,1.37)
                for k in range(2):
                    rock=["aaa091","8e8880","a39683","ad907d"][(i+k)%4]
                    stone.ellipsoid(x+k*.43,z+k*.38,y+s*.32,s,s*.68,s*.63,rock,7,4,
                                    angle=rng.random(),rough=.15)
                if pressure<.3:
                    stone.obstacle_circle(x,z,s*.60)
    return [river,stone,woods,banks]
