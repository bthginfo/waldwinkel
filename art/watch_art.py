"""Hand-modelled island outposts, survey marks and the farm loom."""
import math
import random

import landscape

SITES = [
    ("fern", -64.85, -18.85, -55.00, -12.60),
    ("tide", -6.90, 51.85, -15.45, 50.50),
    ("kiln", 148., -2.25, 158., 1.80),
    ("wind", 44.50, 150.50, 57.35, 145.),
    ("fen", -155.55, 7., -151., 5.50),
]
TIMBER = ("ac8055", "ba8c60", "be966c", "946a49")
MOSS = ("7f874c", "a3a45c", "687d51")
STONE = ("8c8977", "a7a38d", "827e72")


def frame(m, x, z, y, width, depth):
    for dx in [-width/2,width/2]:
        for dz in [-depth/2,depth/2]:
            h = y + (.36 if dz < 0 else 0.)
            m.pole((x+dx,z+dz,.22),(x+dx*.93,z+dz,h),.114,"99724e",7,tip=.088)
            m.pole((x+dx,z+dz,h-.68),(x+dx*.74,z+dz*.5,h-.11),.051,"bc8e5e",6)
    for side in [-1,1]:
        m.pole((x+side*width*.52,z-depth*.6,y+.39),
               (x+side*width*.49,z+depth*.6,y+.015),.10,"8e6746",7)
    for dz,h in [(-depth*.52,y+.39),(depth*.50,y)]:
        m.pole((x-width*.63,z+dz,h),(x+width*.63,z+dz,h),.097,"b2895a",7)


def decking(m, width=3.70, depth=2.45):
    for i in range(10):
        x = (i-4.5)*width/10
        m.box(x,0,.147,width/10-.018,depth,.17,TIMBER[i%4],.024)
    for z in [-.99,.99]:
        m.box(0,z,.076,width+.07,.19,.14,"8f6a4c",.034)
    for i in range(3):
        m.box(.10,1.27+i*.21,.194-i*.057,1.12,.24,.106,TIMBER[(i+1)%4],.025)
    for side in [-1,1]:
        for z in [-.95,.92]:
            m.ellipsoid(side*(width*.42),z,.048,.33,.32,.16,"a19980",7,3,rough=.19)


def shingle_roof(m):
    frame(m,-.12,-.18,2.21,2.96,1.75)
    ridge, peak, eave = -.12, 3.53, 2.25
    front, back = 1.08, -1.39
    for side in (-1,1):
        edge=ridge+side*1.87
        for j in range(10):
            z=back+(front-back)*(j+.5)/10
            for i in range(5):
                dx=(edge-ridge)/5
                dy=(eave-peak)/5
                color=["637b55","76905d","879666","718562","97a573"][(i+j*3)%5]
                m.pole((ridge+dx*i,z,peak+dy*i),(ridge+dx*(i+1),z,peak+dy*(i+1)),
                       .132,color,7,tip=.143)
        for z in (front+.055,back-.055):
            m.pole((edge,z,eave),(ridge,z,peak+.03),.114,"b0895a",6)
            m.pole((edge*.53,z,2.26),(ridge,z,peak-.28),.071,"8c6c48",6)
    m.pole((ridge,back-.10,peak+.084),(ridge,front+.10,peak+.084),.126,"a9a37a",8)
    m.box(-.12,front-.015,2.59,.56,.15,.43,"9b7148",.049)
    m.pole((-.25,front+.08,2.48),(.04,front+.08,2.72),.019,"e0ca87",6)
    for i in range(3):
        x=-.20+i*.094
        y=2.52+i*.077
        m.ellipsoid(x-.046,front+.095,y+.077,.084,.020,.044,"c6cf90",7,4)
        m.ellipsoid(x+.068,front+.095,y-.015,.076,.020,.046,"a9bf77",7,4)


