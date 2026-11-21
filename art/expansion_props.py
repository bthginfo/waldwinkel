"""Hand-built props, wildlife and furniture for the outer districts."""
from math import cos, pi, sin
from random import Random

from build_scene import Model, TAU, blend, point


def boulder(m,x,z,h,size,color,seed=.31):
    m.ellipsoid(x,z,h+size*.32,size,size*.73,size*.61,color,7,3,angle=seed,rough=.17)


def plant(m,x,z,h,scale,color,leaf='708351',stems=5):
    for n in range(stems):
        a=n*2.4
        wide=scale*(.17+(n%3)*.045)
        top=(x+cos(a)*wide,z+sin(a)*wide,h+scale*(.49+.17*(n%4)))
        m.pole((x,z,h+.04),top,.026*scale,leaf,4,tip=.013*scale)
        m.ellipsoid(x+cos(a)*wide*.70,z+sin(a)*wide*.70,h+scale*.33,
                    .18*scale,.068*scale,.027*scale,leaf,5,2,angle=a)
        for i in range(2):
            m.ellipsoid(top[0],top[1],top[2]-.125*scale*i,.080*scale,.074*scale,.116*scale,
                        blend(color,'fff2c6',.14*i),6,3)


def crystal(m,x,z,h,r,height,palette,angle=0.):
    rings=[]
    for k,(scale,dy,tilt) in enumerate([(1.,0.,0.),(1.,.53,.14),(.64,.83,.21),(.04,1.,.28)]):
        rings.append([(x+(cos(angle+i*TAU/5)*scale+tilt)*r,z+sin(angle+i*TAU/5)*r*scale,h+dy*height)
                      for i in range(5)])
    colors=[palette[(i+row)%len(palette)] for row in range(3) for i in range(5)]
    m.rings(rings,colors+[palette[0],palette[-1]])


def lamp(m,x,z,h,paint='7d5536',halo='ffe8a5'):
    m.pole((x,z,h),(x,z,h+1.91),.082,paint,6)
    m.pole((x,z,h+1.80),(x+.34,z,h+1.80),.057,paint,6)
    m.box(x+.28,z,h+1.60,.27,.25,.32,halo,.050,mode='light')
    for shift in (-.125,.125):
        for depth in (-.111,.111):
            m.pole((x+.28+shift,z+depth,h+1.43),(x+.28+shift,z+depth,h+1.79),.016,paint,5)
    m.box(x+.28,z,h+1.78,.39,.33,.075,paint,.021)
    m.box(x+.28,z,h+1.42,.32,.31,.052,paint,.021)


def slats(m,x,z,h,width,depth,thickness,palette,axis=0, angle=0.):
    n=max(1,round((depth if axis==0 else width)/.20))
    ca,sa=cos(angle),sin(angle)
    for i in range(n):
        if axis==0:
            dx,dz=0,(i+.5)/n*depth-depth*.5
            w,d=width,depth/n-.014
        else:
            dx,dz=(i+.5)/n*width-width*.5,0
            w,d=width/n-.014,depth
        m.box(x+dx*ca-dz*sa,z+dx*sa+dz*ca,h,w,d,thickness,palette[i%len(palette)],.010,angle)


def fence(m,x,z,length,h,angle,paint,base):
    count=max(1,round(length/1.32))
    dx,dz=cos(angle),sin(angle)
    spots=[(x+(i/count-.5)*length*dx,z+(i/count-.5)*length*dz) for i in range(count+1)]
    for a,b in spots:
        y=base(a,b)
        m.box(a,b,y+.38,.125,.128,.84,paint,.021)
        m.box(a,b,y+.79,.16,.16,.086,blend(paint,'edd6a8',.33))
    for (a,b),(c,d) in zip(spots,spots[1:]):
        for height in (.39,.68):
            m.pole((a,b,base(a,b)+height),(c,d,base(c,d)+height),.048,paint,5)


def window(m,x,z,h,w,d):
    m.box(x,z,h,w+.13,.102,d+.15,'705742',.036)
    m.box(x,z+.050,h,w,.043,d,'e3b575',.019,mode='light')
    m.box(x,z+.072,h,w,.043,.038,'bc9767',.009)
    m.box(x,z+.074,h,.039,.043,d,'bc9767',.009)
    m.box(x,z+.083,h-d*.5-.026,w+.19,.15,.070,'d2ad7b',.018)

