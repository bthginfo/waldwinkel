"""Draw the original HUD and launcher artwork. The 3D models are built in Blender."""
from pathlib import Path
from math import cos, sin, tau
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
ICON = ROOT / "assets" / "ui"
ICON.mkdir(parents=True, exist_ok=True)
INK = "#3d5946"
LEAF = "#678056"
LIGHT = "#91a670"
GOLD = "#bd8951"


def canvas():
    image = Image.new("RGBA", (288, 288))
    return image, ImageDraw.Draw(image)


def polygon(draw, points, color):
    draw.polygon([(round(x*3), round(y*3)) for x, y in points], fill=color)


def ellipse(draw, rect, color):
    draw.ellipse(tuple(round(x*3) for x in rect), fill=color)


def box(draw, rect, color, radius=0):
    draw.rounded_rectangle(tuple(round(x*3) for x in rect), round(radius*3), fill=color)


def line(draw, points, color, width):
    draw.line([(round(x*3), round(y*3)) for x, y in points],
              fill=color, width=round(width*3), joint="curve")


def save(image, name):
    image.resize((96,96), Image.Resampling.LANCZOS).save(ICON / f"{name}.png")


def sprout(draw):
    polygon(draw, [(46,71),(41,51),(29,48),(23,33),(17,24),(37,25),(48,34),(52,45),
                   (58,28),(74,21),(83,21),(77,38),(64,51),(54,54),(52,72)], LEAF)
    line(draw, [(48,75),(49,51),(61,38),(71,30)], INK, 4)
    line(draw, [(49,54),(38,42),(28,35)], LIGHT, 3)
    box(draw,(29,72,67,80),GOLD,4)


image, draw = canvas()
sprout(draw)
save(image,"sprout")

image, draw = canvas()
box(draw,(27,62,75,76),"#b58353",4)
line(draw,[(35,70),(64,28)],GOLD,8)
line(draw,[(49,28),(66,21),(73,41)],INK,8)
save(image,"hoe")

image, draw = canvas()
line(draw,[(59,41),(71,21)],LEAF,6)
polygon(draw,[(60,40),(58,22),(64,16),(68,30),(77,20),(83,23),(77,38)],LEAF)
polygon(draw,[(27,76),(39,52),(50,40),(63,40),(70,48),(63,61),(49,69),(29,80)],"#cf7a44")
polygon(draw,[(30,73),(43,49),(53,43),(64,43),(56,58),(40,69)],"#e59a56")
line(draw,[(46,57),(51,60),(55,60)],"#b76c3d",3)
line(draw,[(53,48),(58,52)],"#b76c3d",3)
save(image,"carrot")

image, draw = canvas()
for cx, cy, a in [(34,53,-.4),(56,31,.56),(63,62,.62)]:
    (ux,uy),(vx,vy)=(cos(a),sin(a)),(-sin(a),cos(a))
    shape=[(cx+ux*x+vx*y,cy+uy*x+vy*y) for x,y in [(-11,0),(-7,-9),(2,-12),(9,-7),(10,3),(5,11),(-4,10)]]
    polygon(draw,shape,"#ae8756")
    line(draw,[(cx-ux*6,cy-uy*6),(cx+vx*3,cy+vy*3),(cx+ux*4,cy+uy*4)],"#d4b67c",3)
save(image,"seeds")

image, draw = canvas()
box(draw,(27,37,63,69),"#639383",9)
line(draw,[(34,41),(27,27),(32,21),(52,20),(57,35)],INK,5)
line(draw,[(53,53),(69,49),(79,35)],"#639383",9)
line(draw,[(74,33),(84,40)],GOLD,6)
for x,y in [(73,50),(82,53),(76,61)]:
    ellipse(draw,(x-2,y-2,x+3,y+4),"#5b9799")
save(image,"watering")

image, draw = canvas()
polygon(draw,[(30,21),(66,21),(63,32),(73,46),(72,72),(62,78),(31,78),(24,70),(25,44),(34,33)],GOLD)
box(draw,(32,44,65,70),"#f0d9a3",5)
line(draw,[(40,63),(55,50)],INK,4)
ellipse(draw,(47,47,61,57),LEAF)
save(image,"fertilizer")

image, draw = canvas()
for x,z in [(-1,-1),(-1,1),(1,-1),(1,1)]:
    line(draw,[(48+x*12,48+z*21),(48+x*22,48+z*21),(48+x*22,48+z*11)],INK,5)
ellipse(draw,(43,43,53,53),LEAF)
save(image,"choose")

