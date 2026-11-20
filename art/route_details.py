"""Hand-built landmarks and clustered vegetation for the outer routes."""
import math
import random

# Each cluster has a clear side facing the road and a different material palette.
COPPER_YARDS = [(14.6, 0.2), (23.8, 13.8), (26.2, -26.3), (-25.6, 20.4), (-24.5, -15.8)]
WIND_MEADOWS = [(-18.0, -6.1), (8.4, 11.6), (24.15, 15.6), (8.6, 25.8), (-15.2, 19.6)]
MOOR_GROVES = [(-1.4, -7.4), (-17.8, 15.7), (-29.0, -3.2), (-20.9, 31.8), (25.0, -18.2), (9.3, -30.1)]

# These points are also interaction targets. Leave the space in front of the resonator clear.
RESONATORS = {"copper": (16.1, .35), "wind": (6., 16.8), "moor": (-17., 32.1)}


def windmill_sails(Model):
    m = Model("Mühlrad · einzeln animiert", "windmill_sails")
    for i in range(4):
        angle = i * math.tau / 4 + .57
        vx, vy = math.cos(angle), math.sin(angle)
        p, q = -vy, vx
        m.pole((0., 0., 0.), (vx * 3.39, 0., vy * 3.39), .064, "946f52", 5)
        corners = [(vx*t+p*w, .045, vy*t+q*w)
                   for t, w in [(1.12,.05), (3.24,.05), (3.22,.79), (1.22,.73)]]
        m.face(corners, "eddbb2", "raw")
        m.face(corners[::-1], "e0c69e", "raw")
        for j in range(4):
            t = 1.21 + j * .63
            m.pole((vx*t, .082, vy*t), (vx*t+p*.72, .082, vy*t+q*.72), .022, "b08357", 5)
    m.pole((0, -.02, 0), (0, .12, 0), .25, "9d754f", 9, tip=.19)
    return m


