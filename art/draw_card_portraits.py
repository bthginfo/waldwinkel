"""Draw original Waldwinkel card and station illustrations at 2× resolution."""

from pathlib import Path
import math
import random

from PIL import Image, ImageDraw, ImageFilter

OUT = Path(__file__).resolve().parents[1] / "assets" / "ui"
SIZE = (252, 152)
PALETTES = [
    ("#264837", "#658365", "#e9b875"),
    ("#394734", "#7e9866", "#e2d6a1"),
    ("#2f4d50", "#83a59c", "#e8d5a5"),
    ("#354a37", "#75935d", "#dce4ab"),
    ("#3b574b", "#798f74", "#e4c58f"),
    ("#3e5740", "#82945c", "#eac680"),
    ("#294a59", "#668b94", "#c1ddd2"),
    ("#42533d", "#8b9d71", "#f0d19b"),
    ("#414b4c", "#828e73", "#d5ba8c"),
    ("#424961", "#867695", "#d8e8df"),
    ("#573d34", "#a57155", "#efb977"),
    ("#686345", "#a8a67d", "#f0dc9f"),
    ("#473e35", "#957952", "#ffdc8a"),
    ("#3f4953", "#858363", "#d9daa0"),
    ("#303e5e", "#717ca1", "#e4e7c0"),
    ("#2f5260", "#739e9a", "#f3cd86"),
    ("#465453", "#88918b", "#dfc392"),
    ("#643e35", "#b17a56", "#f4ba81"),
    ("#523546", "#99816c", "#e7bd88"),
    ("#434c4d", "#868b80", "#d8c697"),
]


def rgb(hex_color):
    return tuple(int(hex_color[i:i+2], 16) for i in (1, 3, 5))


