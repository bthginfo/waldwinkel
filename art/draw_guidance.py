"""Draw the island atlas and the two new harvest icons used by the HUD."""
from pathlib import Path
import math
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
UI = ROOT / "assets" / "ui"

from expansion_layout import ATLAS_BOUNDS, Ground, SITES, original, phase, smooth

def atlas():
    ox, oz, ex, ez = ATLAS_BOUNDS
    width, height = 740, round(740 * (ez-oz)/(ex-ox))
    image = Image.new("RGB", (width, height))
    pixels = image.load()
    tiles = [Ground(name) for name in SITES]
    climates = {'amber':(150,139,72),'heights':(125,135,111),'reef':(165,155,105)}
    for j in range(height):
        z = oz+j/(height-1)*(ez-oz)
        for i in range(width):
            x = ox+i/(width-1)*(ex-ox)
            tile = next((t for t in tiles if t.bounds[0]<=x<t.bounds[2] and t.bounds[1]<=z<t.bounds[3]),None)
            h, blocked = tile.sample(x,z) if tile else original(x,z)
            if h < -.46:
                lift = 6 * phase(x*.31,z*.31)
                pixels[i, j] = (57, round(107+lift), round(111+lift))
            elif h < .10:
                pixels[i, j] = (180, 164, 117)
            else:
                seed = climates[tile.name] if tile and h > original(x,z)[0]+.10 else (100,135,91)
                ridge = smooth(h/13.)
                shade = 9*phase(x,z) - 6*blocked
                lane = (1-smooth(tile.way(x,z)/1.9))*.65 if tile else 0
                target = (224,218,186)
                terrain = tuple(c + ridge*(a-c) for c,a in zip(seed,target))
                pixels[i, j] = tuple(round(c+shade+lane*(a-c)) for c,a in zip(terrain,(197,174,125)))
    image.save(UI / "island_atlas.png", optimize=True)

def item_icons():
    scale = 3
    def canvas():
        image = Image.new("RGBA", (96 * scale, 96 * scale))
        return image, ImageDraw.Draw(image)
    def poly(draw, points, color):
        draw.polygon([(round(x*scale), round(y*scale)) for x, y in points], fill=color)
    def ellipse(draw, box, color):
        draw.ellipse(tuple(x*scale for x in box), fill=color)
    image, draw = canvas()
    ellipse(draw, (15, 78, 81, 91), (57, 50, 36, 85))
    for n in range(11):
        angle = -.82 + n * .155
        tip = (48 + 52 * math.sin(angle), 77 - 59 * math.cos(angle))
        bend = (48 + 24 * math.sin(angle + .23), 47 - n % 3 * 4)
        color = [(85, 139, 73), (138, 171, 74), (190, 193, 88)][n % 3]
        poly(draw, [(43+n*.85, 80), (bend[0]-5, bend[1]), tip, (bend[0]+4, bend[1]+8), (48+n*.4, 83)], color)
    poly(draw, [(40, 67), (58, 70), (54, 76), (40, 73)], (190, 127, 64))
    image.resize((96, 96), Image.Resampling.LANCZOS).save(UI / "items/grass.png")
    image, draw = canvas()
    ellipse(draw, (15, 78, 84, 92), (52, 45, 29, 78))
    ellipse(draw, (55, 36, 81, 65), (126, 162, 160))
    ellipse(draw, (59, 41, 74, 60), (0, 0, 0, 0))
    poly(draw, [(28, 28), (63, 30), (63, 43), (69, 64), (61, 81), (30, 82),
                (20, 69), (24, 43)], (199, 220, 207))
    poly(draw, [(27, 58), (65, 58), (69, 64), (61, 81), (30, 82), (20, 69)], (120, 165, 165))
    ellipse(draw, (24, 19, 66, 36), (106, 155, 158))
    ellipse(draw, (28, 23, 62, 32), (251, 238, 194))
    poly(draw, [(28, 42), (32, 39), (29, 68), (23, 66)], (240, 244, 218))
    image.resize((96, 96), Image.Resampling.LANCZOS).save(UI / "items/milk.png")
    image, draw = canvas()
    poly(draw, [(48, 6), (57, 37), (88, 47), (57, 56), (48, 89), (39, 56), (8, 47), (39, 37)], (255, 249, 222))
    image.resize((48, 48), Image.Resampling.LANCZOS).save(UI / "gather_glint.png")

if __name__ == "__main__":
    atlas()
    item_icons()