def building(m,x,z,h,w,d,kind,variant=0):
    """A stone footing, a post-and-beam house and individually laid roof shakes."""
    siding,wood,roof,trim,stone = {
        'amber': ('967146','664c32','ac7444','dcbd78','938777'),
        'heights': ('e0d6b6','634f44','876a5c','cfad83','8b9396'),
        'reef': ('8caea2','5f7775','74937f','f0dbaf','aaa69c'),
    }[kind]
    dx,dz=w*.5,d*.5
    wall=1.9 if variant==0 else 1.65
    ridge=wall+w*.36
    floor=.15
    m.box(x,z,h+.04,w+.30,d+.23,.36,stone,.11)
    for a in range(max(6,round(w/.30))):
        xx=x-dx+(a+.50)*w/max(6,round(w/.30))
        m.box(xx,z+dz+.01,h+.12,w/max(6,round(w/.30))*.93,.20,.22,
              blend(stone,'ead4ab',.11+(a%4)*.071),.028)
    m.box(x,z,h+wall*.52,w,d,wall,siding,.055)
    for a in range(-1,2):
        if a==0 and variant!=0:
            continue
        for b in (-1,1):
            m.box(x+a*(dx-.06),z+b*(dz+.025),h+wall*.55,.123,.13,wall+.05,wood,.018)
    for b in (-1,1):
        m.box(x,z+b*(dz+.03),h+wall,w+.20,.14,.161,wood,.019)
        m.box(x+b*(dx+.03),z,h+wall,.152,d+.23,.16,wood,.018)
        if kind in ('amber','reef'):
            for row in range(7):
                height=.26+row*.22
                m.box(x,z+b*(dz+.01),h+height,w-.11,.030,.034,blend(siding,wood,.36))
                m.box(x+b*(dx+.013),z,h+height,.030,d-.06,.034,blend(siding,wood,.36))
        else:
            for row in range(4):
                for i in range(4):
                    xx=x+dx*(i*.44-.65+(row%2)*.18)
                    m.box(xx,z+b*(dz+.024),h+.37+row*.26,.22+(i%3)*.09,.065,.12,
                          blend(stone,siding,.30+i*.13),.029)
        m.face([(x-dx,z+b*dz,h+wall),(x+dx,z+b*dz,h+wall),(x,z+b*dz,h+ridge)],siding)
        m.pole((x-dx-.16,z+b*(dz+.12),h+wall-.05),(x,z+b*(dz+.12),h+ridge+.07),.084,trim,5)
        m.pole((x+dx+.16,z+b*(dz+.12),h+wall-.05),(x,z+b*(dz+.12),h+ridge+.07),.084,trim,5)
    eave=dx+.30; end=dz+.25
    for side in (-1,1):
        for row in range(7):
            f0=row/7; f1=min(1.035,(row+1.10)/7)
            x0=x+side*eave*f0; x1=x+side*eave*f1
            h0=h+ridge-(ridge-wall+.20)*f0+(7-row)*.014
            h1=h+ridge-(ridge-wall+.20)*f1+(7-row)*.014
            columns=max(6,round(d/.35))
            for index in range(columns):
                z0=z-end+2*end*index/columns+.007
                z1=z-end+2*end*(index+1)/columns-.007
                tile=blend(roof,trim,.025+(index*3+row*11)%8*.038)
                m.face([(x0,z0,h0),(x1,z0,h1),(x1,z1,h1),(x0,z1,h0)],tile)
                m.face([(x1,z0,h1),(x1,z0,h1-.049),(x1,z1,h1-.049),(x1,z1,h1)],blend(tile,wood,.36))
        m.pole((x+side*eave,z-end-.05,h+wall-.16),(x+side*eave,z+end+.05,h+wall-.16),.061,trim,5)
    m.pole((x,z-end-.07,h+ridge+.06),(x,z+end+.08,h+ridge+.06),.092,wood,7)
    e=z+dz+.071
    m.box(x-dx*.29,e,h+floor+.60,.65,.11,1.20,wood,.037)
    m.box(x-dx*.29,e+.065,h+floor+.60,.55,.036,1.11,blend(siding,wood,.55),.023)
    m.ellipsoid(x-dx*.29+.18,e+.095,h+.68,.047,.046,.047,'d2ad67',7,3)
    window(m,x+dx*.48,e,h+1.17,.65,.70)
    window(m,x,e+.006,h+wall+.34,.32,.39)
    for i in range(2):
        m.box(x-dx*.29,z+dz+.24+i*.23,h+.07-i*.07,1.21,.39,.17,stone,.041)
    m.box(x+dx*.59,z-dz*.43,h+ridge-.08,.40,.44,1.40,stone,.062)
    m.box(x+dx*.59,z-dz*.43,h+ridge+.64,.51,.55,.12,wood,.027)
    if variant==0:
        shade=blend(siding,'d6b985',.50)
        slats(m,x,z+dz+.50,h+.09,w+.52,1.10,.095,('987753','b99569','c4a57a'))
        for side in (-1,1):
            m.pole((x+side*(dx-.04),z+dz+.92,h),(x+side*(dx-.04),z+dz+.92,h+wall-.25),.084,wood,6)
        for i in range(6):
            d0=i*.19
            m.face([(x-dx-.14,z+dz+d0,h+wall-d0*.24),(x+dx+.14,z+dz+d0,h+wall-d0*.24),
                    (x+dx+.14,z+dz+d0+.21,h+wall-(d0+.21)*.24),
                    (x-dx-.14,z+dz+d0+.21,h+wall-(d0+.21)*.24)],blend(shade,roof,i%3*.17))


