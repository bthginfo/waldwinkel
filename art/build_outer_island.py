"""blender -b -t 3 -P art/build_outer_island.py [-- --bake]"""
from math import atan2, cos, dist, pi, sin
from pathlib import Path
from random import Random
import json
import struct
import sys

import bpy
from mathutils import Vector
from mathutils.bvhtree import BVHTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import build_scene as base
from expansion_layout import Ground, ROOT, SEA, SITES, bake_roofs, original, phase, smooth
from expansion_props import (animal, boulder, building, crate, crystal, fence, furniture, housing,
                             lamp, plant, products, resource, slats, tools)

Model = base.Model
NAMES = {'amber':'Bernsteinhain','heights':'Himmelstufen','reef':'Morgenriff'}
WOOD = ('b08a58','c49a6a','957554','d4aa72')


def terrain(g):
    name=f'extension_{g.name}'
    m=Model('Erdrelief · '+NAMES[g.name],name,casts=False)
    ox,oz,ex,ez=g.bounds
    rng=Random(713)
    step=1.12
    points={}
    # Shared vertices between terrain triangles keep the coastline closed.
    for j in range(int((ez-oz)/step)+1):
        for i in range(int((ex-ox)/step)+1):
            x=ox+i*step + rng.uniform(-.19,.19)
            z=oz+j*step + rng.uniform(-.19,.19)
            if not (ox<=x<ex and oz<=z<ez):
                continue
            h,new,_=g.dominant(x,z)
            points[i,j]=(x,z,h,new)
    grass, moss, trail, sand, shadow, rock=g.spec['palette']
    for (i,j),a in points.items():
        neighbors=((i+1,j),(i+1,j+1),(i,j+1))
        if any(n not in points for n in neighbors):
            continue
        b,c,d=(points[n] for n in neighbors)
        diagonal=((a,b,d),(b,c,d)) if (i+j)%2 else ((a,b,c),(a,c,d))
        for face in diagonal:
            highest=max(p[2] for p in face)
            if highest < SEA-.09 or not any(p[3] for p in face):
                continue
            # The original coast still shows through where the old ground is higher.
            delta=max(p[2] - original(p[0],p[1])[0] for p in face if p[3])
            if delta<.068:
                continue
            x=sum(p[0] for p in face)/3
            z=sum(p[1] for p in face)/3
            steep=highest-min(p[2] for p in face)
            sun=.07*phase(x*.41,z*.42)
            colors=[]
            for ix,iz,h,_ in face:
                d=g.way(ix,iz)
                t=phase(ix,iz)
                tone=base.blend(grass,moss,.30+.24*t)
                tone=base.blend(tone,trail,.91*(1-smooth((d-.72)/1.43)))
                if h<.04:
                    tone=base.blend(tone,sand,(1-smooth((h-SEA)/(.84 if g.name=='reef' else .94)))*.79)
                if steep>.46:
                    tone=base.blend(tone,rock,smooth((steep-.46)/1.08)*.84)
                if g.name=='heights':
                    tone=base.blend(tone,'eee6d1',smooth((h-10.7)/4.8)*(.69+.18*t))
                tone=base.blend(tone,shadow,.053+max(0,-sun)+rng.uniform(0,.044))
                colors.append(base.blend(tone,'fff0ba',max(0.,sun)+.045))
            m.gradient([p[:3] for p in face],colors)
    return m


