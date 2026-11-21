"""Catalogue illustrations for the two meals unlocked on the outer island."""
from pathlib import Path
from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parents[1] / 'assets/ui/items'


def icon(name):
    image = Image.new('RGBA', (384, 384))
    draw = ImageDraw.Draw(image)
    def ellipse(box, color):
        draw.ellipse([int(n*4) for n in box], fill=color)
    def polygon(points, color):
        draw.polygon([(int(x*4), int(y*4)) for x, y in points], fill=color)
    ellipse((8,69,88,86),(33,41,27,87))
    ellipse((8,22,89,79),'#44746f')
    ellipse((7,19,89,73),'#85a39b')
    ellipse((16,24,81,66),'#557c72')
    ellipse((20,29,77,65),'#d6d2b3')
    if name=='sage_cheese':
        polygon([(24,54),(25,38),(64,28),(75,32),(76,55),(69,62),(38,65)],'#e4cc83')
        polygon([(24,41),(25,37),(62,26),(75,31),(63,48),(40,55)],'#f3e1a6')
        ellipse((31,48,41,53),'#e3cd88')
        ellipse((49,37,58,41),'#dbb976')
        ellipse((60,44,69,49),'#d3b16c')
        polygon([(51,56),(57,52),(63,44),(68,46),(60,54),(53,59)],'#71947a')
        polygon([(55,56),(57,47),(62,45),(62,51)],'#8fa889')
    else:
        ellipse((21,34,74,64),'#d6a24e')
        ellipse((21,29,74,57),'#edbf68')
        polygon([(23,33),(43,26),(68,35),(73,39),(63,48),(36,51),(24,45)],'#f6d888')
        polygon([(29,53),(47,58),(65,53),(69,56),(58,61),(37,61)],'#f5d184')
        for x,y in [(33,37),(43,44),(49,35),(59,40),(38,49),(64,46),(54,53)]:
            polygon([(x,y),(x+5,y-2),(x+7,y+1),(x+3,y+3)],'#7d9768')
        for x,y in [(33,41),(50,39),(63,42),(44,52)]:
            ellipse((x,y,x+3,y+2),'#b985a8')
    image.resize((128,128), Image.Resampling.LANCZOS).save(OUT / f'{name}.png')


if __name__=='__main__':
    icon('sage_cheese')
    icon('tide_omelette')
