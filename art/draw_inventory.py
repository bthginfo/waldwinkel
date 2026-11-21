"""Draw legible inventory thumbnails with the colors of the Blender world."""
import math
from PIL import Image, ImageDraw
from draw_icons import canvas, polygon as poly, ellipse as oval, box, line, ROOT

OUT=ROOT/"assets"/"ui"/"items"
OUT.mkdir(parents=True,exist_ok=True)


def draw(name, item):
    image,d=canvas()
    oval(d,(11,73,85,88),"#ad96753d")
    item(d)
    image.resize((88,88),Image.Resampling.LANCZOS).save(OUT/(name+".png"),optimize=True)


def leaf(d,x,y,dx,dy,color):
    poly(d,[(x,y),(x+dx*.38-dy*.14,y+dy*.38+dx*.14),(x+dx,y+dy),
            (x+dx*.64+dy*.16,y+dy*.64-dx*.16)],color)


def root(d,kind):
    for i in range(3):
        leaf(d,57,30, 11+i*6, -21+i*8, ["#73955a","#90a862","#60835a"][i])
    if kind=="carrot":
        poly(d,[(52,23),(67,38),(59,54),(25,81),(27,64),(38,36)],"#cf7037")
        poly(d,[(52,23),(60,32),(41,60),(25,81),(28,59),(39,34)],"#ed9a55")
        for x,y in [(43,40),(36,54),(48,52)]:
            line(d,[(x,y),(x+8,y+4)],"#ba6439",2)
    else:
        oval(d,(23,28,68,72),"#eee2bc")
        oval(d,(32,26,67,48),"#b37d96")
        poly(d,[(45,68),(50,64),(36,81)],"#ddc59d")
        oval(d,(29,42,43,53),"#fff2cf")


def fruit(d,name):
    body, light = {"apple":("#ba5e53","#dd8771"), "tomato":("#c25540","#e7835f"),
                   "pumpkin":("#cb7442","#eeb060"),"strawberry":("#c9535c","#ea8784")}[name]
    line(d,[(48,34),(51,18),(63,16)],"#87664e",5)
    leaf(d,51,26,23,-12,"#799760")
    if name=="pumpkin":
        for x,c in [(14,body),(48,body),(26,light),(44,"#da9750")]:
            oval(d,(x,28,x+33,78),c)
    elif name=="strawberry":
        poly(d,[(19,38),(34,25),(59,27),(76,41),(60,66),(45,79),(30,62)],body)
        poly(d,[(19,38),(34,25),(48,33),(44,70),(30,62)],light)
        for x,y in [(34,44),(55,39),(44,54),(58,55),(38,64)]:
            oval(d,(x,y,x+3,y+5),"#f8d991")
    else:
        oval(d,(17,27,80,78),body)
        oval(d,(19,26,67,70),light)
        oval(d,(28,35,42,43),"#efab8c")


def mineral(d,base,light,tip):
    for x,y,s in [(14,47,.86),(45,43,1.),(35,24,1.04)]:
        shape=lambda points:[(x+a*s,y+b*s) for a,b in points]
        poly(d,shape([(0,23),(8,4),(22,0),(39,12),(37,34),(13,40)]),base)
        poly(d,shape([(0,23),(8,4),(22,0),(28,14),(17,25)]),light)
        poly(d,shape([(28,14),(39,12),(37,34),(17,25)]),tip)


def sack(d,cloth,contents):
    poly(d,[(24,75),(21,55),(30,34),(31,24),(66,23),(68,36),(77,58),(72,77)],cloth)
    oval(d,(29,18,68,34),"#9f7859")
    oval(d,(33,21,64,30),contents)
    box(d,(32,45,63,65),"#f2e0b4",5)
    for x,y in [(40,51),(48,56),(54,48)]:
        oval(d,(x,y,x+6,y+4),contents)


def shells(d,color,edge):
    poly(d,[(19,42),(33,28),(48,24),(65,29),(79,47),(75,57),(53,76),(38,71),(22,56)],color)
    for x,y in [(22,45),(35,32),(49,28),(65,34),(77,47)]:
        line(d,[(49,67),(x,y)],edge,3)
    oval(d,(37,62,61,76),color)


def fish(d,body,light):
    poly(d,[(70,48),(87,30),(83,57),(85,67),(68,58)],body)
    oval(d,(13,31,74,66),body)
    poly(d,[(37,33),(52,21),(62,39)],body)
    line(d,[(26,53),(47,54),(70,47)],light,4)
    oval(d,(24,37,32,44),"#f1e9cd")
    oval(d,(25,38,30,43),"#52675f")


def berries(d,color,light):
    leaf(d,46,38,17,-21,"#7c9464")
    leaf(d,46,38,-20,-14,"#8ea76f")
    for x,y in [(34,29),(50,35),(23,46),(57,50),(40,58)]:
        oval(d,(x,y,x+23,y+22),color)
        oval(d,(x+4,y+4,x+11,y+9),light)


