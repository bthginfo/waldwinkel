"""Model the Waldwinkel diorama and export the runtime assets from Blender.

Run from the project directory: blender -b -t 4 -P art/build_scene.py
Coordinates in this file are (game x, game z, height), with +z facing the player.
"""
import json
import math
import random
import struct
import sys
from pathlib import Path
from types import SimpleNamespace

import bpy
from mathutils import Vector
from mathutils.bvhtree import BVHTree

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "art"))
import landscape
import frontier
import route_details
OUT = ROOT / "assets" / "models"
RNG = random.Random(406)
TAU = math.tau
SUN = Vector((-0.51, -0.37, 0.77)).normalized()
AMBIENT_RAYS = [
    Vector(tuple(r)).normalized()
    for r in [(-.85, -.30, .45), (.77, -.43, .42), (.20, .83, .51),
              (-.35, .55, .70), (.19, -.22, .95)]
]
SOFT_SUN = [SUN, (SUN + Vector((.11, .06, .05))).normalized(),
            (SUN + Vector((-.06, -.13, .02))).normalized()]


def rgb(color):
    return tuple(int(color[i:i+2], 16) / 255 for i in range(0, 6, 2))


def linear(value):
    value = min(1, max(0, value))
    return value / 12.92 if value < .04045 else ((value + .055) / 1.055) ** 2.4


def blend(a, b, amount):
    if isinstance(a, str):
        a = rgb(a)
    if isinstance(b, str):
        b = rgb(b)
    return tuple(x + (y - x) * amount for x, y in zip(a, b))


def point(x, z, h):
    return (x, -z, h)


class Model:
    def __init__(self, name, asset="island", casts=True):
        self.name = name
        self.asset = asset
        self.casts = casts
        self.vertices = []
        self.faces = []
        self.colors = []

    def obstacle_circle(self,x,z,r):
        ox,oz=landscape.ORIGINS.get(self.asset,(0,0))
        landscape.block_circle(ox+x,oz+z,r)

    def obstacle_rect(self,x,z,w,d):
        ox,oz=landscape.ORIGINS.get(self.asset,(0,0))
        landscape.block_rect(ox+x,oz+z,w,d)

    def obstacle_rail(self,x,z,xx,zz,r=.075):
        ox,oz=landscape.ORIGINS.get(self.asset,(0,0))
        landscape.block_segment(ox+x,oz+z,ox+xx,oz+zz,r)

    def add(self, verts, faces, colors, mode="lit"):
        offset = len(self.vertices)
        self.vertices.extend(point(*v) for v in verts)
        for face, color in zip(faces, colors):
            self.faces.append(tuple(offset + i for i in reversed(face)))
            self.colors.append((rgb(color) if isinstance(color, str) else color, mode))

    def face(self, verts, color, mode="lit"):
        self.add(verts, [range(len(verts))], [color], mode)

    def gradient(self, verts, colors):
        self.face(verts, "ffffff", "gradient")
        self.colors[-1] = (list(reversed(colors)), "gradient")

    def rings(self, rings, colors, mode="lit", cap=True):
        count = len(rings[0])
        faces = []
        for ring in range(len(rings) - 1):
            for i in range(count):
                j = (i + 1) % count
                a, b = ring * count, (ring + 1) * count
                faces.append((a+i, a+j, b+j, b+i))
        if cap:
            faces.extend([tuple(reversed(range(count))), tuple(range(len(rings)*count-count, len(rings)*count))])
        if isinstance(colors, str) or (isinstance(colors, tuple) and isinstance(colors[0], float)):
            colors = [colors] * len(faces)
        self.add([v for ring in rings for v in ring], faces, colors, mode)

    def box(self, x, z, h, sx, sz, sy, color, bevel=.035, angle=0, mode="lit"):
        b = min(bevel, sx*.21, sz*.21, sy*.21)
        dx, dz, dy = sx*.5, sz*.5, sy*.5
        def corners(inset, height):
            ex, ez = dx - inset, dz - inset
            clip = min(b*.82, ex*.49, ez*.49)
            verts = [(-ex+clip,-ez),(ex-clip,-ez),(ex,-ez+clip),(ex,ez-clip),
                     (ex-clip,ez),(-ex+clip,ez),(-ex,ez-clip),(-ex,-ez+clip)]
            ca, sa = math.cos(angle), math.sin(angle)
            return [(x+u*ca-v*sa, z+u*sa+v*ca, height) for u,v in verts]
        self.rings([
            corners(b, h-dy), corners(0, h-dy+b), corners(0, h+dy-b), corners(b, h+dy)
        ], color, mode)

    def ellipsoid(self, x, z, h, sx, sz, sy, color, sides=9, levels=4,
                  angle=0, rough=0, mode="lit"):
        shape = [1 + RNG.uniform(-rough,rough) for _ in range(sides)] if rough else [1]*sides
        rings = []
        for j in range(1, levels):
            lat = -math.pi*.5 + math.pi*j/levels
            rings.append([
                (x+sx*math.cos(t)*math.cos(lat)*m,
                 z+sz*math.sin(t)*math.cos(lat)*m,
                 h+sy*math.sin(lat)*(1+(m-1)*.8))
                for i,m in enumerate(shape) for t in [TAU*i/sides+angle+(j%2)*.035]
            ])
        verts = [v for ring in rings for v in ring] + [(x,z,h-sy),(x,z,h+sy)]
        faces = []
        for j in range(len(rings)-1):
            for i in range(sides):
                a = j*sides+i
                b = j*sides+(i+1)%sides
                faces.extend([(a,b,b+sides), (a,b+sides,a+sides)])
        for i in range(sides):
            j = (i+1)%sides
            faces.extend([(len(verts)-2,j,i),(len(verts)-1,len(verts)-2-sides+i,len(verts)-2-sides+j)])
        shades = [blend(color, "827560", RNG.uniform(0,.10)) for _ in faces] if rough else [color]*len(faces)
        self.add(verts, faces, shades, mode)

    def pole(self, a, b, radius, color, sides=7, tip=None, mode="lit"):
        start, end = Vector(a), Vector(b)
        axis = (end-start).normalized()
        temp = Vector((0,1,0)) if abs(axis.y) < .9 else Vector((1,0,0))
        e1 = temp.cross(axis).normalized()
        e2 = axis.cross(e1)
        rings = []
        for center, r in [(start,radius),(end,radius if tip is None else tip)]:
            rings.append([tuple(center+r*(e1*math.cos(t)+e2*math.sin(t)))
                          for i in range(sides) for t in [TAU*i/sides]])
        self.rings(rings, color, mode)


def outline(angle):
    return 1 + .025*math.sin(angle*3+.45) + .023*math.cos(angle*5-1.6) + .012*math.sin(angle*9+1.1)


def rim_height(fraction, angle):
    return -.18 * max(0,(fraction-.84)/.16) + .05*math.sin(4*angle+1)*max(0,(fraction-.80)/.20)


def window(m, x, z, h, w, v, side=False):
    # Ordinary window coordinates are local horizontal depth, frontage and height.
    def b(u,d,y,a,c,k,tint,mode="lit"):
        if side:
            m.box(x+d,z+u,h+y,c,a,k,tint,.023,mode=mode)
        else:
            m.box(x+u,z+d,h+y,a,c,k,tint,.023,mode=mode)
    d = -.06 if side else .06
    b(0,d,0,w+.14,.16,v+.17,"77513b")
    b(0,d*2.25,0,w,.04,v,"edbb72","light")
    for tx in [-1,1]:
        for ty in [-1,1]:
            b(tx*w*.243,d*2.62,ty*v*.249,w*.43,.028,v*.419,
              "ffe4a0" if tx+ty!=2 else "f5ca81","light")
    b(0,d*2.87,0,.048,.048,v+.03,"a37548")
    b(0,d*2.87,0,w+.035,.049,.045,"a37548")
    for dx in [-1,1]:
        b(dx*(w+.049)/2,d*2.64,0,.09,.105,v+.12,"bf8b53")
    b(0,d*2.55,v*.5+.03,w+.19,.13,.104,"bc8752")
    b(0,d*2.90,-v*.5-.075,w+.25,.23,.12,"a7754c")


