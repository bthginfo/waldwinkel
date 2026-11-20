"""Exterior terrain and walkable surfaces shared by Blender and the game."""
import math
import random
import struct

FOREST = (-51., -8.)
BEACH = (4., 47.)
TOWN = (49., 4.)
ORIGINS = {'district_forest': FOREST, 'district_beach': BEACH, 'district_town': TOWN}
LANDFORMS = [(0., -1., 31., 24.), (-50., -11., 34., 26.), (47., 3., 29., 24.),
           (1.5, 47., 29., 25.), (-26., 25., 25., 21.), (31., 29., 23., 19.)]
INLETS = [(5.8, 22.5, 6.45, 6.38), (22.5, 53.8, 13.7, 13.40)]
SEA = -.79
BRIDGES = [.81, -12.0, 12.6]  # Forest-local z coordinates.
BLOCKERS = []

ROADS = [
    [(-4., 4.), (-9., 4.), (-15.5, 1.4), (-22.5, -2.), (-29.5, -3.4),
     (-36.5, -4.4), (-44.4, -5.9), (-47.6, -7.19), (-54.7, -7.19),
     (-60., -5.4), (-67.4, -8.8), (-72.8, -14.4), (-70.2, -21.7),
     (-57., -20.), (-46.5, -20.), (-42., -14.2), (-44.4, -5.9)],
    [(-70.2, -21.7), (-68.1, -29.4), (-59.4, -30.0)],
    [(-67.4, -8.8), (-67.8, -0.8), (-60., 4.6), (-46., 4.6),
     (-39., 8.6), (-32., 16.5), (-29., 24.), (-20., 30.), (-11., 32.)],
    [(6., 4.2), (11., 6.), (17.4, 5.6), (25., 5.2), (34.5, 6.2),
     (42.3, 8.6), (49.0, 9.6), (57.3, 9.6), (63.5, 10.5)],
    [(0., 6.5), (-1.8, 11.2), (-3.2, 17.9), (-2.3, 24.5),
     (-1.6, 31.), (0.9, 36.7), (1.5, 42.5), (1.7, 47.5),
     (-4.8, 50.2), (-9.5, 56.), (-16., 56.4), (-20., 52.4)],
    [(49., 9.6), (50.2, 15.4), (45., 21.2), (39.4, 25.3), (31., 28.7),
     (23.8, 33.), (16., 35.8), (8.7, 39.3), (1.5, 42.5)],
    [(-11., 32.), (-16.5, 37.4), (-20., 44.), (-20., 52.4)],
    [(57.3, 9.6), (57.7, 14.1), (57.8, 20.5), (58., 24.)],
]

TOWN_BUILDINGS = [
    (-14.2, -.9, 3.48, 2.75, 'e2c7a5', 'a47362', .04, 'dormer'),
    (-12.1, -6.4, 3.11, 2.9, 'd7c29a', '8c9c7e', -.20, 'chimney'),
    (-8.1, -8.3, 3.54, 2.85, 'b4c5ac', 'b48263', -.12, 'dormer'),
    (-2.7, -9.0, 4.01, 2.75, 'e5c799', 'b9876e', .36, 'chimney'),
    (3.3, -8.5, 3.27, 2.74, 'd6bba3', '8e978b', .08, 'dormer'),
    (8.3, -7.9, 3.45, 2.68, 'd9c6a5', 'b37c66', -.12, 'chimney'),
    (13.6, -5.7, 3.78, 3.16, 'c6cdaa', '9e806e', .17, 'dormer'),
    (15.1, 2.1, 3.73, 2.8, 'e2b996', 'ac7460', .23, 'dormer'),
    (-7.5, 11.7, 3.59, 2.63, 'bdc4a4', 'a87763', -.04, 'dormer'),
    (-2.3, 13.4, 3.43, 2.64, 'e8ceac', '989a7a', -.12, 'chimney'),
    (13.9, 12.2, 3.23, 2.45, 'd9bc9b', 'b98568', -.23, 'chimney'),
]

CORE_BUILDINGS = [(-7.47,-3.20,3.36,2.8),(-3.44,-4.26,3.65,2.65),(.61,-4.65,3.77,2.65),
                  (4.61,-4.18,3.40,2.55),(8.19,-2.98,3.1,2.63)]
