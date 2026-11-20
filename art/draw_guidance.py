"""Draw the island atlas and the two new harvest icons used by the HUD."""
from pathlib import Path
import math
import struct
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
UI = ROOT / "assets" / "ui"

def atlas():
    data = (ROOT / "assets/world/terrain.bin").read_bytes()
    _, _, _, cols, rows = struct.unpack_from("<fffHH", data, 4)
    width, height = 480, round(480 * rows / cols)
    image = Image.new("RGB", (width, height))
    pixels = image.load()
    for y in range(height):
        z = min(rows - 1, round(y / (height - 1) * (rows - 1)))
        for x in range(width):
            k = min(cols - 1, round(x / (width - 1) * (cols - 1)))
            i = 20 + 3 * (z * cols + k)
            h, blocked = struct.unpack_from("<hB", data, i)
            if h < -46:
                variation = 4 * math.sin(x * .084 + y * .115)
                pixels[x, y] = (58, round(108 + variation), round(108 + variation))
            elif h < 10:
                pixels[x, y] = (178, 162, 113)
            else:
                ridge = min(1, h / 940)
                shade = 12 * math.sin(x * .16 - y * .21)
                pixels[x, y] = tuple(round(n + ridge * lift + shade + blocked * offset)
                                     for n, lift, offset in [(100, 49, -5), (135, 2, -4), (91, 12, 2)])
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