image, draw = canvas()
ellipse(draw,(21,42,27,48),"#59755d")
ellipse(draw,(42,21,48,27),"#59755d")
ellipse(draw,(68,42,74,48),"#59755d")
ellipse(draw,(42,68,48,74),"#59755d")
save(image,"stick")

image, draw = canvas()
box(draw,(4,4,92,92),"#67b8b6",20)
ellipse(draw,(11,56,85,82),"#4e9791")
ellipse(draw,(12,52,85,77),"#aaab60")
polygon(draw,[(18,60),(27,39),(34,60)],"#6e8552")
polygon(draw,[(69,57),(75,34),(82,57)],"#6c8255")
polygon(draw,[(32,64),(32,39),(52,25),(71,42),(71,64)],"#b88958")
polygon(draw,[(25,43),(50,22),(75,42),(69,47),(50,31),(31,47)],"#e1a575")
polygon(draw,[(31,44),(50,29),(55,31),(32,48)],"#efbc8c")
box(draw,(46,47,57,65),"#496a62",1)
box(draw,(61,47,67,54),"#fbe1a0",1)
box(draw,(36,48,42,55),"#fbe1a0",1)
polygon(draw,[(36,65),(67,65),(63,70),(39,70)],"#d3ac73")
box(draw,(22,61,25,64),"#eed7a0",1)
box(draw,(75,65,78,68),"#eed7a0",1)
image.resize((192,192),Image.Resampling.LANCZOS).save(ROOT/"packaging/res/drawable-nodpi/icon.png")

image, draw = canvas()
ellipse(draw,(28,28,68,68),"#e1b571")
ellipse(draw,(34,34,62,62),"#edca84")
for i in range(8):
    a=tau*i/8
    line(draw,[(48+cos(a)*25,48+sin(a)*25),(48+cos(a)*34,48+sin(a)*34)],GOLD,4)
save(image,"day")

image, draw = canvas()
ellipse(draw,(16,16,80,80),"#b8834d")
ellipse(draw,(20,18,76,74),"#e5bd72")
ellipse(draw,(28,26,68,66),"#c89054")
ellipse(draw,(31,29,65,63),"#edcd83")
line(draw,[(48,38),(48,54)],"#a87848",4)
line(draw,[(41,46),(55,46)],"#a87848",4)
save(image,"coin")

# Painted emblems for the five kinds of combat cards.
for kind, halo in [
    ("strike", "#e7b692"), ("guard", "#bdc99a"), ("heal", "#dbb1b1"),
    ("focus", "#b6c3cb"), ("spore", "#c5b8ce"),
]:
    image, draw = canvas()
    ellipse(draw, (14, 11, 82, 79), halo)
    if kind == "strike":
        line(draw, [(28, 67), (62, 29)], "#8b6349", 10)
        line(draw, [(43, 49), (39, 29)], "#8b6349", 7)
        polygon(draw, [(51, 27), (66, 19), (75, 25), (65, 37), (62, 30)], "#f3dcb0")
        line(draw, [(25, 65), (33, 72)], "#f3dcb0", 4)
    elif kind == "guard":
        polygon(draw, [(48, 20), (68, 28), (66, 51), (57, 65), (48, 71),
                       (37, 64), (28, 52), (27, 28)], "#536f55")
        polygon(draw, [(48, 28), (61, 34), (60, 48), (54, 60), (48, 65),
                       (40, 57), (35, 48), (34, 34)], "#98a975")
        line(draw, [(48, 32), (48, 61)], "#d5d5a1", 3)
    elif kind == "heal":
        polygon(draw, [(48, 68), (25, 46), (26, 30), (39, 25), (48, 32),
                       (57, 25), (70, 30), (72, 45)], "#ad6f76")
        line(draw, [(48, 39), (48, 54)], "#f5d8b1", 4)
        line(draw, [(40, 46), (56, 46)], "#f5d8b1", 4)
    elif kind == "focus":
        line(draw, [(24, 45), (46, 26), (71, 45), (46, 63), (24, 45)], "#526b75", 5)
        ellipse(draw, (37, 34, 56, 55), "#526b75")
        ellipse(draw, (44, 38, 51, 44), "#eef2ca")
    else:
        box(draw, (42, 43, 56, 65), "#e9d3b7", 4)
        polygon(draw, [(23, 47), (32, 33), (46, 28), (61, 32), (72, 48),
                       (48, 52)], "#8a729c")
        for x, y in [(33, 41), (49, 35), (63, 43)]:
            ellipse(draw, (x - 2, y - 2, x + 3, y + 3), "#e6d0a8")
    save(image, f"card_{kind}")
