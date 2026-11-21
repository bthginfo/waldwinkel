"""Editable timber planters, copper water pump and lantern trellis for Eichenhafen."""
import math


def plants(model, lush):
    for row, z in enumerate((-.12, .16)):
        for i in range(5):
            x = -.87 + i*.31 + row*.04
            height = .61 + .042*((i+row)%3)
            model.pole((x,z,.39),(x,z,height),.023,"799350",6,tip=.013)
            for side in (-1, 1):
                tip = (x+side*.105,z+.066,height+.075)
                model.face([(x,z,height-.09),(x+side*.051,z-.042,height+.015),tip,
                            (x+side*.037,z+.050,height-.006)], "899d59" if side>0 else "59824e")
            if lush and (i+row)%2 == 0:
                blossom(model,x,z,height+.062)


def blossom(model,x,z,h):
    for i in range(5):
        t = math.tau*i/5
        a,b=math.cos(t),math.sin(t)
        model.face([(x,z,h),(x+a*.047-b*.034,z+b*.047+a*.034,h+.022),
                    (x+a*.077,z+b*.077,h+.038),
                    (x+a*.047+b*.034,z+b*.047-a*.034,h+.016)], "de8146" if i%2 else "f5ac59")
    model.ellipsoid(x,z,h+.023,.032,.032,.024,"f3ca66",7,3)


def planter(model):
    model.box(-.22,0,.31,1.67,.66,.21,"755142")
    model.box(-.22,0,.424,1.51,.55,.032,"624b43")
    for k,tint in enumerate(("b29166","bc986e","c5a275")):
        for z in (-.338,.338):
            model.box(-.22,z,.19+k*.098,1.71,.07,.094,tint,.014)
        for x in (-1.05,.61):
            model.box(x,0,.19+k*.098,.065,.69,.094,tint,.012)
    for x in (-1.035,.595):
        for z in (-.353,.353):
            model.box(x,z,.276,.081,.08,.43,"926d4b",.013)
            model.ellipsoid(x,z+.044,.36,.016,.009,.016,"726654",6,3)
    for x in (-.96,.52):
        model.box(x,.0,.079,.123,.64,.14,"8a694e")
    model.pole((-.75,.215,.41),(-.75,.215,.71),.02,"856444",6)
    model.box(-.75,.222,.70,.20,.035,.10,"e6cca1",.012,angle=.12)
    model.face([(-.80,.248,.68),(-.76,.249,.721),(-.70,.249,.727),(-.74,.249,.686)],"6a885f")


def pump(model):
    for x,z,w in ((.82,.12,.37),(.91,-.16,.31),(.68,-.18,.27)):
        model.ellipsoid(x,z,.095,w,.23,.11,"9d9789",9,3)
    x,z=.81,-.17
    model.pole((x,z,.16),(x,z,.26),.155,"917257",10,tip=.119)
    model.pole((x,z,.245),(x,z,.91),.095,"66968a",10,tip=.087)
    for h in (.32,.71,.92):
        model.pole((x,z,h),(x,z,h+.055),.112,"b39768",10)
    model.ellipsoid(x,z,.99,.103,.103,.075,"759c8d",10,4)
    model.pole((x,z,.84),(x,.086,.85),.05,"71a292",8)
    model.pole((x,.086,.85),(x,.147,.754),.047,"60867b",8)
    model.pole((x,z,.996),(x-.22,z,1.11),.024,"a88761",8)
    model.pole((x-.22,z,1.11),(x-.34,z,1.06),.031,"845e43",8)
    model.pole((x,.17,.18),(x,.17,.38),.16,"9f7859",11,tip=.207)
    model.pole((x,.17,.378),(x,.17,.392),.207,"c4a677",11)
    model.pole((x,.17,.395),(x,.17,.397),.182,"89bbb0",11)


def pergola(model):
    for x in (-.99,.49):
        model.box(x,-.28,1.03,.087,.082,1.84,"996c4b")
        model.pole((x,-.28,1.35),(x+(.22 if x<0 else -.22),-.28,1.72),.034,"b8996c")
    for i in range(8):
        x=-1.09+i*.22
        h=lambda xx: 1.76+.21*math.cos((xx+.32)*1.75)
        model.pole((x,-.27,h(x)),(x+.23,-.27,h(x+.23)),.054,"b28a5f",7)
    for i in range(26):
        t=i/25; x=-1.05+1.69*t; h=1.75+.20*math.sin(math.pi*t)+.058*math.sin(t*28)
        model.ellipsoid(x,-.24+.02*math.cos(t*29),h,.088,.081,.068,["789051","64874f","97a861"][i%3],7,3)
        if i%4==0: blossom(model,x,-.155,h-.007)


def models(Model):
    output=[]
    for stage,(asset,name) in enumerate((("town_planter","Linas Pflanzkisten"),("town_pump","Die kupferne Dorfpumpe"),("town_pergola","Spalier der Abendblüher"))):
        model=Model(name,asset)
        planter(model)
        plants(model,stage>0)
        if stage>0: pump(model)
        if stage==2: pergola(model)
        output.append(model)
    lamp=Model("Windlaterne aus Honigglas","town_lantern")
    lamp.pole((0,0,-.012),(0,0,-.16),.011,"615b4b",6)
    lamp.pole((0,0,-.19),(0,0,-.15),.078,"805e42",8,tip=.039)
    lamp.pole((0,0,-.35),(0,0,-.19),.071,"f7bf69",8,tip=.079)
    lamp.pole((0,0,-.385),(0,0,-.35),.047,"89684b",8,tip=.081)
    for i in range(4):
        a=math.tau*i/4+.30
        x,z=math.cos(a)*.071,math.sin(a)*.071
        lamp.pole((x,z,-.35),(x,z,-.19),.009,"967247",5)
    output.append(lamp)
    return output