def dress(api, theme, model, ground, origin):
    rng = random.Random({"copper": 721203, "wind": 721207, "moor": 721211}[theme])
    ox, oz = origin
    land = api.landscape
    rx, rz = RESONATORS[theme]
    # Preserve sightlines and access to every path, door, bridge and story target.
    def usable(x, z, setback=1.32):
        wx, wz = ox + x, oz + z
        return (ground(x,z) > -.1 and land.edge(wx,wz) > 2.
                and math.hypot(x-rx, z-rz) > 1.65
                and land.road_distance(wx,wz) >= setback)

    def fern(x,z,size):
        api.fern(model,x,z,size,ground(x,z))

    def slab(x,z,size,color):
        y=ground(x,z)
        model.ellipsoid(x,z,y+size*.23,size,size*.72,size*.43,color,7,3,rough=.18)

    if theme=="copper":
        for i,(cx,cz) in enumerate(COPPER_YARDS):
            for _ in range(8):
                x,z=cx+rng.uniform(-4.5,4.5),cz+rng.uniform(-3.9,3.9)
                if not usable(x,z,2.05):
                    continue
                size=rng.uniform(.42,1.53)
                slab(x,z,size,["a78275","c59479","8e7b80"][i%3])
                if size>.85:
                    model.obstacle_circle(x,z,size*.59)
                if rng.random() < .63:
                    y=ground(x,z)+size*.22
                    for j in range(3):
                        a=x+.13*j
                        model.pole((a,z,y),(a+.17,z+.12,y+.22+size*.30),
                                   .076,"c8b8ae" if j else "b9c5b9",5,tip=.012)
            for _ in range(34):
                x,z=cx+rng.uniform(-6.1,6.1),cz+rng.uniform(-5.2,5.2)
                if usable(x,z):
                    api.grass(model,x,z,ground(x,z),rng.uniform(.24,.49),
                              rng.choice(["ad9d68","9b8d5c","c1a474"]),5)
        # Abandoned ore rail with a low timber cart beside the tuning station.
        x,z=19.3,2.95
        y=ground(x,z)
        for j in range(9):
            a=x+(j-4)*.36
            model.box(a,z,ground(a,z)+.072,.26,1.21,.13,"a68966",.018)
        for side in (-1,1):
            model.pole((x-1.58,z+side*.37,y+.14),(x+1.58,z+side*.37,y+.14),.045,"796d69",5)
        model.box(x+.43,z,y+.43,1.14,.81,.32,"a66c50",.053)
        model.box(x+.43,z,y+.58,.92,.61,.08,"715951",.035)
        for j in range(3):
            model.ellipsoid(x+.12+j*.27,z-.12+j%2*.24,y+.69,.19,.17,.15,"b89386",7,3,rough=.15)
        for a in (-.36,.36):
            for b in (-1,1):
                model.pole((x+.43+a,z+b*.31,y+.24),(x+.43+a,z+b*.47,y+.24),.17,"715d59",8)
        model.obstacle_rect(x+.43,z,1.15,1.0)
        # Layer the old siding with waist-high outcrops and low plants so the trail stays readable.
        for x,z,size in [(21.8,.66,1.06),(22.97,1.48,1.20),(22.55,2.76,.88),
                         (15.6,2.79,.78),(14.79,3.70,.93),(12.75,-1.2,.87)]:
            if not usable(x,z,1.52):
                continue
            y=ground(x,z)
            model.ellipsoid(x,z,y+.47*size,size,.64*size,.76*size,"ac837d",8,4,rough=.21)
            model.ellipsoid(x+.22*size,z-.10*size,y+1.03*size,.69*size,.54*size,
                            .42*size,"b79883",8,4,rough=.18)
            model.obstacle_circle(x,z,size*.58)
            for j in range(3):
                a=x+.34+j*.12
                model.pole((a,z+.52*size,y+.51*size),(a+.12,z+.56*size,y+.82*size),
                           .064,"d2b9a5" if j%2 else "9fa99b",5,tip=.017)
        for j in range(104):
            cx,cz=[(14.1,1.9),(18.8,2.2),(21.8,.50),(22.8,4.6),(13.5,-2.9)][j%5]
            x,z=cx+rng.uniform(-1.9,1.9),cz+rng.uniform(-1.5,1.5)
            if not usable(x,z,1.10):
                continue
            if j%7 == 0:
                api.flowers(model,x,z,ground(x,z),rng.uniform(.68,.98),"d59672")
            elif j%4 == 0:
                fern(x,z,rng.uniform(.23,.44))
            else:
                api.grass(model,x,z,ground(x,z),rng.uniform(.28,.54),
                          rng.choice(["9a9c67","a3a577","baa774"]),6)

    if theme=="wind":
        for row,(cx,cz) in enumerate(WIND_MEADOWS):
            for line in range(4):
                for j in range(12):
                    x=cx+(j-5.5)*.47+rng.uniform(-.11,.11)
                    z=cz+(line-1.5)*.68+rng.uniform(-.12,.12)
                    if not usable(x,z):
                        continue
                    y=ground(x,z)
                    for k in range(2):
                        h=rng.uniform(.33,.65)
                        a=x+k*.12
                        model.pole((a,z,y),(a+.093,z+.02,y+h),.022,"8a9c63",4,tip=.011)
                        model.ellipsoid(a+.095,z+.02,y+h,.078,.072,.051,
                                        ["cfbe95","cab3c7","e2cab9"][(row+j+k)%3],5,3)
            for _ in range(21):
                x,z=cx+rng.uniform(-5.4,5.4),cz+rng.uniform(-4.4,4.4)
                if usable(x,z):
                    api.flowers(model,x,z,ground(x,z),rng.uniform(.57,1.03),
                                rng.choice(["f0d495","bca6c5","e5bf9a"]))
        for x,z in [(-15.5,-4.1),(-14.8,-4.55),(9.7,18.9),(10.3,19.5),(18.4,10.7)]:
            if not usable(x,z):
                continue
            y=ground(x,z)
            model.pole((x-.31,z,y+.28),(x+.31,z,y+.28),.31,"d1b776",10)
            for side in [-1,1]:
                model.pole((x+side*.23-.024,z,y+.28),(x+side*.23+.026,z,y+.28),.315,"ac8c56",10)
            model.obstacle_circle(x,z,.29)

    if theme=="moor":
        for group,(cx,cz) in enumerate(MOOR_GROVES):
            for j in range(3):
                x,z=cx+rng.uniform(-3.5,3.5),cz+rng.uniform(-3.4,3.4)
                if not usable(x,z,2.45):
                    continue
                s=rng.uniform(.68,1.35)
                y=ground(x,z)
                model.pole((x,z,y),(x+.24*s,z-.08*s,y+2.12*s),.27*s,"786954",7,tip=.12*s)
                model.obstacle_circle(x,z,.39*s)
                for k in range(5):
                    angle=k*math.tau/5+.7
                    a,b=.81*s*math.cos(angle),.67*s*math.sin(angle)
                    h=y+(2.54+.24*(k%3))*s
                    model.pole((x+.18*s,z,y+1.65*s),(x+a,z+b,h),.088*s,"91775d",6,tip=.045*s)
                    model.ellipsoid(x+a,z+b,h,.71*s,.53*s,.39*s,
                                    ["788873","9a9a7a","879878","979288","b59c9e"][(k+group)%5],8,4,rough=.14)
                    for n in range(3):
                        aa=x+a+(n-1)*.21*s
                        model.pole((aa,z+b,h-.06),(aa+.055*s,z+b+.09*s,h-.64*s),.034*s,
                                   "809275",5,tip=.007*s,mode="leaf")
            for j in range(34):
                x,z=cx+rng.uniform(-4.8,4.8),cz+rng.uniform(-4.4,4.4)
                if usable(x,z):
                    if j%4:
                        fern(x,z,rng.uniform(.26,.57))
                    else:
                        slab(x,z,rng.uniform(.29,.72),"95898a")
        for cx,cz,rx,rz in [(-18.,-12.,10.2,7.2),(19.,21.,13.2,8.4),(-31.,17.,10.8,7.5)]:
            for _ in range(24):
                angle=rng.uniform(0,math.tau)
                radius=rng.uniform(.50,.84)
                x,z=cx+rx*radius*math.cos(angle),cz+rz*radius*math.sin(angle)
                if ground(x,z)>-.88 or abs(z+11.94)<1.36 or abs(z-21.06)<1.36:
                    continue
                r=rng.uniform(.15,.36)
                y=api.landscape.SEA+.021
                rim=[(x+math.cos(n*math.tau/8)*r,z+math.sin(n*math.tau/8)*r*.73,y) for n in range(1,8)]
                model.face([(x,z,y),*rim],rng.choice(["72967d","8ba583","9aaa7a"]),"raw")
                if rng.random()<.26:
                    model.ellipsoid(x,z,y+.06,.10,.098,.049,"e8b2a3",7,3)
        for x,z in [(25.6,-18.35),(16.1,-9.4),(-19.0,30.9),(-20.6,32.1),(-11.9,20.6)]:
            if not usable(x,z,1.):
                continue
            y=ground(x,z)
            model.pole((x,z,y),(x,z,y+1.46),.077,"9b8064",6)
            api.lantern(model,x,z+.23,y+1.24)

    # Small plants along trail shoulders put detail in the walking view as well as at landmarks.
    palette = {
        "copper": ("a7a46b", "b7a773", "cfab82"),
        "wind": ("91a876", "b2bb7a", "d0bfa3"),
        "moor": ("849575", "a0ac79", "b3a594"),
    }[theme]
    for road in land.ROADS:
        for (ax,az),(bx,bz) in zip(road,road[1:]):
            length=math.hypot(bx-ax,bz-az)
            nx,nz=(az-bz)/length,(bx-ax)/length
            steps=math.ceil(length/2.8)
            for i in range(steps):
                t=(i+.48)/steps
                x,z=ax+(bx-ax)*t-ox,az+(bz-az)*t-oz
                if math.hypot(.94*x,z)>54.:
                    continue
                for side in (-1,1):
                    offset=side*rng.uniform(1.78,3.04)
                    a,b=x+offset*nx,z+offset*nz
                    if not usable(a,b,1.38):
                        continue
                    for j in range(3):
                        xx,zz=a+rng.uniform(-.44,.44),b+rng.uniform(-.44,.44)
                        if usable(xx,zz,1.17):
                            api.grass(model,xx,zz,ground(xx,zz),rng.uniform(.23,.46),palette[j],5)
                    if i%5==2:
                        if theme=="moor":
                            fern(a,b,rng.uniform(.27,.48))
                        else:
                            api.flowers(model,a,b,ground(a,b),rng.uniform(.58,.94),palette[2])