# Preserve the original gathering spots and extend exploration to the new trails.
FINDS = [
    (-6.8,2.4),(-4.50,-2.70),(3.62,-2.82),(7.65,2.8),(-5.65,4.75),(5.1,-3.9),(6.8,1.1),
    (2.75,4.15),(4.25,1.86),(4.6,-1.7),(-1.1,4.8),(-6.2,2.8),(-3.0,4.55),
]
EXTRA_FOREST_FINDS = [(-16.,-11.5),(-13.3,11.1),(12.45,-11.6),(-17.3,-20.1),(15.2,11.96),(-16.2,11.15)]
EXTRA_BEACH_FINDS = [(-14.1,11.8),(-18.,5.4),(-7.6,15.2),(.2,16.1),(-17.8,-1.7),(-9.3,9.2)]


def clamp(t, low=0., high=1.):
    return min(high, max(low, t))


def smooth(t):
    t = clamp(t)
    return t*t*(3.-2.*t)


def ellipse(x, z, shape):
    a,b,rx,rz = shape
    return (1.-math.hypot((x-a)/rx,(z-b)/rz))*min(rx,rz)


def edge(x, z):
    positions=[ellipse(x,z,i) for i in LANDFORMS]
    positions.sort(reverse=True)
    a,b=positions[:2]
    soft = max(0.,2.7-(a-b))
    land = a + soft*soft/10.8
    return min(land, *(-ellipse(x,z,i) for i in INLETS))


def stream(z):
    return -1.15+math.sin(z*.45)*1.15


def channel(x,z):
    fz=z-FOREST[1]
    if not -26.4 < fz < 24.7:
        return 0.
    center=FOREST[0]+stream(fz)
    fade=smooth((fz+26.4)/1.35)*smooth((24.7-fz)/1.6)
    return fade*(1.-smooth((abs(x-center)-.48)/.69))


def height(x,z):
    d=edge(x,z)
    sand = beach_mix(x,z)
    # A wider, shallower shelf along the sand and an abrupt bank in the hills.
    shoreline = 4.1 + sand*2.2
    bank=-1.43+1.43*smooth((d+shoreline-1.12)/shoreline)
    return bank - channel(x,z)*.61


def deck_height(x,z):
    fz=z-FOREST[1]
    for b in BRIDGES:
        if abs(fz-b)<.60 and abs(x-(FOREST[0]+stream(b)))<1.73:
            return .244
    if 56.85<x<59.2 and 23.45<z<27.41:
        return .188
    if 10.1<x<15.56 and 47.8<z<49.05:
        return .181
    return None


def beach_mix(x,z):
    return smooth((z-36.)/7.)*smooth((x+25.)/6.)*smooth((30.-x)/9.)


def road_distance(x,z):
    best=1.e9
    for road in ROADS:
        for (ax,az),(bx,bz) in zip(road,road[1:]):
            dx,dz=bx-ax,bz-az
            t=clamp(((x-ax)*dx+(z-az)*dz)/(dx*dx+dz*dz))
            best=min(best,(x-ax-dx*t)**2+(z-az-dz*t)**2)
    return math.sqrt(best)


def block_circle(x,z,r):
    BLOCKERS.append((x-r,z-r,x+r,z+r,(x,z,r)))


def block_rect(x,z,w,l):
    BLOCKERS.append((x-w/2,z-l/2,x+w/2,z+l/2,None))


def is_gathering_spot(x,z):
    for (origin, points) in [(FOREST,FINDS[:7]+EXTRA_FOREST_FINDS),
                             (BEACH,FINDS[7:]+EXTRA_BEACH_FINDS)]:
        if any((x-origin[0]-a)**2+(z-origin[1]-b)**2 < 1.12**2 for a,b in points):
            return True
    return False


def prepare_collisions():
    BLOCKERS.clear()
    for x,z,w,d,*_ in CORE_BUILDINGS+TOWN_BUILDINGS:
        block_rect(TOWN[0]+x,TOWN[1]+z,w+.13,d+.23)
    block_circle(TOWN[0],TOWN[1]+.30,1.41)
    for x,z,w,d in [(-2.71,-4.49,2.84,1.98)]:
        block_rect(BEACH[0]+x,BEACH[1]+z,w,d)


