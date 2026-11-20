"""Seasonal foliage, snow cover, finds and the Eichenhafen gathering table."""
import math

import landscape


SEASONS = ("spring", "summer", "autumn", "winter")


class SeasonalArt:
    def __init__(self, model, blend):
        self.blend = blend
        self.layers = {
            season: model(f"Jahreszeit · {season}", f"season_{season}")
            for season in SEASONS
        }
        self.wood_snow = model("Fällbare Tanne · Schnee", "wood_snow", casts=False)
        self.night_windows = model("Warmes Fensterlicht", "night_windows", casts=False)

    def origin(self, source, x, z):
        ox, oz = landscape.ORIGINS.get(source.asset, (0, 0))
        return x + ox, z + oz

    def crown(self, source, x, z, h, sx, sz, sy, color, sides=9, levels=4,
              *, rough=0, shrub=False):
        wx, wz = self.origin(source, x, z)
        summer = self.layers["summer"]
        first_face, first_vertex = len(summer.faces), len(summer.vertices)
        summer.ellipsoid(wx, wz, h, sx, sz, sy, self.blend(color, "92ae58", .55),
                         sides, levels, rough=rough)
        verts = summer.vertices[first_vertex:]
        faces = summer.faces[first_face:]
        floral = (int(wx * 23 + wz * 38) % 8) < 3
        spring = "dfa6aa" if floral else "abd277"
        autumn = ("d49a4f", "dc8a51", "c66d48", "d2ae61")[int(wx * 19) % 4]
        for season, tint in [("spring", spring), ("autumn", autumn)]:
            model = self.layers[season]
            offset = len(model.vertices) - first_vertex
            model.vertices.extend(verts)
            model.faces.extend(tuple(v + offset for v in f) for f in faces)
            model.colors.extend([(self.blend(color, tint, .81), "lit")] * len(faces))
        if floral and sx >= .41:
            petals = self.layers["spring"]
            for i in range(3):
                a = .7 + i * 2.1 + wx
                petals.ellipsoid(wx + math.cos(a) * sx * .68, wz + math.sin(a) * sz * .69,
                                  h + sy * (.65 + .10 * (i % 2)), sx*.17, sz*.17, sy*.15,
                                  ("f8dbce", "f3bfcb", "ffe5d7")[i], 6, 3)
        if shrub:
            self.layers["winter"].ellipsoid(wx, wz, h*.64, sx*.79, sz*.79, sy*.62,
                                             "6c8c7d", 7, 3)
            self.layers["winter"].ellipsoid(wx-.04, wz+.02, h*.64+sy*.39,
                                             sx*.62, sz*.61, sy*.23, "dee9e4", 7, 3)

    def bare_tree(self, source, x, z, y, scale, height):
        """Winter branching attaches below the top of the year-round trunk."""
        wx, wz = self.origin(source, x, z)
        model = self.layers["winter"]
        for i, reach in enumerate((.71, .92, .68, .50)):
            a = i * math.tau / 4 + wx*.37 + wz*.12
            dx, dz = math.cos(a)*reach*scale, math.sin(a)*reach*scale
            base = y + height*(.64 + .07*i)
            fork = (wx + dx*.61, wz + dz*.61, base + .50*scale)
            tip = (wx + dx, wz + dz, base + 1.01*scale)
            model.pole((wx, wz, base), fork, .080*scale, "8a6c58", 6, tip=.050*scale)
            model.pole(fork, tip, .051*scale, "957965", 5, tip=.018*scale)
            model.pole(fork, (fork[0]-dz*.42, fork[1]+dx*.43, fork[2]+.62*scale),
                       .037*scale, "9e806b", 5, tip=.012*scale)
            if i % 2 == 0:
                model.pole((fork[0], fork[1], fork[2]+.051*scale),
                           (tip[0], tip[1], tip[2]+.02*scale),
                           .030*scale, "e4ebe6", 5, tip=.012*scale)

    def conifer(self, source, rings, scale, tier):
        if tier == 0:
            return
        model = self.wood_snow if source.asset == "wood_tree" else self.layers["winter"]
        ox, oz = self.origin(source, 0, 0)
        shoulder, tip = rings[-2:]
        # A scalloped edge leaves visible needles below the cap.
        edge, shoulder_snow, tip_snow = [], [], []
        for i, (lo, hi) in enumerate(zip(shoulder, tip)):
            t = (.18, .32, .25, .43, .19, .29, .41, .23, .29)[i]
            base = rings[-3][i]
            edge.append((lo[0]+(base[0]-lo[0])*t+ox, lo[1]+(base[1]-lo[1])*t+oz,
                         lo[2]+(base[2]-lo[2])*t+.075*scale))
            shoulder_snow.append((lo[0]+ox, lo[1]+oz, lo[2]+.105*scale))
            tip_snow.append((hi[0]+ox, hi[1]+oz, hi[2]+.065*scale))
        model.rings([edge, shoulder_snow, tip_snow], "e2eae2", cap=True)

    def roof(self, source, ridge, eave, back, front, clearance):
        """Break the snow edge into shallow drifts above the existing roof tiles."""
        model = self.layers["winter"]
        ox, oz = self.origin(source, 0, 0)
        last = None
        for i in range(9):
            z = back + (front-back)*i/8 + oz
            edge = (.81, .91, .86, .96, .88, .93, .85, .91, .83)[i]
            xs = (ridge[0], ridge[0] + (eave[0]-ridge[0])*.42,
                  ridge[0] + (eave[0]-ridge[0])*edge)
            heights = (ridge[1]+clearance+.025,
                       ridge[1]+(eave[1]-ridge[1])*.42+clearance+.065,
                       ridge[1]+(eave[1]-ridge[1])*edge+clearance)
            strip = [(x+ox, z, h) for x, h in zip(xs, heights)]
            if last is not None:
                for j in range(2):
                    quad = (last[j], last[j+1], strip[j+1], strip[j])
                    model.face(quad[::1 if eave[0]>ridge[0] else -1],
                               ("ebefe6", "e2ebe5", "f4f2e8")[i%3])
                a, b = last[-1], strip[-1]
                model.face([a, b, (b[0], b[1], b[2]-.063), (a[0], a[1], a[2]-.063)],
                           "bbcfd3")
            last = strip

    def window(self, source, x, z, h, w, v, side):
        if source.asset not in landscape.ORIGINS and source.asset not in ("island", "mainland"):
            return
        wx, wz = self.origin(source, x, z)
        for u in [-1,1]:
            for t in [-1,1]:
                offset = u*w*.243
                height = h+t*v*.249
                color = "ffd599" if u+t!=2 else "f5b773"
                if side:
                    self.night_windows.box(wx-.173,wz+offset,height,.008,w*.415,v*.40,color,.002,
                                           mode="raw")
                else:
                    self.night_windows.box(wx+offset,wz+.173,height,w*.415,.008,v*.40,color,.002,
                                           mode="raw")