def cabin():
    wood = Model("Hütte · Wände, Fenster und Tür")
    roof = Model("Hütte · Dach und Kamin")
    deck = Model("Hütte · Veranda und Verandastufen")
    decor = Model("Hütte · Laterne, Pflanzkasten und Fässer")
    cx,cz=-2.65,-2.28
    w,d=4.02,3.08
    x0,x1,z0,z1=cx-w/2,cx+w/2,cz-d/2,cz+d/2
    sill,top,peak=.37,2.77,4.57
    # Discrete timber courses with skewed joins, capped by exposed posts.
    wood_colors=["c68e60","cb9562","c98a59","d19b67","c8915e","cc975f","be8256"]
    for j in range(10):
        h=sill+.118+j*.231
        for z in [z0,z1]:
            split=cx+RNG.uniform(-.62,.62)
            wood.box((x0+split)*.5,z,h,split-x0-.012,.14,.224,wood_colors[(j+(1 if z==z0 else 0))%7],.024)
            wood.box((split+x1)*.5,z,h,x1-split-.012,.14,.224,wood_colors[(j+3)%7],.020)
        for x in [x0,x1]:
            wood.box(x,cz,h,.14,d,.224,wood_colors[(j+2)%7],.024)
    for x in [x0,x1]:
        for z in [z0,z1]:
            wood.box(x,z,.23,.38,.37,.44,"8a8180",.07)
            wood.box(x,z,sill+1.16,.20,.20,2.41,"8e5f3f",.022)
        wood.box(x,cz,top,.21,d+.20,.17,"956640")
    for z in [z0,z1]:
        wood.box(cx,z,top,w+.20,.21,.18,"9d6b45")
        for j in range(7):
            y=top+.12+j*.238
            span=w*(1-(y-top)/(peak-top))
            wood.box(cx,z,y,span,.15,.228,wood_colors[(j+3)%7],.020)
        zf=z+(.10 if z==z1 else -.10)
        wood.pole((x0,zf,top),(cx,zf,peak-.07),.089,"a8774e",4)
        wood.pole((x1,zf,top),(cx,zf,peak-.07),.089,"a8774e",4)
        wood.box(cx,zf,3.57,.14,.14,1.57,"997049")
    window(wood,cx-.97,z1+.03,1.64,.76,.89)
    window(wood,cx,z1+.03,3.46,.59,.60)
    window(wood,x0-.025,cz-.64,1.74,.85,.80,True)
    window(wood,x0-.025,cz+.55,1.74,.85,.80,True)
    window(wood,x1+.24,cz+.43,1.68,.75,.85,True)

    x=cx+.57
    wood.box(x,z1+.087,1.40,1.24,.14,1.92,"79573f")
    for j in range(6):
        tint=["406f68","4d8172","48776a","578574","4b7b6f","457569"][j]
        wood.box(x+(.155*(j-2.5)),z1+.178,1.37,.153,.069,1.71,tint,.015)
    for dx in [-.53,.53]:
        wood.box(x+dx,z1+.204,1.4,.13,.16,1.98,"b07b51")
    wood.box(x,z1+.204,2.37,1.29,.18,.166,"bc8755")
    wood.box(x,z1+.215,1.05,.93,.057,.10,"537563")
    wood.box(x,z1+.24,.49,1.02,.24,.15,"926744")
    wood.ellipsoid(x+.32,z1+.25,1.27,.062,.041,.062,"daad55",8,3)
    wood.ellipsoid(x+.32,z1+.28,1.27,.032,.030,.032,"e4bd69",8,3)

    eave=top-.20
    left,right=x0-.31,x1+.31
    front,back=z1+.43,z0-.36
    for edge in [left,right]:
        near=(cx,peak); far=(edge,eave)
        def sample(t):
            return (near[0]+(far[0]-near[0])*t,near[1]+(far[1]-near[1])*t)
        roof.face([(near[0],back,near[1]-.065),(far[0],back,far[1]-.065),
                   (far[0],front,far[1]-.065),(near[0],front,near[1]-.065)][::(1 if edge>cx else -1)],"af694d")
        n=15
        for j in range(n):
            z=back+(front-back)*(j+.5)/n
            offset=RNG.uniform(-.014,.014)
            tint=["d9925f","db9766","cc8056","d58b5d","e2a372","d6895f"][j%6]
            for k in range(4):
                a,b=sample(k*.249+offset),sample((k+1)*.252+offset)
                radius=.138 + RNG.uniform(-.012,.012)
                roof.pole((a[0],z,a[1]+.032),(b[0],z,b[1]+.006),radius,tint,8,tip=radius+.012)
                roof.pole((b[0],z,b[1]+.006),(b[0]+(.040 if edge>cx else -.04),z,b[1]-.025),radius+.012,tint,8,tip=radius*.94)
        for z in [back-.04,front+.05]:
            roof.pole((edge,z,eave-.01),(cx,z,peak+.018),.125,"b68254",4)
        roof.pole((edge,back,eave-.04),(edge,front,eave-.04),.106,"b27a4d",4)
    for j in range(9):
        step=(front-back)/9
        roof.pole((cx,back+j*step-.085,peak+.123),(cx,back+(j+1)*step-.07,peak+.123),.16,
                  ["d39163","e6a777","d99c6d"][j%3],10)
    # The dark chimney opening is bordered by four real stones.
    x,z=cx-1.28,cz-.74
    for i in range(8):
        y=3.05+i*.258
        for k in [-1,1]:
            shift=.027*(i%2)
            roof.box(x+k*.17+shift,z,y,.334,.555,.251,
                     ["8f8c80","a49a8b","aba192","9b9287"][(i+k)%4],.035)
    for k in [-1,1]:
        roof.box(x+k*.258,z,5.15,.19,.74,.205,"b0a898",.040)
        roof.box(x,z+k*.27,5.15,.44,.17,.205,"b6ac9b",.040)
    roof.box(x,z,5.047,.43,.42,.030,"413b33",.005)

    px0,px1=x0-.45,x1+.45
    pz0,pz1=z1-.28,z1+1.37
    for i in range(20):
        x=px0+(px1-px0)*(i+.5)/20
        deck.box(x,(pz0+pz1)/2,.393,(px1-px0)/20-.016,pz1-pz0,.16,
                 ["bd8f60","d5a774","ca9a66","c59865","d2a677"][i%5],.021)
    for z in [pz0+.09,pz1-.09]:
        deck.box(cx,z,.26,px1-px0+.05,.15,.21,"926142")
        for x in [px0+.14,cx-.69,cx+.81,px1-.14]:
            deck.box(x,z,.13,.18,.19,.25,"936544")
    step_center=cx+.57
    for j in range(3):
        deck.box(step_center,pz1+.16+j*.20,.30-j*.107,1.53-.06*j,.40,.122,"c89a67",.034)
    for x in [px0+.12,px1-.13,step_center-.91,step_center+.91]:
        deck.box(x,pz1-.08,.74,.148,.148,.71,"a8764d",.024)
        deck.box(x,pz1-.08,1.10,.174,.174,.05,"cfaa71",.018)
    for z in [pz0+.08,(pz0+pz1)/2,pz1-.08]:
        for x in [px0+.12,px1-.13]:
            deck.box(x,z,.75,.144,.15,.69,"a8764d",.024)
    for y in [.71,1.035]:
        for a,b in [(px0+.12,step_center-.91),(step_center+.91,px1-.13)]:
            deck.box((a+b)/2,pz1-.08,y,b-a,.095,.096,"bc8958",.015)
        for x in [px0+.12,px1-.13]:
            deck.box(x,(pz0+pz1)/2,y,.093,pz1-pz0-.18,.097,"b68657",.016)
    # Flower box, barrel, stacked kindling and a warm lantern.
    x, z = cx-1.25, pz1-.29
    decor.box(x,z,.60,.88,.45,.26,"9f714a",.035)
    decor.box(x,z,.748,.78,.37,.042,"625235",.012)
    for i in range(9):
        flowers(decor,x+RNG.uniform(-.36,.36),z+RNG.uniform(-.145,.145),.77,
                RNG.uniform(.62,.86),["fff1cc","e7aa6e","fff6d9"][i%3])
    x,z=cx+1.77,pz1-.30
    barrel(decor,x,z,.48,.98)
    barrel(decor,x1+.29,z1-.35,0,.79)
    x,z=cx+1.54,z1+.28
    decor.box(x,z,2.46,.10,.47,.11,"555342",.013)
    decor.pole((x,z+.18,2.48),(x,z+.18,2.20),.029,"514b3a",6)
    decor.box(x,z+.18,2.025,.23,.22,.35,"ffc76a",.025,mode="light")
    for i in [-1,1]:
        for j in [-1,1]:
            decor.box(x+i*.107,z+.18+j*.103,2.02,.025,.025,.35,"675539",.006)
    decor.box(x,z+.18,2.215,.32,.32,.091,"695840",.031)
    decor.box(x,z+.18,1.84,.27,.26,.071,"786247",.023)
    for i in range(5):
        for j in range(3-i//2):
            x,z=x0-.12+i*.13,z0-.22
            decor.pole((x,z-.24,.18+.15*j),(x,z+.20,.18+.15*j),.084,"744f35",6)
            decor.pole((x,z+.20,.18+.15*j),(x,z+.231,.18+.15*j),.071,"d5ac78",7)
    return [wood,roof,deck,decor]


def barrel(model,x,z,y,s):
    model.obstacle_circle(x,z,.276*s)
    rings = []
    for h,r in [(0,.21),(.13,.253),(.35,.275),(.59,.236)]:
        rings.append([(x+r*s*math.cos(t),z+r*s*math.sin(t),y+h*s)
                      for i in range(11) for t in [i*TAU/11]])
    colors = ["986a42","a8764a","af8051","9b7149"]*9
    model.rings(rings,colors)
    for height,r in [(.14,.263),(.46,.266),(.57,.247)]:
        model.pole((x,z,y+height*s),(x,z,y+(height+.041)*s),r*s,"6f6957",11)
    model.pole((x,z,y+.603*s),(x,z,y+.617*s),.212*s,"ae8356",11)


def grass(m,x,z,y,s,shade="7f9f4b",blades=5):
    baseangle = RNG.random()*TAU
    for i in range(blades):
        a=baseangle+TAU*i/blades+RNG.uniform(-.52,.52)
        dx,dz=math.cos(a)*s, math.sin(a)*s
        height=RNG.uniform(.59,1.24)*s
        bend=RNG.uniform(.32,.79)
        width=RNG.uniform(.10,.25)
        base=(x+dx*.16,z+dz*.16,y)
        le=(base[0]-dz*width,base[1]+dx*width,y)
        ri=(base[0]+dz*width,base[1]-dx*width,y)
        middle=(x+dx*.35,z+dz*.35,y+height*.66)
        apex=(x+dx*bend,z+dz*bend,y+height)
        color=blend(shade,"c4c063",RNG.uniform(.0,.35))
        m.face([le,ri,middle],color,"leaf")
        m.face([le,middle,apex],color,"leaf")
        m.face([middle,ri,apex],blend(color,"d9cf73",.18),"leaf")


def flowers(m,x,z,y,s,color):
    height=RNG.uniform(.22,.34)*s
    dx=RNG.uniform(-.08,.085)*s
    grass(m,x,z,y,.16*s,"719447",3)
    m.pole((x,z,y),(x+dx,z,y+height),.018*s,"809b4d",5,tip=.009*s)
    for i in range(5):
        a=i*TAU/5
        m.ellipsoid(x+dx+math.cos(a)*.051*s,z+math.sin(a)*.051*s,y+height,
                    .044*s,.036*s,.023*s,color,5,3,mode="leaf")
    m.ellipsoid(x+dx,z,y+height+.012*s,.031*s,.031*s,.025*s,"ecb657",6,3,mode="light")


def pine(m,x,z,h,s,index):
    phase=RNG.uniform(0,TAU)
    r=.32*s
    if m.asset != "wood_tree":
        m.obstacle_circle(x,z,r)
    m.pole((x,z,h-.05),(x+.07*s,z-.07*s,h+2.77*s),r,"7d593e",8,tip=.14*s)
    for i in range(5):
        a=i*TAU/5+phase
        m.pole((x+math.cos(a)*.48*s,z+math.sin(a)*.39*s,h),
               (x,z,h+.59*s),.075*s,"856044",5,tip=.155*s)
    foliage = ["7a914e","6d895b","8ca054","5c8258","879951","748d4d"][index%6]
    for j,(base,rad,high) in enumerate([(1.1,1.31,2.22),(2.10,1.04,2.00),(2.98,.78,1.68),(3.70,.51,1.18)]):
        offset=phase+j*.34
        leanx=s*.19*math.sin(phase+j*1.3)
        leanz=s*.16*math.sin(phase+j)
        shape=[RNG.uniform(.82,1.13) for _ in range(9)]
        rings=[]
        for layer,(height,radius) in enumerate([(base+.035,rad*.83),(base+.12,rad),(base+high*.31,rad*.69),(base+high,rad*.025)]):
            rings.append([(x+leanx+s*radius*m*math.cos(a),z+leanz+s*radius*m*math.sin(a),
                           h+s*(height+(m-1)*(1 if layer<2 else .15)))
                          for i,m in enumerate(shape) for a in [TAU*i/9+offset]])
        tints=[blend(foliage,"b3b369",.10+j*.11+RNG.uniform(-.045,.098)) for _ in range(29)]
        m.rings(rings,tints)


def woods():
    forest=Model("Nadelwald · Kiefern und Tannen")
    stone=Model("Geröll und Uferfelsen")
    greens=Model("Unterholz · Gräser, Farne und Wildblumen",casts=False)
    trees=[
        (-6.39,-2.23,.97),(-5.45,-3.92,.81),(-3.46,-4.92,1.09),(-1.32,-5.22,1.08),
        (.43,-4.89,.96),(2.02,-4.35,1.07),(3.86,-3.51,.94),(5.25,-2.49,1.00),
        (6.48,-1.48,.77),(6.72,1.74,.58),(-6.74,-.36,.58)
    ]
    for i,(x,z,s) in enumerate(trees):
        pine(forest,x,z,0,s,i)
    boulders=[(-5.36,.06,.65),(-6.88,-1.06,.53),(-5.27,-3.42,.67),(1.84,-3.71,.72),
              (.16,-3.04,.52),(4.72,-2.78,.52),(5.8,-.47,.59),(-2.90,4.69,.31),
              (7.07,1.87,.40),(5.95,2.91,.37),(2.48,4.61,.34),(-4.3,4.19,.39)]
    for i,(x,z,s) in enumerate(boulders):
        stone.obstacle_circle(x,z,s*.86)
        tint=["969084","8c8a84","a29688","a0998e"][i%4]
        stone.ellipsoid(x,z,s*.21,s,s*.78,s*.64,tint,8,4,angle=RNG.random(),rough=.19)
        stone.ellipsoid(x+.86*s,z+.54*s,.13,s*.43,s*.35,s*.33,tint,7,4,rough=.15)
    for i,a in enumerate([.23,.60,.82,1.10,1.44,1.94,2.17,2.30,2.55,2.89,3.19,3.99,5.34,5.74]):
        x,z=25.8*math.cos(a)*outline(a),18.9*math.sin(a)*outline(a)
        s=RNG.uniform(.39,1.00)
        stone.ellipsoid(x,z,-1.46+s*.31,s,s*.73,s*.76,["9b9186","988c84","b1a18e"][i%3],
                        8,4,angle=a+.4,rough=.23)
        stone.ellipsoid(x+.67*s,z+.66*s,-1.46,.45*s,.39*s,.44*s,"b2a790",7,4,rough=.16)
    for attempt in range(520):
        x,z=RNG.uniform(-8,8),RNG.uniform(-6,6)
        if (x/8.05)**2+(z/6.02)**2>.975:
            continue
        if -5.23<x<-.21 and -4.06<z<1.27 or .35<x<5.7 and -1.01<z<3.70:
            continue
        if ((x+4.77)/1.73)**2+((z-2.54)/1.39)**2<(1.07**2) or -.89<x<.85 and .18<z<4.8:
            continue
        f=math.hypot(x/8.35,z/6.28)/outline(math.atan2(z/6.28,x/8.35))
        h=rim_height(f,math.atan2(z/6.28,x/8.35))+.007
        scale=RNG.uniform(.11,.28)
        grass(greens,x,z,h,scale,["8c9f4c","a4af55","789144"][attempt%3],3+attempt%3)
        if attempt%9==0:
            flowers(greens,x+.09,z+.08,h,scale*3.7,["fff0d3","eaae70","e9c55e"][attempt%3])
    for i,(x,z) in enumerate([(-4.62,4.25),(-2.27,3.92),(1.78,4.02),(4.22,4.32),(5.95,3.15),
                               (-6.11,1.07),(.15,-.5),(4.56,-.98),(-4.7,-.45)]):
        for j in range(6):
            flowers(greens,x+RNG.uniform(-.36,.36),z+RNG.uniform(-.24,.24),0,RNG.uniform(.52,1.1),
                    ["fff4dd","f8d786","e79166"][i%3])
    for x,z in [(-6.7,1.28),(-2.98,4.1),(6.45,.01),(5.75,3.7),(.64,-2.68)]:
        for j in range(7):
            a=TAU*j/7
            dx,dz=math.cos(a),math.sin(a)
            greens.face([(x,z,.06),(x+dx*.26-dz*.15,z+dz*.26+dx*.15,.24),
                         (x+dx*.52,z+dz*.52,.30),(x+dx*.27+dz*.13,z+dz*.27-dx*.13,.18)],
                        ["d98a52","cf7948","e0a46a"][j%3],"leaf")
    return [forest,stone,greens]


def grounds():
    trail=Model("Schieferplatten und Trampelpfad")
    fences=Model("Gartenzaun, Kisten und Gartenschild")
    pond=Model("Teich · Wasser, Schilf und Seerosen")
    for j in range(11):
        z=.77+j*.426
        x=-2.12+(2.76*(1-math.exp(-j*.123)))+.18*math.sin(j*.44)
        center=(x,z,.020)
        radius=RNG.uniform(.57,.72)
        shade=blend("c0af74","b5ab65",RNG.uniform(0,.52))
        verts=[(x+math.cos(t)*radius,z+math.sin(t)*radius*.79,.009+j*.00083)
               for i in range(12) for t in [i*TAU/12]]
        trail.face(verts,shade,"ground")
        for k in [-1,1]:
            a=RNG.uniform(-.38,.42)
            tint=["c3b7a0","dacbb0","cbbfa7","ccba9c","e2cfad"][(j+k)%5]
            trail.box(x+k*.234,z+RNG.uniform(-.08,.08),.045,RNG.uniform(.33,.50),
                      RNG.uniform(.31,.43),.075,tint,.049,a)
    for j,(x,z) in enumerate([(.17,2.40),(.66,2.36),(1.05,2.25),(.43,.02),(.96,-.20),(-3.69,1.30)]):
        trail.box(x,z,.038,.32+.09*(j%2),.32,.076,"cbbb9d",.046,.14*j)
    # Two sides of a short fence keep the front and left of the kitchen garden open.
    path=[(.91,-.36),(2.07,-.53),(3.37,-.53),(4.73,-.49),(5.84,-.25),(5.96,.73),(5.96,1.87)]
    for x,z in path:
        fences.box(x,z,.43,.175,.181,.85,"9e754e",.028,RNG.uniform(-.16,.16))
        fences.box(x,z,.85,.202,.207,.064,"c39b67",.019)
    for (x,z),(nx,nz) in zip(path,path[1:]):
        fences.obstacle_rail(x,z,nx,nz)
        angle=math.atan2(nz-z,nx-x)
        length=math.hypot(nx-x,nz-z)+.025
        for h,tint in [(.31,"a68256"),(.64,"bb925e")]:
            fences.box((nx+x)/2,(nz+z)/2,h,length,.095,.14,tint,.022,angle)
    for j in range(2):
        x,z=5.55-.31*j,-.70-.34*j
        fences.obstacle_rect(x,z,.51,.49)
        fences.box(x,z,.18,.51,.49,.35,"a8784e",.033,angle=.18+j*.13)
        for k in range(3):
            fences.box(x+(k-1)*.16,z,.358,.151,.49,.037,"c09263",.011,.12)
    fences.box(.44,-.35,.36,.084,.101,.71,"917048",.015)
    fences.box(.44,-.34,.71,.58,.10,.31,"b98e59",.028,-.08)
    fences.face([(.28,-.275,.705),(.40,-.267,.79),(.55,-.27,.79),(.63,-.278,.704),
                 (.55,-.274,.63),(.40,-.273,.63)][::-1],"e9c084","light")
    x,z=-4.77,2.54
    rings=[]
    detail=26
    for layer,(scale,height) in enumerate([(1.05,.014),(1,.040),(.909,.041),(.833,.044)]):
        ring=[]
        for j in range(detail):
            a=j*TAU/detail
            shape=1+.055*math.sin(a*3+.61)+.041*math.cos(a*5)
            ring.append((x+math.cos(a)*1.51*shape*scale,z+math.sin(a)*1.04*shape*scale,height))
        rings.append(ring)
    for layer,palette in enumerate([("8d9860","b3a468"),("726d44","948455"),("589b85","7ab09a")]):
        for j in range(detail):
            k=(j+1)%detail
            pond.face([rings[layer][j],rings[layer][k],rings[layer+1][k],rings[layer+1][j]],blend(*palette,RNG.random()),"ground")
    for j in range(detail):
        pond.face([(x,z,.046),rings[3][j],rings[3][(j+1)%detail]],blend("4f9d98","6dbbb1", .47+.22*math.sin(j*.46)),"raw")
    for i,(dx,dz,s) in enumerate([(-1.35,.31,.28),(-.87,.83,.26),(.22,1.01,.24),(.97,.66,.31),
                                  (1.30,-.16,.29),(.74,-.83,.22),(-.65,-.92,.24),(-1.08,-.55,.19)]):
        pond.ellipsoid(x+dx,z+dz,s*.16,s,s*.74,s*.48,["a29c8d","8f8a81","a39889"][i%3],7,4,rough=.19)
    for dx,dz in [(-1.15,-.43),(1.19,.54),(-.63,.87),(1.08,-.57)]:
        grass(pond,x+dx,z+dz,.023,.39,"8a9b4f",8)
        for i in range(3):
            xx,zz=x+dx+RNG.uniform(-.13,.13),z+dz+RNG.uniform(-.10,.10)
            height=RNG.uniform(.39,.77)
            pond.pole((xx,zz,.04),(xx+.08,zz+.02,height),.022,"7a9252",5)
            pond.pole((xx+.065,zz+.02,height-.11),(xx+.081,zz+.025,height+.047),.039,"9a6b42",7,tip=.031)
    for dx,dz,r in [(-.65,.24,.19),(-.17,.14,.26),(.47,-.25,.17),(.59,-.36,.13)]:
        xx,zz=x+dx,z+dz
        a0=RNG.uniform(0,TAU)
        verts=[(xx+math.cos(a)*r,zz+math.sin(a)*r,.07)
               for i in range(10) for a in [a0+.39+i*(TAU-.63)/9]]
        verts.append((xx,zz,.071))
        pond.face(verts,"aab85a","raw")
        if r in [.26,.17]:
            for i in range(5):
                a=TAU*i/5
                pond.ellipsoid(xx+.052*math.cos(a),zz+.052*math.sin(a),.106,
                                .048,.032,.027,"ece4be",6,3)
            pond.ellipsoid(xx,zz,.13,.036,.036,.023,"ddb760",6,3)
    for dx,dz,l in [(-.40,-.56,.33),(.34,.46,.25),(.81,.21,.21),(-.25,-.38,.20)]:
        xx,zz=x+dx,z+dz
        pond.face([(xx-l,zz,.053),(xx,zz-.02,.053),(xx+l,zz,.053),(xx,zz+.016,.053)],"98d1c4","raw")
    return [trail,fences,pond]


def garden():
    frame=Model("Beet · Bretter und Eckpflöcke","bed")
    lawn=Model("Beet · Wiese","lawn")
    soil=Model("Beet · Erde und Saatreihen","soil")
    for p in [-.617,.617]:
        frame.box(p,0,.074,.086,1.310,.142,"a37d50",.024)
        frame.box(0,p,.076,1.31,.085,.148,"bc9561",.023)
        for q in [-.617,.617]:
            frame.box(p,q,.124,.136,.141,.232,"c29c6a",.020)
            frame.ellipsoid(p,q,.244,.022,.024,.010,"746044",6,3)
    lawn.box(0,0,.034,1.15,1.14,.067,"9eac55",.031)
    for i in range(11):
        x,z=RNG.uniform(-.49,.49),RNG.uniform(-.47,.47)
        grass(lawn,x,z,.067,RNG.uniform(.085,.168),"a2ac53",4)
        if i in [0,5]:
            flowers(lawn,x+.027,z,.069,.37,"faf1d7")
    soil.box(0,0,.041,1.15,1.14,.081,"76503b",.027)
    for j in range(3):
        z=(j-1)*.338
        for k in range(4):
            soil.ellipsoid((k-1.5)*.266,z,.071,.166,.13,.080,
                            ["967052","916549","845b42"][(j+k)%3],7,3,rough=.15)
    crops=[]
    for stage in range(3):
        m=Model(["Keimlinge","Jungpflanzen","Reife Möhren"][stage],["sprout","young","carrot"][stage])
        for j in range(3):
            for k in range(3):
                x,z=(k-1)*.351+RNG.uniform(-.027,.027),(j-1)*.342+RNG.uniform(-.031,.031)
                s=[.58,.88,1.16][stage]*RNG.uniform(.87,1.22)
                h=.132
                if stage==2:
                    m.ellipsoid(x,z,.177,.109*s,.094*s,.115*s,"e49046",9,4,rough=.055)
                    m.ellipsoid(x,z,.224,.082*s,.081*s,.061*s,"efab52",8,3)
                    h=.252
                for i in range(4 if stage else 2):
                    a=TAU*i/4+RNG.uniform(-.3,.3)+k
                    dx,dz=math.cos(a)*s,math.sin(a)*s
                    apex=(x+dx*.194,z+dz*.194,h+s*RNG.uniform(.214,.319))
                    shoulder=(x+dx*.104,z+dz*.104,h+s*.174)
                    left=(shoulder[0]-dz*.057,shoulder[1]+dx*.057,shoulder[2]-.021)
                    right=(shoulder[0]+dz*.061,shoulder[1]-dx*.061,shoulder[2]-.018)
                    foot=(x,z,h)
                    color=["94af4f","719749","acc159","83a747"][i]
                    m.face([foot,left,shoulder],color,"leaf")
                    m.face([left,apex,shoulder],color,"leaf")
                    m.face([shoulder,apex,right],blend(color,"c0c865",.26),"leaf")
                    m.face([shoulder,right,foot],blend(color,"a6b959",.19),"leaf")
        crops.append(m)
    wet=Model("Beet · frisch gegossene Erde","wet_soil")
    wet.vertices,wet.faces=soil.vertices[:],soil.faces[:]
    wet.colors=[(blend(color,"503c34",.45),mode) for color,mode in soil.colors]
    return [frame,lawn,soil,wet,*crops]


def character():
    # The limb assets have their pivots at the top of each limb.
    body=Model("Gärtnerfigur · Gesicht, Outfit und Strohhut","gardener")
    arm=Model("Gärtnerfigur · Arm und Hand","arm")
    leg=Model("Gärtnerfigur · Hosenbein","leg")
    boot=Model("Gärtnerfigur · Stiefel","boot")
    body.ellipsoid(0,0,.64,.243,.149,.271,"64866a",10,5)
    body.box(0,.145,.663,.306,.056,.27,"789776",.049)
    body.box(0,.183,.653,.135,.024,.087,"537965",.020)
    for x in [-.137,.137]:
        body.box(x,.115,.826,.062,.08,.195,"6e8d6c",.021)
        body.ellipsoid(x,.172,.768,.021,.015,.022,"e0b263",7,3)
    body.ellipsoid(0,0,.884,.205,.152,.095,"bd6450",10,4)
    body.pole((-.14,-.04,.900),(-.36,-.158,.775),.054,"b75a48",5,tip=.033)
    body.pole((-.155,-.022,.882),(-.343,-.126,.715),.047,"d4775d",5,tip=.022)
    body.ellipsoid(0,-.035,1.177,.298,.248,.281,"956344",13,6)
    body.ellipsoid(0,.075,1.151,.267,.206,.221,"c49667",13,6)
    for side in [-1,1]:
        x=side*.225
        body.ellipsoid(x,-.014,1.339,.088,.073,.10,"72513d",9,4)
        body.ellipsoid(x,.032,1.343,.053,.035,.060,"c98770",8,4)
        body.ellipsoid(side*.134,.228,1.162,.127,.070,.105,"5d4236",10,5,angle=side*.37)
        body.ellipsoid(side*.093,.292,1.207,.038,.019,.054,"2f2826",9,4,mode="raw")
        body.ellipsoid(side*.093-.008,.307,1.225,.011,.008,.013,"fff0d2",7,3,mode="raw")
        body.ellipsoid(side*.082,.249,1.061,.114,.094,.075,"edcb98",9,5)
    body.ellipsoid(0,.287,1.082,.111,.105,.081,"efd4a8",10,5)
    body.ellipsoid(0,.365,1.118,.044,.029,.028,"49342d",8,4)
    body.pole((-.045,.365,1.042),(0,.390,1.032),.006,"755143",5)
    body.pole((0,.390,1.032),(.045,.365,1.042),.006,"755143",5)
    tail=Model("Tanuki · buschiger Schweif","tail")
    tail.pole((0,0,0),(.09,-.20,.033),.085,"6c4a37",8,tip=.147)
    tail.ellipsoid(.17,-.34,.095,.161,.35,.167,"a6734d",11,5,angle=.40)
    tail.ellipsoid(.27,-.54,.122,.132,.182,.14,"73503d",10,4,angle=.40)
    hat=[]
    for y,r in [(1.371,.424),(1.413,.44),(1.428,.382),(1.453,.276),(1.593,.232),(1.619,.178)]:
        hat.append([(r*math.cos(a),.857*r*math.sin(a)-.020,y+.042*math.cos(a+.62))
                    for i in range(16) for a in [TAU*i/16]])
    body.rings(hat,["bb8f53"]*16+["e4bc76"]*16+["edc984"]*16+["d7ab63"]*16+["e7bc73"]*16+["d9ab65","f0cc84"])
    body.pole((0,-.02,1.455),(0,-.02,1.507),.276,"a87753",14,tip=.255)
    body.pole((0,-.02,1.463),(0,-.02,1.494),.274,"698675",16,tip=.264)
    body.ellipsoid(.123,.222,1.482,.051,.026,.045,"edc27d",8,4)
    for i in range(16):
        a=i*TAU/16
        x,z=.350*math.cos(a),.301*math.sin(a)-.020
        y=1.433+.042*math.cos(a+.62)
        body.pole((x,z,y),(x+.054*math.cos(a),z+.047*math.sin(a),y-.006),
                  .006,"c39960",4)
    body.box(0,-.167,.687,.282,.145,.31,"a97a53",.066)
    body.box(0,-.243,.751,.24,.046,.139,"c69764",.041)
    body.ellipsoid(0,-.273,.73,.03,.016,.025,"dda771",8,4)
    # Thin belt ribbon, then the hem of the sleeve and hands.
    arm.ellipsoid(0,.00,-.094,.085,.080,.135,"e3c79f",8,4)
    arm.ellipsoid(0,.012,-.226,.065,.062,.104,"795741",8,5)
    leg.ellipsoid(0,0,-.099,.097,.12,.153,"577963",8,4)
    boot.box(0,.043,-.045,.175,.237,.131,"76533f",.035)
    boot.box(0,.053,-.104,.187,.244,.050,"503f36",.014)
    hoe=Model("Gärtnerfigur · Gartenhacke","hoe")
    hoe.pole((0,0,.125),(0,0,-.433),.024,"ab784e",7,tip=.018)
    hoe.box(0,.027,-.416,.064,.09,.071,"787c6a",.012)
    hoe.box(0,.097,-.442,.193,.17,.055,"809084",.019)
    hoe.box(0,.166,-.448,.174,.031,.037,"a8b5a2",.008)
    return [body,arm,leg,boot,hoe,tail]


def expansion():
    paths=Model("Waldwege und Küstenwiese")
    forest=Model("Großer Wald und Blütenbäume")
    coast=Model("Sandbucht, Steg und Mineneingang")
    for points in [[(0,5),(0,8),(-5,12),(-12,11),(-17,6),(-16,-3),(-12,-6.75),(0,-6.88),(9,-10)],
                   [(0,5),(5,6),(10,5),(13,-1),(12,-6),(9,-10)],
                   [(5,6),(7,10),(6,15.8),(5.8,17.4)]]:
        for (x,z),(xx,zz) in zip(points,points[1:]):
            for i in range(int(math.hypot(xx-x,zz-z)*2.35)):
                t=i/max(1,int(math.hypot(xx-x,zz-z)*2.35))
                for shift in [-.28,.27]:
                    a=x+(xx-x)*t+shift+RNG.uniform(-.12,.12)
                    b=z+(zz-z)*t+RNG.uniform(-.14,.14)
                    paths.box(a,b,.026,RNG.uniform(.36,.52),RNG.uniform(.29,.46),.067,
                              RNG.choice(["d7c5a4","daceb0","bbac8b","cab58e"]),.052,RNG.uniform(-.45,.45))
    for x,z,s in [(-21,3,1),(-21,-4,.92),(-19,-8,1.12),(-16,-11,1.22),(-12,-13,.94),(-9,-15,1.14),
                   (-6,-15,.9),(-3,-16,1.17),(1,-16,.9),(5,-15,1.03),(8,-14,.95),(12,-13,1.22),
                   (15,-11,1.08),(18,-10,.84),(20,-6,1.15),(21.6,-2.9,.82),(23.0,-.4,.82),(15.7,-2.0,.89),
                   (16,9,.9),(14,11,.71),(11,14,.81),(-19.3,7.91,.88),(-16.4,12.4,.90),(-6.3,14.2,.80),(-9.4,14.3,.68),
                   (-13,4,.85),(-12,1,.85),(-10,-5,.72),(14,-5,.91),(18.5,-2.75,.83)]:
        pine(forest,x,z,0,s,RNG.randrange(6))
    for x,z,s in [(-7.76,4.42,1.0),(-6.42,9.9,.90),(5,9,.94),(11,9,.82),(15,-2,.91),(-5,-9,.98),
                   (3,-10,.76),(-17,-5,.9),(18,-4.3,1),(-6.9,12.2,.72)]:
        forest.obstacle_circle(x,z,.24*s)
        forest.pole((x,z,0),(x+.1,z,2.18*s),.24*s,"856445",8,tip=.13*s)
        for dx,dz,h,sc in [(-.48,.05,2.23,.72),(.42,-.21,2.55,1),(.03,.6,2.39,.72),(0,0,3.0,.76)]:
            forest.ellipsoid(x+dx*s,z+dz*s,h*s,sc*s,sc*.85*s,sc*.75*s,
                             RNG.choice(["c58e83","dba697","b9a461","a8a657","d4bb70"]),9,4,rough=.16)
    flowers_model=Model("Blumen am Waldrand", casts=False)
    for _ in range(420):
        x,z=RNG.uniform(-24,24),RNG.uniform(-17,17)
        if (x/24)**2+(z/17)**2>.90 or abs(x)<9 and abs(z)<6 or (x-6)**2+(z-16)**2<12:
            continue
        grass(flowers_model,x,z,0,RNG.uniform(.16,.37),blades=4)
        if RNG.random()<.32:
            flowers(flowers_model,x+.06,z,0,RNG.uniform(.8,1.3),RNG.choice(["fdf1d7","e3aa70","eac0aa"]))
    for x,z in [(-14,-4),(-12,8),(9,-7),(15,3),(14,7),(-10,-11),(-17,3),(7,13)]:
        for _ in range(4):
            xx,zz=x+RNG.uniform(-1.4,1.4),z+RNG.uniform(-1.4,1.4)
            s=RNG.uniform(.20,.49)
            if s>.34:
                coast.obstacle_circle(xx,zz,s*.80)
            coast.ellipsoid(xx,zz,s*.1,s,s*.8,s*.6,"aaa398",8,4,rough=.18)
    for i in range(14):
        z=16.0+i*.27
        y=.10-max(0,i-5)*.043
        coast.box(5.8,z,y,1.92,.255,.14,["c0925d","bd8d5b","d5a66b"][i%3],.022)
        if i%4==0:
            for x in [4.94,6.66]:
                coast.pole((x,z,-1.73),(x,z,y+.66),.12,"886245",8)
    for x,z in [(4.14,16.0),(7.22,15.12),(3.86,15.82),(7.78,15.72)]:
        grass(coast,x,z,.006,.67,"809352",9)
    for x,z in [(3.73,16.85),(4.5,17.35),(7.40,16.72),(8,16.16)]:
        coast.ellipsoid(x,z,-.09,.67,.51,.52,"b8a58e",8,4,rough=.12)
    barrel(coast,6.76,16.21,0,.86)
    for i in range(11):
        coast.ellipsoid(9+(i%5-2)*.69,-10.64-(i//5)*.61,.35+i%3*.38,.88,.80,.82,
                        ["897482","a38c94","8a7e88"][i%3],8,4,rough=.23)
    coast.box(9,-9.85,1.11,2.18,.31,2.18,"493847",.115)
    for x in [7.90,10.1]:
        coast.obstacle_rect(x,-9.65,.36,.45)
        coast.box(x,-9.65,1.14,.32,.40,2.34,"8d6246",.051)
    coast.obstacle_rect(9,-10.56,3.35,.91)
    coast.box(9,-9.64,2.33,2.65,.42,.40,"b38a57",.068)
    lantern(coast,7.84,-9.38,1.74)
    lantern(coast,10.15,-9.38,1.74)
    return [paths,forest,coast,flowers_model]


def lantern(m,x,z,y):
    m.box(x,z,y,.25,.22,.35,"f4ad54",.045,mode="light")
    for h in [-.18,.18]:
        m.box(x,z,y+h,.34,.31,.077,"78513e",.022)
    for dx in [-.117,.117]:
        for dz in [-.103,.103]:
            m.box(x+dx,z+dz,y,.026,.03,.33,"80533a",.005)


def furniture_models():
    house=Model("Dollhouse · Innenraum","interior")
    for i in range(21):
        x=(i-10)*.34
        for j in range(4):
            house.box(x,(j-1.5)*1.47,-.075,.331,1.462,.19,
                      ["c99268","d1a075","c58c61","d7a97d"][(i+j*3)%4],.027)
    house.box(0,0,-.28,7.3,6.14,.38,"9b664d",.10)
    for i in range(13):
        y=.10+i*.197
        house.box(0,-2.96,y,7.15,.18,.19,["ead0a1","e6c59b","eac99d"][i%3],.019)
        house.box(3.56,0,y,.18,5.9,.19,["d9b68e","e9c99c","e2bd91"][i%3],.019)
    for x in [-3.46,0,3.46]:
        house.box(x,-2.80,1.35,.19,.22,2.68,"916044",.038)
    house.box(0,-2.83,2.66,7.45,.30,.31,"9f6748",.049)
    house.box(3.56,0,2.65,.33,6.14,.31,"9f6748",.049)
    window(house,-1.89,-2.81,1.64,1.15,.91)
    window(house,1.85,-2.81,1.64,1.02,.91)
    for x in [-2.7,-1.1,1.08,2.61]:
        house.box(x,-2.53,1.67,.26,.09,1.14,"eee0bc",.048)
        house.box(x,-2.47,1.45,.27,.091,.063,"bba56d",.012)
    house.box(1.25,.70,.029,2.84,2.03,.038,"afae74",.044)
    for x in [-.14,2.63]:
        house.box(x,.70,.054,.11,2.01,.017,"e7c89b",.01)
    for z in [-.26,1.66]:
        house.box(1.25,z,.054,2.79,.095,.017,"e7c89b",.01)
    for i in range(3):
        house.box(0,3.04+i*.18,-.07-i*.13,1.72,.33,.18,"c78e62",.041)
    for x,z in [(-2.8,-1.15),(2.9,-1.8)]:
        lantern(house,x,z,1.35)
    house.pole((-3.42,-2.78,2.63),(0,-2.78,3.40),.125,"a4714e",4)
    house.pole((0,-2.78,3.40),(3.42,-2.78,2.63),.125,"9f6748",4)
    house.pole((0,-2.62,3.28),(0,-2.62,2.61),.029,"705543",6)
    lantern(house,0,-2.62,2.46)
    for x,z in [(-3.04,-2.34),(3.02,2.37)]:
        house.pole((x,z,0),(x,z,.32),.192,"b98261",9,tip=.24)
        house.pole((x,z,.306),(x,z,.338),.205,"7e684e",8)
        grass(house,x,z,.33,.43,"829950",8)
    bench=Model("Werkbank mit Schubladen und Werkzeug","furniture_bench")
    bench.box(0,0,.46,1.10,.78,.76,"a67851",.056)
    bench.box(0,0,.87,1.21,.90,.17,"c59460",.038)
    for y in [.34,.62]:
        bench.box(0,.397,y,.84,.09,.22,"bc8b5b",.025)
        bench.box(0,.464,y,.135,.071,.045,"6d6755",.018)
    bench.box(-.30,-.19,.978,.31,.27,.052,"85664d",.019)
    bench.box(.22,.03,.999,.36,.083,.075,"8c8072",.015,.36)
    bench.pole((.23,.06,.98),(.21,.37,1.02),.027,"9b7049",6)
    bed=Model("Bett mit Steppdecke","bedroom_bed")
    bed.box(0,0,.28,1.27,1.84,.45,"986845",.072)
    bed.box(0,-.78,.62,1.31,.16,.77,"ac7850",.065)
    bed.box(0,0,.55,1.18,1.65,.21,"f0dbb3",.082)
    bed.box(0,.23,.67,1.20,1.13,.18,"91a164",.076)
    for x in [-.4,-.2,0,.2,.4]:
        bed.box(x,.24,.768,.012,1.025,.009,"b6b080",.003)
    bed.box(0,-.50,.70,.86,.41,.18,"fae6c1",.075)
    table=Model("Runder Holztisch","furniture_table")
    for x in [-.40,.40]:
        for z in [-.35,.35]:
            table.box(x,z,.37,.115,.12,.74,"9d6a48",.023)
    table.pole((0,0,.68),(0,0,.83),.67,"c18b5c",12)
    table.box(0,.03,.842,.44,.94,.034,"e6d2a8",.017)
    table.pole((0,0,.85),(0,0,1.06),.112,"9c7b58",8,tip=.09)
    flowers(table,0,0,1.06,1.0,"ffdf93")
    shelf=Model("Regal mit Körben und Büchern","furniture_shelf")
    for z in [-.27,.27]:
        shelf.box(0,z,.82,1.15,.097,1.65,"aa7450",.03)
    for y in [.15,.64,1.10,1.65]:
        shelf.box(0,0,y,1.25,.60,.15,"c09162",.036)
    for i in range(6):
        x=(i-2.5)*.156
        shelf.box(x,.29,.82,.14,.19,.24,RNG.choice(["9a6150","768866","a99160"]),.024)
    for x in [-.28,.26]:
        shelf.box(x,.33,1.32,.37,.31,.30,"a77e50",.042)
        shelf.box(x,.33,1.48,.40,.34,.056,"c6a477",.023)
    stove=Model("Kleiner Kachelofen","furniture_stove")
    stove.box(0,0,.54,.72,.63,.83,"696078",.068)
    stove.box(0,.333,.54,.39,.05,.42,"e5a248",.04,mode="light")
    stove.box(0,.362,.54,.049,.055,.43,"635668",.008)
    stove.box(0,.364,.54,.40,.055,.046,"635668",.008)
    stove.pole((0,0,.92),(0,0,1.95),.13,"665c67",8)
    return [house,bed,table,shelf,stove,bench]


def resource_models():
    tree=Model("Fällbare Kiefer","wood_tree")
    pine(tree,0,0,0,.89,1)
    tree.pole((.013,-.017,.32),(.016,-.019,.43),.276,"c07854",8,tip=.268)
    tree.box(.38,.39,.22,.08,.082,.43,"9c774e",.013)
    tree.box(.38,.41,.40,.23,.102,.16,"d9b779",.018)
    stump=Model("Baumstumpf","wood_stump")
    stump.pole((0,0,0),(0,0,.32),.32,"79573c",9,tip=.27)
    stump.pole((0,0,.320),(0,0,.337),.224,"dcac76",9)
    rock=Model("Abbaubarer Stein","resource_rock")
    rock.ellipsoid(0,0,.31,.72,.59,.61,"a1939e",9,4,rough=.20)
    copper=Model("Kupferader mit freiliegendem Erz","resource_copper")
    copper.ellipsoid(0,0,.28,.67,.56,.54,"89728b",8,4,rough=.22)
    for x,z,h in [(-.23,.31,.34),(.05,.42,.43),(.24,.23,.57),(.12,-.19,.68)]:
        copper.ellipsoid(x,z,h,.23,.18,.21,"cf8050",7,4,rough=.14)
    tile=Model("Minensohle · polygonales Gestein","mine_floor")
    tile.box(0,0,-.235,2.40,2.40,.47,"81647b",.10)
    for _ in range(14):
        x,z=RNG.uniform(-1.08,1.08),RNG.uniform(-1.08,1.08)
        tile.ellipsoid(x,z,.008,RNG.uniform(.07,.18),.12,.029,
                       RNG.choice(["997883","ab8782","725873"]),6,3,rough=.17)
    wall=Model("Minensohle · Felsrand und Stützbalken","mine_wall")
    for i in range(4):
        x=(i-1.5)*.56
        wall.ellipsoid(x,0,.32,.51,.43,.51+i%3*.18,
                       ["736080","7c6683","8f7390","876989"][i],8,4,rough=.20)
    ladder=Model("Minenleiter mit Öllaterne","mine_ladder")
    for x in [-.30,.30]:
        ladder.box(x,0,.81,.095,.12,1.58,"ae7852",.019)
    for i in range(7):
        ladder.box(0,0,.12+i*.217,.62,.095,.075,"d1a66e",.015)
    ladder.box(.61,0,.80,.11,.12,1.59,"9b6848",.024)
    lantern(ladder,.61,.16,1.46)
    for crop, tint in [("turnip","c88da2"),("pumpkin","de8d46")]:
        m=Model(crop,crop)
        spots=([(-.33,-.34),(.33,-.34),(-.33,.34),(.33,.34)] if crop=="pumpkin"
               else [(x,z) for x in [-.35,0,.35] for z in [-.34,0,.34]])
        for x,z in spots:
            radius=.215 if crop=="pumpkin" else .106
            m.ellipsoid(x,z,.18+radius*.46,radius*1.15,radius,radius*.94,tint,10,5,rough=.075)
            if crop=="pumpkin":
                for j in range(5):
                    a=j*TAU/5
                    m.ellipsoid(x+radius*.53*math.cos(a),z+radius*.47*math.sin(a),.25,
                                radius*.62,radius*.57,radius*.72,"e69e54",7,4)
            grass(m,x,z,.28+radius*.65,radius*.9,"789443",5)
        if crop=="turnip":
            turnip=m
        else:
            pumpkin=m
    support=Model("Minensohle · Streckenbogen","mine_support")
    for x in [-1.03,1.03]:
        support.box(x,0,.90,.21,.27,1.84,"8f6047",.049)
        support.box(x,0,.14,.28,.33,.28,"7b5f69",.047)
    support.box(0,0,1.86,2.36,.33,.239,"b48560",.055)
    for side in [-1,1]:
        support.pole((side*.98,.06,1.30),(side*.55,.06,1.76),.076,"a77556",4)
    lantern(support,.75,.23,1.52)
    bobber=Model("Schwimmer","bobber")
    bobber.ellipsoid(0,0,.02,.103,.103,.089,"c97862",9,5)
    bobber.pole((0,0,.055),(0,0,.103),.083,"faead0",9,tip=.041)
    bobber.pole((0,0,.087),(0,0,.144),.019,"b35740",7,tip=.011)
    axe=Model("Axt · Eschengriff, Stahlblatt und Nieten","axe")
    axe.pole((0,-.018,.13),(0,.026,-.45),.027,"c39260",8,tip=.023)
    for i in range(4):
        h=.074-i*.028
        axe.pole((0,-.015,h),(0,-.014,h+.013),.031,"6f7859",7,tip=.031)
    axe.box(0,.027,-.365,.089,.11,.143,"727e7b",.015)
    axe.box(0,.109,-.375,.072,.105,.194,"8b9b96",.017,.05)
    axe.box(0,.185,-.371,.053,.078,.225,"a6b5a8",.017,.08)
    axe.box(0,.221,-.370,.037,.023,.200,"dbe0c1",.008,.08)
    for side in [-1,1]:
        axe.ellipsoid(side*.047,.027,-.366,.011,.023,.018,"e2bc77",7,3)

    pick=Model("Spitzhacke · Stahlspitzen und Messingkragen","pickaxe")
    pick.pole((0,-.028,.16),(0,.010,-.493),.025,"b58054",8,tip=.023)
    pick.pole((0,-.026,-.04),(0,-.023,.083),.031,"80654d",8)
    pick.box(0,.013,-.412,.083,.109,.137,"c2aa6b",.014)
    for side in [-1,1]:
        pick.pole((0,0,-.443),(0,side*.175,-.426),.056,"768c92",6,tip=.042)
        pick.pole((0,side*.175,-.426),(0,side*.290,-.343),.042,"9aa9a7",6,tip=.008)
    return [tree,stump,rock,copper,tile,wall,ladder,support,bobber,turnip,pumpkin,axe,pick]


def farm_assets():
    chest=Model("Verkaufstruhe · Holz, Leinwand und Messing","sale_crate")
    for i in range(6):
        x=(i-2.5)*.164
        chest.box(x,0,.275,.156,.62,.43,["bc855a","c49165","aa754c"][i%3],.027)
        chest.box(x,-.010,.61,.165,.69,.14,["cf9e70","c69462","d1a373"][i%3],.035)
    for x in [-.44,.44]:
        chest.box(x,0,.32,.14,.70,.53,"926446",.028)
        chest.box(x,0,.617,.14,.74,.17,"a47450",.029)
        chest.box(x,.371,.402,.108,.025,.329,"66796b",.015)
        for y in [.27,.52]:
            chest.ellipsoid(x,.386,y,.022,.012,.022,"deb27a",7,3)
    chest.box(0,0,.075,.99,.66,.11,"8d6245",.028)
    chest.box(0,.345,.454,.155,.066,.159,"d6ae6a",.023)
    chest.pole((-.045,.392,.43),(.045,.392,.43),.016,"7b664e",6)
    chest.box(0,.383,.241,.309,.025,.143,"668161",.012)
    chest.pole((-.052,.402,.196),(.056,.402,.279),.012,"f1d8a4",6)
    chest.ellipsoid(.025,.402,.257,.072,.016,.034,"dfd99f",8,4,angle=.45)

    can=Model("Gießkanne · Grünes Email und Messing","watering_can")
    can.ellipsoid(0,.029,-.155,.133,.142,.124,"638d7f",11,5)
    can.pole((0,.029,-.266),(0,.029,-.234),.092,"537570",10,tip=.112)
    can.pole((0,.030,-.060),(0,.030,-.026),.098,"a9b99a",11,tip=.091)
    can.pole((0,.030,-.035),(0,.030,-.026),.071,"415e56",11)
    can.pole((0,.099,-.185),(0,.270,-.106),.046,"648d80",9,tip=.032)
    can.pole((0,.265,-.108),(0,.321,-.078),.042,"d1b27a",9,tip=.053)
    for a,b in [(-.17,-.125),(-.125,-.086),(-.086,-.030),(-.03,.042)]:
        can.pole((0,a,.032-math.sin((a+.03)*7)**2*.138),
                 (0,b,.032-math.sin((b+.03)*7)**2*.138),.021,"7ea88f",7)
    drop=Model("Gießwasser · Tropfen","water_drop")
    drop.ellipsoid(0,0,0,.027,.027,.046,"a7ddd7",7,4,mode="raw")

    mulch=Model("Mulch · Warme Holzspäne im Beet","fertilized_soil")
    rng=random.Random(41)
    for _ in range(33):
        x,z=rng.uniform(-.61,.61),rng.uniform(-.60,.60)
        size=rng.uniform(.024,.071)
        mulch.ellipsoid(x,z,.086,size,size*.62,.022,
                        rng.choice(["cda271","d8b180","ac7c58"]),6,3,angle=rng.random())

    seeds=Model("Wilde Samenstände","find_seeds")
    for x,z,h in [(-.20,-.09,.41),(.07,-.05,.54),(.19,.15,.38),(-.11,.18,.34)]:
        seeds.pole((x*.45,z*.50,0),(x,z,h),.029,"a6945d",6,tip=.015)
        seeds.pole((x,z,h*.48),(x-.13,z+.084,h*.66),.022,"819357",5,tip=.007)
        seeds.ellipsoid(x,z,h,.075,.064,.102,"d0ad6d",8,4)
        for dx,dy in [(-.047,-.02),(.036,.007),(.009,.05)]:
            seeds.ellipsoid(x+dx,z+.044,h+dy,.033,.023,.034,"ead5a0",6,3)
    return [chest,can,drop,mulch,seeds]


def workshop_assets():
    models=[]
    def make(name, asset):
        model=Model(name,asset)
        models.append(model)
        return model

    plot=make("Bauplatz · Pflöcke und Schnur","building_plot")
    outline=[(-1.29,-.98),(1.29,-.98),(1.29,.98),(-1.29,.98)]
    for x,z in outline:
        plot.box(x,z,.22,.095,.095,.44,"a27b55",.016)
        plot.box(x,z,.41,.109,.105,.065,"d0a86c",.012)
    for (x,z),(xx,zz) in zip(outline,outline[1:]+outline[:1]):
        middle=((x+xx)/2,(z+zz)/2,.22)
        plot.pole((x,z,.34),middle,.019,"d9c597",6)
        plot.pole(middle,(xx,zz,.34),.019,"d9c597",6)

    for kind,wide,depth,high,roof in [
        ("coop",1.92,1.29,1.22,"a4a578"),("barn",2.62,1.89,1.63,"b97458")
    ]:
        pen=make("Hühnerstall" if kind=="coop" else "Schafscheune","farm_"+kind)
        pen.box(0,-.12,.11,wide+.13,depth+.11,.23,"a79a84",.063)
        for i in range(10):
            y=.23+(i+.5)*high/10
            for z in [depth/2-.12,-depth/2-.12]:
                pen.box(0,z,y,wide,.10,high/10-.007,["b68157","c38e61","c89666"][i%3],.018)
            for x in [-wide/2,wide/2]:
                pen.box(x,-.12,y,.12,depth,high/10-.009,"b77f58",.013)
        for x in [-wide/2,wide/2]:
            for z in [depth/2-.10,-depth/2-.10]:
                pen.box(x,z,high*.53+.19,.15,.15,high+.05,"8a6148",.024)
        y=high+.25
        for side in [-1,1]:
            for j in range(9):
                z=(j-4)*(depth+.35)/9-.12
                pen.pole((0,z,y+.47),(side*(wide/2+.13),z,y-.14),(depth+.35)/18,
                         blend(roof,"e2bd82",j%3*.12),6)
        pen.pole((0,-depth/2-.31,y+.51),(0,depth/2+.08,y+.51),.092,"c49161",8)
        front=depth/2-.051
        for x in [-.28,.28]:
            pen.box(x,front,.61,.54,.09,.81,"956b50",.036)
            pen.pole((x-.21,front+.056,.28),(x+.21,front+.056,.92),.045,"c19066",4)
        pen.box(0,front+.18,.159,.89,.29,.124,"c3a378",.024)
        window(pen,wide*.31,front-.012,y-.23,.37,.33)
        lantern(pen,-wide*.31,front+.17,y-.42)
        for i in range(4):
            x=wide*.44+(i%2)*.11
            pen.box(x,front+.23,.11+i*.035,.37,.29,.069,"d5ac70",.025,.15*i)
        pen.box(-wide*.44,front+.19,.15,.53,.31,.27,"a97d55",.036)

    kitchen=make("Hofküche · Herd, Einkochen und Handmühle","farm_kitchen")
    kitchen.box(0,0,.135,2.08,1.02,.27,"b0957a",.055)
    for x in [-.92,.92]:
        kitchen.box(x,-.34,.93,.14,.16,1.83,"94694c",.021)
    kitchen.box(0,-.24,1.71,2.02,.52,.14,"ba9568",.028)
    for j in range(9):
        x=(j-4)*.242
        kitchen.pole((x,-.66,1.84),(x,.29,1.53),.129,
                     ["c9875c","d69a69","be7b53"][j%3],6)
    kitchen.box(-.47,-.05,.49,.79,.69,.67,"9a8070",.057)
    kitchen.box(-.47,.302,.456,.63,.049,.358,"4c4f46",.024)
    kitchen.ellipsoid(-.47,.324,.463,.23,.024,.124,"f3a861",9,4,mode="light")
    kitchen.pole((-.47,-.084,.825),(-.47,-.084,.997),.223,"5b6255",10,tip=.244)
    kitchen.pole((-.47,-.084,1.003),(-.47,-.084,1.047),.224,"829087",10,tip=.163)
    kitchen.ellipsoid(-.47,-.084,1.074,.043,.045,.031,"b9aa86",8,4)
    kitchen.box(.47,-.016,.62,.91,.72,.49,"ab7950",.041)
    kitchen.box(.47,.006,.877,.95,.76,.091,"d0a576",.025)
    kitchen.pole((.67,-.08,.936),(.67,-.08,1.099),.212,"a59c87",10)
    kitchen.pole((.67,-.08,1.085),(.84,-.03,1.176),.029,"725d46",6)
    for i,(x,tint) in enumerate([(.01,"b97773"),(.24,"ebad71"),(.40,"8b9c64")]):
        kitchen.pole((x,.247,.929),(x,.247,1.126),.068,tint,8,tip=.075)
        kitchen.pole((x,.247,1.118),(x,.247,1.152),.081,"ddbd8a",8)
    lantern(kitchen,.88,.22,1.39)

    stall=make("Bauplanstand · Tintenfass und Pergament","pattern_stall")
    stall.box(0,-.04,.27,1.42,.67,.54,"966747",.045)
    stall.box(0,.02,.62,1.54,.78,.125,"d3a36e",.032)
    for x in [-.70,.70]:
        stall.box(x,-.28,1.04,.12,.12,1.47,"8f664a",.025)
    for i in range(7):
        x=(i-3)*.226
        stall.pole((x,-.53,1.70),(x,.40,1.43),.116,
                   "769279" if i%2 else "dfca97",5)
    for i,x in enumerate([-.39,-.11,.19]):
        stall.box(x,.15,.70,.23,.37,.048,"e8c996",.011,.06*(i-1))
        stall.pole((x-.092,.311,.725),(x+.088,.317,.725),.027,"c49a62",7)
    stall.pole((.50,.02,.68),(.50,.02,.751),.08,"527166",7,tip=.051)
    stall.pole((.50,.02,.752),(.59,-.022,.926),.015,"e4d8b0",5,tip=.004)
    roll=make("Gefundener Bauplan · Pergament und Siegel","find_blueprint")
    roll.box(.02,.03,.105,.383,.257,.047,"ecd49e",.018,.22)
    roll.pole((-.23,-.04,.128),(.19,.075,.128),.058,"dab782",8)
    roll.pole((-.17,-.094,.13),(.19,-.008,.13),.039,"ead3a8",8)
    roll.box(.02,-.01,.176,.069,.24,.034,"788d74",.012,-.10)
    roll.ellipsoid(.025,.074,.200,.045,.051,.018,"ad6252",7,3)

    for asset,tint,cushion in [
        ("blue_bench","79918d",False),("cushion","a77c61",True)
    ]:
        bench=make("Kissenbank" if cushion else "Blaue Sitzbank","furniture_"+asset)
        for x in [-.51,.51]:
            bench.box(x,0,.235,.13,.57,.47,"95674b",.025)
            if not cushion:
                bench.box(x,-.237,.673,.081,.081,.56,"72665b",.016)
        bench.box(0,0,.48,1.34,.62,.14,tint,.041)
        if cushion:
            for x,c in [(-.34,"ad7567"),(.34,"d5ad75")]:
                bench.box(x,.025,.601,.62,.56,.149,c,.053)
        else:
            bench.box(0,-.248,.77,1.28,.09,.273,"819b8c",.025)

    lamp=make("Laternenständer","furniture_lantern")
    lamp.pole((0,0,0),(0,0,.097),.24,"a38862",8,tip=.21)
    lamp.pole((0,0,.09),(0,0,1.39),.072,"8d6148",6,tip=.059)
    lantern(lamp,0,.111,1.267)
    lamp.box(0,.05,1.447,.357,.24,.084,"a97653",.024)

    pot=make("Farntopf","furniture_fern")
    pot.pole((0,0,0),(0,0,.397),.204,"b87959",10,tip=.294)
    pot.pole((0,0,.361),(0,0,.407),.30,"cc9671",10)
    pot.pole((0,0,.406),(0,0,.423),.263,"746144",10)
    fern(pot,0,0,.65)
    for j in range(8):
        a=j*TAU/8
        pot.pole((0,0,.411),(.41*math.cos(a),.41*math.sin(a),.73+.14*math.cos(j)),.04,"81a078",5,tip=.007)

    for asset,w,colors in [("armchair",1.01,["a0a475","adb485"]),
                            ("sofa",1.53,["ad7464","dfa878","899b78"])]:
        chair=make("Lesesessel" if asset=="armchair" else "Patchworksofa","furniture_"+asset)
        for x in [-w*.42,w*.42]:
            for z in [0.31,-.31]:
                chair.pole((x,z,0),(x,z,.285),.055,"825c46",5)
            chair.box(x,-.02,.60,.17,.77,.32,"b58d6b",.059)
        chair.box(0,-.015,.35,w,.82,.26,"91735f",.061)
        for i,color in enumerate(colors):
            x=w*((i+.5)/len(colors)-.5)*.83
            chair.box(x,.042,.531,w*.80/len(colors),.64,.17,color,.063)
            chair.box(x,-.36,.815,w*.82/len(colors),.21,.55,color,.067)

    for asset,label,tint in [("display","Glasvitrine","8ca99a"),
                             ("vines","Rankregal","a0875a"),
                             ("herbs","Kräutervitrine","798179")]:
        shelf=make(label,"furniture_"+asset)
        for x in [-.51,.51]:
            shelf.box(x,0,.83,.13,.60,1.63,tint,.026)
        shelf.box(0,-.26,.81,1.12,.09,1.55,"a37f60",.028)
        for i,y in enumerate([.15,.55,1.03,1.58]):
            shelf.box(0,0,y,1.15,.616,.099,tint,.031)
            if i==3:
                continue
            for j,x in enumerate([-.35,.04,.36]):
                h=.102+j*.025
                shelf.pole((x,.075,y+.052),(x,.075,y+.056+2*h),.071,
                           ["e5b377","c28773","93a77c"][(j+i)%3],7,tip=.066)
                if asset=="vines":
                    for k in range(3):
                        shelf.ellipsoid(x+k*.032-.034,.16,y+.19+k*.085,.096,.035,.058,
                                        "829f70",6,4,angle=.34*k)
        if asset=="display":
            for x in [-.283,.283]:
                shelf.box(x,.277,.857,.057,.08,1.36,"9eb6a6",.015)
                for y in [.45,1.14]:
                    shelf.face([(x-.19,.325,y+.10),(x+.145,.325,y-.08),
                                (x+.12,.325,y-.104),(x-.19,.325,y+.072)],"b8d0ba")
        if asset=="herbs":
            shelf.box(0,.28,1.43,1.10,.08,.32,"829579",.027)
            for x in [-.37,0,.37]:
                window(shelf,x,.31,1.434,.25,.12)

    chest=make("Hafentruhe","furniture_harbor_chest")
    chest.box(0,0,.274,1.07,.69,.51,"a87a62",.039)
    chest.box(0,-.005,.57,1.14,.72,.16,"899d90",.056)
    chest.box(0,.361,.432,.225,.045,.146,"cda86d",.015)
    chest.box(0,.395,.443,.10,.066,.074,"7f6852",.014)
    for side in [-1,1]:
        chest.box(side*.43,0,.32,.096,.734,.535,"858c76",.021)

    desk=make("Schreibtisch","furniture_desk")
    for x in [-.55,.55]:
        for z in [-.343,.30]:
            desk.box(x,z,.356,.12,.14,.72,"946a50",.033)
    desk.box(0,0,.733,1.36,.742,.166,"c89166",.029)
    desk.box(.345,.08,.492,.545,.57,.271,"a97854",.034)
    desk.box(.348,.38,.504,.12,.04,.068,"bf9b67",.011)
    desk.box(-.18,.122,.841,.287,.36,.029,"ebd7ab",.012,.25)
    desk.pole((.48,-.21,.816),(.48,-.21,.969),.063,"688379",8,tip=.051)

    stove=make("Blauer Kachelofen","furniture_blue_stove")
    for i in range(6):
        stove.box(0,-.02,.14+i*.13,.84,.75,.128,["688983","77968c","87a398"][i%3],.031)
    stove.pole((0,-.084,.815),(0,-.084,1.91),.16,"62585a",8,tip=.135)
    stove.box(0,.365,.44,.45,.06,.43,"494b4a",.053)
    stove.box(0,.407,.429,.315,.034,.246,"d18e55",.056,mode="light")

    rug=make("Wollteppich","furniture_rug")
    rug.box(0,0,.028,1.30,1.03,.034,"939b73",.058)
    for x in [-.48,.48]:
        rug.box(x,0,.049,.126,.91,.013,"e8ce95",.011)
    for z in [-.453,.453]:
        rug.box(0,z,.051,1.24,.099,.015,"d7b085",.012)
    for x in [-.25,0,.25]:
        rug.box(x,0,.053,.118,.118,.008,"cdab83",.014,angle=math.pi/4)

    copper=make("Kupferlampentisch","furniture_copper_lamp")
    copper.pole((0,0,0),(0,0,.078),.346,"ae795c",9,tip=.326)
    copper.pole((0,0,.081),(0,0,.601),.11,"99694d",9)
    copper.pole((0,0,.582),(0,0,.719),.45,"c69b72",12)
    copper.pole((0,0,.72),(0,0,.982),.087,"b77d56",8,tip=.057)
    copper.pole((0,0,.983),(0,0,1.18),.204,"db9a67",10,tip=.13)
    copper.pole((0,0,.958),(0,0,.979),.198,"f0c985",10,mode="light")

    harp=make("Walnussharfe","furniture_harp")
    harp.box(0,0,.086,1.045,.618,.17,"956b55",.047)
    harp.pole((-.40,0,.16),(-.42,0,1.59),.092,"aa7b57",7,tip=.069)
    harp.pole((-.42,0,1.57),(.46,0,.86),.091,"a57753",7)
    harp.pole((.46,0,.86),(.29,0,.17),.106,"8f634a",7)
    for i in range(8):
        x=-.32+i*.094
        harp.pole((x,.02,.186),(x,.02,1.47-i*.077),.013,"dfbe82",5)
    return models


ORCHARD_POINTS = [
    (-12.35,7.8),(-11.05,5.3),(-8.10,8.90),(-14.40,9.80),
    (-13.30,11.50),(-9.70,10.80),(-15.0,6.50),
]
APPLE_POSITIONS = [(-.89,.64,1.76),(.60,.98,1.87),(.14,1.17,2.15),(-1.12,.31,1.86),(.67,-.70,2.08)]
BERRY_POSITIONS = [(-.41,.28,.59),(.38,.34,.64),(-.08,.64,.59),(.18,-.31,.79)]


def fence(model, points, height=.79):
    for x,z in points:
        model.box(x,z,height*.49,.163,.168,height,"a7764e",.025)
        model.box(x,z,height+.005,.185,.181,.064,"ca9c63",.016)
    for (x,z),(xx,zz) in zip(points,points[1:]):
        model.obstacle_rail(x,z,xx,zz)
        width=math.hypot(xx-x,zz-z)
        for h in [.29,.59]:
            model.box((x+xx)/2,(z+zz)/2,h,width,.113,.116,
                      "a98053" if h==.29 else "bb8e59",.025,math.atan2(zz-z,xx-x))


def produce_crate(model,x,z,h,color):
    if h == 0:
        model.obstacle_rect(x,z,.55,.42)
    model.box(x,z,h+.095,.55,.42,.19,"b28456",.024)
    for side in [-1,1]:
        for dx in [-.24,.24]:
            model.box(x+dx,z+side*.205,h+.103,.053,.055,.21,"d2a56e",.014)
    for dx,dz in [(-.16,-.10),(.04,-.13),(.20,-.09),(-.16,.09),(.05,.08),(.19,.11)]:
        model.ellipsoid(x+dx,z+dz,h+.21,.104,.087,.089,color,7,4,rough=.09)


def booth(model,x,z,kind):
    model.obstacle_rect(x,z,3.10,2.53)
    wood=["bc8858","ae7952","c59161","b58050"]
    for j in range(10):
        model.box(x,z-1.16,.38+j*.195,2.86,.15,.184,wood[j%4],.015)
    for i in range(12):
        model.box(x+(i-5.5)*.245,z,.085,.241,2.44,.153,wood[i%4],.022)
    for dx in [-1.45,1.45]:
        for dz in [-1.18,1.1]:
            model.box(x+dx,z+dz,1.27,.188,.193,2.55,"9f6e4a",.035)
        model.box(x+dx,z,2.51,.16,2.47,.18,"ac7951",.025)
    for dz in [-1.20,1.08]:
        model.box(x,z+dz,2.50,3.08,.19,.22,"ab754e",.033)
    for dz in [-1.17,1.08]:
        model.face([(x-1.45,z+dz,2.49),(x+1.45,z+dz,2.49),(x,z+dz,3.31)],"af7d54")
    for side in [-1,1]:
        model.face([(x,z-1.45,3.30),(x+side*1.73,z-1.45,2.40),
                    (x+side*1.73,z+1.47,2.40),(x,z+1.47,3.30)],"ab6d53")
        for k in range(6):
            u=k*.281
            h=3.31-u*.535
            for i in range(11):
                zz=z+(i-5)*.235
                lead=x+side*(u+.297)
                rise=h-.156
                model.pole((x+side*u,zz,h),(lead,zz,rise),.128,
                           ["af6e51","b97a55","c88a61","b47251"][(i+k)%4],6)
        for dz in [-1.42,1.30]:
            model.pole((x,z+dz,3.34),(x+side*1.76,z+dz,2.42),.101,"b98555",4)
    model.pole((x,z-1.48,3.36),(x,z+1.49,3.36),.139,"c8945d",9)
    window(model,x,z+1.12,2.75,.43,.44)
    model.box(x,z+.60,.84,2.57,.44,.21,"d4a371",.037)
    for j in range(10):
        model.box(x+(j-4.5)*.247,z+.64,.46,.241,.16,.69,wood[j%4],.020)
    model.box(x,z-.95,1.22,2.47,.35,.12,"a56e4d",.022)
    for j in range(8):
        xx=x+(j-3.5)*.29
        model.pole((xx,z-.91,1.29),(xx,z-.91,1.51),.10,
                   ["ba8a50","ceb482","78925f","9d7a52"][j%4],7,tip=.085)
        model.pole((xx,z-.91,1.51),(xx,z-.91,1.55),.093,"ebe1b9",7)
    # Beveled cloth slats form the striped awning and its shallow front valance.
    strips=("7d9662","f1dfb3") if kind==0 else ("b86e53","f3dcb7")
    for j in range(12):
        xx=x+(j-5.5)*.264
        model.face([(xx-.132,z+.73,2.41),(xx+.132,z+.73,2.41),
                    (xx+.132,z+1.73,2.00),(xx-.132,z+1.73,2.00)],strips[j%2])
        model.box(xx,z+1.726,1.946,.257,.051,.103,strips[j%2],.021)
    for dx in [-1.58,1.58]:
        lantern(model,x+dx,z+1.28,1.67)
    if kind==0:
        # The seed merchant has fox ears, a cream muzzle and a linen apron.
        center=x-.36
        model.ellipsoid(center,z+.19,1.15,.21,.14,.24,"8c9a75",9,4)
        model.ellipsoid(center,z+.24,1.48,.24,.194,.217,"ba7852",11,5)
        for side in [-1,1]:
            model.pole((center+side*.145,z+.17,1.55),(center+side*.205,z+.17,1.78),.075,"a86545",5,tip=.011)
            model.ellipsoid(center+side*.094,z+.411,1.475,.028,.015,.035,"453a31",7,4,mode="raw")
        model.ellipsoid(center,z+.389,1.413,.135,.089,.073,"ebcaa0",9,4)
        model.ellipsoid(center,z+.462,1.437,.028,.022,.021,"59443a",8,4)
    else:
        for i,color in enumerate(["ca7257","83a25c","d8a666"]):
            produce_crate(model,x+(i-1)*.7,z+.63,.95,color)
    for i,color in enumerate(["d48a58","9ba965","cf6963"]):
        produce_crate(model,x+(i-1)*.71,z+1.86,0,color)


def homestead_models():
    market=Model("Hafenmarkt · Saatgutladen, Ankauf und Schmiede")
    meadow=Model("Tierweide · Stall, Tröge und Holzpforte")
    orchard=Model("Obstwiese · Apfelbäume und Beerenbüsche")
    street=Model("Markt- und Hofwege")
    booth(market,16.15,1.26,0)
    booth(market,20.05,1.43,1)
    for j in range(10):
        for i in range(16):
            x=14.3+i*.473+(j%2)*.12
            z=3.47+j*.34
            if x>22 and z>6:continue
            street.box(x,z,.039,.459,.326,.071,["cbb99e","d5c4a9","c3b69b","e2cea9"][(i+j*2)%4],.033,RNG.uniform(-.09,.09))
    fence(market,[(22.15,2.70),(22.52,3.82),(22.35,4.90),(22.03,6.0),(21.46,7.02),(20.79,8.0)])
    for x,z in [(15.01,2.96),(20.8,5.91),(14.95,6.0)]:
        barrel(market,x,z,0,.80)
        if z>4:
            grass(market,x,z,.5,.45,"8a9d59",6)
            for i in range(3):
                flowers(market,x+i*.09,z,.51,.81,"f8eacd")
    for x in [15.55,16.55]:
        market.box(x,6.14,.64,.125,.14,1.28,"936643",.022)
    market.box(16.05,6.21,1.10,1.26,.14,.62,"c39864",.043)
    for dx,h in [(-.31,1.18),(.05,1.09),(.30,1.19)]:
        market.box(16.05+dx,6.287,h,.246,.023,.277,"e9d8a4",.011,RNG.uniform(-.10,.10))
    market.box(20.02,6.34,.44,1.30,.75,.86,"a87952",.04)
    market.obstacle_rect(20.02,6.34,1.45,.82)
    market.box(20.02,6.34,.892,1.45,.82,.133,"c49362",.029)
    market.box(20.02,6.34,1.05,.37,.29,.22,"696778",.027)
    market.box(20.02,6.34,1.20,.72,.34,.12,"797c83",.038)
    market.pole((19.47,6.25,1.03),(19.66,6.39,1.08),.043,"ab8053",8)
    for x in [19.13,20.95]:
        market.box(x,6.02,1.12,.16,.17,2.18,"976946",.027)
    market.box(20.05,6.20,2.08,2.17,1.12,.18,"b17753",.052)
    lantern(market,20.82,6.59,1.77)
    # A little clinker boat at the overlook, made from polygon rings.
    boat=[]
    for y,n in [(-1.36,.76),(-1.08,.91),(-.89,1.0)]:
        boat.append([(24.5+math.cos(a)*.47*n,8.27+math.sin(a)*1.15*n,y)
                     for i in range(12) for a in [TAU*i/12]])
    market.rings(boat,["a9684b"]*12+["d1a371"]*12+["926446","b68552"])
    for z in [7.83,8.26,8.71]:
        market.box(24.5,z,-.835,.68,.10,.071,"d7ac77",.022)
    for j in range(11):
        x=21.95+j*.22
        market.box(x,7.23,-.055-j*.069,.217,1.16,.135,
                   ["bc8c5d","cca16b","d4a773"][j%3],.024)
        if j%4==0:
            for z in [6.71,7.74]:
                market.pole((x,z,-1.78),(x,z,.38-j*.069),.106,"8e684e",8)
    lantern(market,22.04,6.67,.71)
    for pts in [
        [(13,-1),(14.2,1.4),(14.75,4.1),(16.05,6.20),(18.3,6.3)],
        [(-5,-6.2),(-8.38,-7.55),(-10.35,-8.37),(-12.56,-9.17)],
        [(-6.4,11.03),(-8.1,8.9),(-10.32,8.04),(-12.2,7.6),(-14.67,6.47)],
    ]:
        for (x,z),(xx,zz) in zip(pts,pts[1:]):
            count=math.ceil(math.hypot(xx-x,zz-z)*2.25)
            for j in range(count):
                t=j/count
                for delta in [-.24,.24]:
                    street.box(x+(xx-x)*t+delta,z+(zz-z)*t+RNG.uniform(-.10,.10),.056,
                               RNG.uniform(.35,.49),RNG.uniform(.29,.39),.072,
                               RNG.choice(["cdbda2","d8c5a4","bfb198"]),.036,RNG.uniform(-.32,.32))
    fence(meadow,[(-14.8,-7.4),(-14.8,-8.7),(-14.8,-10.0),(-14.8,-11.6),
                    (-13.25,-12.1),(-11.7,-12.1),(-10.15,-12.1),(-8.65,-12.1),
                    (-7.16,-12.1),(-5.68,-11.7),(-5.68,-10.1),(-5.68,-8.65),(-5.68,-7.28),
                    (-7.12,-7.28),(-8.58,-7.28),(-9.35,-7.28)])
    fence(meadow,[(-14.8,-7.4),(-13.33,-7.39),(-11.85,-7.31),(-11.36,-7.29)])
    for x,z,w,d,h in [(-12.9,-10.72,2.40,1.62,1.43),(-7.52,-10.82,2.44,1.67,1.20)]:
        meadow.obstacle_rect(x,z,w,d)
        for dx in [-w/2,w/2]:
            for dz in [-d/2,d/2]:
                meadow.box(x+dx,z+dz,.65,.146,.16,1.30,"926645",.025)
        for i in range(8):
            yy=.40+i*.156
            meadow.box(x,z-d/2,yy,w,.12,.148,["b98254","c38c5b","cb9762"][i%3],.022)
            for side in [-1,1]:
                meadow.box(x+side*(w/2-.04),z,yy,.15,d,.148,"c18c5d",.024)
        if x<-10:
            for dx in [-.76,.75]:
                meadow.box(x+dx,z+d/2,.85,.72,.13,1.14,"c99764",.026)
            meadow.box(x,z+d/2,.40,.96,.44,.19,"ae7d50",.024)
            for i in range(5):
                meadow.box(x,z+.88+i*.171,.35-i*.059,.70,.22,.08,"cfa271",.023)
        for side in [-1,1]:
            for j in range(8):
                zz=z+(j-3.5)*.26
                meadow.pole((x,zz,2.12),(x+side*(w/2+.21),zz,1.51),.141,
                            ["b97859","ce9164","c58258","c98a60"][j%4],6)
        meadow.pole((x,z-1.14,2.18),(x,z+1.17,2.18),.14,"dc9f68",9)
        lantern(meadow,x+.81,z+d/2+.25,1.33)
    for x,z in [(-10.35,-8.37),(-7.63,-9.27),(-13.19,-8.98)]:
        meadow.obstacle_rect(x,z,.95,.47)
        meadow.box(x,z,.23,.95,.47,.40,"ac8155",.046)
        meadow.box(x,z,.433,.868,.42,.088,"d9b368",.025)
        for i in range(9):
            dx=RNG.uniform(-.42,.42)
            dz=RNG.uniform(-.19,.19)
            meadow.pole((x+dx,z+dz,.432),(x+dx*.94,z+dz*.79,.51+RNG.random()*.07),.016,"dcc37d",5)
    for x,z in [(-8.4,-7.79),(-11.1,-10.72),(-8.77,-11.25),(-14.09,-8.51)]:
        grass(meadow,x,z,0,.37,"8ca75a",6)
        for i in range(3):
            flowers(meadow,x+.15*i,z+.067*i,0,.80,["f5ebcc","d7a3b5","e9c779"][i])
    for i,(x,z) in enumerate(ORCHARD_POINTS):
        orchard.obstacle_circle(x,z,.25 if i<4 else .54)
        if i<4:
            orchard.pole((x,z,0),(x+.10,z,2.00),.245,"9a694a",9,tip=.12)
            for dx,dz,h,r in [(-.61,.08,1.85,.65),(.48,.27,2.08,.78),(.00,.35,2.38,.88),(-.29,-.46,2.22,.63),
                              (.34,-.54,2.54,.71),(.07,-.02,2.92,.57)]:
                orchard.pole((x,z,.92),(x+dx,z+dz,h),.077,"a77657",6,tip=.042)
                orchard.ellipsoid(x+dx,z+dz,h,r,r*.90,r*.77,
                                  ["96a655","adb362","88a056"][i%3],9,4,rough=.19)
        else:
            for dx,dz,r in [(-.39,.06,.40),(.41,0,.42),(0,.19,.45),(.18,-.26,.46),(-.02,-.35,.43)]:
                orchard.ellipsoid(x+dx,z+dz,.40,r,r*.90,r,
                                  ["78954d","8ca75c","7d9052"][i%3],8,4,rough=.19)
        for j in range(6):
            xx=x+RNG.uniform(-1.12,1.12)
            zz=z+RNG.uniform(-.90,.90)
            if j%3==0:
                flowers(orchard,xx,zz,0,.78,["f2ead0","dca2b6","e7b371"][j%3])
            else:
                grass(orchard,xx,zz,0,.21,"9aab58",4)
    orchard.pole((-10.6,8.35,.016),(-10.6,8.35,.335),.28,"aa8651",10,tip=.36)
    for a in [i*TAU/12 for i in range(12)]:
        orchard.ellipsoid(-10.6+.20*math.cos(a),8.35+.21*math.sin(a),.336,.087,.078,.075,"c56c58",6,4)
    return [market,meadow,orchard,street]


def new_game_models():
    apple=Model("Reife Äpfel","apple_fruit")
    berries=Model("Waldbeeren","berry_fruit")
    for i,(x,z,y) in enumerate(APPLE_POSITIONS):
        apple.ellipsoid(x,z,y,.172,.148,.163,["c86554","ac4f44","d67259"][i%3],9,5,rough=.09)
        apple.pole((x,z,y+.104),(x+.041,z,y+.210),.019,"785840",5)
        apple.face([(x+.036,z,y+.167),(x+.129,z+.04,y+.222),(x+.184,z,y+.23),(x+.109,z-.064,y+.232)],"a3ae5e","leaf")
    for i,(x,z,y) in enumerate(BERRY_POSITIONS):
        for j in range(5):
            a=j*TAU/5
            berries.ellipsoid(x+math.cos(a)*.069,z+math.sin(a)*.058,y,.069,.064,.063,
                              ["775f86","95678e","a66379"][(i+j)%3],7,4)
    hen=Model("Huhn · Laufender Stallbewohner","farm_hen")
    hen.ellipsoid(0,-.019,.339,.200,.266,.229,"e9dbc4",11,5)
    hen.ellipsoid(0,.182,.53,.131,.131,.143,"f9e8c9",9,5)
    for s in [-1,1]:
        hen.ellipsoid(s*.149,-.045,.340,.077,.144,.113,"d7c9b5",8,4)
        hen.ellipsoid(s*.103,.237,.558,.018,.019,.024,"534237",6,3,mode="raw")
        hen.pole((s*.059,.022,.234),(s*.072,.015,.039),.023,"c99556",6,tip=.015)
        hen.box(s*.071,.048,.036,.063,.098,.046,"dbad64",.013)
    hen.pole((0,.279,.521),(0,.408,.505),.057,"d3a754",6,tip=.009)
    for j in range(3):
        hen.ellipsoid(0,.119+j*.052,.657+(.019 if j==1 else 0),.029,.047,.052,"bd6556",8,4)
    for dx in [-.10,0,.10]:
        hen.pole((dx,-.17,.39),(dx,-.35,.57),.049,"ebd6b9",6,tip=.027)
    sheep=Model("Schaf · Wollige Weidenachbarin","farm_sheep")
    for x in [-.155,.155]:
        for z in [-.18,.18]:
            sheep.pole((x,z,.035),(x,z,.321),.058,"665449",7,tip=.083)
    sheep.ellipsoid(0,-.067,.480,.347,.439,.349,"eadbc0",11,5)
    for i in range(10):
        a=i*TAU/10
        sheep.ellipsoid(.272*math.cos(a),-.072+.325*math.sin(a),.51+(.091 if i%2 else 0),
                        .167,.167,.180,["eee0c7","dfd0b7","f9e8cb"][i%3],9,4,rough=.13)
    sheep.ellipsoid(0,.313,.520,.196,.199,.213,"837064",10,5)
    sheep.ellipsoid(0,.316,.690,.185,.143,.119,"f1dfc3",9,4,rough=.11)
    for s in [-1,1]:
        sheep.pole((s*.128,.332,.638),(s*.286,.282,.610),.061,"8e7665",7,tip=.031)
        sheep.ellipsoid(s*.076,.482,.564,.026,.027,.029,"493f37",8,4,mode="raw")
    sheep.ellipsoid(0,.490,.467,.095,.063,.060,"a28b78",9,4)
    strawberry=Model("Reife Erdbeeren","crop_strawberry")
    tomato=Model("Reife Tomaten am Rankstock","crop_tomato")
    sunflower=Model("Reife Sonnenblumen","crop_sunflower")
    for x,z in [(x,z) for x in [-.35,0,.35] for z in [-.34,.02,.34]]:
        grass(strawberry,x,z,.10,.19,"779a50",5)
        for j in [-1,1]:
            strawberry.ellipsoid(x+j*.074,z+.053,.193,.063,.054,.069,"cc6559",8,4)
            for k in [-1,1]:
                strawberry.ellipsoid(x+j*.074+k*.027,z+.098,.201,.009,.006,.009,"edc380",6,3)
    for x,z in [(-.30,-.33),(.30,-.33),(-.30,.28),(.30,.28)]:
        tomato.pole((x-.07,z,.103),(x-.10,z,.84),.022,"aa8153",6)
        tomato.pole((x,z,.113),(x+.024,z,.72),.020,"7d9852",6,tip=.012)
        for j,(u,h) in enumerate([(-.09,.37),(.07,.50),(.04,.65)]):
            tomato.ellipsoid(x+u,z+.07,h,.093,.081,.083,"cc6c54",8,4)
            grass(tomato,x+u,z+.057,h+.072,.080,"92a459",4)
        height=.99+RNG.uniform(-.12,.14)
        sunflower.pole((x,z,.12),(x,z,height),.034,"79944e",7)
        grass(sunflower,x,z,.33,.24,"82a15a",4)
        sunflower.ellipsoid(x,z+.03,height,.139,.052,.139,"a57548",10,4)
        for j in range(11):
            a=TAU*j/11
            sunflower.ellipsoid(x+math.cos(a)*.178,z+.02,height+math.sin(a)*.178,.055,.042,.062,
                                ["e5b85b","ecc36b","deab4e"][j%3],7,4)
        sunflower.ellipsoid(x,z+.066,height,.091,.031,.086,"936347",10,4)
    silver=Model("Silberadern im Schiefer","resource_silver")
    silver.ellipsoid(0,0,.283,.72,.60,.53,"716c83",9,4,rough=.18)
    for x,z,y in [(-.26,.25,.49),(0,.41,.36),(.28,.23,.62),(.05,-.02,.78),(.48,.01,.37)]:
        silver.ellipsoid(x,z,y,.21,.155,.196,"c0c7cc",8,4,rough=.08)
    crystal=Model("Türkises Kristallnest","resource_crystal")
    crystal.ellipsoid(0,0,.137,.62,.48,.25,"65727f",9,4,rough=.18)
    for i,(x,z,h,r) in enumerate([(-.27,.09,.62,.126),(0,0,1.02,.18),(.24,-.06,.72,.13),(.30,.22,.38,.105),(-.08,.27,.45,.106)]):
        crystal.pole((x,z,.18),(x+.07,z-.05,.15+h*.72),r,
                     ["79c1c4","8fd5cf","67b4b8"][i%3],5,tip=r*.81)
        crystal.pole((x+.07,z-.05,.15+h*.72),(x+.11,z-.04,.18+h),r*.81,"acded5",5,tip=.007)
    rain=Model("Regen · Lichtstreifen","weather_rain")
    rain.pole((-.052,0,0),(.069,.018,.55),.012,"accdd6",4,tip=.008,mode="raw")
    snow=Model("Schneeflocke","weather_snow")
    snow.ellipsoid(0,0,0,.052,.05,.049,"e8e9df",7,4,mode="raw")
    return [apple,berries,hen,sheep,strawberry,tomato,sunflower,silver,crystal,rain,snow]


def deep_mine_assets(existing):
    result=[]
    for source,target,tint in [("mine_floor","mine_deep_floor","4b506d"),("mine_wall","mine_deep_wall","444761")]:
        m=next(obj for obj in existing if obj.asset==source)
        copy=Model("Tiefe Mine · "+m.name,target)
        copy.vertices,copy.faces=m.vertices[:],m.faces[:]
        copy.colors=[(blend(color,tint,.70),mode) for color,mode in m.colors]
        result.append(copy)
    return result


def path_stones(m, points, palette=("bfb29a","d5c3a6","ccbaa1"), width=.6):
    for (x,z),(xx,zz) in zip(points,points[1:]):
        count=max(1,int(math.hypot(xx-x,zz-z)*2.1))
        for j in range(count):
            t=j/count
            for off in [-width*.33,width*.33]:
                a,b=x+(xx-x)*t+off,z+(zz-z)*t+RNG.uniform(-.09,.09)
                m.box(a,b,.025,RNG.uniform(.29,.54),RNG.uniform(.30,.45),.071,
                      RNG.choice(palette),.052,RNG.uniform(-.29,.29))


def travel_sign(m,x,z,style=0):
    for dx in [-.46,.46]:
        m.box(x+dx,z,.61,.14,.13,1.2,"a57950",.024)
        m.box(x+dx,z,1.22,.19,.17,.082,"c19865",.018)
    m.box(x,z+.029,1.03,1.16,.16,.55,["b9895c","85a081","bd8e65"][style%3],.039)
    m.face([(x-.30,z+.111,.99),(x+.05,z+.113,.99),(x+.05,z+.113,.89),
            (x+.35,z+.113,1.04),(x+.05,z+.113,1.22),(x+.05,z+.113,1.10),
            (x-.30,z+.113,1.10)],"efdab0","light")
    lantern(m,x+.63,z+.02,.91)


def travel_gates():
    m=Model("Wegweiser · Farnwald, Muschelbucht und Eichenhafen")
    for i,(x,z) in enumerate([(-18.4,-5.8),(3.8,14.2),(21.50,5.52)]):
        travel_sign(m,x,z,i)
        if i<2:
            for j in range(6):
                m.box(x+(j%2-.5)*.51,z+(j//2-.7)*.37,.029,.48,.36,.071,
                      ["bdaf96","d4bea3","c6b497"][j%3],.053,RNG.uniform(-.22,.27))
    return [m]


def fern(m,x,z,s,lift=0.):
    for i in range(6):
        angle=i*TAU/6+RNG.uniform(-.23,.23)
        dx,dz=math.cos(angle)*s,math.sin(angle)*s
        m.pole((x,z,lift+.01),(x+dx*.86,z+dz*.86,lift+s*.36),s*.018,"6b8854",5,tip=s*.012)
        for j in range(1,5):
            t=j/5
            xx,zz,y=x+t*dx,z+t*dz,lift+s*(.20+.27*math.sin(t*2.0))
            for side in [-1,1]:
                m.face([(xx,zz,y),(xx+dx*.20-dz*.30*side,zz+dz*.20+dx*.30*side,y+.067*s),
                        (xx+dx*.27,zz+dz*.27,y+.025*s)],
                       ["91aa64","a2b16a","718f52"][(i+j)%3],"leaf")


def farnwald():
    forest=Model("Farnwald · Tannen, Herbstbäume und Felsen","district_forest")
    trail=Model("Farnwald · Steine, Bachbrücke und Waldhütte","district_forest")
    water=landscape.stream
    for j in range(13):
        x=-2.47+j*.272
        trail.box(x,.81,.16,.261,1.53,.16,["c09566","aa7b55","d2a775"][j%3],.026)
        if j%4==0:
            for z in [.15,1.47]:
                trail.box(x,z,.46,.139,.139,.85,"986847",.026)
    for z in [.15,1.47]:
        trail.obstacle_rail(-2.48,z,.80,z)
        trail.pole((-2.48,z,.70),(.80,z,.70),.045,"c4a87c",6)
    path_stones(trail,[(-6.75,4.6),(-5.7,3.2),(-6.6,1.7),(-5.15,.9),(-2.53,.81)])
    path_stones(trail,[(.98,.81),(3.3,1.0),(5.75,1.79),(7.10,2.85),(7.1,-.5),(5.1,-3.9),(3.4,-3.0),(2.2,-4.7)])
    path_stones(trail,[(-4.7,1.7),(-4.86,-.3),(-4.49,-2.64),(-5.55,-4.0),(-7.2,-4.9)])
    for i,(x,z,s) in enumerate([(-10.4,-2.4,1.05),(-9.82,-4.32,1.1),(-8.0,-5.7,.99),(-6.24,-6.2,.96),
          (-4.37,-6.5,1.08),(-2.86,-6.67,1.0),(.36,-7.04,.97),(2.24,-6.5,1.08),
          (4.63,-6.49,1.05),(6.76,-5.7,1.20),(8.95,-3.65,1.03),(10.10,-1.6,.93),
          (9.50,1.47,.95),(8.84,4.02,.71),(-9.26,2.29,.92),(-8.94,4.3,.61),(-7.95,-1.9,.73),
          (-7.2,-3.2,.71),(3.1,4.23,.61),(5.91,4.96,.64),(-4.5,5.8,.58),(-2.83,-3.18,.62)]):
        pine(forest,x,z,0,s,i)
    for i,(x,z,sc) in enumerate([(-7.69,2.05,1.0),(-5.75,-4.5,.95),(.5,-4.9,1.12),(5.2,-2.5,.91),(6.2,3.9,.73),(-3.8,3.6,.79)]):
        forest.obstacle_circle(x,z,.27*sc)
        forest.pole((x,z,0),(x+.11,z,2.24*sc),.27*sc,"8b6449",9,tip=.142)
        for dx,dz,h,r in [(-.61,.14,2.13,.79),(.49,.1,2.47,.79),(.12,-.40,2.67,.88),(0,.24,3.0,.71)]:
            forest.ellipsoid(x+dx*sc,z+dz*sc,h*sc,r*sc,r*.87*sc,r*.70*sc,
                             ["c9a752","dbb35b","b9a057","e2bf71"][(i+int(h*10))%4],9,4,rough=.17)
    for i,(x,z,h) in enumerate([(-3.35,-6.74,.82),(-1.84,-7.02,1.20),(-.6,-7.16,1.44),(.78,-6.85,1.11),(-1.29,-5.95,.67)]):
        trail.obstacle_circle(x,z,.72)
        trail.ellipsoid(x,z,.14+h*.36,.93,.7,h,"969892",8,4,rough=.21)
    for _ in range(94):
        x,z=RNG.uniform(-10.3,10.1),RNG.uniform(-6.49,6.49)
        if (x/10.6)**2+(z/6.8)**2>.92 or abs(x-water(z))<1.03:continue
        if RNG.random()<.59:fern(forest,x,z,RNG.uniform(.25,.53))
        else:flowers(forest,x,z,0,RNG.uniform(.72,1.01),RNG.choice(["f1dfbb","d9a5bc","c2a5cd","e4b683"]))
    travel_sign(trail,-6.75,4.6,0)
    xx,zz=-7.52,-4.66
    for dx in [-.96,.96]:
        for dz in [-.65,.65]:
            trail.obstacle_circle(xx+dx,zz+dz,.11)
            trail.box(xx+dx,zz+dz,.93,.15,.16,1.87,"9c7350",.025)
    trail.obstacle_rail(xx-.96,zz-.67,xx+.96,zz-.67)
    for j in range(5):
        trail.box(xx,zz-.67,.30+j*.183,1.94,.12,.171,"ab815b",.022)
    for j in range(8):
        trail.box(xx+(j-3.5)*.279,zz,1.98,.267,1.73,.18,["b18c61","9e845e","c39b6d"][j%3],.031)
    trail.box(xx,zz-.17,.43,1.57,.43,.14,"c29b72",.027)
    lantern(trail,xx+.72,zz+.7,1.57)
    return [forest,trail]


def muschelbucht():
    props=Model("Muschelbucht · Fischerhütte, Dünen und Felsbogen","district_beach")
    ox,oz=landscape.BEACH
    for i,(x,z,r) in enumerate([(-9.8,-3.3,.92),(-8.2,-5.3,1.44),(-6.2,-6.1,1.58),(-4.26,-6.1,1.34),(-1.12,-6.45,1.4),
                      (1.27,-6.39,.92),(3.6,-5.77,1.06),(-6.84,-.05,.85),(1.77,-3.36,.77),
                      (4.74,2.75,.43),(5.12,4.53,.68),(-7.4,2.45,.55),(-3.84,5.69,.49),(6.5,-.55,.65)]):
        h=max(landscape.SEA-.12,landscape.height(ox+x,oz+z))+r*.32
        props.ellipsoid(x,z,h,r,r*.74,r*.92,["b5a596","a99e91","c5b3a0"][i%3],9,4,rough=.23)
    for i in range(10):
        a=i*math.pi/9
        x,z,y=9.6+math.cos(a)*1.90,-.9,math.sin(a)*2.18
        props.ellipsoid(x,z,y-0.1,.63,.64,.79,["a7a6a8","bab0a6","95969d"][i%3],8,4,rough=.21)
    for i in range(52):
        x,z=RNG.uniform(-9.0,5.2),RNG.uniform(-5.4,5.6)
        if (x/10.5)**2+(z/6.57)**2>.93 or abs(x+2.7)<1.9 and z<-2.5:continue
        if i%4==0:
            for j in range(4):
                xx=x+RNG.uniform(-.09,.09)
                props.pole((xx,z,0),(xx+.16,z+.08,.42+RNG.random()*.27),.024,"9c9f62",5,tip=.009)
        else: grass(props,x,z,0,RNG.uniform(.15,.34),"9aab65",4)
    cutoff=[(-8.0,5.12),(-6.60,3.71),(-4.7,1.41),(-3.0,-1.31),(-2.56,-2.48)]
    path_stones(props,cutoff,("d1bda0","d6c7ab","c4b495"),.62)
    travel_sign(props,-6.65,4.90,1)
    x,z=-2.71,-4.49
    for dx in [-1.43,1.43]:
        for dz in [-.92,.98]:
            props.box(x+dx,z+dz,.80,.197,.20,1.6,"937154",.033)
    for k in range(11):
        h=.54+.15*k
        props.box(x,z-.89,h,2.80,.15,.144,"c5af83",.021)
        for dx in [-1.35,1.35]:
            props.box(x+dx,z,h,.15,1.85,.144,["bdb08b","d0ba96"][k%2],.023)
        if k>2:
            for dx in [-.97,.9]:
                props.box(x+dx,z+.88,h,.72,.13,.144,"d6c4a2",.019)
    props.box(x,z+.91,1.19,.87,.12,1.27,"93795a",.048)
    window(props,x+.77,z+.99,1.46,.40,.49)
    for side in [-1,1]:
        props.face([(x,z-1.12,2.69),(x+side*1.6,z-1.12,2.08),
                    (x+side*1.6,z+1.20,2.08),(x,z+1.20,2.69)],"8b9f76")
        for j in range(9):
            dz=(j-4)*.263
            props.pole((x,z+dz,2.72),(x+side*1.65,z+dz,2.09),.135,
                       ["909e75","9aa67d","a9af84"][j%3],6)
    props.pole((x,z-1.19,2.81),(x,z+1.29,2.81),.124,"af8967",8)
    for j in range(14):
        props.box(x+(j-6.5)*.23,z+1.15,.51,.216,1.21,.166,"bd946a",.024)
    for j in range(5):
        props.box(x,z+1.76+j*.21,.43-j*.091,1.22,.23,.145,"c39a6e",.024)
    for xt,zt in [(x-1.51,z+1.67),(x+1.51,z+1.67)]:
        props.box(xt,zt,.77,.125,.136,.52,"997956",.022)
    lantern(props,x-.89,z+1.08,1.63)
    barrel(props,x+1.33,z+1.53,.59,.71)
    for j in range(13):
        xx=4.8+j*.289
        props.box(xx,-3.15,.114,.275,1.36,.161,
                  ["b4885f","c69c72","d0a77b"][j%3],.024)
        if j%4==0:
            for zz in [-3.77,-2.55]:
                props.pole((xx,zz,-1.29),(xx,zz,.57),.112,"8f7255",8)
    for x,z in [(1.86,1.48),(-.25,3.10),(-3.55,3.86),(3.52,4.62),(5.37,-2.25)]:
        for k in range(6):
            a=k*TAU/6
            props.pole((x,z,.028),(x+math.cos(a)*.32,z+math.sin(a)*.32,.31+RNG.random()*.15),
                       .035,"d69883",6,tip=.021)
    return [props]


def city_house(m,x,z,w,d,plaster,roof,lift,detail):
    y=0.38
    wall=2.22+lift
    m.box(x,z,.17,w+.09,d+.05,.39,"afa192",.052)
    m.box(x,z,y+wall*.5,w,d,wall,plaster,.034)
    for sign in [-1,1]:
        m.box(x+sign*(w*.5-.062),z+d*.5+.037,1.49+lift*.5,.157,.15,2.37+lift,"9f7857",.026)
    m.box(x,z+d*.5+.07,.48,w+.13,.18,.23,"a98665",.026)
    m.box(x,z+d*.5+.07,2.48+lift,w+.16,.189,.22,"aa7b54",.030)
    m.box(x,z+d*.5+.093,1.02,.69,.13,1.20,"89664e",.018)
    for j in range(5):
        m.box(x+(j-2)*.118,z+d*.5+.171,1.03,.115,.042,1.08,"6d7866",.012)
    m.ellipsoid(x+.195,z+d*.5+.20,.98,.033,.023,.033,"d6b36c",7,4)
    for dx in [-w*.31,w*.31]:
        wx=x+dx
        window(m,wx,z+d*.5+.045,1.61,w*.193,.61)
        m.box(wx,z+d*.5+.238,1.226,.61,.225,.13,"936a4b",.017)
        for j in range(4):
            flowers(m,wx+(j-1.5)*.138,z+d*.5+.25,1.30,.57,["fff0cb","ddaa95","f3cf86"][j%3])
    for dz in [-d*.5,d*.5]:
        m.face([(x-w*.5,z+dz,2.55+lift),(x+w*.5,z+dz,2.55+lift),(x,z+dz,3.64+lift)],plaster)
        m.pole((x-w*.54,z+dz+.04,2.47+lift),(x,z+dz+.04,3.67+lift),.094,"ad8056",4)
        m.pole((x+w*.54,z+dz+.04,2.47+lift),(x,z+dz+.04,3.67+lift),.094,"ad8056",4)
    for side in [-1,1]:
        m.face([(x,z-d*.5-.24,3.64+lift),(x+side*w*.59,z-d*.5-.24,2.42+lift),
                (x+side*w*.59,z+d*.5+.29,2.42+lift),(x,z+d*.5+.29,3.64+lift)],roof)
        for i in range(12):
            zz=z+(i-5.5)*(d+.42)/12
            tint=blend(roof,"e6ad7f",.10+(i%4)*.063)
            m.pole((x,zz,3.66+lift),(x+side*w*.594,zz,2.43+lift),(d+.43)/23,tint,7)
    m.pole((x,z-d*.5-.29,3.77+lift),(x,z+d*.5+.31,3.77+lift),.148,blend(roof,"c79568",.55),9)
    if detail=="dormer":
        window(m,x,z+d*.5+.05,2.97+lift,.41,.40)
        for dx in [-.30,.30]:
            m.box(x+dx,z+d*.5+.13,2.97+lift,.12,.08,.41,"8a9a7a",.018)
    elif detail=="chimney":
        m.box(x-w*.26,z-.32,3.48+lift,.37,.42,.76,"a89689",.028)
        m.box(x-w*.26,z-.32,3.9+lift,.46,.50,.128,"8d8075",.022)
    for dx in [-.61,.61]:
        lantern(m,x+dx,z+d*.5+.22,1.43)


def eichenhafen():
    town=Model("Eichenhafen · Giebelhäuser, Docks und Rathaus","district_town")
    court=Model("Eichenhafen · Pflaster, Brunnen und Marktstände","district_town")
    for j in range(27):
        z=-4.55+j*.385
        for i in range(37):
            x=(i-18)*.551+(j%2)*.105
            if (x/10.72)**2+(z/6.65)**2>1.01:continue
            court.box(x,z,.035,.538,.375,.077,
                      ["d9c4a8","c9b8a5","d2c1ab","dfc9b0","bfae9e"][(j+2*i)%5],.036,RNG.uniform(-.042,.041))
    for house in [(-7.47,-3.20,3.36,2.8,"c0cfad","ac795d",.14,"dormer"),
        (-3.44,-4.26,3.65,2.65,"ebc9a8","b98569",-.15,"chimney"),
        (.61,-4.65,3.77,2.65,"f0d9af","be805e",.18,"bell"),
        (4.61,-4.18,3.40,2.55,"e6cea3","87987f",.08,"dormer"),
        (8.19,-2.98,3.1,2.63,"d9b9a4","a67364",-.12,"chimney")]:
        city_house(town,*house)
    town.box(.67,-4.55,3.71,1.22,1.24,1.35,"ddc49d",.040)
    town.box(.67,-3.90,3.97,.67,.15,.62,"8a6c55",.038)
    town.pole((.67,-3.76,3.80),(.67,-3.76,4.17),.195,"cc9a5d",9,tip=.109)
    town.pole((.67,-4.55,4.35),(.67,-4.55,5.05),.92,"cb8e68",4,tip=.035)
    town.pole((.67,-4.55,5.04),(.67,-4.55,5.29),.058,"9a7954",6)
    for x,z,colors in [(-5.97,-2.20,("979d70","f2ddb6")),(6.45,-2.13,("ca8b65","ead7b2"))]:
        for dx in [-1.09,1.09]:
            court.box(x+dx,z,.97,.11,.12,1.92,"a9825f",.018)
        for j in range(8):
            xx=x+(j-3.5)*.28
            court.face([(xx-.14,z-.62,2.15),(xx+.14,z-.62,2.15),(xx+.14,z+.64,1.83),(xx-.14,z+.64,1.83)],
                       colors[j%2])
            court.box(xx,z+.625,1.79,.270,.048,.108,colors[j%2],.017)
        for i in range(3):
            produce_crate(court,x+(i-1)*.62,z-.10,.37,["cf7a5f","87a26b","d8ac67"][i%3])
        court.box(x,z,.23,2.0,.59,.43,"ad835b",.033)
    x,z=0,.30
    sides=16
    def ring(radius,h):
        return [(x+radius*math.cos(a),z+radius*math.sin(a),h) for i in range(sides) for a in [i*TAU/sides]]
    court.rings([ring(1.37,.05),ring(1.35,.15),ring(1.25,.18)],"bcae9f")
    for j in range(3):
        court.rings([ring(1.195,.17+j*.16),ring(1.16,.318+j*.16),
                     ring(1.049,.332+j*.16),ring(1.045,.26+j*.16)],
                    ["b1a59b","c1b29d","c9b9aa","a5988d"]*12,cap=False)
    court.face(ring(1.046,.587),"86bbaf","raw")
    court.pole((x,z,.28),(x,z,1.28),.245,"b4ad9c",10,tip=.13)
    court.pole((x,z,1.25),(x,z,1.37),.52,"c5b9a5",14,tip=.48)
    court.pole((x,z,1.31),(x,z,1.39),.43,"92c5b2",14,mode="raw")
    court.ellipsoid(x,z,1.55,.098,.31,.14,"a4b2a1",9,4,angle=.03)
    court.face([(x-.05,z-.25,1.58),(x-.17,z-.48,1.74),(x+.12,z-.47,1.73),(x+.052,z-.24,1.61)],"abb9aa")
    for i in range(5):
        a=i*TAU/5
        dx,dz=math.cos(a),math.sin(a)
        curve=[(dx*.42,dz*.42,1.39),(dx*.63,dz*.63,1.19),(dx*.75,dz*.75,.83),(dx*.80,dz*.80,.57)]
        for start,end in zip(curve,curve[1:]):
            court.pole(start,end,.028,"a4d3c7",5,tip=.021,mode="light")
    for x,z in [(-8.03,1.36),(-4.72,2.74),(2.69,1.64),(8.61,1.47),(6.61,4.02),(-2.91,-2.61)]:
        court.box(x,z,.18,.52,.49,.314,"a9845d",.045)
        court.box(x,z,.346,.47,.47,.046,"726748",.021)
        for j in range(6):
            flowers(court,x+RNG.uniform(-.21,.21),z+RNG.uniform(-.20,.20),.38,
                    RNG.uniform(.62,.91),["e6c6bb","fff1cc","c1a4c7"][j%3])
    travel_sign(town,.13,5.81,2)
    for x,z in [(-2.41,4.74),(2.58,5.23),(-8.01,-.24),(7.88,-.64)]:
        town.box(x,z,1.12,.139,.153,2.22,"927151",.021)
        lantern(town,x+.20,z,1.84)
    fence(town,[(-7.37,4.46),(-5.98,5.16),(-4.69,5.79),(-3.34,6.22),(-1.50,6.33)])
    fence(town,[(1.58,6.33),(3.58,6.05),(4.89,5.59),(6.35,4.92),(7.71,4.38)])
    for x,z in [(-1.11,6.58),(2.51,5.72),(-8.11,2.65)]:
        barrel(town,x,z,.08,.68)
    for x,z in [(-8.86,2.59),(8.91,2.94)]:
        court.obstacle_circle(x,z,.46)
        court.pole((x,z,.05),(x,z,.42),.43,"ae8767",10,tip=.51)
        court.pole((x,z,.391),(x,z,.415),.44,"777153",10)
        court.pole((x,z,.40),(x+.04,z,1.55),.092,"977153",8,tip=.07)
        for dx,dz,h,rad in [(-.33,.04,1.31,.49),(.33,.05,1.47,.47),(0,-.12,1.76,.57)]:
            court.ellipsoid(x+dx,z+dz,h,rad,rad*.81,rad*.79,"9daa72",9,4,rough=.17)
    for x,z in [(-5.39,4.48),(5.61,4.39)]:
        for dx in [-.52,.52]:
            court.box(x+dx,z,.215,.219,.40,.34,"a39a90",.043)
        court.box(x,z,.41,1.51,.44,.13,"ab865f",.033)
        court.box(x,z+.205,.72,1.51,.11,.16,"bea27c",.025)
        for dx in [-.58,.58]:
            court.box(x+dx,z+.20,.64,.106,.11,.52,"987b60",.018)
    boat_x,boat_z=12.12,24.57
    hull=[[ (boat_x+math.sin(a)*width,boat_z+math.cos(a)*length,h)
            for i in range(12) for a in [i*TAU/12] ]
          for width,length,h in [(.47,1.04,-.96),(.68,1.30,-.63),(.72,1.34,-.39),
                                 (.59,1.23,-.37),(.52,1.09,-.70)]]
    town.rings(hull,["987056"]*12+["aa8261"]*12+["bf9f72"]*12+["a78361"]*12,cap=False)
    town.face(hull[-1],"b68e67")
    for dz in [-.48,.48]:
        town.box(boat_x,boat_z+dz,-.405,1.12,.23,.074,"cea67b",.022)
    town.pole((boat_x-.51,boat_z-.57,-.40),(10.19,23.51,.21),.026,"c7b38d",6)
    return [town,court]


def mainland_details():
    forest=Model("Farnwald · Alte Fichten, Steinlichtungen und Fernwege","district_forest")
    bank=Model("Farnwald · Zwei weitere Bachbrücken und der Rastplatz","district_forest")
    dunes=Model("Muschelbucht · Dünen, Steg und Fischerunterstand","district_beach")
    court=Model("Eichenhafen · Seitenstraßen, Hafengasse und Wohnhäuser","district_town")
    rng=random.Random(14731)
    fx,fz=landscape.FOREST
    bx,bz=landscape.BEACH
    tx,tz=landscape.TOWN
    trees=[]
    for i in range(320):
        x,z=rng.uniform(-28.5,23.),rng.uniform(-25.5,22.)
        wx,wz=fx+x,fz+z
        h=landscape.height(wx,wz)
        if (abs(x)<11.9 and abs(z)<8.2) or h<-.09 or landscape.edge(wx,wz)<1.9:
            continue
        if abs(x-landscape.stream(z))<1.88 or landscape.road_distance(wx,wz)<1.53:
            continue
        if landscape.is_gathering_spot(wx,wz) or any((x-a)**2+(z-b)**2<2.22**2 for a,b in trees):
            continue
        trees.append((x,z))
        sc=rng.uniform(.63,1.18)
        if i%5:
            pine(forest,x,z,h,sc,i)
        else:
            forest.pole((x,z,h),(x+.04,z,h+2.04*sc),.24*sc,"8e6950",8,tip=.14)
            for dx,dz,r,y in [(-.51,.15,.83,1.87),(.49,.02,.86,2.26),(.01,-.21,.88,2.56)]:
                forest.ellipsoid(x+dx*sc,z+dz*sc,h+y*sc,r*sc,r*.85*sc,r*.80*sc,
                                 rng.choice(["b1a053","caa350","d8ad5d","dec17d"]),9,4,rough=.16)
        landscape.block_circle(wx,wz,.42*sc)
        if len(trees)>=92:
            break
    for i in range(208):
        x,z=rng.uniform(-29,25.),rng.uniform(-24.2,22.)
        wx,wz=fx+x,fz+z
        if landscape.edge(wx,wz)<.62 or landscape.road_distance(wx,wz)<.95 or abs(x-landscape.stream(z))<1.25:
            continue
        if landscape.is_gathering_spot(wx,wz):
            continue
        h=landscape.height(wx,wz)
        if i%8==0:
            r=rng.uniform(.36,.93)
            forest.ellipsoid(x,z,h+r*.21,r,r*.74,r*.68,rng.choice(["aaa498","a19b93","a4a49d"]),8,4,rough=.22)
            landscape.block_circle(wx,wz,r*.67)
        elif i%3:
            fern(forest,x,z,rng.uniform(.26,.57))
        else:
            flowers(forest,x,z,h,rng.uniform(.63,1.06),rng.choice(["dfbd99","decaa6","c2a4c4","e7dca4"]))

    for z in landscape.BRIDGES[1:]:
        x=landscape.stream(z)
        for j in range(13):
            xx=x+(j-6)*.276
            bank.box(xx,z,.151,.267,1.56,.176,["b88b5c","a87e58","c39b71"][j%3],.027)
            if j%4==0:
                for dz in [-.71,.71]:
                    bank.box(xx,z+dz,.43,.142,.136,.78,"a37853",.021)
        for dz in [-.71,.71]:
            bank.obstacle_rail(x-1.71,z+dz,x+1.71,z+dz)
            bank.pole((x-1.71,z+dz,.65),(x+1.71,z+dz,.65),.047,"c5a37a",6)
        for dx in [-1.97,1.97]:
            bank.box(x+dx,z,.066,.41,1.52,.16,"b9956f",.028)
    for x,z in [(-16.23,-14.82),(14.33,8.83)]:
        for dx in [-.63,.63]:
            bank.box(x+dx,z,.235,.20,.39,.34,"9c8f7d")
        bank.box(x,z,.45,1.57,.49,.14,"b18b65",.024)
        bank.box(x,z-.19,.74,1.57,.113,.18,"b8926c",.029)
        bank.box(x+.99,z-.06,.69,.132,.135,1.3,"9b7959",.027)
        lantern(bank,x+1.08,z+.11,1.11)

    for i in range(200):
        x,z=rng.uniform(-26.,14.9),rng.uniform(-10.5,22.)
        wx,wz=bx+x,bz+z
        if landscape.beach_mix(wx,wz)<.24 or landscape.edge(wx,wz)<.36:
            continue
        if (abs(x)<9 and abs(z)<6.8) or landscape.is_gathering_spot(wx,wz) or landscape.road_distance(wx,wz)<.85:
            continue
        h=landscape.height(wx,wz)
        if i%10==0:
            r=rng.uniform(.30,1.07)
            dunes.ellipsoid(x,z,h+r*.32,r,r*.81,r*.84,rng.choice(["beb0a0","b1a499","a99e94"]),9,4,rough=.18)
            landscape.block_circle(wx,wz,r*.67)
        else:
            grass(dunes,x,z,h,rng.uniform(.28,.62),rng.choice(["b0aa64","a6a86c","aaa675"]),5)
    for x,z in [(-12.5,-1.9),(-16.8,9.6),(-3.5,16.),(-15.5,2.45)]:
        dunes.ellipsoid(x,z,.06,1.14,.77,.53,"b8a993",8,4,rough=.21)
        landscape.block_circle(bx+x,bz+z,.77)
    for j in range(22):
        x=5.87+j*.264
        dunes.box(x,1.39,.11,.253,1.43,.147,["be956b","a88059","c6a176"][j%3],.02)
        if j%5==0:
            for z in [.79,1.98]:
                dunes.pole((x,z,-1.09),(x,z,.52),.12,"9b7554",8)
    for x,z in [(-12.2,7.5),(-9.8,7.5)]:
        dunes.pole((x,z,0),(x,z,1.78),.086,"987452",7)
    dunes.pole((-12.2,7.5,1.63),(-9.8,7.5,1.63),.036,"ddbe8f",6)
    for i in range(13):
        x=-12.12+i*.18
        dunes.pole((x,7.49,1.58),(x+.08,7.49,.83+.12*(abs(i-6)/6)),.015,"b5986f",5)
    for i in range(5):
        h=.91+i*.142
        dunes.pole((-12.1,7.495,h),(-9.91,7.495,h),.014,"b99b71",5)
    for x,z in [(-11.6,7.07),(-12.3,7.15)]:
        barrel(dunes,x,z,.04,.66)

    for house in landscape.TOWN_BUILDINGS:
        city_house(court,*house)
    for x,z,width,length in [(-10.5,-4.55,2.0,12.), (11.3,-2.47,2.15,15.8),
                             (1.6,8.05,18.3,3.05), (9.2,14.7,2.18,11.1),(-2.26,-6.65,13.7,1.65)]:
        for i in range(max(3,int(width/.52))):
            for j in range(max(3,int(length/.45))):
                wx=x-width/2+i*.52
                wz=z-length/2+j*.45
                if landscape.height(tx+wx,tz+wz)<-.10:
                    continue
                court.box(wx,wz,.022,.502,.43,.055,["d6c1a5","dfc9ab","c9b69d"][(i+2*j)%3],.018)
    for i,(x,z) in enumerate([(-12.2,2.25),(-8.70,7.35),(-5.2,10.2),(-.35,10.0),(5.4,8.8),
                              (12.1,5.4),(14.15,9.55),(10.3,13.1),(-10.9,-3.5),(10.7,-5.2)]):
        court.pole((x,z,.04),(x,z,.35),.46,"9b8769",10,tip=.52)
        court.pole((x,z,.34),(x,z,1.68),.092,"936f54",8,tip=.066)
        for dx,dz,y,r in [(-.29,.07,1.50,.58),(.33,.04,1.65,.54),(.02,-.12,1.94,.62)]:
            court.ellipsoid(x+dx,z+dz,y,r,r*.78,r*.73,
                            ["a6ae73","b3ad68","b9a169","9ea876"][i%4],9,4,rough=.17)
        landscape.block_circle(tx+x,tz+z,.50)
    for x,z in [(-11.3,7.7),(-2.1,8.6),(5.1,12.8),(10.8,16.6),(8.1,19.78)]:
        court.box(x,z,1.04,.132,.142,2.03,"937455",.023)
        lantern(court,x+.19,z,1.75)
    for j in range(18):
        z=19.3+j*.252
        court.box(9.0,z,.113,2.54,.243,.153,["c1996d","b68b62","cda377"][j%3],.020)
        if j%5==0:
            for x in [7.83,10.2]:
                court.pole((x,z,-1.55),(x,z,.60),.132,"917453",8)
    for x,z in [(7.1,19.04),(7.7,18.3),(10.82,18.9)]:
        barrel(court,x,z,.061,.72)
    for x,z,c in [(10.78,19.4,"c38561"),(7.35,18.31,"a0a16a")]:
        produce_crate(court,x,z,.08,c)
    travel_sign(court,8.42,17.73,2)
    # One more wooded connector brings the darker greens toward the pasture.
    meadow=Model("Waldwinkel · Hecken und Weidepfad","island")
    for i in range(195):
        x,z=rng.uniform(-36.,38.),rng.uniform(-26.,41.0)
        if (x/22.)**2+(z/16.5)**2<1.13 or (x<-36 and z<15) or x>32 and z<18:
            continue
        if landscape.edge(x,z)<1.5 or landscape.road_distance(x,z)<1.35:
            continue
        y=landscape.height(x,z)
        if i%7==0 and z<33:
            sc=rng.uniform(.48,.83)
            pine(meadow,x,z,y,sc,i)
            landscape.block_circle(x,z,.31*sc)
        elif i%3==0:
            flowers(meadow,x,z,y,rng.uniform(.69,1.1),rng.choice(["edd6b1","d7adc2","dacba0"]))
        else:
            grass(meadow,x,z,y,rng.uniform(.19,.37),"9da75e",4)
    # Distinct groves frame the longer stretches of road and the back of town.
    for cx,cz,rad,count in [(-28,17,6,12),(-21,25,7.4,14),(-31,31,6.5,12),
                            (-10,36,6.1,11),(16,30,5,9),(29,19,7,12),
                            (41,-10,5.5,12),(58,-10,8.1,18),(66,-1,5,9)]:
        planted=[]
        for j in range(count*5):
            angle=rng.random()*TAU
            radius=rad*math.sqrt(rng.random())
            x,z=cx+math.cos(angle)*radius,cz+math.sin(angle)*radius
            if landscape.edge(x,z)<1.7 or landscape.road_distance(x,z)<1.65 or landscape.is_gathering_spot(x,z):
                continue
            if any(math.hypot(x-a,z-b)<1.45 for a,b in planted):
                continue
            planted.append((x,z))
            y=landscape.height(x,z)
            sc=rng.uniform(.49,.88)
            if j%3==0:
                meadow.pole((x,z,y),(x,z,y+1.78*sc),.182*sc,"987654",8,tip=.089)
                for dx,dz,hh,r in [(-.40,.08,1.49,.66),(.38,.08,1.82,.73),(.07,-.08,2.12,.70)]:
                    meadow.ellipsoid(x+dx*sc,z+dz*sc,y+hh*sc,r*sc,r*.87*sc,r*.70*sc,
                                     ["afad68","babb73","c8a65e","d4b268"][j%4],9,4,rough=.14)
            else:
                pine(meadow,x,z,y,sc,j)
            landscape.block_circle(x,z,.31*sc)
            if len(planted)>=count:
                break
    for x,z,r in [(-35,15,1.5),(-26,23,1.05),(-20,32,1.17),(-12,28,1.23),
                  (15,30,1.1),(25,22,1.37),(36,-7,1.6),(61,-12,1.28),(71,2,1.07)]:
        if landscape.road_distance(x,z)<1.95:
            continue
        y=landscape.height(x,z)
        for j in range(3):
            dx=(j-1)*r*.85
            meadow.ellipsoid(x+dx,z+(j%2)*.60,y+r*.22,r,r*.82,r*.60,
                             ["a79e8e","aca798","b4ab98"][j],9,4,rough=.22)
            landscape.block_circle(x+dx,z+(j%2)*.60,r*.62)
    for x,z in [(-28.7,-1.85),(-2.6,31.2),(14.2,36.4),(30.2,27.1),(35.6,7.6)]:
        y=landscape.height(x,z)
        meadow.box(x,z,y+.289,.159,.158,.58,"9a7b56",.025)
        meadow.box(x,z,y+.53,.196,.194,.103,"c0a378",.019)
    return [forest,bank,dunes,court,meadow]


def find_assets():
    fungi=Model("Farnwald · Pilze","find_mushroom")
    for j,(x,z,h,s) in enumerate([(-.17,.09,.32,.18),(.12,-.10,.38,.23),(.29,.15,.22,.11)]):
        fungi.pole((x,z,.025),(x+.04,z,h),.056*s/.18,"dfc59e",7,tip=.044*s/.18)
        fungi.ellipsoid(x+.04,z,h-.014,s,s*.96,s*.47,["b785a2","d59a71","a97b9f"][j],9,4)
        for k in range(5):
            a=k*TAU/5+.21
            fungi.ellipsoid(x+.04+s*.55*math.cos(a),z+s*.55*math.sin(a),h+s*.36,.032,.029,.022,"ead6aa",7,3)
    nuts=Model("Farnwald · Haselnüsse","find_hazel")
    for i,(x,z) in enumerate([(-.16,.02),(.05,.12),(.23,-.076)]):
        nuts.ellipsoid(x,z,.11,.112,.104,.112,["ae8353","916946","c59962"][i],9,4,rough=.08)
        nuts.ellipsoid(x-.017,z-.038,.169,.081,.076,.057,"c3a465",8,3)
        grass(nuts,x,z,.024,.13,"8e9d61",4)
    shells=Model("Muschelbucht · Herzmuscheln","find_shell")
    for j,(x,z,r) in enumerate([(-.16,.05,.183),(.19,-.07,.151)]):
        palette=("dc9c87","f1ccb5","e7b9a2") if j==0 else ("eac3a3","f5e1c4","ecd3b5")
        for i in range(10):
            a=math.pi*(.07+i*.088)
            b=a+math.pi*.092
            c=(x,z-.04,.044)
            p=(x+math.cos(a)*r,z+math.sin(a)*r,.051+math.sin(a)*r*.48)
            q=(x+math.cos(b)*r,z+math.sin(b)*r,.051+math.sin(b)*r*.48)
            shells.face([c,p,q],palette[i%3],"light")
            shells.pole(c,p,.014,palette[(i+1)%3],5,tip=.010)
    drift=Model("Muschelbucht · Treibholz","find_driftwood")
    drift.pole((-.36,-.10,.049),(.33,.12,.16),.083,"b58e68",7,tip=.06)
    drift.pole((-.06,-.02,.10),(-.18,.30,.16),.062,"ae8969",6,tip=.028)
    drift.pole((.14,.07,.11),(.34,-.16,.066),.044,"bea080",6,tip=.024)
    return [fungi,nuts,shells,drift]


def npc_models():
    specs=[
        ("npc_lina","rabbit","d8b69c","ad684f","ecd2ab","a37a6d"),
        ("npc_miro","fox","c88857","6d8d73","ebd3ac","86634a"),
        ("npc_mara","cat","968c88","85718a","e6cbb0","706761"),
        ("npc_fiete","squirrel","b77951","bd9158","f0cda2","805a44"),
        ("npc_bo","badger","726e6a","6a867a","e1d0b4","595a56"),
        ("npc_juna","cat","ad907a","b68464","e4c7a7","846254"),
        ("npc_runa","badger","8b7874","ad6d54","ecd2b3","69534d"),
        ("npc_nele","rabbit","b39d86","8b9e81","f0dcbc","78604d"),
        ("npc_edda","fox","a98283","738d80","e1c8b5","715c63"),
    ]
    models=[]
    legacy_rng=None
    for asset,species,fur,cloth,light,dark in specs:
        if asset=="npc_runa":
            legacy_rng=RNG.getstate()
        m=Model(asset,asset)
        for side in [-1,1]:
            m.ellipsoid(side*.114,.012,.275,.089,.106,.226,fur,8,4)
            m.box(side*.119,.067,.083,.160,.214,.123,dark,.034)
            m.ellipsoid(side*.239,-.004,.674,.084,.081,.228,cloth,8,4)
            m.ellipsoid(side*.246,.029,.502,.073,.067,.082,fur,8,4)
        m.ellipsoid(0,0,.662,.244,.162,.322,cloth,11,5)
        m.box(0,.152,.66,.292,.043,.306,light,.033)
        for side in [-1,1]:
            m.ellipsoid(side*.108,.166,.846,.026,.022,.028,"c8a465",8,4)
        m.ellipsoid(0,-.028,1.156,.269,.211,.256,fur,12,6)
        m.ellipsoid(0,.145,1.085,.207,.158,.153,light,11,5)
        m.ellipsoid(0,.276,1.128,.046,.025,.03,dark,8,4)
        for side in [-1,1]:
            if species=="badger":
                m.ellipsoid(side*.151,.134,1.204,.086,.083,.124,"565453",8,5)
            m.ellipsoid(side*.109,.207,1.203,.035,.025,.042,"483a32",8,4,mode="raw")
            m.ellipsoid(side*.106-.01,.229,1.222,.010,.007,.011,"fff0d2",7,3,mode="raw")
            if species=="rabbit":
                m.ellipsoid(side*.14,-.012,1.49,.073,.064,.234,fur,9,5,angle=side*.13)
                m.ellipsoid(side*.14,.043,1.515,.047,.024,.159,"d9a98f",8,4)
            else:
                m.pole((side*.169,-.001,1.31),(side*.228,-.034,1.53),
                       .093,fur,6,tip=.015)
        if species in ["fox","squirrel"]:
            m.ellipsoid(.22,-.29,.62,.127,.29,.159,fur,10,5,angle=.27)
            m.ellipsoid(.32,-.465,.63,.094,.104,.111,light,8,4)
        if asset=="npc_fiete":
            m.ellipsoid(.32,-.31,.88,.164,.16,.35,"be8757",10,5,rough=.11)
            m.ellipsoid(.41,-.35,1.12,.147,.138,.174,fur,9,4)
            for dx,h in [(-.09,.874),(0,.843),(.09,.871)]:
                m.ellipsoid(dx,.169,h,.038,.022,.027,"e8d7ac",8,4)
        if asset=="npc_lina":
            m.box(.241,.078,.48,.145,.132,.17,"ab8a5b",.029)
            m.ellipsoid(.222,.137,.511,.061,.024,.048,"9ea874",7,4)
        if asset=="npc_miro" or asset=="npc_mara":
            scarf="b65e4a" if asset=="npc_miro" else "b8a991"
            m.ellipsoid(0,.029,.935,.172,.134,.069,scarf,10,4)
            m.face([(-.086,.171,.932),(.104,.174,.934),(-.028,.188,.743)],scarf)
        if asset=="npc_juna":
            m.box(.225,-.086,.54,.169,.125,.222,"ad875f",.032)
            m.pole((-.142,.166,.893),(.217,.166,.606),.024,"ae8d66",5)
        if asset=="npc_bo" or asset=="npc_juna":
            m.pole((0,.027,1.373),(0,.027,1.428),.257,cloth,12,tip=.238)
            m.ellipsoid(0,-.023,1.45,.217,.155,.089,cloth,10,4)
            m.ellipsoid(0,.207,1.409,.042,.023,.04,"dfb96c",8,4)
        if asset=="npc_mara":
            for side in [-1,1]:
                ring=[(side*.105+.081*math.cos(a),.230,1.183+.070*math.sin(a)) for i in range(9) for a in [i*TAU/9]]
                for i in range(9):m.pole(ring[i],ring[(i+1)%9],.012,"775e4e",5)
            m.pole((-.029,.238,1.188),(.029,.238,1.188),.010,"775e4e",5)
        models.append(m)
    if legacy_rng is not None:
        RNG.setstate(legacy_rng)
    return models


def regional_assets():
    assets=[]
    for lit in [False,True]:
        bell=Model("Wegglocke" if not lit else "Reparierte Wegglocke","waybell_lit" if lit else "waybell")
        bell.pole((0,0,-.08),(0,0,.08),.62,"a59881",12,tip=.52)
        for x in [-.45,.45]:
            bell.box(x,0,.72,.17,.19,1.33,"8d6950",.028)
            bell.box(x,0,.13,.26,.26,.17,"a89c83",.04)
        bell.box(0,0,1.37,1.22,.24,.151,"b4845e",.041)
        for side in [-1,1]:
            bell.pole((0,0,1.64),(side*.63,0,1.309),.089,"a47452",5)
        bell.pole((0,0,1.36),(0,0,1.24),.042,"635d56",7)
        bell.pole((0,0,1.02),(0,0,1.24),.18,"caac65" if lit else "a29262",9,tip=.117)
        bell.pole((0,0,.96),(0,0,1.03),.23,"deba78" if lit else "9c8762",9,tip=.183)
        bell.pole((0,0,.88),(0,0,1.02),.027,"796d50",6)
        bell.ellipsoid(0,0,.883,.057,.055,.049,"ccab6a",7,4)
        if lit:
            lantern(bell,.51,.14,1.071)
            bell.box(0,.106,.31,.25,.040,.17,"90a989",.021)
        else:
            bell.box(0,.106,.31,.25,.040,.17,"b59d7c",.021)
        assets.append(bell)
    for opened in [False,True]:
        gate=Model("Freier Weg" if opened else "Wegschranke","frontier_open" if opened else "frontier_gate")
        for x in [-1.43,1.43]:
            gate.box(x,0,.81,.222,.27,1.62,"8b684e",.044)
            gate.box(x,0,.13,.38,.38,.26,"998c7a",.049)
            gate.box(x,0,1.60,.29,.31,.11,"c19a6c",.028)
            lantern(gate,x,.12,1.29)
        if not opened:
            for y in [.46,1.00]:
                gate.box(0,0,y,2.81,.147,.19,"a98359",.023)
            gate.pole((-1.28,.07,.31),(1.30,.07,1.13),.075,"c5a17a",5)
            gate.box(0,.13,.72,.51,.08,.34,"947552",.039)
            gate.box(0,.178,.74,.27,.014,.105,"d8b98c",.014)
        assets.append(gate)

    quartz=Model("Klammquarz · helle Kristalle und Kupferader","find_gorge_quartz")
    quartz.ellipsoid(0,0,.17,.46,.39,.27,"a48077",9,4,rough=.17)
    for x,z,h,r in [(-.14,.12,.52,.102),(.09,.09,.65,.133),(.19,-.09,.41,.094),(-.02,-.18,.47,.083)]:
        quartz.pole((x,z,.24),(x+.065,z,h),r,"d4a18a",5,tip=r*.68)
        quartz.pole((x+.065,z,h),(x+.089,z-.01,h+.137),r*.68,"e5ba9e",5,tip=.008)
    flax=Model("Wiesenflachs · violettblaue Blüten","find_meadow_flax")
    herb=Model("Nebelminze · türkise Blattspitzen","find_marsh_herb")
    for i in range(9):
        a=i*TAU/9
        x,z=.21*math.cos(a),.21*math.sin(a)
        height=.37+.22*(i%3)/2
        flax.pole((x,z,0),(x+.075,z, height),.022,"719467",5,tip=.014)
        for j in range(5):
            t=j*TAU/5
            flax.ellipsoid(x+.075+.045*math.cos(t),z+.045*math.sin(t),height+.023,
                           .045,.038,.025,"9ba8d0" if i%2 else "af9cca",6,3)
        flax.ellipsoid(x+.075,z,height+.046,.027,.027,.021,"e5c47f",6,3)
        dx,dz=.53*math.cos(a),.53*math.sin(a)
        herb.face([(x/2,z/2,.022),(x+dx*.50-dz*.15,z+dz*.50+dx*.15,height*.5),
                   (dx,dz,height*.79),(x+dx*.53+dz*.16,z+dz*.53-dx*.16,height*.53)],
                  ["75a593","8ab392","669381"][i%3],"leaf")
        herb.pole((x/2,z/2,.043),(dx*.91,dz*.91,height*.75),.017,"aec2a0",5,tip=.005)
    return assets+[quartz,flax,herb]


def effects():
    clouds=[]
    for i in range(3):
        m=Model("Wolke "+str(i+1),"cloud_"+str(i+1))
        for x,z,h,s in [(-.77,.04,.075,.49),(-.34,-.15,.25,.69),(.32,-.08,.26,.76),(.89,.06,.07,.42)]:
            m.ellipsoid(x,z,h,s,s*.68,s*.62,["f4e6cc","f1e9d8","eceddd"][i],9,4,rough=.08)
        clouds.append(m)
    marker=Model("Auswahl · Leuchtende Beetecken","selection")
    for x in [-1,1]:
        for z in [-1,1]:
            marker.box(x*.727,z*.666,.030,.044,.252,.044,"fff0aa",.010,mode="raw")
            marker.box(x*.628,z*.778,.030,.231,.042,.044,"fff0aa",.010,mode="raw")
    shadow=Model("Gärtnerfigur · Bodenschatten","shadow")
    for j in range(32):
        a,b=j*TAU/32,(j+1)*TAU/32
        shadow.face([(0,0,0),(math.cos(a), math.sin(a)*.77,0),(math.cos(b), math.sin(b)*.77,0)],
                    "4c592f","raw")
    return [*clouds, marker, shadow]


def light_color(color, normal, pos, mode, bvh):
    if mode=="raw":
        return (*map(linear,color),1)
    if mode=="light":
        return (*map(linear, blend(color,"fff1c3",.12)),1)
    orient=abs(normal.dot(SUN)) if mode=="leaf" else max(0,normal.dot(SUN))
    diffuse=.75+.32*orient+.041*max(0,normal.z)
    shadow=0
    ambient=0
    if bvh and pos.z>-.50:
        origin=pos+normal*.019
        shadow=sum(bvh.ray_cast(origin,direction,16)[0] is not None for direction in SOFT_SUN)/3
        if mode=="ground":
            ambient=sum(bvh.ray_cast(origin,direction,1.15)[0] is not None for direction in AMBIENT_RAYS)/5
        elif mode!="leaf":
            ambient=sum(bvh.ray_cast(origin,direction,.48)[0] is not None
                        for direction in AMBIENT_RAYS if direction.dot(normal)>.1)/5
    shade=diffuse-(.095+.18*orient)*shadow-.16*ambient
    warmth=(.049,.022,-.013)
    result=tuple(linear(c*shade+warmth[i]*orient*(1-shadow)+.040*shadow*(i==2))
                 for i,c in enumerate(color))
    return (*result,1)


def build_object(model, bvh, material, collection, offset=Vector((0,0,0))):
    mesh=bpy.data.meshes.new(model.name)
    mesh.from_pydata(model.vertices,[],model.faces)
    mesh.update()
    obj=bpy.data.objects.new(model.name,mesh)
    collection.objects.link(obj)
    mesh.materials.append(material)
    attr=mesh.color_attributes.new(name="COLOR_0",type="FLOAT_COLOR",domain="CORNER")
    mesh.color_attributes.active_color=attr
    for p,(color,mode) in zip(mesh.polygons,model.colors):
        normal=p.normal.copy()
        sampling=p.center*.12
        for corner, loop_index in enumerate(p.loop_indices):
            v=mesh.vertices[mesh.loops[loop_index].vertex_index].co*.88+sampling+offset
            tint, style = (color[corner], "raw") if mode=="gradient" else (color, mode)
            attr.data[loop_index].color=light_color(tint,normal,v,style,bvh)
    return obj


def scenery_patches(model, width=24.0):
    """Export spatial bounds small enough for the mobile renderer to cull distant scenery."""
    if not model.asset.startswith("district_") or len(model.faces) < 2600:
        yield model
        return
    patches={}
    for face,color in zip(model.faces,model.colors):
        key=tuple(math.floor(sum(model.vertices[v][axis] for v in face)/len(face)/width)
                  for axis in (0,1))
        if key not in patches:
            patches[key]=(Model(f"{model.name} · {key[0]},{key[1]}",model.asset,model.casts),{})
        patch,mapping=patches[key]
        indices=[]
        for source in face:
            if source not in mapping:
                mapping[source]=len(patch.vertices)
                patch.vertices.append(model.vertices[source])
            indices.append(mapping[source])
        patch.faces.append(tuple(indices))
        patch.colors.append(color)
    yield from (patch for patch,_ in patches.values())


def glb_unlit(path, alpha=None):
    """The Blender source uses diffuse vertex color for editing. Game uses the baked light."""
    with path.open("rb") as f:
        magic, version, _ = struct.unpack("<4sII", f.read(12))
        size, chunk_type = struct.unpack("<I4s",f.read(8))
        assert magic==b"glTF" and chunk_type==b"JSON"
        doc=json.loads(f.read(size))
        remainder=f.read()
    doc.setdefault("extensionsUsed",[]).append("KHR_materials_unlit")
    for material in doc["materials"]:
        material.setdefault("extensions",{})["KHR_materials_unlit"]={}
        material["doubleSided"]=True
        if alpha is not None:
            material["alphaMode"]="BLEND"
            material.setdefault("pbrMetallicRoughness",{})["baseColorFactor"]=[1,1,1,alpha]
    data=json.dumps(doc,separators=(",",":"),ensure_ascii=True).encode()
    data += b" "*(-len(data)%4)
    path.write_bytes(struct.pack("<4sIII4s",magic,version,20+len(data)+len(remainder),len(data),b"JSON")+data+remainder)


def setup_preview(scene, scale=62.0, center=(.28,.03,1.24)):
    camera_data=bpy.data.cameras.new("Spielkamera")
    camera=bpy.data.objects.new("Spielkamera",camera_data)
    scene.collection.objects.link(camera)
    target=Vector(point(*center))
    camera.location=target+Vector(point(-8.78,22.77,16.26))*scale/26
    direction=target-camera.location
    camera.rotation_euler=direction.to_track_quat("-Z","Y").to_euler()
    camera_data.type="ORTHO"
    camera_data.ortho_scale=scale
    scene.camera=camera
    scene.world=bpy.data.worlds.new("Türkise Morgenluft")
    scene.world.color=(.55,.70,.68)
    scene.view_settings.view_transform="Standard"
    scene.render.resolution_x=1600
    scene.render.resolution_y=900



def dungeon_assets():
    styles=[]
    rng=random.Random(7324)
    for variant in range(3):
        floor=Model("Dämmergänge · Platten und Moos",f"dusk_floor_{variant}")
        floor.box(0,0,-.29,2.40,2.40,.57,["695b72","78657c","6f606f"][variant],.087)
        for x in range(4):
            for z in range(4):
                xx,zz=(x-1.5)*.581,(z-1.5)*.581
                face=["93828c","9d8490","aa8d95","88778b"][(x+z*2+variant)%4]
                floor.box(xx,zz,.006,.553,.555,.033,face,.067,rng.uniform(-.044,.044))
                if (x*4+z+variant)%4==0:
                    floor.ellipsoid(xx-.19,zz+.18,.030,.12,.075,.026,"829067",7,3,rough=.16)
        styles.append(floor)

    wall=Model("Dämmergänge · brüchiger Stein mit Quarz","dusk_wall")
    for i in range(5):
        x=(i-2)*.467
        height=.49+(i%3)*.12
        wall.ellipsoid(x,0,height*.54,.36,.43,height,
                       ["746680","806e83","84798a","94808a"][i%4],9,4,rough=.18)
        wall.ellipsoid(x+.13,.23,.19,.25,.23,.32,"8e7985",7,4,rough=.19)
        if i%2==0:
            wall.ellipsoid(x-.09,.03,height*.77,.23,.29,.10,"82966c",8,4,rough=.15)
    for x,z,h in [(-.37,.31,.47),(-.12,.34,.60),(.35,.36,.44)]:
        wall.pole((x,z,.09),(x+.026,z,h),.087,"6a9e96",6,tip=.043)
        wall.pole((x+.026,z,h),(x+.044,z,h+.099),.043,"a3c9b2",5,tip=.007,mode="light")
    styles.append(wall)

    sconce = Model("Dämmergänge · Mauervorsprung und Laternenlicht", "dusk_lantern_wall")
    for i in range(6):
        x = (i-2.5)*.37
        h = .68 + .27 * (i in (1,4))
        sconce.ellipsoid(x,0,h*.27,.30,.48,.71,
                         ["665a72","70627b","817185"][i%3],8,4,rough=.17)
        sconce.ellipsoid(x,-.064,h*.61,.28,.39,.39,
                         ["887989","8f7b8d","9c8793"][i%3],8,4,rough=.14)
    sconce.box(0,0,.63,.58,.71,.13,"9d8890",.055)
    lantern(sconce,0,0,.68)
    for x,z,s in [(-.69,.27,.90),(-.71,-.29,.74),(.56,.29,.94),(.72,-.29,.68)]:
        sconce.ellipsoid(x,z,.104,.21*s,.14*s,.084,"788a66",8,3,rough=.12)
        for ox,oz,scale in [(-.07,.03,.70),(.014,-.04,.94),(.107,.044,.80)]:
            sconce.pole((x+ox,z+oz,.11),(x+ox+.021,z+oz,.11+.143*scale),.024,"d4bc94",6)
            sconce.ellipsoid(x+ox+.021,z+oz,.11+.16*scale,.092*scale,.077*scale,.046,
                             "d3a987" if x<0 else "bea4aa",8,3)
    styles.append(sconce)

    court = Model("Dämmergänge · Steinrosette", "dusk_rosette")
    for i in range(18):
        a = i * TAU / 18
        x,z = math.cos(a)*.88, math.sin(a)*.88
        court.box(x,z,.029,.244,.244,.036,["ad9390","877181","ad987d"][i%3],.047,a)
        if i%3 == 0:
            court.ellipsoid(x*.79,z*.79,.045,.052,.05,.021,"8da37a",7,3)
    court.pole((0,0,.026),(0,0,.041),.56,"82778a",12,tip=.54)
    for i in range(6):
        a = i * TAU / 6
        court.ellipsoid(math.cos(a)*.29,math.sin(a)*.29,.045,.252,.086,.014,
                        "cfaf85",7,3,angle=a)
    styles.append(court)

    arch=Model("Dämmergänge · alter Rundbogen","dusk_arch")
    for i in range(13):
        a=i*TAU/24
        xx,yy=math.cos(a)*.925,.67+math.sin(a)*1.07
        arch.box(xx,0,yy,.32,.37,.28,["9a8590","9d8e93","a38d90"][i%3],
                 .055,angle=0.)
        if i%3==0:
            arch.ellipsoid(xx+.071,.10,yy+.105,.19,.19,.081,"839271",7,3)
    for x in [-.91,.91]:
        arch.box(x,0,.33,.39,.47,.66,"8f7885",.052)
        arch.box(x,0,.11,.51,.56,.18,"9c858a",.043)
        lantern(arch,x-.08,.21,.69)
    styles.append(arch)

    for opened in [False,True]:
        seal=Model("Freigegebener Abstieg" if opened else "Versiegelter Abstieg",
                   "dusk_open" if opened else "dusk_seal")
        for i in range(5):
            z=-.48-i*.172
            seal.box(0,z,.02-i*.061,1.18,.204,.12,"9b8790" if i%2 else "ae9294",.032)
        for side in [-1,1]:
            seal.box(side*.65,-.34,.22,.18,.84,.49,"8d7a82",.049)
        y=.38 if opened else .44
        seal.box(0,-.345,y,1.17,.126,.16,"886745" if opened else "826471",.023)
        seal.ellipsoid(0,-.239,y+.073,.13,.062,.14,"9dc09a" if opened else "b78075",9,4)
        styles.append(seal)

    camp=Model("Glockenrast · Bank und ruhiges Licht","dusk_camp")
    camp.pole((0,0,-.04),(0,0,.088),1.04,"a99388",13,tip=.99)
    camp.pole((0,0,.092),(0,0,.101),.77,"b5a48e",13)
    camp.pole((0,0,.101),(0,0,.105),.60,"89977a",12)
    for i in range(7):
        a=i*TAU/7
        camp.ellipsoid(.34*math.cos(a),.34*math.sin(a),.113,.14,.09,.015,"d3bb87",6,3,angle=a)
    for x in [-.47,.47]:
        camp.box(x,-.56,.223,.14,.41,.39,"967153",.027)
        camp.box(x,-.725,.55,.091,.083,.52,"a27b5a",.027)
    camp.box(0,-.553,.404,1.14,.37,.13,"c39b70",.029)
    camp.box(0,-.723,.667,1.17,.11,.25,"b69169",.035)
    camp.box(.33,-.582,.494,.44,.32,.053,"d7ba87",.023)
    lantern(camp,.72,.18,.219)
    lantern(camp,-.72,.18,.219)
    styles.append(camp)

    binder=Model("Kartenbinder · Pult und Ledermappe","card_binder")
    binder.box(0,0,.085,.66,.49,.17,"926752",.044)
    binder.pole((0,0,.112),(0,0,.78),.104,"ab7e57",8,tip=.087)
    binder.box(0,0,.796,.75,.55,.113,"c49970",.033)
    binder.box(0,.045,.881,.629,.391,.070,"6b8272",.029)
    for x in [-.147,.147]:
        binder.box(x,.047,.922,.265,.34,.052,"ead5a6",.018)
        binder.box(x,.073,.953,.119,.166,.022,"d1ab77",.012,.08 if x<0 else -.08)
    binder.pole((0,-.123,.936),(0,.230,.936),.027,"887460",7)
    styles.append(binder)

    for index, asset in enumerate(("gear_staff", "gear_sickle", "gear_silver", "gear_crystal")):
        blade = Model(asset.replace("gear_", ""), asset)
        blade.pole((0,0,-.21),(0,0,.47),.046,"815e4b",8,tip=.021)
        for z in [-.04,.005,.050,.095]:
            blade.pole((0,0,z),(0,0,z+.028),.048,"ad8b67",7,tip=.047)
        if index == 0:
            blade.pole((0,0,.25),(.154,0,.41),.04,"9c7753",6,tip=.017)
            blade.pole((.026,0,.40),(.071,0,.61),.038,"ad865d",7,tip=.012)
            blade.ellipsoid(.043,.009,.455,.055,.027,.073,"90a373",6,3,angle=.28)
        elif index == 1:
            for bend in range(6):
                a = bend * .37
                b = (bend + 1) * .37
                blade.pole((.12*(1-math.cos(a)),0,.35+.18*math.sin(a)),
                           (.12*(1-math.cos(b)),0,.35+.18*math.sin(b)),
                           .056-bend*.006,"b68662" if bend%2 else "d5a77c",5,
                           tip=.05-bend*.007)
        else:
            metal = "b6c5c5" if index == 2 else "9fb5cf"
            blade.box(0,0,.20,.22,.082,.067,metal,.037)
            blade.pole((0,0,.22),(0,0,.54),.086,metal,4,tip=.065)
            blade.pole((0,0,.54),(0,0,.70),.066,"e2ddd6" if index == 2 else "badbd0",4,tip=.007)
            blade.ellipsoid(0,.034,.27,.066,.024,.057,"9d7669" if index == 2 else "7aabac",7,3)
        styles.append(blade)

    for kind,asset,fur,face,eye in [
        ("moss","duel_mosskin","80976b","d5c4a0","4e5148"),
        ("clay","duel_kiln","9c796b","d6a889","594b50"),
        ("owl","duel_owl","8b849e","c7bcd1","574b59"),
        ("keeper","duel_keeper","84718d","d1a985","564453"),
        ("keeper","duel_keeper_clay","a17663","deaa82","644e4f"),
        ("keeper","duel_keeper_mist","7d91a5","b4c8c3","514b62"),
        ("keeper","duel_keeper_root","7f8c61","c2b38b","535843"),
        ("keeper","duel_keeper_bell","88735e","e2ba84","5b4b50"),
    ]:
        m=Model(asset.replace("duel_",""),asset)
        big=1.33 if kind=="keeper" else 1.
        m.ellipsoid(0,-.10,.456*big,.42*big,.39*big,.42*big,fur,12,5,rough=.08)
        m.ellipsoid(0,.258,.56*big,.30*big,.24*big,.261*big,face,12,5)
        for side in [-1,1]:
            m.ellipsoid(side*.22*big,.20,.106,.14*big,.16,.102,fur,8,4)
            m.ellipsoid(side*.112*big,.454,.621*big,.052,.038,.061,eye,8,4,mode="raw")
            m.ellipsoid(side*.112*big-.012,.479,.642*big,.013,.012,.017,"fff0cc",7,3,mode="raw")
            if kind=="owl":
                m.ellipsoid(side*.356,-.02,.52,.22,.173,.352,fur,9,4,angle=side*.34)
                m.pole((side*.155,.081,.791),(side*.254,.098,.983),.098,fur,6,tip=.009)
            elif kind=="clay":
                m.box(side*.41,.007,.411,.198,.237,.247,"b0866e",.046)
                m.box(side*.28,.128,.746,.147,.147,.211,"ae8276",.039)
            elif kind=="keeper":
                m.pole((side*.301,-.16,.95),(side*.387,-.12,1.55),.091,"a58c9a",6,tip=.018)
            else:
                for j in range(3):
                    a=.45+j*.56
                    m.ellipsoid(side*.35*math.sin(a),-.13-.25*math.cos(a),.58+j*.086,
                                .16,.19,.068,"9baa75",7,4,angle=side*.33)
        if kind=="moss":
            m.pole((0,-.25,.72),(0,-.33,1.07),.035,"819160",6,tip=.014)
            m.ellipsoid(0,-.302,1.088,.145,.084,.044,"a9b47d",8,4,angle=.43)
        elif kind=="keeper":
            m.pole((0,-.06,1.09),(0,-.06,1.312),.235,"c39771",9,tip=.132)
            m.pole((0,-.06,1.08),(0,-.06,1.134),.258,"dab485",9)
            if asset=="duel_keeper_clay":
                for side in [-1,1]:
                    m.box(side*.44,.10,.659,.208,.184,.273,"c39c77",.046,.2*side)
                    m.box(side*.47,.22,.759,.141,.120,.155,"d0a886",.032)
            elif asset=="duel_keeper_mist":
                for side in [-1,1]:
                    for j in range(3):
                        m.ellipsoid(side*(.38+j*.055),-.21,.71+j*.16,.122,.19,.26,
                                    ["9babbe","aaa9bd","bbc6c9"][j],8,4,angle=side*.45)
            elif asset=="duel_keeper_root":
                for side in [-1,1]:
                    for j in range(4):
                        m.ellipsoid(side*(.18+j*.071),-.12,1.15+j*.103,
                                    .17,.115,.068,["9ca87a","afb183"][j%2],8,4,angle=side*.56)
            elif asset=="duel_keeper_bell":
                lantern(m,-.42,.16,.79)
                lantern(m,.42,.16,.79)
                m.pole((0,-.06,1.17),(0,-.06,1.56),.157,"deb877",9,tip=.059,mode="light")
        else:
            m.ellipsoid(0,.485,.515*big,.066,.048,.048,"bc8d6e",7,4)
        styles.append(m)
    for theme, base, tiles, accent in [
        ("clay","835e64",["a3786f","bb8b77","94706f","bd927a"],"c7a177"),
        ("mist","536779",["7a8997","84969b","778598","8d9aa3"],"a1bfc3"),
        ("root","665f55",["8c8469","979171","998970","858165"],"a3a16f"),
        ("bell","6b626b",["938576","958587","ad9883","877d88"],"c8a974"),
    ]:
        floor=Model(f"Dämmergänge · {theme}",f"dusk_floor_{theme}")
        floor.box(0,0,-.29,2.40,2.40,.57,base,.08)
        for x in range(4):
            for z in range(4):
                u,v=(x-1.5)*.58,(z-1.5)*.58
                floor.box(u,v,.006,.551,.551,.033,tiles[(x+2*z)%4],.058,rng.uniform(-.045,.045))
                if (2*x+z)%5==0:
                    floor.ellipsoid(u-.11,v+.11,.027,.16,.08,.024,accent,7,3,rough=.15)
        styles.append(floor)
    for theme,rock,accent in [
        ("clay","95746d","c49a72"),("mist","627783","89b3bd"),
        ("root","827962","a6a477"),("bell","817183","c2a276"),
    ]:
        wall=Model(f"Dämmergänge · Wand mit {theme}",f"dusk_wall_{theme}")
        for i in range(5):
            x=(i-2)*.45
            wall.ellipsoid(x,0,.27+(i%3)*.07,.34,.37,.57+(i%3)*.13,
                           rock,8,4,rough=.19)
        if theme=="clay":
            for x,z,s in [(-.65,.09,.31),(-.34,.03,.23),(.58,.07,.36)]:
                wall.pole((x,z,.47),(x,z,.61),s,accent,9,tip=s*.91)
                wall.pole((x,z,.60),(x,z,.84),s*.91,"b8866b",9,tip=s*.55)
                wall.pole((x,z,.84),(x,z,.90),s*.60,"cc9f7f",9,tip=s*.55)
        elif theme=="mist":
            for x,z,h in [(-.71,.12,.89),(-.47,-.11,1.16),(-.31,.09,.84),(.63,.06,1.04)]:
                wall.pole((x,z,.49),(x+.05,z,h),.13,accent,6,tip=.048)
                wall.pole((x+.05,z,h),(x+.09,z,h+.16),.048,"b0d3ca",6,tip=.006,mode="light")
        elif theme=="root":
            for side in (-1,1):
                wall.pole((side*.81,0,.14),(side*.49,0,.78),.15,"92775c",7,tip=.071)
                wall.pole((side*.49,0,.78),(0,0,1.07),.086,"7f6c56",7,tip=.039)
                for j in range(3):
                    wall.ellipsoid(side*(.29+j*.20),.18,.62+j*.035,.17,.14,.064,
                                   accent,7,4,rough=.16)
            lantern(wall,0,0,.42)
        else:
            wall.box(0,0,.63,.61,.37,.24,"9f8b81",.051)
            wall.pole((0,0,1.13),(0,0,.70),.125,accent,12,tip=.28,mode="light")
            for side in (-1,1):
                wall.pole((side*.49,.015,.52),(side*.43,.015,1.22),.07,"a88d73",7)
            wall.pole((-.43,.015,1.22),(.43,.015,1.22),.055,"a88d73",7)
        styles.append(wall)
    return styles

def generate():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    OUT.mkdir(parents=True,exist_ok=True)
    landscape.prepare_collisions()
    with landscape.original_layout():
        world=[*cabin(),*woods(),*grounds(),*expansion(),*homestead_models(),*travel_gates()]
        districts=[*farnwald(),*muschelbucht(),*eichenhafen(),*mainland_details()]
    existing_rng=RNG.getstate()
    districts.extend(frontier.models(SimpleNamespace(Model=Model,blend=blend,landscape=landscape,
                                                     city_house=city_house,pine=pine,barrel=barrel,lantern=lantern,
                                                     window=window,flowers=flowers,fern=fern,grass=grass)))
    RNG.setstate(existing_rng)
    terrain=landscape.terrain_models(SimpleNamespace(Model=Model, blend=blend))
    landscape.bake_heightmap(ROOT)
    resources=resource_models()
    extra=[*garden(),*character(),*effects(),*furniture_models(),*resources,*new_game_models(),*farm_assets(),
           *deep_mine_assets(resources),*find_assets(),*npc_models(),*workshop_assets(),*regional_assets(),*dungeon_assets()]
    extra.extend([route_details.windmill_sails(Model),*route_details.story_models(Model,lantern)])
    exterior=[*world,*districts,*terrain]
    exterior_assets={model.asset for model in exterior}
    offsets={asset:Vector(point(*location,0)) for asset,location in landscape.ORIGINS.items()}
    zero=Vector((0,0,0))
    verts,faces=[],[]
    for m in exterior:
        if m.casts:
            faces.extend(tuple(i+len(verts) for i in f) for f in m.faces)
            offset=offsets.get(m.asset,zero)
            verts.extend(tuple(Vector(v)+offset) for v in m.vertices)
    shadow=BVHTree.FromPolygons(verts,faces)
    material=bpy.data.materials.new("Gebackenes Sonnenlicht")
    material.use_nodes=True
    material.diffuse_color=(.65,.53,.33,1)
    nodes=material.node_tree.nodes
    color=nodes.new("ShaderNodeVertexColor")
    color.layer_name="COLOR_0"
    material.node_tree.links.new(color.outputs["Color"],nodes["Principled BSDF"].inputs["Base Color"])
    nodes["Principled BSDF"].inputs["Roughness"].default_value=.93

    collections={}
    objects={}
    master=bpy.context.scene.collection
    district_names={"district_forest":"03 · Farnwald","district_beach":"04 · Muschelbucht","district_town":"05 · Eichenhafen",
                    "district_copper":"06 · Kupferklamm","district_wind":"07 · Windweiden","district_moor":"08 · Nebelmoor"}
    for m in (patch for model in [*exterior,*extra] for patch in scenery_patches(model)):
        if m.asset not in collections:
            name="01 · Waldinsel" if m.asset=="island" else district_names.get(m.asset,m.name)
            coll=bpy.data.collections.new(name)
            master.children.link(coll)
            collections[m.asset]=coll
        obj=build_object(m,shadow if m.asset in exterior_assets else None,material,collections[m.asset],offsets.get(m.asset,zero))
        objects.setdefault(m.asset,[]).append(obj)

    for asset, selection in objects.items():
        bpy.ops.object.select_all(action="DESELECT")
        for obj in selection:
            obj.select_set(True)
        bpy.context.view_layer.objects.active=selection[0]
        path=OUT/(asset+".glb")
        bpy.ops.export_scene.gltf(filepath=str(path),use_selection=True,export_format="GLB",
                                  export_yup=True,export_normals=True,export_vertex_color="ACTIVE")
        glb_unlit(path, .30 if asset=="shadow" else None)

    # Arrange the asset source pieces for inspection while leaving exported model origins intact.
    for asset, selection in objects.items():
        if asset in ("island","mainland"):
            continue
        for obj in selection:
            obj.hide_set(True)
    preview = bpy.data.collections.new("02 · Spielszene Vorschau")
    master.children.link(preview)
    for asset,offset in offsets.items():
        for base in objects[asset]:
            replica=base.copy()
            preview.objects.link(replica)
            replica.location=offset
            replica.hide_set(False)
    for i, (x, z) in enumerate([(x,z) for z in [.48,2.10] for x in [1.59,3.15,4.71]]):
        groups = [("lawn",), ("soil","young"), ("soil","carrot"),
                  ("soil","sprout"), ("lawn",), ("soil",)][i]
        for asset in ("bed", *groups):
            replica=objects[asset][0].copy()
            preview.objects.link(replica)
            replica.location=point(x,z,0)
            replica.hide_set(False)
    for asset,x,z,h in [("gardener",-.62,3.47,0),("tail",-.62,3.29,.58),("arm",-.864,3.47,.81),("arm",-.376,3.47,.81),
                         ("leg",-.732,3.47,.356),("leg",-.508,3.47,.356),
                         ("boot",-.732,3.484,.132),("boot",-.508,3.484,.132),("hoe",-.371,3.512,.548),
                         ("cloud_1",-8.28,-6.64,3.72),("cloud_2",11.95,-4.03,2.62),("cloud_3",13.10,-2.99,2.21),
                         ("windmill_sails",16.,124.31,landscape.height(16.,122.8)+4.94),
                         ("resonance_klamm_idle",153.1,-4.65,landscape.height(153.1,-4.65)),
                         ("resonance_weide_idle",34.,146.8,landscape.height(34.,146.8)),
                         ("resonance_moor_idle",-154.,19.1,landscape.height(-154.,19.1)),
                         ("resonance_hafen_idle",56.4,11.1,landscape.height(56.4,11.1))]:
        replica=objects[asset][0].copy()
        preview.objects.link(replica)
        replica.location=point(x,z,h)
        replica.hide_set(False)
    bpy.ops.object.select_all(action="DESELECT")
    bpy.context.scene.name="Waldwinkel · Zusammenhängende Welt"
    setup_preview(bpy.context.scene,445.,(0.,49.,0.65))
    for screen in bpy.data.screens:
        for area in screen.areas:
            if area.type=="VIEW_3D":
                area.spaces.active.shading.color_type="VERTEX"
                area.spaces.active.shading.type="SOLID"
                area.spaces.active.region_3d.view_perspective="CAMERA"
    bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/"art"/"Waldwinkel.blend"),compress=True)


if __name__=="__main__":
    generate()