def crate(m,x,z,h,paint='b88a57'):
    m.box(x,z,h+.227,.45,.40,.46,paint,.032)
    for i in range(3):
        m.box(x,z+.204,h+.104+i*.145,.44,.026,.104,blend(paint,'ead0a4',.18+i*.045),.015)
    for a in (-.171,.171):
        m.box(x+a,z+.23,h+.228,.064,.028,.46,'775638',.011)


def resource(variant, winter=False):
    name=f'extension_node_{variant}'+('_snow' if winter else '')
    m=Model(name,name)
    if variant in (9,10,15):
        trunk,leaf,h,spread = {
            9: ('d6c3a2','dfae4d',3.45,1.24), 10: ('705746','a38339',4.02,1.18),
            15: ('947b61','547e61',3.16,1.33)}[variant]
        if winter:
            leaf=blend(leaf,'d2e2de',.67)
        m.pole((0,0,-.035),(.13,.11,h*.78),.175,trunk,8,tip=.079)
        for n in range(5):
            a=n*2.39
            sz=spread*(.68+.18*(n%3))
            end=(cos(a)*sz*.56,sin(a)*sz*.56,h*(.64+n*.067))
            m.pole((.04,.02,h*.36),end,.081,trunk,6,tip=.023)
            color=blend(leaf,'f6d493' if variant!=15 or winter else '9fa667',.07+.11*(n%3))
            m.ellipsoid(*end,sz*.71,sz*.65,sz*.54,color,8,4,angle=a,rough=.18,mode='leaf')
            m.ellipsoid(end[0]-.24,end[1]-.17,end[2]+.25,sz*.55,sz*.46,sz*.40,blend(color,'fff2cb',.17),7,3,rough=.12,mode='leaf')
        for i in range(13):
            a=i*2.38
            hh=.28+i*.16
            radius=.16-hh*.02
            m.ellipsoid(.052*hh+cos(a)*radius,.035*hh+sin(a)*radius,hh,.090,.080,.025,
                        '806b54' if variant==9 else 'a68d70',5,2,angle=a)
        for i in range(5):
            a=i*TAU/5
            m.pole((0,0,.13),(cos(a)*.37,sin(a)*.37,.015),.078,trunk,5,tip=.011)
    else:
        colors={
            11: ('777c87',('4c596b','738eab','bacedd','7584a1')),
            12: ('6f7d84',('6bbac1','afe2dc','d9f2dc','8dcfda')),
            13: ('95816a',('d1913c','f7c658','f0aa43','ddb06c')),
            14: ('a59d86',('a8bdaa','d9ccc1','e7edc9','bdb8c2')),
        }
        rock,tones=colors[variant]
        boulder(m,0,0,-.09,.72,rock)
        boulder(m,.44,-.11,-.102,.43,blend(rock,'c5b59c',.18))
        boulder(m,-.48,.17,-.08,.37,blend(rock,'756957',.22))
        for i in range(7):
            a=i*2.39
            crystal(m,cos(a)*(.18+.076*(i%3)),sin(a)*.26,.30+.055*sin(i),.15+(i%3)*.034,.28+(i%4)*.13,tones,a)
    return m