def blossom(d,petal,center):
    line(d,[(45,76),(47,37)],"#688556",5)
    leaf(d,47,66,-25,-9,"#92a45d")
    for i in range(8):
        x=48+math.cos(i*math.tau/8)*16
        y=39+math.sin(i*math.tau/8)*15
        oval(d,(x-9,y-9,x+9,y+9),petal)
    oval(d,(38,30,58,50),center)


def jar(d,contents,label):
    box(d,(28,29,68,79),contents,10)
    box(d,(24,21,72,34),"#c6a274",5)
    box(d,(36,47,61,64),label,4)
    line(d,[(34,36),(34,68)],"#ffffff48",3)


def soup(d,contents,herb):
    poly(d,[(12,48),(85,48),(74,67),(61,76),(35,76),(21,68)],"#799b94")
    oval(d,(11,39,86,61),"#bfd2bc")
    oval(d,(17,42,80,57),contents)
    for x,y in [(29,46),(47,50),(65,44)]:
        oval(d,(x,y,x+8,y+5),herb)


def logs(d):
    for x,y in [(13,56),(35,62),(26,36)]:
        poly(d,[(x,y),(x+35,y-23),(x+57,y-11),(x+21,y+14)],"#a67950")
        oval(d,(x,y,x+24,y+23),"#d4ac78")
        oval(d,(x+4,y+4,x+20,y+19),"#bf945f")
        oval(d,(x+8,y+8,x+16,y+16),"#e1bc8d")


draw("wood",logs)
for name,colors in {
    "stone":("#918d80","#bab5a6","#807f75"),"copper":("#b57458","#d19a70","#936951"),
    "silver":("#aebaad","#d7e3d5","#889b9f"),"crystal":("#6b8fa3","#b3d9db","#88aabc"),
    "quartz":("#b5a6b9","#eadbe0","#a98b9d"),"seal":("#b27c63","#dba887","#916d58"),
    "fittings":("#b77d59","#e5ae79","#996c52"),
}.items():
    draw(name,lambda d: mineral(d,*colors))
for name,colors in {
    "fertilizer":("#a49773","#6e8964"),"feed":("#c39a67","#a27646"),"seed":("#bba17c","#889658"),
    "flour":("#cebb92","#d6c198"),"meal":("#cdad76","#a68156"),"dust":("#ac99bb","#d2c7d9"),
}.items():
    draw(name,lambda d: sack(d,*colors))
for name in ["carrot","turnip"]:
    draw(name,lambda d:root(d,name))
for name in ["apple","strawberry","tomato","pumpkin"]:
    draw(name,lambda d:fruit(d,name))
draw("sunflower",lambda d:blossom(d,"#e8ba52","#997651"))
draw("blossom",lambda d:blossom(d,"#eab7c3","#eba96b"))
draw("flax",lambda d:blossom(d,"#9c9bc6","#ded4b4"))
draw("mint",lambda d:berries(d,"#779866","#a2bb85"))
draw("berry",lambda d:berries(d,"#a46e95","#cc9cac"))
draw("frost",lambda d:berries(d,"#7e91bc","#e3ece4"))
draw("moss",lambda d:berries(d,"#889764","#afba7b"))
draw("wool",lambda d:berries(d,"#e3d6c5","#f3e9d9"))
draw("hazel",lambda d:berries(d,"#9f7756","#cb9c68"))
draw("amber",lambda d:berries(d,"#b58445","#e5b563"))
draw("shell",lambda d:shells(d,"#dec0aa","#b99482"))
draw("sun_shell",lambda d:shells(d,"#ecc58e","#c38d60"))
draw("fish",lambda d:fish(d,"#729d93","#bfcdac"))
draw("herring",lambda d:fish(d,"#7996b2","#cad6c8"))
draw("egg",lambda d:(oval(d,(28,20,67,79),"#eedbb3"),oval(d,(33,27,59,57),"#f7e9cc")))
draw("mushroom",lambda d:(box(d,(40,41,56,76),"#e7d8b2",6),oval(d,(16,24,79,57),"#b87d5a"),oval(d,(27,29,58,42),"#dba175")))
draw("soup",lambda d:soup(d,"#dfa46b","#8ca266"))
draw("stew",lambda d:soup(d,"#c4694e","#e6b067"))
draw("jam",lambda d:jar(d,"#9b6e8f","#eac5c7"))
draw("compote",lambda d:jar(d,"#b66863","#ebcfb0"))
for seed,color in {
    "carrot":"#d48f53", "turnip":"#b68b9d", "pumpkin":"#cfa156",
    "strawberry":"#c27782", "tomato":"#c36955", "sunflower":"#d0af4e",
}.items():
    draw("seed_"+seed,lambda d: sack(d,"#bba17c",color))