def scene(index):
    shade, sky, light = PALETTES[index]
    rng = random.Random(19040 + index)
    a, b = rgb(shade), rgb(sky)
    image = Image.new("RGB", SIZE)
    pixels = image.load()
    for y in range(SIZE[1]):
        for x in range(SIZE[0]):
            center = max(0, 1-math.hypot((x-127)/166, (y-57)/111))
            value = .21+.62*center+.09*math.sin(x*.044+y*.038)+rng.uniform(-.035,.035)
            pixels[x,y] = tuple(round(u+(v-u)*value) for u,v in zip(a,b))
    d = ImageDraw.Draw(image, "RGBA")
    for row in range(3):
        horizon = 115+row*12
        points = [(x, horizon+(9+row*3)*math.sin(.052*x+row+.25*index))
                  for x in range(0, 258, 8)]
        d.polygon(points+[(256,152),(0,152)], fill=(*a, 75+row*55))
    for side in (0, 1):
        for i in range(5):
            x = 10+side*231+(i-2)*12
            y = rng.randint(37,96)
            d.polygon([(x,y),(x-16,145),(x+16,145)], fill=(*a, 93))
    halo = Image.new("RGBA", SIZE)
    h = ImageDraw.Draw(halo)
    h.ellipse((72,10,181,130), fill=(*rgb(light),105))
    image = Image.alpha_composite(image.convert("RGBA"), halo.filter(ImageFilter.GaussianBlur(26)))
    d = ImageDraw.Draw(image, "RGBA")

    def poly(coords, color, edge=None, width=2):
        d.polygon(coords, fill=color)
        if edge:
            d.line(coords+[coords[0]], fill=edge, width=width, joint="curve")

    def line(coords, color=light, width=3):
        d.line(coords, fill=color, width=width, joint="curve")

    def leaf(x, y, width, length, angle, color):
        ca, sa = math.cos(angle), math.sin(angle)
        pts = [(x + u*ca-v*sa, y + u*sa+v*ca)
               for u, v in [(0,0),(.82*width,-.40*length),(.51*width,-.71*length),
                             (0,-length),(-.59*width,-.69*length),(-width,-.43*length)]]
        poly(pts, color, shade)
        line([pts[0],pts[3]],light,1)

    if index in (0, 2, 9, 10, 14):
        for i in range(3):
            d.arc((41+i*10,21+i*7,207-i*7,152-i*12), 193, 300, fill=(*rgb(light),160-i*31), width=3)
        if index == 14:
            poly([(184,31),(175,61),(149,80),(117,84),(91,76),(75,56),(85,89),
                  (111,107),(149,105),(179,84),(188,56)], light,shade,3)
            line([(143,98),(113,121),(98,127)],"#b2b6a8",8)
        elif index == 9:
            for x,y,h,w in [(122,122,92,25),(94,119,53,17),(153,119,61,17)]:
                poly([(x-w,y-9),(x-w+5,y-h+14),(x+5,y-h),(x+w,y-h+17),(x+w,y)],
                     light,shade,3)
                poly([(x,y-2),(x+5,y-h),(x+w,y-h+17),(x+w,y)],"#8a9da6")
        elif index == 10:
            for i in range(5):
                d.arc((58+i*11,27+i*5,185-i*6,133-i*5), 35, 329, fill=light, width=5)
            poly([(89,102),(113,74),(104,68),(161,40),(146,63),(155,70),(116,110)],"#fff0ad",shade)
        else:
            line([(79,115),(118,81),(149,49),(172,42)],shade,17)
            line([(79,113),(118,79),(149,47),(172,40)],light,9)
            line([(118,79),(117,55),(127,40)],"#bc9b70",6)
            if index == 2:
                for i in range(4):
                    line([(61,79+i*10),(92,74+i*10),(135,69+i*10)],"#e5e7c4",3)
    elif index in (1, 3, 8, 11):
        if index == 1:
            border = [(126,29),(173,52),(165,93),(144,112),(126,124),(100,109),(83,87),(79,53)]
            poly(border,light,shade,4)
            poly([(126,41),(159,58),(151,90),(126,111),(102,91),(91,58)],"#735943",shade,3)
            line([(125,46),(125,106)],"#d2a879",6)
            leaf(127,94,14,38,.58,"#89a466")
        else:
            for i in range(5):
                leaf(124,115,11+i*2,58+5*(i%3),-.85+i*.42,
                     ["#c0bd8b","#799757",light][i%3])
            for i in range(3):
                d.arc((82-i*3,60+i*8,173+i*3,122+i*5),30,178,fill=light,width=6)
            line([(126,96),(153,106),(171,90),(150,89)],"#f3e6bb",3)
    elif index in (4, 5, 6, 13):
        for i in range(5):
            x=73+i*26
            y=103-int(57*math.sin((i+.75)*.55))
            if index == 13:
                leaf(x,y+42,13,38,(i-2)*.30,"#8da864")
                d.ellipse((x-19,y-15,x+17,y+12), fill=light, outline=shade, width=2)
                d.ellipse((x-3,y-5,x+4,y+2), fill="#e29f78")
            elif index == 6:
                poly([(x,y-19),(x+9,y-3),(x+6,y+6),(x-6,y+6),(x-9,y-2)],light,shade)
                d.arc((x-19,y+14,x+19,y+30),0,350,fill=light,width=2)
            elif index == 4:
                d.ellipse((x-8,y+4,x+7,y+19), fill=light)
                for j in range(3):
                    d.ellipse((x-11+j*7,y-3-3*(j==1),x-6+j*7,y+3),fill=light)
            else:
                leaf(x,y+15,9,26,(i-2)*.31,"#deb877")
                line([(x-20,y+29),(x-11,y+17)],light,2)
    elif index in (7, 17):
        d.ellipse((71,116,178,130),fill=shade)
        poly([(75,70),(181,70),(166,111),(145,121),(107,121),(91,108)],
             "#a36347" if index==17 else "#689880",light,3)
        d.ellipse((74,62,183,86),fill=light,outline=shade,width=3)
        d.ellipse((82,67,174,80),fill="#ba7744" if index==17 else "#8a7048")
        d.arc((162,78,201,107),280,100,fill=light,width=6)
        for i in range(3):
            d.arc((102+i*16,20+i*4,128+i*16,67),115,273,fill=light,width=3)
    elif index == 12:
        line([(108,37),(108,28),(145,28),(145,37)],light,4)
        poly([(110,35),(141,35),(164,57),(153,109),(99,109),(88,57)],shade,light,4)
        poly([(103,58),(149,58),(145,98),(107,98)],"#e5a96a",light,2)
        for x in (111,140):
            line([(x,58),(x,102)],shade,3)
        poly([(124,94),(115,84),(124,71),(131,66),(136,85),(130,94)],"#fff0ad")
    elif index == 15:
        poly([(94,109),(96,45),(104,32),(134,53),(164,37),(177,50),
              (167,113),(149,124),(109,122)],shade,light,5)
        for i in range(6):
            line([(110+i*9,48+i%3*3),(111+i*9,110)],light,2)
        line([(103,118),(153,118),(164,108)],"#b99366",5)
    elif index == 18:
        poly([(92,40),(156,40),(170,59),(169,117),(151,126),(96,125),(82,113),(82,59)],
             "#955360",light,4)
        d.ellipse((88,47,167,68),fill="#c6817a",outline=shade,width=3)
        d.rounded_rectangle((89,32,163,51),radius=6,fill="#b98d65",outline=light,width=3)
        line([(94,63),(94,111)],"#e7bdad",4)
        d.rounded_rectangle((107,76,155,103),radius=7,fill="#e7d6ab",outline=shade,width=2)
        d.ellipse((119,84,132,95),fill="#9f535e")
        leaf(129,94,5,14,.53,"#83935d")
    elif index == 19:
        d.ellipse((58,92,190,137),fill=shade)
        poly([(72,73),(181,73),(177,111),(159,123),(97,123),(77,110)],
             "#847b68",light,3)
        d.ellipse((72,54,182,95),fill="#afaa92",outline=shade,width=5)
        d.ellipse((105,63,147,83),fill="#645e52",outline=light,width=2)
        line([(126,72),(151,52),(174,51),(174,29)],"#70533c",9)
        d.ellipse((167,24,181,34),fill=light,outline=shade,width=2)
        for i in range(12):
            x, y = rng.randrange(90,165), rng.randrange(108,124)
            d.ellipse((x-2,y-1,x+2,y+1),fill=light)
    else:
        poly([(96,119),(85,89),(100,45),(132,31),(155,44),(171,106),(155,122)],
             "#8e9180",shade,4)
        line([(108,80),(124,64),(138,74),(119,88),(137,101),(150,87)],light,5)
        d.arc((53,65,194,134),3,178,fill=light,width=3)

    for i in range(38):
        x,y=rng.randrange(18,234),rng.randrange(9,139)
        if (x-126)**2+(y-75)**2 < 30**2:
            continue
        r=rng.choice((.5,.8,1.5,2.))
        d.ellipse((x-r,y-r,x+r,y+r),fill=(*rgb(light),rng.randrange(70,173)))
    d.rounded_rectangle((1,1,250,150),radius=12,outline=(120,94,61,255),width=3)
    d.rounded_rectangle((5,5,246,146),radius=9,outline=(*rgb(light),185),width=2)
    for x in (11,241):
        for y in (11,141):
            poly([(x-5,y),(x,y-4),(x+5,y),(x,y+4)],light,shade,1)
    return image.convert("RGB")


if __name__ == "__main__":
    for i in range(18):
        scene(i).save(OUT / f"portrait_card_{i:02}.png", optimize=True)
    for index, station in enumerate(("cook","preserve","mill"),start=17):
        scene(index).save(OUT / f"station_{station}.png", optimize=True)
