"""Surface relief and lighting for furnished rooms and modular underground levels."""

import math


def refine(models, lantern):
    for m in models:
        key = m.asset
        if key == "interior":
            m.box(0, -2.80, .13, 6.99, .098, .23, "9c6d50", .021, mode="timber")
            for x in [-2.82, -.89, .89, 2.82]:
                m.box(x, -2.81, .67, .88, .055, .76, "cf9f76", .046, mode="timber")
                m.box(x, -2.77, .67, .73, .048, .60, "d9ae82", .035, mode="timber")
            for x in [-1.89, 1.85]:
                m.box(x, -2.60, 1.17, 1.36, .39, .087, "986949", .032, mode="timber")
                for i in range(3):
                    m.box(x+(i-1)*.23, -2.65, 1.245, .133, .13, .071,
                          ("a98c6d", "9fa27a", "c28b72")[i], .019, mode="plaster")
            for x in range(7):
                for z in range(5):
                    if x % 2 == 0 or z % 2 == 0:
                        m.box(.18+x*.35, .01+z*.34, .054, .123, .108, .007,
                              ("c3a06b","8e945f","c6ae7b")[(x+z)%3],
                              .011, .78, mode="weave")
        elif key in ("mine_floor", "mine_deep_floor"):
            deep = key == "mine_deep_floor"
            for i in range(7):
                x = -.95+i*.316
                z = -.8+((i*5)%7)*.27
                m.ellipsoid(x, z, .015, .16, .19, .03, "656984" if deep else "9a7b83",
                            6, 3, angle=i*.72, rough=.19, mode="stone")
        elif key in ("mine_wall", "mine_deep_wall"):
            deep = key == "mine_deep_wall"
            for i in range(6):
                x = (i-2.6)*.31
                h = .21+(i%4)*.087
                m.ellipsoid(x, .27+.045*(i%3), h, .228, .196, .082,
                            "83828f" if deep else "a98b83", 6, 4,
                            angle=.26+i*.19, rough=.15, mode="stone")
            if deep:
                for x, z, h in [(-.29,.34,.72),(-.16,.31,.53),(.02,.36,.41)]:
                    m.pole((x-.04,z,.27), (x,z,h), .059, "748b9e", 5, tip=.036, mode="stone")
                    m.pole((x,z,h), (x+.03,z,h+.064), .036, "afc2bd", 5, tip=.006, mode="light")
        elif key == "mine_support":
            for side in (-1, 1):
                m.box(side*1.02, .15, 1.72, .27,.10,.33, "766e70", .021, mode="metal")
                for y in (1.63,1.80):
                    m.ellipsoid(side*1.02,.212,y,.042,.018,.042,"b7a482",7,3,mode="metal")
            m.box(-.75, .03, 1.52, .234,.13,.125,"7b6d69",.027,mode="metal")
        elif key.startswith("dusk_wall_"):
            theme = key.removeprefix("dusk_wall_")
            shades = {
                "clay": ("b29371", "b58565"),
                "mist": ("719294", "a2bab7"),
                "root": ("88835d", "9eaa75"),
                "bell": ("9b8d86", "ba9a6f"),
            }[theme]
            for s in (-1, 1):
                for i in range(4):
                    m.ellipsoid((.15+i*.21)*s,.33,.08+.025*(i%3),.133,.132,.066,
                                shades[i%2],7,3,angle=i*.26,rough=.20,mode="stone")
        elif key == "dusk_camp":
            lantern(m,-.55,-.40,.36)


def lighting(Model):
    for key, tone in (("lamplight_pool", (1., .71, .37)),
                      ("crystallight_pool", (.51, .81, .92))):
        m = Model("Weicher Lichtschein", key, casts=False)
        near, mid, edge = ((*tone, opacity) for opacity in (.24, .12, 0.))
        for i in range(20):
            a, b = math.tau*i/20, math.tau*(i+1)/20
            def at(r, t):
                return (r*math.cos(t), .84*r*math.sin(t), 0.)
            al, bl, ah, bh = at(.53,a), at(.53,b), at(1.42,a), at(1.42,b)
            m.gradient([(0,0,0),al,bl],[near,mid,mid])
            m.gradient([al,ah,bh,bl],[mid,edge,edge,mid])
        yield m


def practice_dummy(Model):
    m = Model("Strohwächter · Übung am Kartenbinder", "training_dummy")
    m.ellipsoid(0,0,.068,.49,.33,.09,"8e6549",10,4,mode="timber")
    m.pole((0,0,.105),(0,0,1.12),.069,"a9764f",7,mode="timber")
    m.pole((-.45,0,.84),(.45,0,.84),.054,"ab7854",7,mode="timber")
    m.ellipsoid(0,0,.81,.27,.19,.35,"ccaf6d",10,4,rough=.09,mode="weave")
    m.ellipsoid(0,0,1.23,.215,.176,.194,"e2bc76",10,4,rough=.09,mode="weave")
    m.pole((0,0,1.38),(0,0,1.45),.31,"ae8955",12,tip=.28,mode="timber")
    m.ellipsoid(0,0,1.46,.184,.174,.095,"c39b5c",10,4,mode="weave")
    for x in (-.085,.085):
        m.ellipsoid(x,.164,1.24,.033,.022,.034,"746045",7,3)
    m.pole((-.071,.17,1.178),(.071,.17,1.178),.013,"ae7755",5)
    for side in (-1,1):
        for i in range(5):
            x = side*(.37+i*.029)
            m.pole((x,-.051,.84), (x+side*.043,.076,.73+(i%3)*.035),
                   .023,"dcbb78",5,tip=.012,mode="weave")
    for y in (.65,.71):
        m.pole((0,0,y),(0,0,y+.024),.246,"8f6e4c",10,tip=.244,mode="weave")
    m.ellipsoid(-.25,.18,.83,.177,.084,.195,"627d67",9,4,mode="timber")
    return m