def animal(kind):
    name='farm_'+kind
    m=Model(name,name)
    poses=[]
    def limb(label,x,z,pivot,low,r,color):
        part=Model(label,name)
        part.pivot=point(x,z,pivot)
        part.pole((x,z,pivot+.035),(x,z,low),r,color,7,tip=r*.68)
        part.ellipsoid(x,z+.035,low,.95*r,1.26*r,.56*r,'665144' if kind!='duck' else 'c97b49',7,3)
        poses.append(part)

    if kind=='rabbit':
        fur,pale='c2a186','efe3cf'
        m.ellipsoid(0,-.09,.243,.244,.344,.239,fur,12,5)
        m.ellipsoid(0,.196,.404,.185,.174,.180,pale,11,5)
        m.ellipsoid(0,-.382,.245,.112,.104,.137,'f2e6d8',9,4)
        for s in (-1,1):
            m.ellipsoid(s*.10,.137,.660,.062,.067,.249,fur,10,5,angle=s*.16)
            m.ellipsoid(s*.107,.186,.667,.034,.021,.194,'e3ae9e',8,4)
            m.ellipsoid(s*.111,.318,.44,.038,.023,.049,'403432',9,4)
            m.ellipsoid(s*.105,.337,.459,.013,.009,.013,'fff2d7',6,3,mode='raw')
            m.ellipsoid(s*.064,.343,.369,.077,.056,.068,'e9d7be',8,4)
        m.ellipsoid(0,.384,.392,.036,.019,.023,'bc8a80',8,4)
    elif kind=='goat':
        fur='e2d9be'
        m.ellipsoid(0,-.052,.55,.242,.518,.286,fur,12,5,rough=.10)
        m.ellipsoid(0,.321,.62,.165,.199,.286,'ede5ce',10,5)
        m.ellipsoid(0,.443,.829,.170,.229,.170,'e8debf',11,5)
        m.ellipsoid(0,.579,.764,.130,.135,.096,'c9b5a5',10,4)
        m.pole((0,.58,.74),(-.024,.53,.53),.061,'e0cab0',7,tip=.010)
        m.pole((0,-.48,.62),(.02,-.61,.82),.057,'d4c3a4',7,tip=.021)
        for s in (-1,1):
            for a,b,r in zip([(s*.103,.387,.945),(s*.131,.297,1.045),(s*.16,.240,1.11)],
                    [(s*.131,.297,1.045),(s*.16,.240,1.11),(s*.17,.188,1.143)], [.062,.046,.029]):
                m.pole(a,b,r,'ac9071',7,tip=r*.71)
            m.ellipsoid(s*.179,.349,.834,.121,.056,.078,'ccb89f',8,4,angle=s*.27)
            m.ellipsoid(s*.118,.586,.868,.038,.028,.037,'403634',8,4)
            m.ellipsoid(s*.112,.602,.885,.013,.010,.013,'fff0cb',6,3,mode='raw')
        m.pole((-.112,.444,.633),(.112,.444,.633),.036,'638878',7)
        m.ellipsoid(0,.482,.591,.054,.040,.049,'c99a55',8,4)
    else:
        fur='e9ddbe'
        m.ellipsoid(0,-.077,.361,.185,.328,.259,fur,11,5,angle=.1)
        m.ellipsoid(0,.135,.565,.106,.129,.249,fur,10,5)
        m.ellipsoid(0,.199,.785,.127,.151,.123,'e8dfc8',11,5)
        m.ellipsoid(0,.338,.749,.101,.126,.045,'dc9a5f',9,3)
        m.pole((0,-.23,.406),(0,-.36,.486),.086,'cdbaa1',7,tip=.024)
        for s in (-1,1):
            m.ellipsoid(s*.139,-.115,.410,.080,.207,.123,'d1b99b',9,4)
            m.ellipsoid(s*.081,.307,.805,.024,.018,.029,'39302a',8,4)
            m.ellipsoid(s*.077,.322,.817,.008,.006,.009,'fff3da',6,3,mode='raw')
    zs=(.16,) if kind=='duck' else (-.21,.23)
    for z in zs:
        for s in (-1,1):
            scale,pivot=(.067,.23) if kind=='rabbit' else (.062,.44) if kind=='goat' else (.046,.24)
            limb(f"Paw_{s}_{z}",s*(.106 if kind=='duck' else .137),z,pivot,.043,scale,
                 fur if kind!='duck' else 'c2804f')
    return [m,*poses]