def story_models(Model, lantern):
    active = ["dab078","cbb562","9cbaa5","e6c67f"]
    grounded = ["79605a","8f7657","718784","99817a"]
    names = ["klamm", "weide", "moor", "hafen"]
    for i,name in enumerate(names):
        for lit in (False,True):
            model = Model(f"Klangstein {name} · {'wach' if lit else 'still'}",
                          f"resonance_{name}_{'lit' if lit else 'idle'}")
            base, shade = grounded[i], active[i]
            model.pole((0,0,0),(0,0,.17),.55,base,9,tip=.49)
            model.pole((0,0,.17),(0,0,.89),.34,base,7,tip=.23)
            model.pole((0,0,.88),(0,0,1.07),.45,base,10,tip=.37)
            for side in [-1,1]:
                model.pole((side*.21,0,.93),(side*.29,0,1.52),.092,base,7,tip=.068)
            model.pole((-.29,0,1.51),(.29,0,1.51),.081,base,7)
            model.pole((0,0,1.44),(0,0,1.14),.13,shade if lit else "ad8b65",10,tip=.25,
                       mode="light" if lit else "lit")
            model.ellipsoid(0,0,1.117,.069,.069,.053,"d8b37d",8,4)
            if lit:
                for j in range(5):
                    angle=j*math.tau/5
                    model.ellipsoid(.36*math.cos(angle),.36*math.sin(angle),1.16+.12*j,
                                    .045,.045,.046,shade,6,3,mode="light")
                if i == 3:
                    lantern(model,.65,-.18,.26)
                    lantern(model,-.65,-.18,.26)
                    for a in (-1,1):
                        for j in range(6):
                            x=(.71+j*.19)*a
                            z=-.04-j*.17
                            model.pole((x,z,.44),(x,z,.47),.065,"907a62",7)
                            model.ellipsoid(x,z,.55,.047,.056,.11,
                                            ["ddb77b","c7ab7a","a1b093"][j%3],7,3,mode="light")
            yield model
