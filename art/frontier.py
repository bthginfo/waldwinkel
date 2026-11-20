"""Geometry and terrain parameters for the three outer regions."""
import math
import random
import route_details

ORIGINS = {
    "district_copper": (137., -5.),
    "district_wind": (28., 130.),
    "district_moor": (-137., -13.),
}
LANDFORMS = [
    (137.,-5.,61.,62.), (28.,130.,62.,58.), (-137.,-13.,62.,62.),
    (78.,8.,17.5,8.), (11.,78.,9.0,19.), (-87.,-14.,15.,8.5),
]
LAKES = [(-18.,-12.,10.2,7.2), (19.,21.,13.2,8.4), (-31.,17.,10.8,7.5), (-15.,8.,3.6,4.3)]
COPPER_BRIDGES = [-21., 4., 24.]
WIND_BRIDGES = [-22., 1., 18.]
ROADS = [
    [(63.5,10.5),(73.,10.2),(81.,8.8),(97.,7.3),(110.,6.4),(125.,8.),(134.,2.),
     (143.,-1.),(155.,-8.),(164.,-19.),(172.,-28.)],
    [(125.,8.),(126.,-10.),(134.,-26.),(141.,-26.),(157.,-30.),(172.,-28.)],
    [(125.,8.),(126.,16.),(133.,19.),(142.,19.),(159.,25.),(168.,10.),(155.,-8.)],
    [(1.7,47.51),(1.8,61.),(8.5,72.),(10.,78.),(13.,91.),(17.,116.),(19.,121.),
     (29.,123.),(29.,131.),(32.,147.),(43.,153.),(63.,152.)],
    [(17.,116.),(4.,122.),(6.,132.),(9.,148.),(15.,160.),(32.,166.),(49.,161.),(63.,152.)],
    [(29.,123.),(47.,118.),(46.,127.),(46.,139.),(55.,144.),(63.,152.)],
    [(-72.8,-14.4),(-83.8,-14.),(-97.,-13.3),(-115.,-11.),(-130.,-11.),(-144.,-8.),
     (-153.,-9.1),(-166.,-14.),(-178.,-25.)],
    [(-144.,-8.),(-150.,1.),(-162.,9.),(-155.,21.),(-140.,27.),(-119.,17.),(-107.,6.),(-112.,-7.)],
    [(-144.,-8.),(-135.,-18.),(-123.,-22.),(-111.,-35.),(-124.,-46.),(-140.,-48.)],
]


def copper_stream(z):
    return 4. + 2.6*math.sin(z*.085)


def wind_stream(x):
    return .6 + 1.23*math.sin(x*.093)


def relief(x,z,bank,edge,smooth):
    if bank < -.5:
        return bank
    if x > 87. and z < 65.:
        a,b=x-137.,z+5.
        broad=.23+.38*(.5+.5*math.cos(a*.11+b*.077))
        river=1.-smooth((abs(a-copper_stream(b))-1.07)/1.65)
        fade=smooth((b+44.)/4.)*smooth((49.-b)/4.)
        return bank+smooth(edge/4.)*broad-river*fade*1.67
    if z > 79.:
        a,b=x-28.,z-130.
        hill=.24+.45*(.5+.5*math.sin(a*.085-b*.10))
        river=1.-smooth((abs(b-wind_stream(a))-.76)/1.22)
        fade=smooth((a+43.)/3.)*smooth((38.-a)/3.)
        return bank+smooth(edge/5.)*hill-river*fade*1.60
    if x < -87.:
        a,b=x+137.,z+13.
        h=bank+smooth(edge/4.)*.11*(1.+math.sin(a*.14-b*.09))
        for ox,oz,w,d in LAKES:
            distance=math.hypot((a-ox)/w,(b-oz)/d)
            h-=1.51*(1.-smooth((distance-.64)/.43))
        return h
    return bank


def deck(x,z):
    if x > 87. and z < 65.:
        a,b=x-137.,z+5.
        if any(abs(b-j)<=.83 and abs(a-copper_stream(j))<=3.43 for j in COPPER_BRIDGES):
            return .39
    if z > 79.0:
        a,b=x-28.,z-130.
        if any(abs(a-j)<=.82 and abs(b-wind_stream(j))<=2.82 for j in WIND_BRIDGES):
            return .35
    if x < -87.:
        a,b=x+137.,z+13.
        if -29.7<a<-6.3 and -12.68<b<-11.21:
            return .25
        if 5.4<a<32.3 and 20.36<b<21.76:
            return .25
    return None


