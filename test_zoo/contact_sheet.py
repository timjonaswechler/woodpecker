"""Lay out real Bevy captures; no replacement models or painted renders."""
import argparse
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

parser = argparse.ArgumentParser()
parser.add_argument('directory', type=Path)
args = parser.parse_args()
font_path = Path(__file__).resolve().parent.parent / 'test_app/asset/fonts/Barlow-Regular.ttf'
font = ImageFont.truetype(str(font_path), 23)
small = ImageFont.truetype(str(font_path), 19)
names = ['Auftragstafel', 'Montage', 'Lagerhebel', 'Wasserventil', 'Archivscanner',
         'Generatorbedienung', 'Sendeschalter', 'Rückkehrpunkt', 'Belohnungstruhe',
         'Bauteilablage', 'Montageausgabe', 'Gartenausgabe', 'Archivausgabe']
files = sorted(args.directory.glob('*-front.png'))
width, height = 420, 350
sheet = Image.new('RGB', (width * 4, height * 4 + 70), '#E8EBE7')
draw = ImageDraw.Draw(sheet)
draw.text((24, 18), 'Modell-Zoo | 13 Stationen und ihre Zustände', font=font, fill='#203338')
samples = [(path, f'{i + 1:02}  {names[i]}') for i, path in enumerate(files)]
for index, label in [(4, 'Archivscanner · benutzt'), (9, 'Bauteilablage · leer'), (8, 'Belohnungstruhe · leer')]:
    samples.append((files[index].with_name(files[index].name.replace('-front', '-used')), label))
for i, (path, label) in enumerate(samples):
    x, y = (i % 4) * width, (i // 4) * height + 70
    image = Image.open(path).convert('RGB').crop((240, 115, 1200, 805))
    image.thumbnail((width, height - 42))
    sheet.paste(image, (x + (width - image.width) // 2, y))
    draw.text((x + 18, y + height - 37), label, font=small, fill='#203338')
sheet.save(args.directory / 'all-models.jpg', quality=94)
# Four views per prop make defects and state changes easy to compare.
for i, path in enumerate(files):
    stem = path.name.removesuffix('-front.png')
    panel = Image.new('RGB', (1440, 400), '#E8EBE7')
    draw = ImageDraw.Draw(panel)
    draw.text((24, 16), names[i], font=font, fill='#203338')
    for j, (view, label) in enumerate([('front', 'Bereit'), ('back', 'Rückseite'), ('side', 'Seite'), ('used', 'Benutzt / leer')]):
        image = Image.open(args.directory / f'{stem}-{view}.png').convert('RGB').crop((260, 120, 1180, 815))
        image.thumbnail((360, 510))
        panel.paste(image, (j * 360 + (360 - image.width) // 2, 70))
        draw.text((j * 360 + 16, 365), label, font=small, fill='#203338')
    panel.save(args.directory / f'{stem}-review.jpg', quality=94)
print(args.directory / 'all-models.jpg')