def bake_heightmap(root):
    """Pack a 0.4 m heightfield with a blocked bit in every sample."""
    ox,oz,step=-87.,-43.,.4
    cols,rows=426,305
    samples=[]
    for j in range(rows):
        z=oz+j*step
        for i in range(cols):
            x=ox+i*step
            deck=deck_height(x,z)
            y=height(x,z) if deck is None else deck
            samples.append([round(y*100),False])
    for left,top,right,bottom,circle in BLOCKERS:
        for j in range(max(0,math.ceil((top-oz)/step)),min(rows,math.floor((bottom-oz)/step)+1)):
            z=oz+j*step
            for i in range(max(0,math.ceil((left-ox)/step)),min(cols,math.floor((right-ox)/step)+1)):
                x=ox+i*step
                if circle is None or (x-circle[0])**2+(z-circle[1])**2 < circle[2]**2:
                    samples[j*cols+i][1]=True
    out=root/'assets'/'world'/'terrain.bin'
    out.parent.mkdir(parents=True,exist_ok=True)
    data=bytearray(struct.pack('<4sfffHH',b'WLD5',ox,oz,step,cols,rows))
    for h,obstacle in samples:
        data += struct.pack('<hB',h,obstacle)
    out.write_bytes(data)


def terrain_models(api):
    Model,blend=api.Model,api.blend
    rng=random.Random(4839)
    land=Model('Durchgehende Küste · Erde und Wiesen','mainland',casts=False)
    sea=Model('Gezeitenwasser · ein Wasserspiegel','mainland',casts=False)
    surf=Model('Brandung an der Küste','mainland',casts=False)
    paths=Model('Wanderwege, Sandpfade und Hafenstraße','mainland',casts=False)
    waterway=Model('Der Farnbach','mainland',casts=False)

    def water_color(x,z):
        v=clamp(.54-z*.004+.13*math.cos(x*.06+z*.13))
        return blend('53aeb3','92d7ca',v)

    for x in range(-202,203,12):
        for z in range(-140,173,12):
            poly=[(x,z,SEA),(x+12,z,SEA),(x+12,z+12,SEA),(x,z+12,SEA)]
            sea.gradient(poly,[water_color(a,b) for a,b,_ in poly])
    for _ in range(450):
        x,z=rng.uniform(-100,100),rng.uniform(-66,93)
        if height(x,z)>SEA-.12:
            continue
        l,w=rng.uniform(.16,1.02),rng.uniform(.014,.065)
        sea.face([(x-l,z,SEA+.013),(x,z-w,SEA+.013),(x+l,z,SEA+.013),(x,z+w,SEA+.013)],
                 blend('ceedda','73c1bc',rng.uniform(.25,.82)),'raw')

    step=1.06
    verts={}
    for iz in range(114):
        for ix in range(162):
            x,z=-86.+ix*step,-42.+iz*step
            if (ix*iz)%3 and 0<ix<161 and 0<iz<113:
                x+=rng.uniform(-.17,.17)
                z+=rng.uniform(-.17,.17)
            verts[ix,iz]=(x,z,height(x,z))

    def paint(tri):
        x,z,h=(sum(v[c] for v in tri)/3 for c in range(3))
        forest=clamp((-x-17)/23)*clamp((22-z)/19)
        grain=.52+.22*math.sin(x*.16+z*.31)+.14*math.cos(z*.48-x*.62)+rng.uniform(-.09,.09)
        grass=blend(blend('9fac65','c3c479',grain),blend('758a5e','aab569',grain),forest)
        sand=blend('e4c088','f1dca1',grain)
        tint=blend(grass,sand,beach_mix(x,z))
        if h<-.09:
            rock=blend('b89572','a79581',grain)
            tint=blend(tint,rock,clamp((-h-.07)*1.46))
        land.face(tri,tint,'ground')
        # Draw the shoreline at the actual intersection with the water plane.
        intersections=[]
        for a,b in zip(tri,tri[1:]+tri[:1]):
            if (a[2]>SEA)!=(b[2]>SEA):
                v=(SEA-a[2])/(b[2]-a[2])
                intersections.append((a[0]+(b[0]-a[0])*v,a[1]+(b[1]-a[1])*v))
        if len(intersections)!=2:
            return
        (ax,az),(bx,bz)=intersections
        mx,mz=(ax+bx)*.5,(az+bz)*.5
        nx,nz=height(mx-.12,mz)-height(mx+.12,mz),height(mx,mz-.12)-height(mx,mz+.12)
        length=math.hypot(nx,nz)
        if length<.001:
            return
        nx,nz=nx/length,nz/length
        width=.64+.86*beach_mix(mx,mz)
        wet=blend(water_color(mx,mz),'b2e1c3',.66)
        poly=[(ax,az,SEA+.008),(bx,bz,SEA+.008),
              (bx+nx*width,bz+nz*width,SEA+.008),(ax+nx*width,az+nz*width,SEA+.008)]
        surf.gradient(poly,[wet,wet,water_color(bx,bz),water_color(ax,az)])
        if rng.random()<.71:
            w=rng.uniform(.047,.138)
            foam=[(ax+nx*.12,az+nz*.12,SEA+.019),(bx+nx*.12,bz+nz*.12,SEA+.019),
                  (bx+nx*(.12+w),bz+nz*(.12+w),SEA+.019),(ax+nx*(.12+w),az+nz*(.12+w),SEA+.019)]
            surf.face(foam,blend('c6e9d0','f1e7c3',beach_mix(mx,mz)),'raw')

    for iz in range(113):
        for ix in range(161):
            a,b,c,d=[verts[x,z] for x,z in [(ix,iz),(ix+1,iz),(ix+1,iz+1),(ix,iz+1)]]
            if max(v[2] for v in (a,b,c,d))<SEA-.28:
                continue
            for tri in [(a,b,c),(a,c,d)] if (ix+iz)%2 else [(a,b,d),(b,c,d)]:
                paint(tri)
    for road in ROADS:
        centers=[]
        for j,(b,c) in enumerate(zip(road,road[1:])):
            a,d=road[max(0,j-1)],road[min(len(road)-1,j+2)]
            segments=max(2,math.ceil(math.dist(b,c)/.50))
            for i in range(segments):
                t=i/segments
                centers.append(tuple(.5*(2*p+(q-u)*t+(2*u-5*p+4*q-v)*t*t+
                                            (v-u+3*p-3*q)*t*t*t)
                                     for u,p,q,v in zip(a,b,c,d)))
        centers.append(road[-1])
        rows=[]
        for i,(x,z) in enumerate(centers):
            a,b=centers[max(0,i-1)],centers[min(len(centers)-1,i+1)]
            dx,dz=b[0]-a[0],b[1]-a[1]
            direction=math.hypot(dx,dz)
            nx,nz=-dz/direction,dx/direction
            width=(.80 if x < -24 else 1.03)*(.96+.08*math.sin(x*.81+z*.54))
            rows.append([(x+nx*width*k,z+nz*width*k,height(x+nx*width*k,z+nz*width*k)+.034)
                         for k in [-1.,-.73,.73,1.]])
        for a,b in zip(rows,rows[1:]):
            if min(a[1][2],b[1][2])<-.38:
                continue
            for lane in range(3):
                tint=blend('cbad7d','ead1a1',rng.uniform(.25,.84))
                if lane!=1:
                    tint=blend(tint,'b7b879',.53)
                paths.face([a[lane],b[lane],b[lane+1],a[lane+1]],tint,'ground')

    for i in range(151):
        z=-26.15+i*.334
        zz=z+.334
        a,b=FOREST[0]+stream(z),FOREST[0]+stream(zz)
        z,zz=z+FOREST[1],zz+FOREST[1]
        for j in range(5):
            x=j*.377-.9425
            poly=[(a+x,z,-.275),(a+x+.378,z,-.275),(b+x+.378,zz,-.275),(b+x,zz,-.275)]
            waterway.face(poly,blend(['91b9a0','78beb2','5eaaa7','72b5a7','8fbcaa'][j],'c4dbc2',rng.uniform(0,.19)),'raw')
        if i%4==0:
            x=a+rng.uniform(-.53,.51)
            waterway.face([(x-.12,z,-.265),(x,z-.03,-.265),(x+.13,z,-.265),(x,z+.035,-.265)],'b8ddc8','raw')

    return [land,sea,surf,paths,waterway]
