"""Blender geometry for the restoration stages of Eichenhafen's fishing pier."""
import math
import random


def models(Model, lantern, barrel):
    assets = []
    woods = ["bd8d63", "cba074", "aa7e5d", "d2a67a", "ba916a"]
    for stage, name, length in [
        (0, "damaged", 0.), (1, "landing", 3.50),
        (2, "rails", 6.39), (3, "open", 9.56),
    ]:
        model = Model(f"Fischersteg · {name}", f"pier_{name}")
        assets.append(model)
        rng = random.Random(307)
        for z in [.18, 1.82, 3.37, 5.03, 6.36, 7.99, 9.32]:
            for side in [-1, 1]:
                x = side * (.94 if z < 7.4 or stage < 3 else 1.53)
                h = .40 if length >= z else rng.uniform(-.12, .32)
                model.pole((x, z, -1.68), (x+.038, z, h), .141,
                           "9a7757" if length >= z else "756b57", 7, tip=.116)
                model.pole((x, z, -.72), (x, z, -.47), .149, "7e8e69", 7)
                if stage >= 2 and length >= z:
                    model.pole((x, z, h-.145), (x+.025, z, h-.067), .151, "a87355", 7)
            if z <= length:
                model.box(0,z,-.036,2.4 if z<7.4 else 3.31,.19,.18,"977659",.022)
        if length:
            planks = math.ceil(length/.29)
            for j in range(planks):
                z = (j+.5)*length/planks
                width = 2.25 if stage < 3 or z < 7.4 else 3.47
                model.box(0,z,.115,width,length/planks-.013,.148,woods[j % 5],.020)
                for side in [-1,1]:
                    model.ellipsoid(side*(width/2-.20),z,.190,.029,.028,.013,
                                    "8a7766",6,3)
            if stage >= 2:
                for side in [-1,1]:
                    positions = [(side*.994,.16), (side*.994,1.82), (side*.994,3.36),
                                 (side*.994,4.94), (side*.994,6.30)]
                    if stage == 3:
                        positions.extend([(side*1.60,7.43), (side*1.60,8.96), (side*1.60,9.39)])
                    for x,z in positions:
                        model.box(x,z,.494,.142,.14,.64,"aa7d57",.025)
                        model.box(x,z,.825,.177,.171,.06,"cbaa7f",.016)
                    for (x,z),(a,b) in zip(positions,positions[1:]):
                        model.pole((x,z,.727), (a,b,.727), .065, "b38e66", 5)
                        model.pole((x,z,.451), (a,b,.451), .036, "ceac83", 6)
            else:
                for side in [-1,1]:
                    model.pole((side*1.04,.16,.46),(side*1.04,3.38,.46),.035,"c5a986",6)
            bucket(model, .94, 2.16, .193)
            if stage == 3:
                for x,z in [(-1.63,7.68),(1.64,9.32)]:
                    model.box(x,z,.845,.14,.15,1.34,"9c7657",.025)
                    lantern(model,x-.16,z,1.43)
                model.pole((-1.47,9.41,.65),(1.47,9.41,.65),.073,"bc976a",6)
                barrel(model,-1.39,8.33,.19,.54)
                bucket(model,1.27,8.86,.19)
            else:
                barrier(model,length-.1)
        else:
            barrier(model,.11)
            for x,z,a in [(-.54,.92,.19),(.57,1.03,-.28),(-.53,2.33,.5),(.83,3.41,-.32)]:
                model.box(x,z,-.23,.66,.29,.14,"8b7d63",.027,a)
            for x,z,a in [(-.33,.58,.55),(.47,1.24,-.52),(.83,1.68,.92)]:
                model.box(x,z,-.19,1.26,.17,.107,"aa8d6d",.031,a)
            model.pole((-.83,.46,-.02),(-.33,1.34,-.26),.067,"b09871",6)
    board = Model("Fischersteg · Bautisch und Aushang","pier_board")
    for x in [-.64,.64]:
        for z in [-.30,.30]:
            board.box(x,z,.34,.135,.14,.68,"8f6c4e",.023)
    for i in range(6):
        board.box((i-2.5)*.255,0,.708,.247,.82,.123,woods[i%5],.022)
    for x in [-.64,.64]:
        board.box(x,-.29,1.13,.126,.119,.87,"936a48",.02)
    board.box(0,-.27,1.24,1.26,.093,.77,"668b81",.03)
    board.box(0,-.21,1.26,.91,.037,.47,"eddfbb",.026)
    for i in range(4):
        board.box(-.06+(i%2)*.06,-.184,1.36-i*.083,.59-(i%2)*.09,.009,.016,
                  "a38d69",.002)
    for x in [-.33,.34]:
        board.ellipsoid(x,-.178,1.46,.026,.010,.025,"ad754f",6,3)
    board.pole((-.49,.17,.78),(-.17,.18,.81),.037,"997b55",6)
    board.box(-.46,.17,.832,.104,.16,.114,"888f89",.020)
    board.pole((-.78,-.29,1.62),(.78,-.29,1.62),.092,"bf9568",4)
    lantern(board,.52,.21,.935)
    assets.append(board)

    salvage = Model("Fischersteg · Gebündeltes Treibholz","pier_salvage")
    for x,z,h,a in [(-.10,.14,.098,.16),(.08,-.14,.12,-.22),(-.05,-.04,.234,.04),
                    (.08,.06,.187,.24)]:
        salvage.box(x,z,h,1.12,.144,.14,woods[1],.032,a)
        salvage.box(x+.13,z,h+.074,.273,.057,.012,"ab9369",.004,a)
    for x in [-.32,.29]:
        salvage.pole((x,-.20,.106),(x,.23,.106),.027,"8c886d",5)
        salvage.pole((x,-.095,.305),(x,.106,.278),.035,"cbbc8d",6)
    assets.append(salvage)

    feast = Model("Fischersteg · Bänder und Einweihung","pier_feast")
    for x in [-1.04,1.04]:
        feast.box(x,7.59,1.12,.105,.105,1.84,"997858",.019)
    for j in range(8):
        x=-1.04+j*.26
        y=1.92+.132*abs(x)
        z=7.59
        feast.pole((x,z,y),(x+.26,z,1.92+.132*abs(x+.26)),.015,"cfb38b",5)
        feast.face([(x+.015,z,y-.03),(x+.247,z,y-.03),(x+.14,z+.032,y-.25)],
                   ["cf8867","e7c776","83a799","d39b88"][j%4],"raw")
    for z in [7.82,8.63,9.10]:
        feast.box(-1.4,z,.32,.45,.39,.25,"ad8459",.026)
        feast.ellipsoid(-1.4,z,.502,.151,.138,.055,"d99d72",8,3)
        feast.ellipsoid(-1.37,z+.07,.578,.065,.054,.073,"be7569",7,3)
    assets.append(feast)

    lamp = Model("Hafenlaterne · Möbel aus Eichenhafen","furniture_harbor_lantern")
    lamp.box(0,0,.095,.62,.46,.19,"a58b76",.072)
    lamp.box(0,0,.39,.142,.155,.68,"947155",.029)
    lamp.pole((-.03,0,.699),(-.03,0,1.40),.075,"88694d",7)
    lamp.pole((-.03,0,1.36),(.22,.02,1.36),.056,"a18154",6)
    lantern(lamp,.216,.024,1.105)
    for j in range(3):
        r=.136-j*.014
        lamp.ellipsoid(-.19,.092,.215+j*.022,r,r*.72,.023,"c1aa83",10,3)
    assets.append(lamp)
    rod = Model("Angel · Haselrute mit Spule","fishing_rod")
    rod.pole((0,-.09,-.24),(.015,.045,.30),.027,"a17d59",7,tip=.021)
    rod.pole((.015,.045,.30),(.027,.25,.84),.019,"bc966d",6,tip=.012)
    rod.pole((.027,.25,.84),(.031,.51,1.22),.011,"cbac85",6,tip=.005)
    rod.pole((0,-.033,-.09),(.014,.047,.20),.045,"708676",7)
    rod.pole((.037,.06,.20),(.086,.06,.20),.067,"a29a80",8)
    rod.pole((.031,.51,1.22),(.04,.19,1.24),.004,"d8cfb1",4)
    assets.append(rod)
    return assets


def barrier(model, z):
    for x in [-.95,.95]:
        model.box(x,z,.448,.135,.127,.72,"826849",.019)
    model.box(0,z,.633,1.98,.118,.158,"9f7f59",.022)
    for i in range(4):
        model.box((i-1.5)*.41,z+.065,.63,.142,.012,.139,"d7ae78",.003)


def bucket(model, x, z, y):
    model.pole((x,z,y),(x,z,y+.23),.12,"9b8e78",9,tip=.155)
    model.pole((x,z,y+.229),(x,z,y+.246),.161,"c3ad87",10)
    model.pole((x,z,y+.244),(x,z,y+.249),.13,"8daaa3",9)
    model.pole((x-.13,z,y+.221),(x,z,y+.347),.014,"c8b58f",5)
    model.pole((x,z,y+.347),(x+.13,z,y+.221),.014,"c8b58f",5)