def housing(kind):
    key={'rabbit':'farm_hutch','goat':'farm_goat_shed','duck':'farm_duck_house'}[kind]
    m=Model(key,key)
    style={'rabbit':'amber','goat':'heights','duck':'reef'}[kind]
    building(m,0,-.31,0,2.73,1.49,style,1)
    paint={'rabbit':'966e44','goat':'87775d','duck':'7b8f7e'}[kind]
    for side in (-1,1):
        fence(m,side*1.42,.48,1.46,0,pi/2,paint,lambda x,z:0.)
    crate(m,1.04,.68,-.04,paint)
    m.ellipsoid(-.83,.74,.12,.24,.17,.10,'aa8b57',9,4)
    m.ellipsoid(-.83,.74,.165,.19,.136,.037,'bcbc77',9,3)
    if kind=='duck':
        m.ellipsoid(-.69,.57,.049,.42,.30,.06,'5c8e9c',10,3)
        m.ellipsoid(-.70,.59,.09,.34,.23,.035,'84c9d0',10,3)
    return m


def products():
    p=[]
    for name in ('gold_resin','ironwood','amber','star_ore','alpine_sage','sky_quartz',
                 'mother_pearl','salt_bloom','reefwood','rabbit_down','goat_milk','duck_egg','duck_feather'):
        m=Model('find_'+name,'find_'+name)
        p.append(m)
        if name in ('gold_resin','amber','star_ore','sky_quartz'):
            colors={'gold_resin':('b87d38','edaa48','eac568','ffdc85'),
                    'amber':('b98347','e69d3e','f4c681','ffcf69'),
                    'star_ore':('5f6474','98b5c4','c9daea','65768b'),
                    'sky_quartz':('649aaa','a4e1e3','dbede2','93cedb')}[name]
            m.ellipsoid(0,0,.08,.29,.23,.12,colors[0],8,3,rough=.14)
            for i,(x,z,h) in enumerate([(-.14,.05,.22),(.01,-.035,.35),(.15,.06,.25),(.04,.14,.21)]):
                crystal(m,x,z,.08,.11,h,colors,i)
        elif name in ('salt_bloom','alpine_sage'):
            plant(m,0,0,0,.68,'be88b8' if name=='salt_bloom' else '91a47f',
                  '61826e' if name=='alpine_sage' else '76905f')
        elif name in ('ironwood','reefwood'):
            color='845d43' if name=='ironwood' else 'a79671'
            for i,z in enumerate((-.14,-.015,.13)):
                m.pole((-.26,z,.10+i*.02),(.29-i*.03,z+.02,.15+i*.02),.094,color,8)
                m.pole((.283-i*.03,z+.02,.15+i*.02),(.303-i*.03,z+.02,.15+i*.02),.069,'e2ba88',8)
        elif name=='mother_pearl':
            m.ellipsoid(0,0,.087,.28,.23,.108,'b59da4',12,4,rough=.08)
            m.ellipsoid(0,-.074,.22,.26,.128,.13,'dccab3',11,4)
            m.ellipsoid(0,.06,.146,.197,.151,.075,'b8c9b2',10,4)
            m.ellipsoid(0,.06,.217,.094,.084,.087,'f1e9cf',10,5)
        elif name=='goat_milk':
            m.ellipsoid(0,0,.213,.164,.166,.213,'dfeddd',12,6)
            m.pole((0,0,.30),(0,0,.52),.096,'d3dcc3',9,tip=.076)
            m.pole((0,0,.49),(0,0,.55),.082,'bba16e',9)
            m.box(0,.154,.239,.199,.029,.160,'9eb8a5',.022)
        elif name=='duck_egg':
            for i,(x,z) in enumerate([(-.116,.00),(.078,-.10),(.097,.11)]):
                m.ellipsoid(x,z,.135,.109,.100,.145,['e8dabe','d8d5b2','e2e7cb'][i],9,5,angle=i*.9)
        elif name=='rabbit_down':
            for i in range(7):
                a=i*2.4
                m.ellipsoid(cos(a)*.133,sin(a)*.108,.076+(i%3)*.056,.145,.101,.111,
                            ['e2cbb2','e9d9c6','f1e6d3'][i%3],8,4,rough=.13)
        else:
            for i in range(3):
                x=(i-1)*.114
                m.pole((x,.152,.015),(x+.09,-.16,.39),.016,'ae8d68',5)
                m.ellipsoid(x+.063,-.066,.239,.113,.049,.201,'f1dec1',9,4,angle=i*.12)
    return p