def surfaces(g):
    m=Model('Erweiterte Küste · '+NAMES[g.name],f'extension_{g.name}',casts=False)
    water=Model('Gezeitentümpel' if g.name=='reef' else 'Bachlauf am Außenweg',m.asset,casts=False)
    rng=Random(134+len(g.name))
    if g.name=='reef':
        for x,z,rx,rz in g.spec['pools']:
            rim=[(x+cos(i*pi/13)*rx*.89,z+sin(i*pi/13)*rz*.89,SEA+.012) for i in range(26)]
            for i,a in enumerate(rim):
                b=rim[(i+1)%len(rim)]
                water.gradient([(x,z,SEA+.012),a,b],[base.rgb(c) for c in ('5bafac','8bcdc0','99d3c1')])
            for i in range(9):
                a=rng.uniform(0,2*pi)
                rr=rng.uniform(.22,.84)
                xx,zz=x+cos(a)*rx*rr,z+sin(a)*rz*rr
                ww=rng.uniform(.17,.62)
                m.face([(xx-ww,zz,SEA+.029),(xx,zz-.06,SEA+.029),
                        (xx+ww,zz,SEA+.029),(xx,zz+.045,SEA+.029)],'a5d9ce','raw')
    else:
        centers=g.spec['water']
        for (x,z),(a,b) in zip(centers,centers[1:]):
            length=dist((x,z),(a,b))
            for i in range(int(length*2)):
                f=i/int(length*2); t=(i+1)/int(length*2)
                xx,zz=x+(a-x)*f,z+(b-z)*f
                u,v=x+(a-x)*t,z+(b-z)*t
                if not (g.sample(xx,zz)[0] > SEA+.15 and g.sample(u,v)[0] > SEA+.15):
                    continue
                if any(dist((xx,zz),p)<1.55 for p in g.spec['crossings']):
                    continue
                width=.77+.19*cos(zz*.24)
                h=g.sample(xx,zz)[0]+.087
                nxt=g.sample(u,v)[0]+.087
                water.gradient([(xx-width,zz,h),(xx+width,zz,h),(u+width,v,nxt),(u-width,v,nxt)],
                               [base.rgb(c) for c in ('7eb7a4','5c9fa8','65b0b7','89c4b5')])
                if i%9==0:
                    m.face([(xx-.12,zz,h+.019),(xx+.59,zz+.09,h+.019),(xx+.16,zz+.15,h+.019)],'add3bd','raw')
    return m,water


def paths(g, m):
    rng=Random(944)
    approach=g.spec['approach']
    gate_index={'amber':1,'heights':2,'reef':2}[g.name]
    gate=approach[gate_index]
    h=g.sample(*gate)[0]
    axial=abs(approach[gate_index+1][0]-gate[0])>abs(approach[gate_index+1][1]-gate[1])
    for side in (-1,1):
        f=(gate[0]+(.0 if axial else side*1.83),gate[1]+(side*1.83 if axial else .0))
        lamp(m,*f,g.sample(*f)[0], '8e724b' if g.name!='reef' else '73867b')
    # Low side rails along the new causeway, including at the gate.
    a,b=approach[1],approach[-2]
    angle=0 if axial else pi*.5
    for side in (-1,1):
        cx=(a[0]+b[0])*.5+(.0 if axial else side*1.78)
        cz=(a[1]+b[1])*.5+(side*1.78 if axial else .0)
        fence(m,cx,cz,dist(a,b)-1.04,0,angle,'a3825b',lambda x,z:g.sample(x,z)[0])
    for a,b in zip(approach,approach[1:]):
        length=dist(a,b)
        n=int(length/.27)
        for i in range(n):
            t=(i+.5)/n
            x,z=a[0]+(b[0]-a[0])*t,a[1]+(b[1]-a[1])*t
            y,blocked=g.sample(x,z)
            if y<-.30 or blocked:
                continue
            m.box(x,z,y+.006, .254 if axial else 2.96, 2.96 if axial else .254, .075,
                  WOOD[i%len(WOOD)],.012,mode="ground")
    for road in g.spec['roads']:
        for a,b in zip(road,road[1:]):
            length=dist(a,b)
            for i in range(max(1,int(length/.62))):
                t=(i+.46)/max(1,int(length/.62))
                x,z=a[0]+(b[0]-a[0])*t,a[1]+(b[1]-a[1])*t
                x+=rng.uniform(-.85,.85);z+=rng.uniform(-.85,.85)
                y,blocked=g.sample(x,z)
                if blocked or y<-.34:
                    continue
                color={'amber':'b9a278','heights':'c5b69c','reef':'cabfa6'}[g.name]
                m.ellipsoid(x,z,y-.020,rng.uniform(.18,.55),rng.uniform(.13,.39),.083,
                            base.blend(color,'e0ceaa',rng.uniform(0,.42)),6,3,angle=rng.uniform(0,pi),rough=.15)
    for x,z in g.spec.get('crossings',()):
        y=g.sample(x,z)[0]
        slats(m,x,z,y+.05,6.55,2.08,.125,WOOD,axis=1)
        for side in (-1,1):
            fence(m,x,z+side*1.025,6.20,0,0,'98744d',lambda a,b:g.sample(a,b)[0])
            for xx in (x-2.8,x+2.8):
                m.pole((xx,z+side,-.02+y),(xx,z+side,y-.74),.13,'77563f',7)


