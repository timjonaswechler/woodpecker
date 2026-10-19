"""Rasterize the four inventory symbols; matching silhouettes live in test_app/src/level/objects/pickups/models.rs."""
from math import sin, cos, pi
from pathlib import Path
from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parents[1] / 'test_app' / 'asset' / 'ui'
COLORS = {'coin': (255,193,62), 'part': (255,139,62), 'cell': (68,215,153), 'blueprint': (83,170,255)}
for name, color in COLORS.items():
    image = Image.new('RGBA', (256,256))
    d = ImageDraw.Draw(image)
    white = (240,255,250)
    if name == 'coin':
        d.ellipse((34,34,222,222), fill=color)
        d.ellipse((56,56,200,200), outline=white, width=9)
        d.rounded_rectangle((117,78,139,178), radius=6, fill=white)
    elif name == 'part':
        points=[]
        for i in range(32):
            a = i*pi/16-pi/32
            r = 109 if i%4 in (0,1) else 82
            points.append((128+sin(a)*r,128+cos(a)*r))
        d.polygon(points, fill=color)
        d.ellipse((91,91,165,165), fill=(0,0,0,0))
    elif name == 'cell':
        d.rounded_rectangle((88,22,168,48),radius=5,fill=white)
        d.rounded_rectangle((62,43,194,228),radius=16,fill=color)
        d.rounded_rectangle((93,123,163,141),radius=3,fill=white)
        d.rounded_rectangle((119,97,137,167),radius=3,fill=white)
    else:
        d.rounded_rectangle((51,24,205,231),radius=8,fill=color)
        d.polygon([(153,24),(205,76),(153,76)],fill=white)
        for y in (107,149,191): d.rounded_rectangle((77,y,174,y+10),radius=3,fill=white)
    image.resize((96,96),Image.Resampling.LANCZOS).save(OUT/f'{name}.png')
