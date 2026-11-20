"""Hand-built village masonry and planting beds, using game X/Z/height coordinates."""

import math
import random


STONE_COLORS = ("d7c2a5","cbb9a1","ddcbb0","bfb4a3","d9b892","d2c1ac","beafa4")


def paver(model, blend, scatter, x, z, width, depth, tilt, height):
    shape = [(x+sx*(width*.48+scatter.uniform(-.021,.018))+tilt*sz,
              z+sz*(depth*.48+scatter.uniform(-.018,.017))+tilt*sx)
             for sx,sz in [(-1,-1),(1,-1),(1,1),(-1,1)]]
    shade = blend(scatter.choice(STONE_COLORS),"c2a17d",.13+.11*math.sin(x*.55-z*.72))
    t = height+scatter.uniform(-.008,.010)
    def ring(s,h):
        return [(x+(a-x)*s,z+(b-z)*s,h) for a,b in shape]
    rim = ring(.982,t-.021)
    top = ring(scatter.uniform(.896,.951),t)
    faces = [(k,(k+1)%4,(k+1)%4+4,k+4) for k in range(4)]
    faces += [(k+4,(k+1)%4+4,8) for k in range(4)]
    model.add(rim+top+[(x,z,t+scatter.uniform(-.006,.006))],faces,
              [blend(shade,"ad9b87",.25)]*4+[shade]*4,"stone")


def walkstones(model, blend, rng):
    scatter = random.Random(4713)
    for j in range(27):
        for i in range(37):
            x,z=(i-18)*.551+(j%2)*.105,-4.55+j*.385
            edge = (x/10.72)**2 + (z/6.65)**2
            if edge > 1.01:
                continue
            tilt = rng.uniform(-.042,.041)
            if edge > .91 and scatter.random() < (edge-.87)*3.1:
                continue
            paver(model,blend,scatter,x,z,.548,.381,tilt,.061)


def doorway(model, blend, x, z, lift, accent, carved):
    radius = .405
    spring = 1.48 + lift*.24
    bottom = .23
    z += .12
    arch = [(x+radius*math.cos(math.pi*i/10), z, spring+radius*math.sin(math.pi*i/10))
            for i in range(11)]
    outline = [(x-radius,z,bottom),(x+radius,z,bottom),*arch]
    model.face(outline,"4c4541","timber")
    for i in range(7):
        dx=(i-3)*.107
        top=spring+math.sqrt(radius*radius-(abs(dx)+.055)**2)-.028
        model.box(x+dx,z+.036,(bottom+top)*.5,.101,.071,top-bottom,
                  blend(accent,"739582",.05+(i%4)*.13),.009,mode="timber")
    model.box(x,z+.087,1.04,.72,.037,.048,"715749",.009,mode="timber")
    model.ellipsoid(x+.254,z+.087,1.08,.033,.030,.033,"d5a963",8,4)
    stones=("d8c3a4","ddc9ae","c8b297","d2bea0")
    for sign in (-1,1):
        for i in range(6):
            h=.354+i*(spring-.30)/6
            model.box(x+sign*(radius+.072),z,h,.163,.258,.184,
                      stones[(i+sign)%4],.028,mode="stone")
    for i in range(9):
        lo,hi=math.pi*(i+.045)/9,math.pi*(i+.962)/9
        inner,outer=radius,radius+.149+(i==4)*.057
        face=[(x+r*math.cos(a),z+.139,spring+r*math.sin(a))
              for r,a in [(inner,lo),(outer,lo),(outer,hi),(inner,hi)]]
        verts=face+[(px,z-.044,y) for px,_,y in face]
        model.add(verts,[(0,1,2,3),(0,4,5,1),(1,5,6,2),(2,6,7,3),(3,7,4,0)],
                  [stones[i%4]]*5,"stone")
    model.box(x,z+.16,.089,1.13,.51,.151,"b3a38d",.036,mode="stone")
    model.box(x,z+.108,.171,.97,.38,.107,"c9b89c",.026,mode="stone")
    if carved:
        model.box(x,z+.146,spring+.59,.55,.113,.111,"af967b",.031,mode="stone")
        model.box(x,z+.211,spring+.597,.34,.034,.046,"7d8977",.013)