def landmarks(g,m):
    for index,(x,z,label) in enumerate(g.spec['hubs']):
        y,_=g.sample(x,z)
        size=(4.05,2.86) if index==0 else (3.30,2.59)
        building(m,x,z,y,*size,g.name,0 if index==0 else 1)
        for dx,dz in [(-1.76,1.80),(1.56,1.78),(1.99,-.69)]:
            xx,zz=x+dx,z+dz
            hh=g.sample(xx,zz)[0]
            if dz>1:
                crate(m,xx,zz,hh)
            else:
                lamp(m,xx,zz,hh)
        if index==0:
            for dx,dz in [(2.95,.2),(3.1,1.04),(3.15,-.61)]:
                xx,zz=x+dx,z+dz;hh=g.sample(xx,zz)[0]
                palette={'amber':'a77a43','heights':'677e87','reef':'98a990'}
                base.barrel(m,xx,zz,hh,.77)
                m.ellipsoid(xx,zz,hh+.57,.19,.17,.036,palette[g.name],9,3)
    if g.name=='amber':
        for z in (-19.,-17.5,-16.0):
            x=-254.1; h=g.sample(x,z)[0]
            fence(m,x,z,2.95,0,0,'9e7349',lambda a,b:g.sample(a,b)[0])
            for i in range(3):
                dx=i*.43-.43
                m.pole((x+dx,z+.04,h+.42),(x+dx,z+.04,h+.43),.129,'c89b57',7)
                m.ellipsoid(x+dx,z+.04,h+.44,.096,.095,.034,'e5ad55',8,3,mode='light')
        x,z=-246.5,-18.25
        h=g.sample(x,z)[0]
        for xx in (x-.20,x+1.15):
            m.pole((xx,z+.92,h-.05),(xx,z+.92,h+1.63),.080,'735639',7)
        for i in range(7):
            z0=z-.52+i*.21
            m.face([(x-.46,z0,h+1.92),(x+1.36,z0,h+1.61),
                    (x+1.36,z0+.22,h+1.61),(x-.46,z0+.22,h+1.92)],
                   ('bb9254','9b7447','a68051')[i%3])
        for j in range(3):
            for i in range(4-j):
                xx=x+(i-1.5+j*.5)*.254
                yy=h+.11+j*.187
                m.pole((xx,z-.36,yy),(xx,z+.73,yy+.025),.115,'806344',8)
                m.pole((xx,z+.718,yy+.025),(xx,z+.744,yy+.025),.098,'d2ad75',8)
        x,z=-286.9,-7.4; h=g.sample(x,z)[0]
        lamp(m,x,z,h)
        for n in range(7):
            boulder(m,x+cos(n*.9)*2.42,z+sin(n*.9)*2.05,h,.47,'8f8673',n*.45)
    if g.name=='heights':
        x,z=254.,-28.; y=g.sample(x,z)[0]
        for a in (-1,1):
            for b in (-1,1):
                m.pole((x+a*1.1,z+b*.80,y),(x+a*.84,z+b*.52,y+4.28),.133,'72584a',7)
        for i in range(3):
            yy=y+.55+i*1.32
            m.pole((x-1.04,z-.66,yy),(x+1.04,z-.66,yy+1.08),.065,'ad8160',6)
            m.pole((x+1.04,z+.66,yy),(x-1.04,z+.66,yy+1.08),.065,'ad8160',6)
        slats(m,x,z,y+4.17,2.02,1.55,.098,WOOD)
        m.pole((x-1.1,z,y+4.44),(x+1.45,z+.00,y+4.44),.103,'99785d',7)
        m.pole((x+1.42,z,y+4.39),(x+1.42,z,y+.49),.019,'d1ac75',5)
        drum=(x+1.05,z+.58,y+3.75)
        m.pole((drum[0],drum[1]-.88,drum[2]),(drum[0],drum[1]+.22,drum[2]),.114,'80695a',9)
        for i in range(12):
            theta=2*pi*i/12
            following=2*pi*(i+1)/12
            a=(drum[0]+.68*cos(theta),drum[1],drum[2]+.68*sin(theta))
            b=(drum[0]+.68*cos(following),drum[1],drum[2]+.68*sin(following))
            m.pole(a,b,.074,'b18a64',6)
            if i%2==0:
                m.pole(drum,a,.043,'936f57',6)
        for dx in (-1.65,1.67):
            for j in range(3):
                m.box(251.6+dx,-24.4+1.40,y+.28+j*.31,.33,.30,.32,
                      ('aaa69f','c4b8a0','948b7d')[j],.057)
        for lo,hi in [((260,-41),(263,-43)),((275,-63),(280,-65)),((281,-15),(284,-21))]:
            xx,zz=(lo[0]+hi[0])/2,(lo[1]+hi[1])/2
            fence(m,xx,zz,dist(lo,hi),0,atan2(hi[1]-lo[1],hi[0]-lo[0]),
                  'a58c67',lambda a,b:g.sample(a,b)[0])
    if g.name=='reef':
        for x,z,w,d in [(22.,225.,3.2,1.45),(27.,224.,4.1,2.45),(47.,250.,3.5,2.60),(51.,253.,3.1,1.90)]:
            y=g.sample(x,z)[0]
            slats(m,x,z,y+.04,w,d,.106,('ab906b','cbb18a','b8a079'))
            for a in (-1,1):
                for b in (-1,1):
                    xx=x+a*w*.46;zz=z+b*d*.44
                    m.pole((xx,zz,y+.56),(xx,zz,y-.65),.12,'92755b',8)
        x,z=30.0,225.6
        y=g.sample(x,z)[0]
        for dx in (-1.0,1.0):
            m.pole((x+dx,z,y),(x+dx,z,y+2.17),.073,'7f8c77',7)
        m.pole((x-1.03,z,y+1.98),(x+1.13,z,y+1.98),.049,'948b6b',6)
        for i in range(6):
            xx=x+(i-2.5)*.28
            m.face([(xx-.13,z,y+1.99),(xx+.13,z,y+1.99),(xx+.085,z,y+1.77),
                    (xx-.055,z,y+1.73)],('b891a4','e0ba7c','a3b9a5')[i%3])
        slats(m,x,z+.24,y+.56,1.75,.74,.081,('ac9371','c0a77e','a79170'))
        for i in range(4):
            xx=x+(i-1.5)*.32
            m.ellipsoid(xx,z+.21,y+.647,.147,.122,.058,('cbb6ab','dbc6a8','e0d5b5','acc5b8')[i],9,3)
        for x,z in ((35.6,225.7),(8.,244.8),(42.,238.6),(53.,247.5),(31.,265.)):
            y=SEA+.11
            wood='a88b64'
            outline=[(x+cos(i*pi/6)*1.20,z+sin(i*pi/6)*.47,y+abs(cos(i*pi/6))*.12) for i in range(12)]
            base.figure_art.hem(m,outline+[outline[0]],.073,wood)
            m.face(outline,'98795a')
            for dx in (-.42,.34):
                m.box(x+dx,z,y+.11,.17,.71,.107,'cfb489',angle=.14)
            m.pole((x-.30,z-.62,y+.05),(x+.60,z+.77,y+.06),.027,'d3bc92',7)