def furniture():
    models=[]
    for name in ('harzlaterne','flaumbank','bernsteinvitrine','salbeitrog','almbank','sternlampe',
                 'gezeitenwindspiel','perlmuttkasten','strandtisch','harzregal','gipfelglobus','entenboje'):
        m=Model('furniture_'+name,'furniture_'+name)
        models.append(m)
        if name in ('flaumbank','almbank','strandtisch'):
            coast=name=='strandtisch'; alpine=name=='almbank'
            paint='8ca697' if coast else '957951' if alpine else 'a57147'
            seat=.96 if coast else .47
            for x in (-.57,.57):
                for z in (-.27,.27):
                    m.box(x,z,seat*.53,.116,.116,seat+.04,paint,.027)
                m.box(x,0,.23,.106,.54,.11,paint,.016)
            slats(m,0,0,seat,1.52,1.06 if coast else .66,.098,
                  (paint,blend(paint,'eed5a7',.19),blend(paint,'eed5a7',.30)))
            if not coast:
                for x in (-.67,.67):
                    m.box(x,-.28,.74,.096,.10,.50,paint,.026)
                m.box(0,-.31,.84,1.42,.12,.23,paint,.045)
                m.box(0,.024,.55,1.31,.45,.097,'ad7c60' if alpine else 'e5caba',.044)
                for i in range(11):
                    m.box(-.64+.127*i,.024,.598,.014,.424,.017,'c6aa82',.008)
            else:
                m.box(0,0,.86,1.32,.26,.19,paint,.026)
                for i in range(4):
                    m.box(0,-.14+i*.09,1.029,1.44,.041,.022,'d7d5aa',.008)
        elif name in ('bernsteinvitrine','harzregal','perlmuttkasten'):
            short=name=='perlmuttkasten'
            frame='83a69a' if short else '966e44'
            depth=.66 if short else .58
            height=.82 if short else 1.54
            for x in (-.50,.50):
                m.box(x,0,height*.55,.115,depth,height*.9,frame,.03)
            for i,y in enumerate([.14,height*.54,height]):
                m.box(0,0,y,1.21,depth+.09,.102,blend(frame,'e6ba7e',i*.12),.035)
            m.box(0,-depth*.46,height*.57,.99,.10,height*.84,blend(frame,'d3b181',.24),.034)
            for i in range(6):
                m.box(-.42+.168*i,-depth*.30,height*.45,.013,.02,height*.58,'a18961',.005)
            if short:
                for y in (.30,.60):
                    m.box(0,.31,y,.97,.05,.211,'b8bf9b',.035)
                    m.ellipsoid(0,.349,y,.041,.025,.042,'e5d0b9',9,4)
            else:
                for i in range(4):
                    x=-.36+i*.237
                    crystal(m,x,.080,.203,.09,.15+i%3*.073,
                            ('bc883f','e1a448','eac568','f0d48b'),i)
                for i in range(3):
                    m.ellipsoid(-.32+i*.31,0,.924,.106,.114,.109,('aa8754','d2b774','88975c')[i],8,4)
        elif name=='salbeitrog':
            m.box(0,0,.23,1.20,.70,.41,'a49b87',.081)
            m.box(0,0,.455,1.22,.747,.093,'c9bca3',.044)
            m.box(0,0,.493,1.045,.56,.044,'74654d',.021)
            for i,x in enumerate((-.38,-.12,.15,.39)):
                plant(m,x,(i%2-.5)*.18,.507,.42+i%3*.07,'929d81','6f8976',4)
        elif name in ('harzlaterne','sternlampe','gezeitenwindspiel'):
            metal='6d8d80' if name!='harzlaterne' else '896640'
            m.ellipsoid(0,0,.071,.36,.32,.083,metal,9,4)
            m.pole((0,0,.06),(0,0,1.41),.071,metal,8,tip=.043)
            if name=='harzlaterne':
                m.box(0,0,1.28,.33,.31,.29,'eeb35d',.056,mode='light')
                for x in (-.151,.151):
                    for z in (-.14,.14):
                        m.pole((x,z,1.10),(x,z,1.45),.022,metal,5)
                m.box(0,0,1.45,.407,.365,.084,metal,.030)
            elif name=='sternlampe':
                crystal(m,0,0,1.05,.24,.47,('92cfca','a8dfcd','ededd4','aacecb'),.42)
                for i in range(10):
                    a=i*TAU/10
                    m.pole((cos(a)*.36,sin(a)*.29,1.15),(cos(a+TAU/10)*.36,sin(a+TAU/10)*.29,1.15),
                           .025,'b99a5f',6)
            else:
                m.pole((-.28,0,1.38),(.32,0,1.46),.047,metal,7)
                for i in range(5):
                    x=(i-2)*.115
                    hh=.93+(i%3)*.09
                    m.pole((x,0,1.42),(x,0,hh),.010,'bfa87b',5)
                    m.ellipsoid(x,0,hh,.068,.038,.128,('e4cfbb','bdcec2','d7b4b3')[i%3],9,4)
        elif name=='gipfelglobus':
            m.box(0,0,.08,.78,.63,.14,'927655',.051)
            for x in (-.31,.31):
                m.pole((x,0,.13),(x*.89,0,.52),.052,'ae925c',6)
            m.pole((-.29,0,.52),(.29,0,.52),.031,'c6a770',7)
            m.ellipsoid(0,0,.54,.29,.28,.28,'89a7a0',13,7,rough=.07)
            for i in range(12):
                a=i*TAU/12
                m.pole((cos(a)*.307,sin(a)*.303,.55),(cos(a+TAU/12)*.307,sin(a+TAU/12)*.303,.55),
                       .015,'dbb882',5)
            for i in (0,2,3,7,11):
                a=i*TAU/12
                m.ellipsoid(cos(a)*.23,sin(a)*.23,.62+(i%3)*.04,.128,.095,.064,'b8be85',6,3,rough=.12)
        else:
            m.ellipsoid(0,0,.2,.38,.35,.22,'ca9b64',11,5)
            m.ellipsoid(0,0,.31,.35,.32,.084,'ece0b8',10,4)
            m.pole((0,0,.27),(0,0,.74),.059,'967657',7)
            m.ellipsoid(0,.048,.745,.169,.199,.103,'e3d4a9',10,4)
            m.ellipsoid(0,.219,.738,.099,.106,.039,'c48359',9,3)
            for x in (-.077,.077):
                m.ellipsoid(x,.195,.787,.024,.020,.027,'5e4732',8,4)
    return models


