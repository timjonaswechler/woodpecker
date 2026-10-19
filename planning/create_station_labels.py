"""Readable physical station plaques, rendered with the demo's own font."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1] / 'test_app'
OUT = ROOT / 'asset/ui/stations'
LABELS = {
    'board': ('AUFTRAG', 'Drei Zellen für den Sendemast', 'Auftrag lesen', 'Auftrag gelesen'),
    'assembly': ('MONTAGE', 'Zwei Bauteile benötigt', 'Energiezelle bauen', 'Montage abgeschlossen'),
    'rack': ('LAGERHEBEL', 'Regal verschieben', 'Regal verschieben', 'Regal ist geöffnet'),
    'valve': ('WASSERVENTIL', 'Bewässerung öffnen', 'Bewässerung öffnen', 'Ventil ist offen'),
    'scanner': ('ARCHIVSCAN', 'Tafel einlesen', 'Tafel einlesen', 'Tafel ist gescannt'),
    'generator': ('GENERATOR', 'Energiezellen benötigt', 'Energiezellen einsetzen', 'Generator ist bereit'),
    'sender': ('SENDEMAST', 'Generator zuerst versorgen', 'Sender einschalten', 'Signal gesendet'),
    'checkpoint': ('CHECKPOINT', 'Rückkehrpunkt setzen', 'Rückkehrpunkt setzen', 'Rückkehrpunkt gesetzt'),
    'chest': ('BAUPLAN', 'Zielübung abschließen', 'Bauplan mitnehmen', 'Bauplan mitgenommen'),
    'part': ('BAUTEIL', 'Lagerregal zuerst öffnen', 'Zur Montage mitnehmen', 'Bauteil mitgenommen'),
    'cell_assembly': ('FERTIGE ZELLE', 'Montageschalter bedienen', 'Energiezelle mitnehmen', 'Zelle mitgenommen'),
    'cell_water': ('ENERGIEZELLE', 'Beide Wasserventile öffnen', 'Energiezelle mitnehmen', 'Zelle mitgenommen'),
    'cell_archive': ('ENERGIEZELLE', 'Drei Archivtafeln scannen', 'Energiezelle mitnehmen', 'Zelle mitgenommen'),
}

OUT.mkdir(parents=True, exist_ok=True)
font = ROOT / 'asset/fonts/Barlow-Regular.ttf'
for name, (title, *states) in LABELS.items():
    for suffix, description in zip(('', '_ready', '_done'), states):
        image = Image.new('RGB', (768, 238), '#1F3338' if suffix != '_done' else '#215039')
        draw = ImageDraw.Draw(image)
        draw.text((384, 68), title, font=ImageFont.truetype(str(font), 62),
                  anchor='mm', fill='#F1F4EA')
        draw.text((384, 160), description, font=ImageFont.truetype(str(font), 38),
                  anchor='mm', fill='#CCDACE')
        image.save(OUT / f'{name}{suffix}.png')
print(f'{len(LABELS) * 3} station plaques written to {OUT}')