HARD = {
    'amber': {10:[(-241,-15),(-245,-7),(-256,-17),(-261,-19),(-272,-9),(-278,11),(-249,17),(-239,9),
                   (-273,-32),(-286,5),(-248,-30),(-275,19),(-282,-19),(-261,10)],
              13:[(-280,-8),(-282,-13),(-286,0),(-278,14),(-287,-20),(-273,-37),(-284,9)]},
    'heights': {11:[(237,1),(243,-17),(249,-9),(264,-25),(260,-32),(268,-42),(258,-38),(254,-48),
                    (270,-18),(286,-25),(264,-55),(280,-32),(275,-11),(260,3)],
                12:[(263,-57),(266,-54),(278,-53),(280,-58),(270,-62),(263,-63),(274,-66),(281,-48)]},
    'reef': {14:[(30,230),(25,236),(20,245),(34,245),(38,252),(11,246),(2,236),(-9,233),(51,230),
                  (50,250),(49,244),(58,248),(66,232),(57,258)],
             15:[(41,227),(50,226),(58,234),(15,233),(10,240),(8,251),(-10,239),(31,250),
                  (48,252),(56,255),(51,258),(44,261),(24,260)]},
}


def scatter(g):
    rng=Random({'amber':894,'heights':342,'reef':909}[g.name])
    ox,oz,ex,ez=g.bounds
    name=f"extension_{g.name}"
    m=Model('Unterwuchs und Steinbänder · '+NAMES[g.name],name)
    used=[]
    nodes=[]
    def distance(p, positions):
        return min((dist(p, q) for q in positions), default=1e3)
    exclusions=g.spec['reserved']+[h[:2] for h in g.spec['hubs']]
    site_distance=lambda p:distance(p,[h[:2] for h in g.spec['hubs']])

    for variant,sites in HARD[g.name].items():
        for x,z in sites:
            options=[(x,z),*((x+cos(i*2.40)*(1.1+i*.18),z+sin(i*2.40)*(1.1+i*.18)) for i in range(19))]
            for xx,zz in options:
                y,blocked=g.sample(xx,zz)
                if (y<.0 or blocked or g.way(xx,zz)<1.32 or distance((xx,zz),used)<1.90 or
                    distance((xx,zz),exclusions)<1.60 or site_distance((xx,zz))<3.4):
                    continue
                scale=rng.uniform(.90,1.16)
                used.append((xx,zz))
                nodes.append((xx,zz,y,scale,rng.uniform(0,6.28),variant))
                break

    counts={'amber':297,'heights':186,'reef':84}
    for _ in range(counts[g.name]*12):
        x,z=rng.uniform(ox+2,ex-2),rng.uniform(oz+2,ez-2)
        if g.name=='amber' and x>=-219 or g.name=='heights' and x<=222 or g.name=='reef' and z<=211:
            continue
        y,blocked=g.sample(x,z)
        if blocked or y<-.04 or g.way(x,z)<2.05 or site_distance((x,z))<4.30:
            continue
        if distance((x,z),used)<2.69 or distance((x,z),exclusions)<1.57:
            continue
        if g.name=='heights' and (y>7.8 or rng.random()<.43 and z<-42):
            continue
        if abs(phase(x*.52,z*.69))<.18 and rng.random()<.59:
            continue
        v=9 if g.name=='amber' else rng.randrange(0,6)
        if g.name=='reef':
            v=9 if rng.random()<.29 else rng.choice((3,4,5))
        scale=rng.uniform(.68,1.33) if g.name!='reef' else rng.uniform(.54,.96)
        used.append((x,z))
        nodes.append((x,z,y,scale,rng.uniform(0,6.28),v))
        if len(nodes)>=counts[g.name]:
            break
    # Clusters frame the trail and buildings while keeping the path completely clear.
    for _ in range(4100):
        x,z=rng.uniform(ox+2,ex-2),rng.uniform(oz+2,ez-2)
        if g.name=='amber' and x>=-215 or g.name=='heights' and x<=218 or g.name=='reef' and z<=207:
            continue
        y,blocked=g.sample(x,z)
        d=g.way(x,z)
        if y<-.12 or d<1.27 or site_distance((x,z))<2.7 or distance((x,z),exclusions)<.63:
            continue
        if d>6 and rng.random()>.36 or d<4 and rng.random()>.67:
            continue
        if rng.random()<(.34 if g.name=='heights' else .23):
            # Large features are collidable harvest nodes. These stones are ground dressing.
            color={'amber':'9d9480','heights':'a9a7a1','reef':'b7ab98'}[g.name]
            r=rng.uniform(.18,.48)
            boulder(m,x,z,y-.05,r,color,rng.uniform(0,pi))
        elif not blocked:
            t=phase(x,z)
            s=rng.uniform(.39,.99)
            if g.name=='amber':
                if t<-.22:
                    plant(m,x,z,y,s*.66,'e4ac65','8f9459',3)
                else:
                    base.fern(m,x,z,s*.52,lift=y)
            elif g.name=='heights' and y<6.8 and t>-.05:
                plant(m,x,z,y,s*.55,'ad9bb0' if t>.28 else 'e4ce96','778462',3)
            elif g.name=='reef' and t>.02:
                plant(m,x,z,y,s*.85,'c69eb0' if t>.15 else 'dfbd9b','89976a',4)
            else:
                base.grass(m,x,z,y,s*.78,shade='9ca56d',blades=4)
    return m,nodes


