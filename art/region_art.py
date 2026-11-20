"""Hand-placed scenery and plant communities for the connected island."""

import math
import random


PROFILES = {
    "farm": ("829252", "b6a65c", "ead398", "ba8391"),
    "forest": ("587c50", "789251", "debcae", "b092b1"),
    "beach": ("949752", "b6a86c", "f0d8aa", "c18d8f"),
    "town": ("768e54", "a2a66b", "edd1aa", "c98189"),
    "copper": ("8d8d56", "bea268", "e8bd84", "c98b69"),
    "wind": ("879b53", "c6ad66", "f0d899", "a794b4"),
    "moor": ("678163", "89926b", "c9a5b8", "ad87ae"),
}


def district(x, z):
    if x < -88:
        return "moor"
    if z > 81:
        return "wind"
    if x > 90:
        return "copper"
    if z > 33:
        return "beach"
    if x > 27:
        return "town"
    if x < -24:
        return "forest"
    return "farm"


def models(api):
    land, Model = api.landscape, api.Model
    rng = random.Random(25919)
    clusters = {
        "farm": [(4.6, -7.5, 1.5), (8.5, -3.1, 1.3), (-5.6, 7.1, 1.6),
                 (-8.3, 4.3, 1.3), (5.9, 13.9, 1.5), (13.2, 7.5, 1.1)],
        "forest": [(-41, -8.3, 1.65), (-44.5, -3.3, 1.45), (-47.8, -11.4, 1.7),
                   (-53.9, -5.5, 1.25), (-53.8, -10.4, 1.3), (-57.7, -4.2, 1.4),
                   (-68, -9.1, 1.9), (-66, 1.7, 2.0), (-58.4, 1.7, 1.75)],
        "beach": [(-1, 39.5, 1.5), (-5.6, 43.7, 1.4), (3.5, 45, 1.03),
                  (7.6, 44, 1.8), (9, 50.8, 1.85), (-3.5, 51.7, 1.8),
                  (13.3, 39.6, 1.5), (13.6, 50.9, 1.6)],
        "town": [(37.4, 5.8, 1.0), (43.3, 11.8, 1.25), (48.6, 12.8, 1.2),
                 (50.6, 14.3, 1.4), (48.8, 18.4, 1.7), (54.9, 10.9, 1.25),
                 (53.8, 18.4, 1.2), (57.6, 8.5, 1.08)],
        "copper": [(119.5, 7.9, 1.9), (121.3, 11.5, 1.3), (128.4, 9.7, 1.9),
                   (133.7, 7.5, 1.65), (136.2, -3.7, 1.7), (140.7, -4.6, 2.0),
                   (142.6, 5.2, 2.45), (150.4, -2.9, 1.9)],
        "wind": [(13.4, 115.9, 2.2), (16.1, 119.4, 1.2), (22.9, 119.2, 2.1),
                 (23.2, 125.7, 2.4), (11.5, 121.7, 2.1), (27.7, 127.0, 2.0),
                 (33.6, 121.2, 2.9), (37.1, 132, 2.5), (31.2, 136.8, 2.0)],
        "moor": [(-133.8, -8.3, 1.5), (-139.9, -11.6, 1.5),
                 (-146.9, -12.7, 1.7), (-151.7, -13.9, 2.1), (-151, -6.6, 1.8),
                 (-156.9, -7.3, 1.7), (-157, -21.6, 2.2), (-148.2, -19.3, 2.0),
                 (-167.6, 1.2, 2.6)],
    }
    names = ("Hof · Gartenränder", "Farnwald · Bachpflanzen", "Muschelbucht · Dünenflora",
             "Eichenhafen · Vorgärten", "Kupferklamm · Trockenwiesen",
             "Windweiden · Goldhafer und Lein", "Nebelmoor · Erikafluren")
    plants = {key: Model(name, "island_relief", casts=False)
              for key, name in zip(clusters, names)}

    def ground(x, z, clearance=1.05):
        level = land.height(x, z)
        if (level < -.30 or land.road_distance(x, z) < clearance
                or land.is_gathering_spot(x, z) or land.deck_height(x, z) is not None):
            return None
        for a, b, c, d, circle in land.BLOCKERS:
            if a < x < c and b < z < d:
                return None
        return level + .019

    def stalk(m, x, z, y, size, stem, bloom):
        bend = size * rng.uniform(-.31, .31)
        m.pole((x, z, y), (x+bend, z+.13*size, y+size), size*.016,
               stem, 4, tip=size*.009, mode="leaf")
        for i in range(3):
            level = .68+.14*i
            m.ellipsoid(x+bend*level, z+size*(.13*level+.036*(-1)**i),
                        y+level*size, .063*size, .068*size, .104*size,
                        bloom, 5, 3, rough=.1, mode="leaf")

    def growth(key, x, z, y, size, i):
        m = plants[key]
        leaf, gold, pale, petal = PROFILES[key]
        if key in ("forest", "moor") and i % 5 == 0:
            api.fern(m, x, z, size*2.1, y)
        elif key in ("beach", "wind", "moor") and i % 3 == 0:
            api.grass(m, x, z, y, size*1.47, leaf, 5)
            for j in range(3):
                stalk(m, x+rng.uniform(-.16, .16), z+rng.uniform(-.14, .14), y,
                      size*rng.uniform(1.3, 2.05), leaf, petal if i % 2 else pale)
        else:
            api.grass(m, x, z, y, size, gold if i % 3 == 0 else leaf, 4)
            if i % 4 == 0:
                api.flowers(m, x+.08, z, y, size*2.1, petal if i % 8 else pale)
            if key == "forest" and i % 9 == 0:
                for dx, dz, h, cap in [(0, 0, .19, "bd8760"), (.12, -.10, .13, "d0a375")]:
                    m.pole((x+dx, z+dz, y), (x+dx, z+dz, y+h), .018, "dfc7a0", 5)
                    m.ellipsoid(x+dx, z+dz, y+h, h*.38, h*.41, h*.23, cap, 7, 3,
                                mode="plaster")

    for key, sites in clusters.items():
        for x, z, radius in sites:
            for i in range(22):
                angle = rng.random()*math.tau
                r = radius*math.sqrt(rng.random())
                xx, zz = x+math.cos(angle)*r, z+math.sin(angle)*r
                y = ground(xx, zz)
                if y is not None:
                    growth(key, xx, zz, y, rng.uniform(.26, .53), i)

    # Link the landmarks with clusters of the plants encountered in the next district.
    used = set()
    for road in land.ROADS:
        for (ax, az), (bx, bz) in zip(road, road[1:]):
            dx, dz = bx-ax, bz-az
            length = math.hypot(dx, dz)
            n = int(length/5.2)
            for i in range(n):
                t = (i+.45)/n
                width = rng.uniform(1.75, 3.6)*rng.choice((-1, 1))
                x, z = ax+t*dx-width*dz/length, az+t*dz+width*dx/length
                cell = round(x/2.4), round(z/2.4)
                if cell in used or (abs(x) < 6.5 and abs(z) < 8.5):
                    continue
                used.add(cell)
                for j in range(8):
                    xx, zz = x+rng.uniform(-.71, .71), z+rng.uniform(-.68, .68)
                    y = ground(xx, zz, 1.30)
                    if y is not None:
                        growth(district(xx, zz), xx, zz, y, rng.uniform(.23, .46), j)

    strata = Model("Insel · Schieferbänke, Düne und Kupferadern", "island_relief")
    for x, z, span, heading, key in [
        (-45.5,-8.5,.81,.14,"forest"),(-52.7,-4.43,.96,-.45,"forest"),
        (-55.6,-12.3,1.05,.68,"forest"),(-57.5,-2.4,.69,-.38,"forest"),
        (-3.6,43.2,.74,-.43,"beach"),(6.6,44.9,.83,.31,"beach"),(9.3,51.4,1.02,.44,"beach"),
        (13.1,52.4,.96,-.41,"beach"),(116.9,8.5,1.12,.21,"copper"),
        (125.5,11.7,1.04,.41,"copper"),(130.2,8.5,1.44,-.42,"copper"),
        (135.6,6.9,1.24,-.29,"copper"),(139.9,-2.0,1.25,.37,"copper"),
        (143.3,5.9,1.51,-.38,"copper"),(150.7,-3.4,1.31,-.24,"copper"),
        (-144.5,-12.8,.89,-.29,"moor"),(-153.0,-13.4,.95,-.12,"moor"),
        (-158.6,-24.4,1.24,.23,"moor"),
    ]:
        y = ground(x, z, 2.1+span*.40)
        if y is None:
            continue
        palette, seam = {
            "forest": (("777d7e","969a88","8a8d87"), "b4ad91"),
            "beach": (("ad9580","c0ab92","cbb69a"), "e1ceb0"),
            "copper": (("a47a63","bc8e70","aa8169"), "c5a67c"),
            "moor": (("606d68","778277","81907b"), "9e9a81"),
        }[key]
        for i in range(3):
            offset = (i-1)*.15*span
            cx, cz = x+offset, z-offset*.32
            h = .12+span*(.074+i*.105)
            strata.ellipsoid(cx, cz, y+h, span*(1-i*.17), .51*span*(1-i*.15),
                             .27*span, palette[i], 8, 4, heading,
                             rough=.18, mode="stone")
            if i != 2:
                strata.ellipsoid(cx+.04, cz+.02, y+h+.088, span*(.94-i*.15),
                                 .48*span*(1-i*.16), .12*span, seam, 8, 4, heading,
                                 rough=.13, mode="stone")
        strata.obstacle_circle(x, z, span*.53)
        for i in range(8):
            a = rng.random()*math.tau
            xx, zz = x+1.17*span*math.cos(a), z+.65*span*math.sin(a)
            if (g := ground(xx, zz)) is not None:
                strata.ellipsoid(xx, zz, g, .13, .10, .064, palette[i%3], 7, 3, mode="stone")
        if key == "copper":
            for i in range(5):
                dx = (i-2)*span*.29
                strata.ellipsoid(x+dx, z+.36*span, y+span*(.17+.06*i),
                                 .095, .126, .105, ("ba865c","9bba9b","7b9e84")[i%3],
                                 6, 4, heading, rough=.13, mode="stone")

    walls = Model("Hof und Windweiden · alte Lesesteinmauern", "island_relief")
    for x, z, span, heading in [(-8.8,7.2,2.4,.27),(6.8,-6.1,2.75,-.10),
                                (12.3,121,3.0,-.54),(20.5,125.2,3.4,.14),
                                (29.9,128.5,3.2,.27),(44.,12.2,1.85,-.19)]:
        bases = []
        for row in range(2):
            n = int(span/.36)
            for i in range(n):
                u = (i-(n-1)*.5)*.39+row*.13
                xx, zz = x+u*math.cos(heading), z+u*math.sin(heading)
                if (y := ground(xx, zz, 1.65)) is None:
                    continue
                walls.box(xx, zz, y+.093+row*.154, rng.uniform(.36,.46), .30-row*.03,
                          rng.uniform(.144,.184), ("b5a588","c4b39a","a7987d")[(i+row)%3],
                          .048, heading+rng.uniform(-.14,.14), mode="stone")
                if row == 0:
                    bases.append((xx, zz))
                    if i % 3 == 1:
                        api.grass(plants[district(xx, zz)], xx, zz+.16, y, .24, "7a8c56", 4)
        for xx, zz in bases:
            walls.obstacle_circle(xx, zz, .14)
    return [*plants.values(), strata, walls]