def planting(api, base, low, landscape):
    seasonal = api.seasonal
    rng = random.Random(277)

    def ground(x,z):
        return max(-.05,landscape.height(x+49.,z+4.))

    # The town's taller silhouettes stand behind the houses, out of the market lanes.
    for x,z,s in [(-10.05,-6.53,1.17),(-6.21,-7.07,1.08),(-4.91,-7.77,1.49),
                  (-.30,-6.91,1.35),(2.52,-6.89,1.11),(6.44,-6.89,1.34),
                  (10.31,-5.95,1.25),(11.94,-6.41,.93),(11.62,-4.57,1.04)]:
        y=ground(x,z)
        base.obstacle_circle(x,z,.245)
        base.pole((x,z,y),(x+.08,z,y+s*3.43),.122*s,"68543d",8,tip=.049*s)
        for i in range(6):
            u=i/5
            r=(.52-.31*u)*s
            lean=.17*math.sin(.8+i*.83)
            base.ellipsoid(x+lean,z+(.13 if i%2 else -.09),y+s*(.92+i*.54),
                           r,r*.76,s*(.70-.29*u),
                           ("637b57","758b53","879854","477259","6e895b","a1a367")[i],
                           9,5,angle=rng.random(),rough=.14,mode="leaf")

    # Low retaining courses and meadow plants sit beyond the north façades.
    for x,z,r in [(-9.06,-5.12,1.45),(-5.67,-5.63,1.09),(-1.89,-6.16,1.08),
                  (2.72,-6.07,.61),(6.27,-5.80,1.28),(9.90,-5.33,1.07)]:
        for i in range(6):
            u=(i-2.5)*r*.33
            zz=z+math.sin(i*.59)*.21
            y=ground(x+u,zz)
            base.box(x+u,zz,y+.12,r*.333,.35,.236,rng.choice(("b6a188","bca792","c9b69c")),
                     .055,rng.uniform(-.29,.29),mode="stone")
        for i in range(13):
            a,b=x+rng.uniform(-r,r),z+rng.uniform(-.22,.36)
            h=ground(a,b)+.08
            api.grass(low,a,b,h,rng.uniform(.17,.36),"80965b",4)
            if i%2==0:
                api.flowers(low,a,b,h,rng.uniform(.55,.88),
                            rng.choice(("b97b98","f3cf87","f5e0ae","bea1bb")))

    for x,z,s in [(-8.86,2.59,1.03),(8.91,2.94,.77)]:
        # A generous canopy over the rim of the plaza, with a forked trunk.
        y=.39
        tips=[(-.76,.10,2.17),(-.37,-.57,2.55),(.55,-.22,2.35),(.63,.37,2.07)]
        base.pole((x,z,y),(x-.11,z+.043,y+1.41*s),.138*s,"7a684f",9,tip=.105*s)
        seasonal.bare_tree(base,x,z,y,.95*s,1.51*s)
        for i,(dx,dz,h) in enumerate(tips):
            end=(x+dx*s,z+dz*s,y+h*s)
            base.pole((x-.09,z+.043,y+s*(1.03+.13*i)),end,.099*s,"78624d",7,tip=.041*s)
            for j,(a,b,k,q) in enumerate([(0,0,0,.62),(-.40,.39,.05,.49),(.47,.02,.22,.53),
                                            (.15,-.25,.48,.44),(-.16,.12,.52,.46)]):
                seasonal.crown(base,end[0]+a*s,end[1]+b*s,end[2]+k*s,
                               s*q,s*q*.83,s*q*.75,
                               ("718c54","909650","7a9860","9da960")[(i+j)%4],
                               9,4,rough=.16)

    climbs=[(-8.75,-1.73,2.18,"c87d97"),(-2.19,-2.82,1.92,"dda08e"),
            (3.24,-2.81,2.04,"a781a4")]
    for x,z,height,petal in climbs:
        bough=[]
        for i in range(10):
            y=.34+height*i/9
            bough.append((x+.13*math.sin(i*.61), z+.08+.026*math.cos(i), y))
        for i,(a,b) in enumerate(zip(bough,bough[1:])):
            base.pole(a,b,.024-i*.0016,"8b7657",5)
            xx,zz,h=b
            wx,wz=seasonal.origin(base,xx,zz)
            for which,tint in [("spring","658649"),("summer","718940"),("autumn","a77354")]:
                layer=seasonal.layers[which]
                for side in (-1,1):
                    leaf=wx+side*(.095+.031*math.cos(i*.77))
                    layer.ellipsoid(leaf,wz+.025,h-.05,.126,.084,.054,tint,7,3,
                                    angle=side*.25,mode="leaf")
                    if which!="autumn" and (i+side)%3!=0:
                        for j in range(3):
                            hin=h+.041*math.sin(j*math.tau/3)
                            layer.ellipsoid(leaf+.041*math.cos(j*math.tau/3),wz+.091,hin,
                                            .053,.034,.047,petal if j!=2 else "f0bdac",6,3,
                                            mode="leaf")