def tools():
    p=[]
    tones=('678b94','9db9b8','c6ded6')
    for kind in ('axe','pickaxe','can','hoe'):
        asset='star_'+kind
        m=Model(asset,asset)
        p.append(m)
        # Match the original tool's hand-height origin in Bevy.
        if kind=='can':
            m.ellipsoid(0,0,-.101,.173,.161,.145,tones[1],10,5)
            m.box(0,0,.016,.25,.246,.067,tones[0])
            m.pole((0,.055,-.15),(0,.31,-.06),.043,tones[0],8,tip=.026)
            m.pole((0,.292,-.067),(0,.331,-.056),.060,tones[2],9)
            for a,b in zip([(0,-.139,-.14),(0,-.24,-.062),(0,-.21,.04)],
                           [(0,-.24,-.062),(0,-.21,.04),(0,-.12,.041)]):
                m.pole(a,b,.021,'966e50',7)
            continue
        m.pole((0,.0,-.29),(0,.0,.37 if kind=='axe' else .43),.034,'9d7755',8,tip=.025)
        for i in range(6):
            m.pole((0,0,-.27+i*.025),(0,0,-.257+i*.025),.037,'80604d',7)
        top=.31 if kind=='axe' else .42
        if kind=='axe':
            m.box(0,.115,top,.062,.263,.166,tones[1],.035)
            m.box(0,.246,top,.048,.046,.199,tones[2],.018)
        elif kind=='pickaxe':
            m.pole((0,-.23,top-.055),(0,0,top+.047),.059,tones[1],7,tip=.056)
            m.pole((0,0,top+.047),(0,.264,top-.041),.057,tones[0],7,tip=.012)
            m.pole((0,-.186,top-.04),(0,-.265,top-.105),.036,tones[2],7,tip=.006)
        else:
            m.box(0,.071,top-.024,.07,.192,.078,tones[1],.020)
            m.box(0,.154,top-.078,.173,.087,.101,tones[2],.018)
    return p