def cloth_roof(m, wind=False):
    frame(m,-.18,-.28,2.16,3.03,1.64)
    def point(x,z):
        return (x,z,2.36-z*.23-.15*(1-(x/1.82)**2)+.023*math.sin(x*5+z*2))
    for i in range(10):
        x=(i-5)*.36
        for j in range(5):
            z=-1.21+j*.43
            m.face([point(a,b) for a,b in [(x,z),(x+.36,z),(x+.36,z+.43),(x,z+.43)]],
                   ("e6cf9e","ecdeb8","ded0a8")[((i+j)//2)%3],mode="leaf")
    if wind:
        for pos in [-1.70,1.60]:
            m.pole((pos,-1.21,2.30),(pos,1.09,2.03),.041,"d0ad75",5)
    else:
        for j in range(11):
            x=-1.55+j*.28
            z=.31+.11*math.sin(j*.9)
            def stripe(a,b):
                xx,zz,yy=point(a,b)
                return xx,zz,yy+.014
            m.face([stripe(a,b) for a,b in [(x,z),(x+.29,z+.065),(x+.29,z+.22),(x,z+.15)]],
                   ("799caf","85a6b4")[j%2],mode="leaf")


def crate(m,x,z,y,s=.46):
    m.box(x,z,y+s*.34,s,s*.82,s*.69,"9f7651",.024)
    for i in range(3):
        m.box(x,z+s*.41,y+s*(.16+i*.21),s+.017,.04,s*.18,TIMBER[i],.011)
    for side in [-1,1]:
        m.box(x+s*side*.36,z+s*.46,y+s*.35,.045,.037,s*.76,"c49968",.010)


def lantern_mount(m,x,z,y,shade="967e60"):
    m.ellipsoid(x,z,y+.26,.46,.43,.29,shade,8,4,rough=.12)
    m.pole((x,z,y+.29),(x,z,y+.69),.29,"a9a187",8,tip=.25)
    m.pole((x,z,y+.67),(x,z,y+.75),.32,"c0ac83",9,tip=.29)


def lantern(m,y=0,lit=False,tone="warm"):
    brass = "a47b5a" if tone == "warm" else "777e66"
    glass = ("f5d975" if tone=="warm" else "b7d69a") if lit else "797964"
    m.pole((0,0,y),(0,0,y+.16),.216,brass,9,tip=.191)
    m.ellipsoid(0,0,y+.35,.196,.187,.24,glass,10,5,mode="light" if lit else "lit")
    for i in range(8):
        a=i*math.tau/8
        x,z=math.cos(a)*.169,math.sin(a)*.169
        m.pole((x,z,y+.12),(x*1.27,z*1.27,y+.38),.020,brass,4)
        m.pole((x*1.27,z*1.27,y+.38),(x*.83,z*.83,y+.61),.018,brass,4)
    m.pole((0,0,y+.57),(0,0,y+.70),.223,brass,9,tip=.112)
    m.pole((0,0,y+.69),(0,0,y+.77),.052,"9a8771",7)


def shelves(m):
    m.box(-.78,-.69,.86,1.37,.43,.105,"bd925f",.028)
    for x in [-1.34,-.29]:
        m.pole((x,-.70,.21),(x,-.70,.84),.087,"a57654",6)
    for i in range(2):
        crate(m,.92+i*.4,-.74,.24,.37)
    m.box(-.94,-.67,.934,.34,.21,.027,"cfb689",.012,-.21)
    m.box(-.85,-.61,.949,.16,.08,.011,"ece0b2",.006,-.21)


def fern_watch(m):
    decking(m)
    shingle_roof(m)
    shelves(m)
    for j in range(3):
        for i in range(3-j):
            x=-1.72+i*.21+j*.105
            m.pole((x,.52,.2+j*.181),(x,-.16,.2+j*.181),.11,"8a694c",7)
            m.pole((x,.54,.2+j*.181),(x,.58,.2+j*.181),.096,"d1ae79",7)
    lantern_mount(m,1.43,.72,.21)
    # A faded canvas pennant makes the entrance legible between the trees.
    m.pole((-1.61,.87,2.15),(-.84,.87,2.14),.038,"b99b6b",5)
    m.face([(-1.57,.90,2.13),(-.89,.90,2.13),(-1.00,.99,1.56),
            (-1.29,1.02,1.43),(-1.55,.97,1.56)],"ddc690",mode="leaf")
    m.face([(-1.31,.992,1.60),(-1.46,.972,1.85),(-1.21,.925,1.98),
            (-1.11,.982,1.74)],"80915b",mode="leaf")
    m.pole((-1.27,1.022,1.56),(-1.24,.936,1.94),.013,"9b9067",5)


def tide_watch(m):
    decking(m,3.54,2.55)
    cloth_roof(m)
    shelves(m)
    for p in [(-1.69,.69),(1.43,.72)]:
        lantern_mount(m,*p,.16,"a99d8b")
    for i in range(7):
        z=-.62+.21*(i%3)
        x=1.7+i*.13
        m.ellipsoid(x,z,.06,.39,.27,.16,["c6ab89","b99b7d","cfb191"][i%3],8,3,rough=.19)
    m.pole((1.18,-.93,.25),(1.18,-.93,2.52),.128,"886b53",7)
    m.pole((1.15,-.93,2.38),(2.10,-.93,2.39),.091,"a6855d",6)
    m.pole((1.81,-.93,1.66),(1.81,-.93,2.04),.26,"938269",12,tip=.13)
    m.pole((1.81,-.93,1.60),(1.81,-.93,1.67),.308,"b79a6c",12,tip=.26)
    m.pole((1.81,-.93,2.00),(1.81,-.93,2.31),.047,"98805f",7)


def kiln_watch(m):
    decking(m,3.46,2.47)
    frame(m,-.90,-.23,2.03,1.42,1.68)
    for i in range(7):
        m.pole((-1.81+i*.27,-1.31,2.38),(-1.8+i*.27,.98,2.02),.136,TIMBER[i%4],7)
    for i in range(4):
        for j in range(8):
            a=(j+.5*(i%2))*math.tau/8
            if 1.0<a<2.05 and i<3:
                continue
            m.ellipsoid(.63+math.cos(a)*.90,-.31+math.sin(a)*.69,.38+i*.29,
                        .36,.32,.20,STONE[(i+j)%3],7,4,rough=.13)
    m.ellipsoid(.63,-.31,1.46,.94,.78,.44,"66877c",12,6,rough=.095)
    for i in range(9):
        a=i*math.tau/9
        m.pole((.63+math.cos(a)*.82,-.31+math.sin(a)*.68,1.34),
               (.63+math.cos(a)*.34,-.31+math.sin(a)*.31,1.831),.034,"bd8b65",5)
    m.pole((.63,-.31,1.54),(.63,-.31,2.62),.23,"96765e",10,tip=.17)
    m.pole((.63,-.31,2.60),(.63,-.31,2.76),.33,"b68f69",10,tip=.106)
    for x in [-.43,.17]:
        m.pole((x,.88,.28),(x,2.41,.037),.031,"a39a84",5)
    for j in range(6):
        m.box(-.13,.96+j*.239,.14-j*.014,1.01,.14,.08,TIMBER[j%3],.019)
    crate(m,-1.01,-.48,.21,.72)
    lantern_mount(m,1.43,.72,.20)


def loom(m):
    for x in [-.61,.61]:
        m.box(x,0,.092,.177,.87,.18,"9c764f",.025)
        for z in [-.34,.27]:
            m.pole((x,z,.08),(x,z,1.08-(.42 if z>0 else 0.)),.072,"af8759",6)
        m.pole((x,-.35,1.01),(x,.31,.61),.061,"c59a63",6)
    for y,z in [(.62,.31),(.91,-.19),(1.04,-.35)]:
        m.pole((-.67,z,y),(.67,z,y),.072,"c59862",8)
    for i in range(15):
        x=(i-7)*.070
        m.pole((x,-.34,1.093),(x,.314,.681),.012,("d9c889","d7b674","94a6ab")[i%3],4)
    m.face([(-.53,.32,.69),(.53,.32,.69),(.46,-.02,.875),(-.48,-.02,.875)],"a8baae",mode="leaf")
    m.box(.31,.37,.727,.23,.057,.036,"b17e51",.012,-.25)


def wind_watch(m):
    decking(m,3.83,2.50)
    cloth_roof(m,True)
    shelves(m)
    for j in range(8):
        x=-1.55+j*.08
        m.pole((x,.61,.22),(x*1.1,.59+(j%2)*.13,1.22+j%3*.10),.021,"c9b480",5)
    m.pole((-1.38,.62,.59),(-1.31,.62,.677),.19,"7b96a5",7,tip=.16)
    m.pole((1.34,-.68,.23),(1.34,-.68,3.08),.076,"9b7956",6)
    m.pole((.85,-.68,2.76),(1.83,-.68,2.76),.061,"c09a68",6)
    m.face([(1.46,-.68,2.75),(2.09,-.78,2.88),(1.89,-1.01,3.19),
            (1.50,-.85,2.96)],"89a9b7",mode="leaf")
    lantern_mount(m,1.43,.72,.21)


def fen_watch(m):
    decking(m,3.5,2.8)
    for i,(x,z,s) in enumerate([(-1.58,-.49,.98),(1.61,-.55,.95),(-1.92,.82,.74),(1.88,.62,.64)]):
        m.ellipsoid(x,z,s*.84,s*.55,s*.46,s*1.09,STONE[i%3],7,5,rough=.19)
        m.ellipsoid(x-.04,z,s*1.50,s*.49,s*.35,s*.42,MOSS[i%3],7,4,rough=.21)
        for j in range(5):
            zz=z+s*.49
            a=j*math.tau/5
            xx=x+math.cos(a)*s*.26
            y=s*.87+math.sin(a)*s*.26
            a+=math.tau/5
            m.pole((xx,zz,y),(x+math.cos(a)*s*.26,zz,s*.87+math.sin(a)*s*.26),.022,"bbab86",5)
    m.pole((-1.38,-.58,1.77),(0,-.63,2.55),.15,"896a4d",7)
    m.pole((0,-.63,2.55),(1.51,-.63,1.75),.16,"92765b",7)
    for j in range(9):
        t=j/9
        m.ellipsoid(-1.11+t*2.26,-.62,1.89+math.sin(t*math.pi)*.73,.31,.19,.12,
                    MOSS[j%3],7,3,rough=.21)
    lantern_mount(m,1.43,.72,.20)


def marker(Model, style):
    m=Model("Wegzeichen · eingeritzte Tafel",style)
    m.ellipsoid(0,0,.18,.47,.36,.23,"a3987e",7,4,rough=.13)
    m.box(0,0,.41,.54,.29,.36,"9e8d6e",.044)
    m.box(0,.17,.44,.38,.04,.21,"c9b57d",.026)
    m.pole((-.137,.200,.442),(.136,.200,.442),.021,"696f56",5)
    m.pole((.043,.201,.522),(.136,.201,.443),.021,"75816a",5)
    m.pole((.136,.201,.444),(.048,.201,.395),.020,"75816a",5)
    return m


def models(api):
    Model=api.Model
    builders=[fern_watch,tide_watch,kiln_watch,wind_watch,fen_watch]
    styles=["a6a456","bea887","b18562","a7a36a","798353"]
    extra=[]
    for (kind,x,z,tx,tz),build,color in zip(SITES,builders,styles):
        m=Model(f"Wegwacht · {kind}",f"watch_{kind}")
        build(m)
        y=landscape.height(x,z)
        rng=random.Random(505+x)
        # Paths and vegetation stay below the interactable silhouette and clear of the road.
        for j in range(76):
            a=rng.uniform(0,math.tau)
            r=rng.uniform(2.10,5.3)
            dx,dz=math.cos(a)*r,math.sin(a)*r
            wx,wz=x+dx,z+dz
            yy=landscape.height(wx,wz)-y
            if landscape.road_distance(wx,wz)<.9 or abs(yy)>.69 or yy+y<-.15:
                continue
            if j%8==0:
                m.ellipsoid(dx,dz,yy+.064,.35,.29,.17,color,7,3,rough=.16)
            elif j%5==0:
                api.flowers(m,dx,dz,yy,rng.uniform(.68,1.02),["ded0a8","9ca5bd","d9c179"][j%3])
            else:
                api.grass(m,dx,dz,yy,rng.uniform(.16,.38),color,4)
        for i in range(8):
            dx=(-.22,.18,-.05,.37)[i%4]
            dz=1.92+i*.24
            h=landscape.height(x+dx,z+dz)-y
            m.ellipsoid(dx,dz,h+.017,.27,.19,.040,["bfb393","ac9f85","c7bca7"][i%3],6,3,
                        rough=.19)
        extra.append(m)
    for tone in ["warm","green"]:
        for active in [False,True]:
            m=Model("Wegwachtlaterne",f"watch_{tone}_{'lit' if active else 'dark'}",casts=False)
            lantern(m,lit=active,tone=tone)
            extra.append(m)
    hearth=Model("Wegwachtlampe · Möbel", "furniture_watch_lamp")
    lantern_mount(hearth,0,0,.07)
    lantern(hearth,.78,True,"green")
    extra.append(hearth)
    frame=Model("Hof · Webrahmen und Stoffbahnen","workshop_loom")
    loom(frame)
    extra.extend([frame,marker(Model,"watch_trace")])
    return extra