def farm_borders(api):
    Model = api.Model
    moss=Model("Hof · Schichtsteine und Pflanzränder","island")
    flowers=Model("Hof · Lupinen, Mohn und Ziersalbei","island",casts=False)
    rng=random.Random(27625)
    # The beds and the front path keep their established clearances.
    for x,z,w,palette in [
        (-5.56,.69,.69,("ecd69e","cc937c","b7858b")),
        (-6.56,2.15,.79,("dc927f","eecb8c","d9ad91")),
        (-3.32,3.77,.56,("b283a0","eac277","e2d1a3")),
        (-5.26,3.74,.76,("d1afce","d98884","f0d285")),
        (5.87,-.72,.61,("cbc78f","c5949c","efa48c")),
        (6.33,2.44,.68,("dc9a73","e9c583","e7aec2")),
    ]:
        for j in range(5):
            ox=x+(j-2)*w*.37
            zz=z+.14*math.cos(j)
            moss.ellipsoid(ox,zz,.055,w*.22,w*.18,.087,
                           ("9c8e77","b8a38a","a49682")[j%3],8,4,rough=.16)
        for j in range(18):
            a,b=x+rng.uniform(-w*.63,w*.63),z+rng.uniform(-.19,.20)
            scale=rng.uniform(.67,1.12)
            api.grass(flowers,a,b,.006,scale*.31,"7e964c",4)
            if j%3!=0:
                api.flowers(flowers,a,b,.018,scale,palette[j%3])
            else:
                h=rng.uniform(.43,.61)
                flowers.pole((a,b,.006),(a-.06,b+.02,h),.017,"687a52",6,tip=.007)
                for k in range(5):
                    flowers.ellipsoid(a-.008*k,b+.016*math.sin(k*1.7),h-.031*k,
                                      .035+.007*k,.034+.005*k,.041,palette[j%3],6,3,mode="leaf")
    return [moss,flowers]


def harbor_banks(api, landscape):
    """Dress the banks around the pier and keep the restoration route open."""
    Model, blend = api.Model, api.blend
    bank=Model("Fischersteg · Geschichtete Uferfelsen","district_town")
    flora=Model("Fischersteg · Strandhafer und Salzwiesen","district_town",casts=False)
    ox,oz=landscape.TOWN
    rng=random.Random(854)
    for x,z,r in [(52.04,25.66,.83),(53.29,26.82,.54),(54.64,28.08,.59),
                  (60.67,23.37,.71),(62.66,23.82,.79),(63.5,24.17,.56)]:
        for j in range(6):
            xx,zz=x+(j-2.5)*r*.29,z+r*.33*math.sin(j*1.2)
            ground=landscape.height(xx,zz)
            if ground < -.71:
                continue
            scale=r*rng.uniform(.38,.71)
            bank.ellipsoid(xx-ox,zz-oz,ground+scale*.12,
                           scale*1.09,scale*.72,scale*.62,
                           ("9a9c96","b3a997","aba191","898d89")[j%4],
                           8,4,rough=.15,angle=rng.uniform(-.29,.29))

    plants = [ (52.29,22.2,1.38), (53.48,23.14,.80), (51.76,25.32,1.05),
               (53.01,26.80,.62), (55.26,28.54,.62), (60.82,21.48,1.08),
               (61.42,23.16,1.10), (63.20,23.53,1.08), (60.17,24.74,.59) ]
    for i,(x,z,r) in enumerate(plants):
        if i in (0,2,5,6):
            for j in range(6):
                a=rng.random()*math.tau
                reach=rng.uniform(.1,.61)*r
                xx,zz=x+math.cos(a)*reach,z+math.sin(a)*reach
                y=landscape.height(xx,zz)
                foliage=rng.uniform(.22,.39)*r
                bank.ellipsoid(xx-ox,zz-oz,y+foliage*.71,foliage,foliage*.77,foliage*.77,
                               ("6b915b","8c9f58","8b985d","638861")[j%4],9,4,
                               rough=.15,mode="leaf")
        for j in range(24):
            a=rng.random()*math.tau
            reach=r*math.sqrt(rng.random())
            xx,zz=x+math.cos(a)*reach,z+math.sin(a)*reach
            y=landscape.height(xx,zz)
            if y < -.36 or landscape.road_distance(xx,zz)<.82:
                continue
            scale=rng.uniform(.19,.38)
            green=blend("80905b","b5ad65",rng.random())
            api.grass(flora,xx-ox,zz-oz,y+.021,scale,green,4)
            if j%7==0:
                api.flowers(flora,xx-ox,zz-oz,y+.02,scale*2.85,
                            ("bf8da9","e4bd86","e8d8b0")[i%3])
            if j%6==3:
                start=(xx-ox,zz-oz,y+.019)
                for stem in range(3):
                    tip=(start[0]+.071*stem,start[1]+.038*stem,y+.29+scale+.032*stem)
                    flora.pole(start,tip,.012,"8e9c5f",5,tip=.005)
                    flora.ellipsoid(*tip,.019,.020,.050,"c8b77a",5,3)
    return [bank,flora]