def workshop_gardens(g, m):
    rng=Random(512+len(g.name))
    colors={'amber':('daa35e','c87943','f2c071'),
            'heights':('bebad8','b789a3','e6d5a2'),
            'reef':('c894b5','cfaf98','e6c186')}[g.name]
    hubs=g.spec['hubs']
    for hx,hz,_ in hubs:
        for row in range(4):
            x=hx+(-1 if row%2 else 1)*(2.25+row*.42)
            z=hz+1.4+(row//2)*.92
            for k in range(5):
                a,b=x+rng.uniform(-.65,.65), z+rng.uniform(-.62,.62)
                h,blocked=g.sample(a,b)
                if h<-.20 or blocked or g.way(a,b)<.93:
                    continue
                plant(m,a,b,h,rng.uniform(.36,.66),colors[k%3],stems=4)
                base.grass(m,a+.15,b+.24,h,rng.uniform(.37,.58),shade='8d9a67',blades=6)
        for dx,dz in [(-2.47,.86),(-2.84,.13),(2.42,1.47),(2.52,-.51)]:
            x,z=hx+dx,hz+dz
            y,blocked=g.sample(x,z)
            if y<-.24 or g.way(x,z)<.61:
                continue
            boulder(m,x,z,y-.12,.36,'aa9c80',dx*.8)
    for path in g.spec['roads']:
        for (x,z),(a,b) in zip(path,path[1:]):
            distance=dist((x,z),(a,b))
            nx,nz=-(b-z)/distance,(a-x)/distance
            n=max(1,int(distance/.56))
            for i in range(n):
                if rng.random()>.38:
                    continue
                side=-1 if rng.random()<.5 else 1
                dx=(1.48+rng.uniform(0,.56))*side
                dd=(i+.39)/n
                xx,zz=x+(a-x)*dd+nx*dx,z+(b-z)*dd+nz*dx
                y,blocked=g.sample(xx,zz)
                if blocked or y<-.25 or g.way(xx,zz)<1.20:
                    continue
                if rng.random()<.27:
                    plant(m,xx,zz,y,rng.uniform(.36,.59),colors[i%3],stems=3)
                else:
                    base.grass(m,xx,zz,y,rng.uniform(.33,.62),shade='8e9b64',blades=5)
    if g.name=='amber':
        for x,z in [(-252.5,-18),(-256.1,-10.),(-259.,-8.1),(-261.5,-4.8),(-271.4,-6.2)]:
            y=g.sample(x,z)[0]
            base.fern(m,x,z,.81,lift=y)
            plant(m,x+.21,z+.26,y,.61,'e9b167','869357',6)
            for i in range(4):
                h=y+.028*i
                m.ellipsoid(x-.24+i*.16,z+.23,h,.145,.086,.031,('b67b40','e7b365','cc9352')[i%3],6,2,angle=i*1.2)
    if g.name=='heights':
        for x,z in [(248.5,-28.9),(251.1,-31.1),(256.2,-27.8),(267.,-36.),(268.6,-38.4)]:
            y=g.sample(x,z)[0]
            for i in range(5):
                xx,zz=x+(i-2)*.23,z+(i%2)*.18
                boulder(m,xx,zz,y-.11,.40+i%3*.08,'a7a79d',i*.67)
            plant(m,x+.48,z+.37,y,.58,'bfa9c4','879778',5)
    if g.name=='reef':
        for x,z in [(33.1,223.4),(36.5,223.9),(34.4,226.6),(38.0,225.9),(18.,233.0),(27.8,234.6)]:
            y,_=g.sample(x,z)
            if y>-.45:
                plant(m,x,z,y,.75,'c398b6','91966b',5)
                base.grass(m,x-.29,z+.17,y,.52,shade='a4a16e',blades=6)

def region_geometry(g):
    structure=Model('Werkhöfe und Wege · '+NAMES[g.name],f'extension_{g.name}')
    paths(g,structure)
    landmarks(g,structure)
    greenery,nodes=scatter(g)
    workshop_gardens(g, greenery)
    return [terrain(g),*surfaces(g),structure,greenery],nodes


def model_groups():
    nodes=[]
    scenery=[]
    for name in SITES:
        ground=Ground(name)
        assets,harvest=region_geometry(ground)
        scenery.extend(assets)
        nodes.extend(harvest)
        print(NAMES[name],len(harvest),'Ressourcen ·',sum(len(m.faces) for m in assets),'Flächen',flush=True)
    output=bytearray(struct.pack('<4sI',b'WHV1',len(nodes)))
    for node in nodes:
        output+=struct.pack('<fffffB',*node)
    (ROOT/'assets/world/harvest_extension.bin').write_bytes(output)

    catalogue=[*products(),*furniture(),*tools()]
    for kind in ('rabbit','goat','duck'):
        catalogue.extend(animal(kind))
        catalogue.append(housing(kind))
    catalogue.extend(resource(variant) for variant in range(9,16))
    catalogue.extend(resource(variant,True) for variant in (9,10,15))
    residents=base.npc_models([
        ("npc_ivo","squirrel","b38155","7f8061","e7c9a2","685d53"),
        ("npc_sanna","badger","9d9487","59757b","e2d1ad","75666a"),
        ("npc_tilda","rabbit","c3a89d","7d9a8f","edddbb","986c62"),
    ])
    for m in residents:
        if m.name==m.asset:
            color={'npc_ivo':'bb844b','npc_sanna':'806d63','npc_tilda':'aab9a0'}[m.asset]
            m.ellipsoid(0,-.055,1.373,.35,.296,.061,color,12,4,mode="cloth")
            m.ellipsoid(0,-.078,1.46,.236,.210,.128,color,11,5,mode="cloth")
            m.ellipsoid(0,-.08,1.422,.232,.191,.035,'d3ae78',11,4,mode="cloth")
            m.box(.11,.172,.69,.071,.036,.119,'a98b55',.019,mode='cloth')
    return scenery, catalogue+residents


def daylight_bvh(models):
    verts,faces=[],[]
    for m in models:
        if not m.casts:
            continue
        faces.extend(tuple(i+len(verts) for i in f) for f in m.faces)
        verts.extend(m.vertices)
    return BVHTree.FromPolygons(verts,faces)


def compact_colors(path):
    """Use normalized glTF byte colors for static vertex-lit scenery."""
    raw=path.read_bytes()
    size,=struct.unpack_from('<I',raw,12)
    doc=json.loads(raw[20:20+size])
    source=memoryview(raw)[28+size:]
    colors={p['attributes']['COLOR_0'] for mesh in doc['meshes'] for p in mesh['primitives']}
    views={doc['accessors'][a]['bufferView']:doc['accessors'][a] for a in colors}
    data=bytearray()
    for i,view in enumerate(doc['bufferViews']):
        start,count=view.get('byteOffset',0),view['byteLength']
        content=source[start:start+count]
        assert len(data)%4==0
        view['byteOffset']=len(data)
        if i in views:
            accessor=views[i]
            assert accessor['type']=='VEC3' and accessor['componentType']==5126
            for color in struct.iter_unpack('<fff',content):
                data.extend([*(round(max(0.,min(1.,c))*255) for c in color),255])
            accessor.update(normalized=True,componentType=5121,type='VEC4')
            view['byteLength']=accessor['count']*4
        else:
            data.extend(content)
        data.extend(b'\x00' * (-len(data)%4))
    doc['buffers'][0]['byteLength']=len(data)
    body=json.dumps(doc,separators=(',',':')).encode()
    body+=b' '*(-len(body)%4)
    header=struct.pack('<4sIII4s',b'glTF',2,28+len(body)+len(data),len(body),b'JSON')
    path.write_bytes(header+body+struct.pack('<I4s',len(data),b'BIN\x00')+data)

def export():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    base.RNG.seed(88109)
    scenery, catalogue=model_groups()
    material=bpy.data.materials.new("Gebackenes Sonnenlicht · Außenbezirke")
    material.use_nodes=True
    shader=material.node_tree.nodes.get('Principled BSDF')
    shader.inputs['Roughness'].default_value=.94
    color=material.node_tree.nodes.new("ShaderNodeVertexColor")
    color.layer_name="COLOR_0"
    material.node_tree.links.new(color.outputs["Color"],shader.inputs["Base Color"])
    material.diffuse_color=(.65,.56,.38,1.)
    objects={}
    for m in [piece for model in scenery for piece in base.scenery_patches(model, width=18.)]+catalogue:
        if m.asset not in objects:
            coll=bpy.data.collections.new(NAMES.get(m.asset.removeprefix('extension_'),m.asset))
            bpy.context.scene.collection.children.link(coll)
            objects[m.asset]=(coll,[])
        coll,group=objects[m.asset]
        local=not m.asset.startswith('extension_') and m.casts and len(m.faces)>50 and not base.figure_art.is_figure(m.asset)
        shadow=daylight_bvh([m]) if local else None
        item=base.build_object(m,shadow,material,coll,Vector((0,0,0)),{},local)
        group.append(item)
    for asset,(_,group) in objects.items():
        bpy.ops.object.select_all(action="DESELECT")
        for obj in group:
            obj.select_set(True)
        bpy.context.view_layer.objects.active=group[0]
        path=base.OUT/(asset+'.glb')
        bpy.ops.export_scene.gltf(filepath=str(path),use_selection=True,export_format='GLB',
                                  export_yup=True,export_normals=True, export_vertex_color='ACTIVE',
                                  export_animations=False)
        base.glb_unlit(path)
        if asset in {f"extension_{name}" for name in SITES}:
            compact_colors(path)
    coords={"amber":(-249.,-12.),"heights":(248.,-21.),"reef":(25.,230.)}
    for name,(x,z) in coords.items():
        assets={'amber':['npc_ivo','farm_rabbit'],'heights':['npc_sanna','farm_goat'],'reef':['npc_tilda','farm_duck']}[name]
        coll=objects['extension_'+name][0]
        for i,asset in enumerate(assets):
            xx,zz=x-1.2*i,z+.79*i
            position=Vector(base.point(xx,zz,Ground(name).sample(xx,zz)[0]))
            for obj in objects[asset][1]:
                copy=obj.copy()
                copy.location=position+obj.location
                coll.objects.link(copy)
    for asset,(coll,group) in objects.items():
        if asset not in ('extension_amber','extension_heights','extension_reef'):
            for obj in group:
                obj.hide_set(True)
    bpy.context.scene.name='Waldwinkel · Die äußeren Wege'
    base.setup_preview(bpy.context.scene,27.,(-250.,-12.,1.3))
    for screen in bpy.data.screens:
        for area in screen.areas:
            if area.type=='VIEW_3D':
                area.spaces.active.shading.color_type='VERTEX'
                area.spaces.active.region_3d.view_perspective='CAMERA'
    bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'art/Waldwinkel-Aussenbezirke.blend'),compress=True)


if __name__=='__main__':
    arguments=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else []
    if '--bake' in arguments:
        for name in SITES:
            Ground(name).bake()
    bake_roofs()
    export()