def find_model(model, season, x=0, z=0, y=0):
    """The same handcrafted objects sit on the ground and on the festival table."""
    if season == "spring":
        model.pole((x-.25,z+.15,y+.04),(x+.24,z-.13,y+.25),.029,"80634c",5,tip=.012)
        for i in range(3):
            a,b,h=x-.13+i*.16,z+.09-i*.10,y+.105+i*.057
            model.ellipsoid(a,b,h,.13,.15,.062,"d9869e",7,3)
            for k in range(5):
                p=k*math.tau/5+.1*i
                model.ellipsoid(a+.085*math.cos(p),b+.085*math.sin(p),h+.035,
                                .065,.062,.038,"ffd7dd",6,3)
            model.ellipsoid(a,b,h+.074,.039,.035,.029,"ecc05f",6,3)
    elif season == "summer":
        model.ellipsoid(x,z,y+.12,.24,.21,.137,"edb580",9,4)
        for k in range(7):
            a=(k-3)*.33
            model.pole((x,z+.14,y+.088),(x+.231*math.sin(a),z-.18*math.cos(a),y+.151),
                       .026,"ffe1ad",5,tip=.020)
        model.ellipsoid(x,z+.139,y+.104,.104,.088,.079,"c67a60",7,3)
    elif season == "autumn":
        for dx,dz,d in [(-.115,.061,.13),(.10,-.05,.155)]:
            model.ellipsoid(x+dx,z+dz,y+d*.85,d,d*.83,d,"c38a43",8,4)
            model.ellipsoid(x+dx,z+dz,y+d*1.36,d*1.02,d*.89,d*.58,"876249",8,3)
            model.pole((x+dx,z+dz,y+d*1.64),(x+dx+.047,z+dz,y+d*2.12),.025,"6f6e46",5,tip=.012)
        model.face([(x-.04,z+.15,y+.04),(x+.12,z+.27,y+.06),(x+.27,z+.18,y+.08),
                    (x+.06,z+.083,y+.07)],"d69751","leaf")
    else:
        for dx,dz,h in [(-.14,.03,.11),(.11,-.03,.14),(.02,.115,.078)]:
            model.ellipsoid(x+dx,z+dz,y+h,.107,.102,.11,"647aac",8,4)
            model.ellipsoid(x+dx,z+dz,y+h+.096,.078,.076,.036,"eff0ea",7,3)
        for side in [-1, 1]:
            model.face([(x,z,y+.09),(x+side*.17,z-.14,y+.092),(x+side*.25,z-.15,y+.17),
                        (x+side*.16,z-.04,y+.18)],"769995","leaf")


