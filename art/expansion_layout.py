"""Authored roads and landforms for the three outer districts.

Coordinates match Bevy's X/Z plane. The Blender exporter uses the same
heights for the rendered ground, movement grid and island atlas.
"""
import math
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SEA = -.79
STEP = .25
ATLAS_BOUNDS = (-323., -104., 321., 291.)
DATA = (ROOT / 'assets/world/terrain.bin').read_bytes()
_, OX, OZ, _, COLS, ROWS = struct.unpack_from('<4sfffHH', DATA)

SITES = {
    'amber': {
        'bounds': (-317, -74, -185, 51),
        'lobes': [(-254,-17,49,42),(-277,-6,27,37),(-240,9,40,24),(-272,-37,29,29)],
        'approach': [(-189,-14),(-203,-14),(-213,-14),(-224,-13),(-234,-10)],
        'roads': [[(-234,-10),(-243,-12),(-249,-12),(-256,-9),(-263,-5),(-270,-4),(-283,-5),(-291,0)],
                  [(-283,-5),(-286,4),(-282,12),(-274,16),(-264,14),(-253,9),(-249,-12)],
                  [(-270,-4),(-270,-11),(-272,-20),(-271,-27),(-263,-31),(-252,-28),(-241,-20),(-243,-12)],
                  [(-241,-20),(-240,-29),(-247,-38),(-258,-43),(-273,-44),(-285,-36)]],
        'spurs': [(-286,-36,9,15,5.9),(-265,-50,12,13,4.5),(-244,24,13,10,3.5),(-288,16,12,11,3.1)],
        'hubs': [(-249,-19,'Harzhütte'),(-277,3,'Harzschuppen'),(-265,-24,'Trockenscheune'),(-238,3,'Unterstand')],
        'reserved': [(-244,-11),(-246,-11),(-249,-12),(-250,-11),(-252,-8),(-255,-5),(-254,-15),(-257,-12),
                     (-243,-14),(-247,-11),(-254,-9),(-259,-15),(-261,-1),(-274,-4),(-275,-23),(-244,-27),
                     (-237,-2),(-258,-31),(-286,-8)],
        'palette': ('a6a15e','839352','d6b178','b69567','617447','674935'),
        'water': [(-267,-47),(-265,-32),(-269,-18),(-266,-5),(-262,8),(-266,20),(-270,39)],
        'crossings': [(-266,-5),(-264,14)],
    },
    'heights': {
        'bounds': (193, -101, 318, 44),
        'lobes': [(252,-25,43,44),(268,-58,32,34),(256,6,33,26),(284,-33,23,35)],
        'approach': [(195,4),(202,4),(206,4),(216,4),(227,4),(236,-2)],
        'roads': [[(236,-2),(243,-8),(247,-15),(248,-21),(251,-28),(261,-33),(266,-40),(267,-48),(275,-57),(274,-67)],
                  [(243,-8),(250,3),(266,5),(279,-2),(284,-11),(279,-23),(261,-33)],
                  [(266,-40),(255,-38),(253,-46),(255,-56),(260,-62),(267,-63),(274,-67)],
                  [(279,-23),(288,-29),(290,-37),(285,-47),(276,-52)]],
        'spurs': [(239,-37,11,19,12),(246,-62,12,14,16),(265,-81,17,11,17),
                  (291,-60,8,22,17),(299,-39,8,17,13),(278,-37,7,8,5.6),(263,16,12,12,4.9)],
        'hubs': [(251.6,-24.4,'Windenhaus'),(257,-44,'Geröllhütte'),(274,-8,'Schieferhof'),(269,-67,'Ausguck')],
        'reserved': [(254,-17),(249,-20),(248,-21),(247,-19),(246,-17),(245,-16),(256,-20),(259,-21),
                     (236,-14),(241,-21),(245,-14),(238,-28),(256,-38),(263,-46),(258,-17),(271,-55),(267,-34),(254,-52),(269,-64)],
        'palette': ('919873','738970','d2c6ac','9e9a91','52685f','625d62'),
        'water': [(285,-58),(279,-48),(276,-39),(274,-31),(269,-22),(271,-10),(275,1),(282,15),(285,28)],
        'crossings': [(277,-35),(274,-6)],
    },
    'reef': {
        'bounds': (-42, 178, 88, 289),
        'lobes': [(28,240,45,38),(3,234,30,24),(48,257,31,23),(50,230,27,25)],
        'approach': [(31,180),(31,185),(31,192),(31,202),(30,212),(31,222)],
        'roads': [[(31,222),(29,227),(25,230),(22,234),(20,239),(15,248),(15,258)],
                  [(22,234),(15,231),(7,232),(2,235),(-6,236),(-13,232),(-17,228)],
                  [(15,248),(24,250),(31,251),(39,254),(48,254),(58,252),(63,245)],
                  [(29,227),(36,231),(44,231),(52,228),(61,230),(66,236),(63,245)]],
        'spurs': [(-10,216,11,8,3.3),(50,216,14,8,3.0),(64,266,13,9,4.2),(14,271,12,5,2.1)],
        'hubs': [(25.8,225.4,'Muschelwerkstatt'),(46,227,'Salzschuppen'),(42,249,'Fischerhütte'),(-4,232,'Tidenwarte')],
        'reserved': [(30,234),(28,232),(25,230),(24,231),(22,234),(21,237),(28.4,229),(31.1,228),
                     (33,225),(21,227),(14,236),(15,251),(4,248),(2,231),(39,239),(40,255),(34,257),(1,253),(48,254)],
        'palette': ('d7bd8a','a9ad6d','ebd0a0','bbae83','738159','816f61'),
        'pools': [(35.4,225.5,3.6,2.15),(7,243,6.8,3.2),(22,252,4.6,2.7),(11,226,7.5,3.3),(41,236,6.2,2.1),
                  (53,245,7.4,4.1),(30,264,8.4,4.2),(60,237,3.1,3.1),(-1,245,4.1,3.6)],
    },
}