def tint(x,z,grain,base,blend):
    if x > 87. and z < 65.:
        clay=math.exp(-(((x-154.)/19.)**2+((z+4.)/14.)**2))
        weathered=blend("a89765","c2ac76",grain)
        return blend(weathered,blend("aa8a72","c6a283",grain),.67*clay)
    if z > 79.:
        return blend("8eab6f","c2c783",grain)
    if x < -87.:
        return blend("75875e","a4aa73",grain)
    return base


def models(api):
    Model,blend,landscape=api.Model,api.blend,api.landscape
    rng=random.Random(61983)
    result=[]

    def region(asset):
        m=Model(asset.replace("district_",""),asset)
        result.append(m)
        ox,oz=ORIGINS[asset]
        return m, lambda x,z: landscape.height(ox+x,oz+z)

    def house(m,ground,x,z,w,d,body,roof,detail="window"):
        y=ground(x,z)
        api.city_house(m,x,z,w,d,body,roof,y,detail)
        m.obstacle_rect(x,z,w+.19,d+.23)

    def bridge(m,x,z,half,wide,top,axis):
        # Local planks are level with the matching hand-authored deck in the height field.
        for i in range(math.ceil(half*8)):
            u=-half+(i+.5)*2*half/math.ceil(half*8)
            p,q=(x+u,z) if axis=="x" else (x,z+u)
            w,d=(.245,wide) if axis=="x" else (wide,.245)
            m.box(p,q,top-.082,w,d,.16,["bd9168","c9a474","aa7d58"][i%3],.026)
        for side in [-1,1]:
            shift=side*(wide*.48)
            a,b=(x-half,z+shift) if axis=="x" else (x+shift,z-half)
            c,d=(x+half,z+shift) if axis=="x" else (x+shift,z+half)
            m.obstacle_rail(a,b,c,d)
            m.pole((a,b,top+.57),(c,d,top+.57),.061,"a88560",5)
            for i in range(math.ceil(half*1.8)+1):
                t=i/math.ceil(half*1.8)
                xx,zz=a+(c-a)*t,b+(d-b)*t
                m.box(xx,zz,top+.18,.15,.15,.77,"89664e",.024)
                m.box(xx,zz,top+.585,.172,.171,.062,"cda37a",.014)

    copper,h=region("district_copper")
    for args in [(-15,2,3.8,3.0,"c98e62","827c7c","chimney"),(-12,-8,3.5,2.5,"d1af7a","bb7959","dormer"),
                 (-20,9,4.1,2.8,"c5976c","b87a58","dormer"),(14,-9,3.3,2.8,"d5a474","8f7f76","chimney"),
                 (20,-18,4.3,2.9,"ba885e","99766c","chimney"),(25,12,3.8,2.8,"cea26a","b9825d","window")]:
        house(copper,h,*args)
    for p in COPPER_BRIDGES:
        bridge(copper,copper_stream(p),p,3.49,1.96,.39,"x")
    for i,(x,z) in enumerate([(-36,-17),(-27,-29),(-23,-35),(13,-39),(25,-31),(31,-30),(38,-22),
                               (39,-13),(23,31),(-20,36),(-35,25)]):
        y=h(x,z)
        for j in range(4):
            a=x+rng.uniform(-3.8,3.8)
            b=z+rng.uniform(-2.8,2.8)
            radius=rng.uniform(1.38,3.76)
            height=rng.uniform(1.55,5.91)
            copper.ellipsoid(a,b,y+height*.31,radius,radius*.75,height,
                             ["b38979","a9847c","957980","c89478"][(i+j)%4],8,4,rough=.19)
            copper.obstacle_circle(a,b,radius*.76)
            copper.pole((a-radius*.63,b+radius*.57,y+height*.13),
                        (a+radius*.22,b+radius*.64,y+height*.91),.11,
                        "cb8e72",5,tip=.053)
    center=(-13.,6.2)
    y=h(*center)
    x,z=center
    copper.box(x,z,y+.12,2.48,1.11,.24,"9d8b77",.079)
    copper.box(x,z-.24,y+.46,2.29,.52,.82,"8c756d",.074)
    copper.box(x,z+.18,y+.34,1.92,.41,.35,"7c746b",.064)
    copper.ellipsoid(x,z+.407,y+.56,.51,.077,.33,"5c4843",11,5)
    copper.ellipsoid(x,z+.460,y+.53,.37,.060,.208,"d39059",10,5,mode="light")
    for j in range(7):
        a=j*math.pi/6
        copper.box(x+.58*math.cos(a),z+.389,y+.36+.52*math.sin(a),
                   .242,.20,.22,["a78f7b","b59c82","9e8779"][j%3],.047)
    for side in [-1,1]:
        copper.box(x+side*.98,z+.35,y+.99,.16,.19,1.82,"8c654a",.025)
    for j in range(9):
        u=x+(j-4)*.253
        copper.pole((u,z-.52,y+1.96),(u,z+.58,y+1.64),.139,
                    ["827972","928478","a49885"][j%3],7)
    copper.pole((x+.68,z-.32,y+.67),(x+.68,z-.32,y+2.47),.163,"ad8066",8,tip=.132)
    copper.pole((x+.68,z-.32,y+2.47),(x+.68,z-.32,y+2.56),.201,"847666",8)
    copper.box(x-1.53,z+.38,y+.20,.43,.38,.39,"947255",.044)
    copper.ellipsoid(x-1.53,z+.41,y+.43,.26,.19,.09,"80786e",8,4)
    copper.obstacle_rect(*center,2.3,.76)
    for x,z in [(-14.1,7.03),(-11.4,7.43),(17,-15.1),(21,-16.3)]:
        api.barrel(copper,x,z,h(x,z),.82)
    for x,z in [(-19.4,11.6),(-10.3,8.6),(15,-6.4),(30,-21.8)]:
        y=h(x,z)
        copper.pole((x,z,y),(x,z,y+1.45),.089,"9f744f",5)
        api.lantern(copper,x,z+.29,y+1.29)
    for i,(x,z,scale) in enumerate([(-17.7,13.9,1.10),(-16.9,16.4,.87),(-9.1,15.4,.73),
                                       (-6.4,11.8,.65),(-4.7,10.3,1.02),(-6.6,17.6,1.14),
                                       (-11.2,1.75,1.13),(-15.8,-2.6,.87),(9.23,11.8,1.12)]):
        api.pine(copper,x,z,h(x,z),scale,i)
        copper.ellipsoid(x+.55,z+.31,h(x+.55,z+.31)+.16,.39,.34,.31,"baa38d",8,4,rough=.20)
    for i in range(280):
        cx,cz=[(-17.7,15.7),(-9.8,13.0),(-7.8,5.1),(-23,10.5),(1.8,7.6),(10.7,13.3)][i%6]
        x,z=cx+rng.uniform(-2.8,2.8),cz+rng.uniform(-2.4,2.4)
        if h(x,z)<-.07 or landscape.road_distance(137+x,-5+z)<1.16:
            continue
        if i%7:
            api.grass(copper,x,z,h(x,z),rng.uniform(.18,.44),["a99860","c6ae65","b5a56a"][i%3],4+i%3)
        else:
            api.flowers(copper,x,z,h(x,z),rng.uniform(.67,1.19),["f3d49a","d38f72","e6ba7f"][i%3])

    wind,h=region("district_wind")
    for args in [(-19.,-10.,3.5,2.8,"d0a174","c58d60","chimney"),(11,9,4.4,3.1,"b88a5e","d49c63","window"),
                 (-23,12,4.1,2.9,"d0a77c","a78361","dormer"),(26,14,4.4,3.1,"bf9468","c58c63","chimney"),
                 (16,-16,3.4,2.4,"e3ba8d","b98061","window")]:
        house(wind,h,*args)
    for p in WIND_BRIDGES:
        bridge(wind,p,wind_stream(p),2.91,1.96,.35,"z")
    x,z=-12.,-7.2
    y=h(x,z)
    wind.obstacle_circle(x,z,2.06)
    wind.pole((x,z,y),(x,z,y+.54),2.1,"aa9a83",12,tip=1.93)
    wind.pole((x,z,y+.51),(x,z,y+5.39),1.57,"a47954",11,tip=.99)
    wind.pole((x,z,y+5.37),(x,z,y+6.53),1.27,"bd815c",9,tip=.056)
    api.window(wind,x,z+1.30,y+2.05,.47,.58)
    api.window(wind,x,z+1.10,y+3.62,.46,.53)
    for cx,cz in [(8.4,11.6),(24.15,15.6),(-23,-6.4)]:
        for side in [-1,1]:
            for j in range(7):
                x,z=cx+(j-3)*.91,cz+side*2.37
                y=h(x,z)
                wind.box(x,z,y+.35,.155,.159,.70,"a7825b",.027)
                if j<6 and (j!=3 or side<0):
                    wind.obstacle_rail(x,z,x+.91,z)
                    wind.pole((x,z,y+.52),(x+.91,z,h(x+.91,z)+.52),.061,"ad8962",5)
                    wind.pole((x,z,y+.26),(x+.91,z,h(x+.91,z)+.26),.047,"b29468",5)
    for j in range(130):
        cx,cz=[(-12,-3),(10,7),(-27,21),(24,-16)][j%4]
        x,z=cx+rng.uniform(-2.6,2.6),cz+rng.uniform(-1.8,1.8)
        api.flowers(wind,x,z,h(x,z),rng.uniform(.8,1.36),"b192be")
    for i in range(290):
        cx,cz=[(-16,-8),(-9,-10),(-13,-17),(-19,-17),(-15,-24),(-22,-4),(6,-13)][i%7]
        x,z=cx+rng.uniform(-3.6,3.6),cz+rng.uniform(-3.1,3.1)
        if h(x,z)<-.14 or landscape.road_distance(28+x,130+z)<1.12:
            continue
        api.grass(wind,x,z,h(x,z),rng.uniform(.22,.45),["b1b77d","c7bc86","a8b277"][i%3],5)
    for i,(x,z,s) in enumerate([(-16.8,-6.2,.92),(-20.9,-5.8,.88),(-23.4,-8.4,1.04),
                                (-8.35,-17.9,.82),(-9.2,-21.5,1.17),(-18.1,-18.8,.74)]):
        api.pine(wind,x,z,h(x,z),s,i)
        wind.ellipsoid(x+.58,z+.31,h(x+.58,z+.31)+.163,.39,.36,.35,"b3ae8f",8,4,rough=.17)

    moor,h=region("district_moor")
    for args in [(-10,15,4.2,3.1,"b58864","b77e5e","chimney"),(9,7,3.4,2.8,"b89672","a3896a","dormer"),
                 (26,13,3.4,2.6,"c6a780","9e856a","dormer"),(-22,-3,3.8,3.0,"cea275","b98264","dormer"),
                 (-25,24,3.6,2.9,"b8986f","baa06c","window")]:
        house(moor,h,*args)
    bridge(moor,-18.,-11.94,11.65,1.89,.25,"x")
    bridge(moor,18.83,21.06,13.43,1.89,.25,"x")
    for cx,cz,rx,rz in LAKES:
        for i in range(61):
            angle=rng.random()*math.tau
            x=cx+rx*math.cos(angle)*rng.uniform(.71,.96)
            z=cz+rz*math.sin(angle)*rng.uniform(.71,.96)
            y=h(x,z)
            if y<-.82 or (abs(z+11.94)<1.20 or abs(z-21.06)<1.20):
                continue
            for j in range(3):
                size=rng.uniform(.35,.94)
                a,b=x+j*.071,z+j*.069
                moor.pole((a,b,y),(a+.06,b,y+size),.024,"b3a365",5,tip=.011)
                moor.pole((a+.058,b,y+size-.13),(a+.067,b,y+size+.036),.053,"a77e57",7,tip=.034)
    x,z=26.,-22.
    y=h(x,z)
    for i in range(13):
        a=i*math.tau/24
        xx,yy=x+math.cos(a)*1.37,y+.42+math.sin(a)*1.46
        moor.ellipsoid(xx,z,yy,.40,.40,.37,
                       ["958985","9e928a","ad9b87"][i%3],8,4,rough=.14)
        if i%2==0:
            api.fern(moor,xx+.09,z+.39,.30,y)
    for side in [-1,1]:
        moor.box(x+side*1.38,z,y+.31,.64,.91,.65,"91837a",.12)
        moor.obstacle_circle(x+side*1.38,z,.45)
        api.lantern(moor,x+side*1.55,z+.58,y+.67)
    moor.box(x,z-.46,y+.033,2.40,1.21,.056,"aba083",.049)
    for i,(x,z,s) in enumerate([(-10.7,3.1,1.05),(-4.4,1.25,1.15),(-11.2,10.7,.88),
                                (-18.7,6.5,1.14),(-8.8,12.62,1.05),(-11.9,13.9,1.12),
                                (-4.75,14.6,.93),(-22.9,5.5,1.10),(-9.9,-2.5,1.00)]):
        y=h(x,z)
        if y<-.12:
            continue
        moor.pole((x,z,y),(x+.06,z,y+2.11*s),.207*s,"9c8271",8,tip=.108*s)
        moor.obstacle_circle(x,z,.24*s)
        for dx,dz,hh,rad in [(-.47,.09,2.04,.86),(.58,-.20,2.33,.80),(.07,.22,2.69,.83)]:
            moor.ellipsoid(x+dx*s,z+dz*s,y+hh*s,rad*s,rad*.91*s,rad*.75*s,
                          ["a37887","af828d","987789"][(i+int(hh*10))%3],9,4,rough=.15)
    for i in range(230):
        cx,cz=[(-9.0,4.8),(-12.1,2.7),(-15.3,12.8),(-4.5,9.2),(-18.7,10.4)][i%5]
        x,z=cx+rng.uniform(-2.1,2.1),cz+rng.uniform(-2.1,2.1)
        y=h(x,z)
        if y<-.14 or landscape.road_distance(-137+x,-13+z)<1.0:
            continue
        if i%6==0:
            moor.pole((x,z,y),(x+.05,z,y+.18),.036,"d8bf9c",6,tip=.029)
            moor.ellipsoid(x+.05,z,y+.194,.094,.092,.058,"cb998f",7,4)
        else:
            api.grass(moor,x,z,y,rng.uniform(.21,.43),["9ca167","95ac77","8c9a6b"][i%3],5)

    palettes=[
        ("copper",copper,["937b62","b19a71","a79468"]),
        ("wind",wind,["a59681","abb68a","c1a581"]),
        ("moor",moor,["a28786","a6a780","998277"]),
    ]
    for theme,m,colors in palettes:
        ox,oz=ORIGINS[m.asset]
        planted=[]
        for i in range(560):
            x,z=rng.uniform(-56,56),rng.uniform(-54,54)
            wx,wz=ox+x,oz+z
            y=landscape.height(wx,wz)
            if math.hypot(x*.97,z)<23.8 or landscape.edge(wx,wz)<3.2 or y<-.1 or landscape.road_distance(wx,wz)<1.32:
                continue
            if theme=="moor" and x>15. and z<-11.:
                continue
            if any(math.hypot(x-a,z-b)<2.8 for a,b in planted):
                continue
            if i%7==0:
                scale=rng.uniform(.64,1.34)
                planted.append((x,z))
                if theme=="moor" and i%3!=0:
                    m.pole((x,z,y),(x+.07,z,y+2.55*scale),.223*scale,"886e57",8,tip=.105)
                    m.obstacle_circle(x,z,.27*scale)
                    for dx,dz,dy,r in [(-.65,.11,2.16,.71),(.56,-.24,2.48,.76),(.14,.17,2.84,.82)]:
                        m.ellipsoid(x+dx*scale,z+dz*scale,y+dy*scale,r*scale,r*.85*scale,r*.80*scale,
                                    ["a37483","b48b90","947386"][(i+int(dy))%3],8,4,rough=.16)
                else:
                    api.pine(m,x,z,y,scale,i)
            elif i%5==0:
                radius=rng.uniform(.42,1.02)
                m.ellipsoid(x,z,y+.25,radius,radius*.84,radius*.6,colors[i%3],8,4,rough=.21)
                m.obstacle_circle(x,z,radius*.68)
            elif theme=="moor":
                api.fern(m,x,z,rng.uniform(.23,.61),y)
            else:
                api.grass(m,x,z,y,rng.uniform(.22,.39),colors[i%3],4)
    for theme,m,_ in palettes:
        ox,oz=ORIGINS[m.asset]
        route_details.dress(api,theme,m,lambda x,z: landscape.height(ox+x,oz+z),(ox,oz))
    return result