def gathering_assets(Model, lantern):
    assets = []
    palettes = [
        ("b97281", "f0d1ca", "afcb83"),
        ("c48b46", "f7dfa3", "80a9a2"),
        ("a85c43", "eab17f", "e3c17d"),
        ("628987", "d9ebe0", "a992ad"),
    ]
    for season, (cloth, trim, ribbon) in zip(SEASONS, palettes):
        piece = Model(f"Saisonfund · {season}", f"season_find_{season}")
        find_model(piece, season)
        assets.append(piece)
        table = Model(f"Dorftisch · {season}", f"season_table_{season}")
        for x in [-.67,.67]:
            for z in [-.31,.31]:
                table.box(x,z,.391,.139,.136,.76,"947155",.028)
            table.box(x,0,.26,.09,.69,.113,"a77b57",.02)
        for k in range(7):
            table.box((k-3)*.239,0,.775,.235,.82,.14,
                      ("ba8f66","c49c72","ae835b")[k%3],.027)
        table.box(0,0,.854,1.23,.84,.025,cloth,.006)
        table.box(0,.416,.682,1.23,.029,.324,cloth,.008)
        table.box(0,.438,.555,1.23,.015,.054,trim,.004)
        for x in [-.47,0,.47]:
            table.ellipsoid(x,.439,.67,.033,.013,.036,trim,6,3)
        for dx,z in [(-.43,.02),(.11,-.12)]:
            table.pole((dx,z,.848),(dx,z,.91),.215,"a18768",10,tip=.27)
            table.pole((dx,z,.906),(dx,z,.932),.271,"bda17a",10)
            find_model(table, season, dx, z, .917)
        for x in [-.78,.78]:
            table.pole((x,-.31,.79),(x,-.31,1.82),.046,"8e7256",6,tip=.035)
            table.ellipsoid(x,-.31,1.82,.061,.058,.067,trim,7,3)
        for i in range(6):
            a=-.78+i*.26
            b=a+.26
            def string(x):
                return 1.61+.21*(x/.78)**2
            table.pole((a,-.31,string(a)),(b,-.31,string(b)),.012,"927e6a",4)
            table.face([(a+.029,-.302,string(a)-.018),(b-.028,-.302,string(b)-.019),
                        (a+.131,-.287,(string(a)+string(b))*.5-.228)],
                       (cloth,trim,ribbon)[i%3],"raw")
        lantern(table,.575,.205,1.08)
        assets.append(table)
    shelf = Model("Jahreszeitenregal · Vier Erinnerungsfächer", "furniture_seasons")
    for x in [-.48,.48]:
        shelf.box(x,-.11,.73,.128,.52,1.42,"916748",.036)
        shelf.box(x,.11,.085,.12,.14,.17,"9b7554",.024)
    shelf.box(0,-.334,.735,.91,.061,1.36,"89694f",.019)
    shelf.box(0,-.063,.72,.096,.46,1.22,"a77e56",.021)
    for y in [.156,.755,1.44]:
        shelf.box(0,-.075,y,1.11,.54,.092,"bc9467",.029)
    for i, season in enumerate(SEASONS):
        x,y=(-.246 if i%2==0 else .246),(.207 if i<2 else .804)
        shelf.box(x,-.035,y,.41,.31,.025,palettes[i][0],.008)
        # Miniature specimens fitted to the cubbies from the same seasonal mesh.
        sample = Model("Kleines Fundstück")
        find_model(sample,season)
        base = len(shelf.vertices)
        shelf.vertices.extend((x+xx*.69, zz*.69+.008, y+hh*.73)
                              for xx,zz,hh in sample.vertices)
        shelf.faces.extend(tuple(base+v for v in face) for face in sample.faces)
        shelf.colors.extend(sample.colors)
    assets.append(shelf)
    lawn = Model("Beet · Schnee","lawn_winter",casts=False)
    lawn.box(0,0,.035,1.15,1.14,.067,"cddfdb",.031)
    for x,z in [(-.37,-.29),(.24,-.31),(-.06,.32),(.40,.26)]:
        lawn.ellipsoid(x,z,.076,.158,.127,.037,"e1e9e1",7,3)
        lawn.pole((x,z,.077),(x+.036,z,.142),.019,"a7a381",5,tip=.009)
    assets.append(lawn)
    return assets