def original(x, z):
    ix, iz = round((x-OX)/STEP), round((z-OZ)/STEP)
    if 0 <= ix < COLS and 0 <= iz < ROWS:
        h, blocked = struct.unpack_from('<hB', DATA, 20 + (iz*COLS+ix)*3)
        return h*.01, bool(blocked)
    return -1.43, False


def smooth(t):
    t = min(1., max(0., t))
    return t*t*(3.-2.*t)


def flatten(paths):
    return tuple((x,z,a-x,b-z,1/((a-x)**2+(b-z)**2)) for path in paths
                 for (x,z),(a,b) in zip(path,path[1:]))


def route_dist(x, z, segments):
    best = 4e6
    for a,b,dx,dz,inv in segments:
        px,pz = x-a,z-b
        t = max(0., min(1.,(px*dx+pz*dz)*inv))
        best = min(best, (px-t*dx)**2+(pz-t*dz)**2)
    return math.sqrt(best)


def oval(x,z,blob):
    px,pz,rx,rz=blob[:4]
    return ((x-px)/rx)**2+((z-pz)/rz)**2


def phase(x,z):
    return .56*math.sin(x*.17+z*.14)+.32*math.cos(x*.063-z*.238)+.12*math.sin(x*.34-z*.47)


class Ground:
    def __init__(self, name):
        self.name = name
        self.spec = SITES[name]
        self.approach = flatten([self.spec['approach']])
        self.roads = flatten(self.spec['roads'])
        self.water = flatten([self.spec['water']]) if 'water' in self.spec else ()
        self.bounds = self.spec['bounds']
        self.blockers = [(x,z,2.35,1.75) for x,z,_ in self.spec['hubs']]
        self.cols = round((self.bounds[2]-self.bounds[0])/STEP)+1
        self.rows = round((self.bounds[3]-self.bounds[1])/STEP)+1
        self.cache = None

    def way(self, x,z):
        return min(route_dist(x,z,self.approach), route_dist(x,z,self.roads))

    def height(self,x,z):
        a = route_dist(x,z,self.approach)
        r = route_dist(x,z,self.roads)
        f = phase(x,z)
        edge = max(1-math.sqrt(oval(x,z,l)) for l in self.spec['lobes']) * 23 + f*.87
        neck = (2.75-a) * 1.27
        if self.name == 'reef':
            neck = (2.9-a) * 1.3
        edge=max(edge,neck)
        if edge < -.12:
            return max(-1.52, -.72 + .33 * edge)
        coast=smooth(edge/2.45)
        undulation = (f*.26 if self.name=='reef' else f*.42)
        if self.name=='heights':
            incline = smooth((-z+5)/68)*4.6 + smooth((x-226)/62)*1.9
        elif self.name=='amber':
            incline=.56 + smooth((-x-232)/63)*1.22
        else:
            incline=0.24
        route=(.14 + incline + undulation*smooth(r/5))
        h=-.67+coast*(.67+route)
        for p in self.spec['spurs']:
            d = oval(x,z,p)
            if d < 1:
                h += (1-d)**1.55 * p[4] * smooth((min(r,a)+.15)/5.1)
        if a < 5.35:
            lane=.08 + smooth((z-184)/21)*.21 if self.name=='reef' else .11 + smooth((-z if self.name=='heights' else -x-218)/48)*.61
            h=h*(1-smooth((5.35-a)/2.65))+lane*smooth((5.35-a)/2.65)
        if self.water:
            d=route_dist(x,z,self.water)
            x0,z0=min(self.spec['crossings'],key=lambda v:(v[0]-x)**2+(v[1]-z)**2)
            bridge=abs(z-z0) if self.name=='amber' else abs(z-z0)*.86
            bridge *= smooth((4.-abs(x-x0))/1.5)
            if d < 3.7:
                cut=(1-smooth(d/3.7))*(.70 if self.name=='amber' else 1.32)
                core=1-smooth((bridge-1.18)/1.34)
                keep=core if abs(x-x0)<4.1 and abs(z-z0)<2.4 else 0.
                h -= cut*(1-keep)
        if self.name=='reef':
            for p in self.spec['pools']:
                d=oval(x,z,p)
                if d < 1.35:
                    h=min(h, -.98 + smooth((d-.36)/1.10)*1.57)
        return h

    def dominant(self,x,z):
        old, blocked=original(x,z)
        h=self.height(x,z)
        if old > -.82 and old+.045 >= h:
            return old, False, blocked
        return h, True, False

    def solid(self,x,z,y):
        near_road = self.way(x,z) < 1.38
        if self.water and not near_road:
            if route_dist(x,z,self.water) < (1.12 if self.name=='amber' else 1.03):
                return True
        if any(abs(x-a)<rx and abs(z-b)<rz for a,b,rx,rz in self.blockers):
            return True
        return y > 1.2 and max(abs(y-self.dominant(x+dx,z+dz)[0])
            for dx,dz in ((STEP,0),(-STEP,0),(0,STEP),(0,-STEP))) > .25

    def bake(self):
        path = ROOT/'assets/world'/f'extension_{self.name}.bin'
        low_x,low_z,_,_=self.bounds
        output=bytearray(struct.pack('<4sfffHH',b'WLD5',low_x,low_z,STEP,self.cols,self.rows))
        for j in range(self.rows):
            z=low_z+j*STEP
            for i in range(self.cols):
                x=low_x+i*STEP
                y, new, old_blocked=self.dominant(x,z)
                blocked=old_blocked if not new else y > -.41 and self.solid(x,z,y)
                output+=struct.pack('<hB',round(y*100),blocked)
        path.write_bytes(output)
        self.cache=output

    def sample(self,x,z):
        if self.cache is None:
            self.cache=(ROOT/'assets/world'/f'extension_{self.name}.bin').read_bytes()
        low_x,low_z,_,_=self.bounds
        i,j=round((x-low_x)/STEP),round((z-low_z)/STEP)
        h, blocked = struct.unpack_from('<hB', self.cache,20+(j*self.cols+i)*3)
        return h*.01, bool(blocked)


def bake_roofs():
    """Precipitation occlusion from the same footprint as the visible gable roof."""
    data=bytearray(b'WRF1')
    for name,spec in SITES.items():
        ground=Ground(name)
        for i,(x,z,_) in enumerate(spec['hubs']):
            width,depth,wall=(4.05,2.86,1.90) if i==0 else (3.30,2.59,1.65)
            floor=ground.sample(x,z)[0]
            data.extend(struct.pack('<6f',x,z,floor+wall-.20,width*.5+.30,depth*.5+.25,width*.36+.20))
    (ROOT/'assets/world/extension_roofs.bin').write_bytes(data)