(OUT/"seed.png").unlink(missing_ok=True)


def construction_plan(d, outline):
    poly(d,[(17,21),(71,16),(80,71),(27,80)],"#c9b68b")
    poly(d,[(15,15),(69,10),(78,65),(25,74)],"#f2dfa9")
    poly(d,[(69,10),(77,21),(70,23)],"#ddc28d")
    for x in range(28,69,9):
        line(d,[(x,26),(x+6,64)],"#e6cd98",1)
    for y in range(32,64,8):
        line(d,[(25,y),(68,y-4)],"#e6cd98",1)
    if outline == "barn":
        line(d,[(30,43),(44,29),(63,39),(66,57),(33,61),(30,43),(63,39)],"#66857b",3)
        line(d,[(43,59),(42,46),(54,45),(56,58)],"#66857b",3)
    elif outline == "room":
        line(d,[(28,33),(57,29),(62,39),(68,38),(71,56),(50,59),(48,53),(33,55),(28,33)],"#66857b",3)
        line(d,[(60,39),(63,52)],"#66857b",2)
    else:
        line(d,[(32,51),(32,44),(60,41),(62,49),(32,51),(35,62)],"#66857b",3)
        line(d,[(58,50),(60,58)],"#66857b",3)
        line(d,[(36,37),(54,35)],"#66857b",2)


def tool(d, kind, metal, shine):
    if kind == "can":
        oval(d,(17,31,44,61),metal)
        oval(d,(23,36,43,54),"#f0d7a8")
        poly(d,[(61,49),(79,34),(86,34),(85,39),(69,59)],metal)
        box(d,(31,37,68,73),metal,10)
        oval(d,(32,33,67,47),shine)
        oval(d,(40,36,63,42),metal)
        return
    line(d,[(30,77),(58,29)],"#a57450",9)
    line(d,[(29,75),(55,34)],"#c89865",3)
    if kind == "axe":
        poly(d,[(49,33),(47,25),(69,16),(76,24),(80,38),(73,48),(64,41),(54,41)],metal)
        line(d,[(69,17),(76,25),(79,37),(73,46)],shine,3)
    elif kind == "pickaxe":
        poly(d,[(26,34),(35,24),(52,20),(65,22),(75,31),(81,44),(65,35),(53,32),(38,33)],metal)
        line(d,[(34,26),(51,22),(64,24),(75,32)],shine,3)
    else:
        poly(d,[(53,33),(55,23),(81,30),(79,43),(70,49),(68,36)],metal)
        line(d,[(57,25),(79,31),(77,41)],shine,3)


for plan in ("furniture", "barn", "room"):
    draw(f"plan_{plan}", lambda d: construction_plan(d, plan))
for kind in ("axe", "pickaxe", "can", "hoe"):
    for tier,(metal,shine) in enumerate([
        ("#7f8987","#b9beb1"),("#b57850","#e8aa7d"),("#8caaa7","#e3eadf"),
    ]):
        draw(f"tool_{kind}_{tier}",lambda d: tool(d,kind,metal,shine))


def boards(d):
    for x,y in [(17,48),(21,34),(27,24)]:
        poly(d,[(x,y),(x+36,y-11),(x+55,y+5),(x+50,y+14),(x+8,y+27),(x,y+17)],"#a97951")
        poly(d,[(x,y),(x+36,y-11),(x+55,y+5),(x+13,y+18)],"#d9b284")
        line(d,[(x+11,y+6),(x+35,y-3),(x+44,y-1)],"#b88e61",2)


def tideglass(d):
    for x,y,r in [(19,33,27),(44,41,31),(46,17,24)]:
        poly(d,[(x,y+r*.3),(x+r*.4,y),(x+r,y+10),(x+r*.75,y+r),
                (x+r*.2,y+r*.86)],"#6d9d98")
        poly(d,[(x,y+r*.3),(x+r*.4,y),(x+r*.70,y+10),(x+r*.48,y+r*.64)],"#b0d1b4")
        line(d,[(x+r*.4,y+3),(x+r*.67,y+11)],"#e8eac8",2)


def rivets(d):
    for x,y in [(23,43),(46,32),(58,52)]:
        line(d,[(x+4,y+5),(x+9,y+24)],"#9e644b",8)
        oval(d,(x-7,y-8,x+15,y+9),"#c38760")
        oval(d,(x-5,y-7,x+11,y+3),"#e9b17f")


def canvas_roll(d):
    poly(d,[(21,29),(64,20),(76,39),(77,61),(36,77),(24,66)],"#d8bb89")
    poly(d,[(21,29),(64,20),(76,39),(36,52)],"#f0dbad")
    oval(d,(24,40,53,77),"#b79b75")
    oval(d,(29,44,47,71),"#e3c798")
    oval(d,(35,51,44,65),"#9f8666")
    for y in [39,46]:
        line(d,[(45,y+9),(73,y)],"#8ea9a3",3)


def oil(d):
    box(d,(33,25,62,42),"#a98b58",4)
    box(d,(29,34,66,77),"#65897a",9)
    box(d,(33,52,62,73),"#d2ad5d",5)
    line(d,[(35,42),(35,55)],"#cddebc",3)
    box(d,(33,21,63,29),"#cfa270",4)


for slug, picture in [("boards",boards),("tideglass",tideglass),("rivets",rivets),
                      ("canvas",canvas_roll),("lampoil",oil)]:
    draw(slug,picture)


def harbor_broth(d):
    # The bowl repeats the harbor cart's turquoise enamel and cream rim.
    oval(d,(16,39,81,78),"#477c75")
    oval(d,(17,29,80,61),"#fff0cc")
    oval(d,(21,33,76,56),"#db9145")
    for x,y in [(38,42),(61,41)]:
        oval(d,(x-6,y-3,x+5,y+3),"#f8dab1")
    for x,y in [(31,45),(51,37),(56,49)]:
        line(d,[(x-2,y+1),(x+4,y-1)],"#638851",2)
    line(d,[(56,44),(72,20)],"#734d35",4)
    oval(d,(26,58,39,62),"#82b0a0")
    for x in (32,44):
        line(d,[(x,30),(x-2,24),(x+1,20),(x,16)],"#d0ddcd",2)


draw("harbor_broth", harbor_broth)


def broth_lesson(phase):
    # The same counter stays in view as the board gives way to the pot and bowls.
    image = Image.new("RGBA", (252, 152), "#557d78")
    d = ImageDraw.Draw(image)
    for y in range(0, 90, 17):
        d.line((0, y, 252, y), fill="#6c9287")
        for x in range(-30 + (y // 17 % 2) * 22, 253, 44):
            d.line((x, y, x, y + 16), fill="#62877d")
    d.rectangle((0, 92, 252, 152), fill="#b78760")
    for y in (102, 119, 140):
        d.line((0, y + 3, 252, y - 2), fill="#a87955", width=2)
    d.rounded_rectangle((4, 4, 247, 147), radius=11, outline="#ead6a9", width=2)
    for x in (14, 237):
        for y in (14, 137):
            d.polygon([(x-2,y),(x,y-3),(x+2,y),(x,y+3)], fill="#e9d6a5")

    def item(slug, size, x, y):
        with Image.open(OUT / (slug + ".png")) as icon:
            image.alpha_composite(icon.resize((size, size), Image.Resampling.LANCZOS), (x, y))

    if phase == "board":
        d.rounded_rectangle((35, 67, 215, 134), 12, fill="#9c6c49")
        d.rounded_rectangle((35, 63, 215, 128), 12, fill="#d8b282", outline="#e6c695", width=3)
        item("fish", 95, 38, 41)
        item("carrot", 65, 121, 55)
        for x, y in [(164,99), (179,90), (180,105)]:
            d.ellipse((x-6,y-3,x+6,y+3), fill="#db8246", outline="#af633e")
    elif phase == "pot":
        d.ellipse((55, 114, 198, 134), fill="#926b50")
        d.rounded_rectangle((43, 85, 209, 99), 5, fill="#415e57")
        d.rounded_rectangle((63, 68, 191, 124), 18, fill="#ae7355", outline="#684f42", width=3)
        d.ellipse((63, 54, 191, 85), fill="#f4dec2", outline="#835e49", width=2)
        d.ellipse((69, 58, 185, 81), fill="#df964b")
        for x, y in [(88,69), (114,64), (144,72)]:
            d.ellipse((x-5,y-2,x+5,y+2), fill="#fae1b5")
            d.line((x+5,y+4,x+11,y+1), fill="#6d8659", width=2)
        d.line((143, 70, 182, 31), fill="#744e39", width=5)
        d.line((148, 65, 181, 32), fill="#c0905d", width=2)
        d.arc((73,68,179,81), 28, 140, fill="#f1bf7f", width=2)
        for x in (95,116):
            d.arc((x,32,x+12,50), 230, 340, fill="#e1e1c4",width=2)
            d.arc((x+6,23,x+18,41), 65, 156, fill="#e1e1c4",width=2)
    else:
        d.polygon([(39,100),(173,83),(213,120),(79,139)], fill="#70988c")
        item("harbor_broth", 89, 112, 39)
        item("harbor_broth", 105, 49, 42)
    image.convert("RGB").save(OUT.parent / f"broth_{phase}.png", optimize=True)


for phase in ("board", "pot", "bowls"):
    broth_lesson(phase)
