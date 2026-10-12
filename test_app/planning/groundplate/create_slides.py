"""Create the six-slide PDF for the ground-plate proposal.

Requires reportlab. Render create_model.py in Blender first.
This script only writes the planning PDF and does not modify the game.
"""
from pathlib import Path
import layout as plan
from math import atan2, cos, sin, pi
from xml.sax.saxutils import escape

from reportlab.pdfgen import canvas
from reportlab.lib.colors import HexColor
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import Paragraph
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.utils import ImageReader

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "planning/grundplatte_levelplan.pdf"
ART = ROOT / "artifacts/groundplate-plan"
pdfmetrics.registerFont(TTFont("Barlow", str(ROOT / "asset/fonts/Barlow-Regular.ttf")))
W, H = 960, 540
BG = HexColor("#F5F4ED")
WHITE = HexColor("#FFFFFF")
INK = HexColor("#173438")
MUTED = HexColor("#50696C")
TEAL = HexColor("#237F78")
TEAL_PALE = HexColor("#D2E7E1")
GOLD = HexColor("#AC701B")
GOLD_PALE = HexColor("#F3D49C")
PURPLE = HexColor("#7950A0")
PURPLE_PALE = HexColor("#E1D5EB")
STONE = HexColor("#DBDFD6")
LINE = HexColor("#C8D0C7")
FLOORS = [HexColor(h) for h in ["#F0E1BC", "#DCE5D3", "#EFD7CB", "#D4E6E8"]]
c = canvas.Canvas(str(OUT), pagesize=(W, H), pageCompression=1)
c.setTitle("Acht Räume zum Erkunden | Levelplan")
c.setAuthor("Woodpecker Demo")
c.setSubject("Erweiterter Levelentwurf mit Sammelaufgaben, Räumen und Abkürzungen")
page = 0


def rect(x, y, w, h, fill=WHITE, radius=0):
    c.setFillColor(fill)
    if radius:
        c.roundRect(x, H-y-h, w, h, radius, fill=1, stroke=0)
    else:
        c.rect(x, H-y-h, w, h, fill=1, stroke=0)


def text(s, x, y, size=16, color=INK, center=False):
    c.setFillColor(color)
    c.setFont("Barlow", size)
    draw = c.drawCentredString if center else c.drawString
    draw(x, H-y-size*.85, s)


def para(s, x, y, w, size=16, color=MUTED, max_h=80):
    p = Paragraph(escape(s), ParagraphStyle("body", fontName="Barlow", fontSize=size,
                  leading=size*1.24, textColor=color))
    _, h = p.wrap(w, max_h)
    assert h <= max_h + .1, (page, s, h, max_h)
    p.drawOn(c, x, H-y-h)
    return h


def line(points, color=TEAL, width=2, dash=None):
    c.saveState()
    c.setStrokeColor(color)
    c.setLineWidth(width)
    c.setLineJoin(1)
    c.setLineCap(1)
    if dash:
        c.setDash(dash)
    p = c.beginPath()
    p.moveTo(points[0][0], H-points[0][1])
    for x, y in points[1:]:
        p.lineTo(x, H-y)
    c.drawPath(p)
    c.restoreState()


def arrow(points, color=TEAL, width=2.5, dash=None, head=6):
    line(points, color, width, dash)
    x, y = points[-1]
    a = atan2(y-points[-2][1], x-points[-2][0])
    line([(x-head*cos(a-pi/6), y-head*sin(a-pi/6)), (x, y),
          (x-head*cos(a+pi/6), y-head*sin(a+pi/6))], color, width)


def circle(x, y, r, color):
    c.setFillColor(color)
    c.circle(x, H-y, r, fill=1, stroke=0)


def note(title, body, x, y, w=278, color=TEAL, size=16):
    text(title, x, y, 18, color)
    para(body, x, y+27, w, size, max_h=81)


def new(title, subtitle):
    global page
    if page:
        c.showPage()
    page += 1
    rect(0, 0, W, H, BG)
    text("TEST_APP  /  ERWEITERTER LEVELPLAN", 44, 23, 11, TEAL)
    text(title, 44, 51, 31)
    text(subtitle, 44, 99, 16, MUTED)
    line([(44, 510), (916, 510)], LINE, .8)
    text("Planungsentwurf · 12.10.2026", 44, 519, 10, MUTED)
    text(f"{page:02d} / 06", 879, 519, 10, TEAL)


def cropped_image(path, crop, x, y, w, h):
    """Place a crop with vector clipping, preserving the source image."""
    image = ImageReader(str(path))
    iw, ih = image.getSize()
    left, top, right, bottom = crop
    scale = min(w/(right-left), h/(bottom-top))
    ox = x+(w-(right-left)*scale)/2
    oy = y+(h-(bottom-top)*scale)/2
    c.saveState()
    p = c.beginPath()
    p.rect(ox, H-oy-(bottom-top)*scale, (right-left)*scale, (bottom-top)*scale)
    c.clipPath(p, stroke=0)
    c.drawImage(image, ox-left*scale, H-oy-ih*scale+top*scale,
                iw*scale, ih*scale, mask="auto")
    c.restoreState()



# Shared coordinates keep the maps aligned with the Blender model.
S, OX, OY = 6.1, 78, 480
BLUE = HexColor("#387DA8")
ORANGE = HexColor("#B85F2C")


def xy(x,y):
    return OX+x*S, OY-y*S


def slab(bounds,color):
    x0,y0,x1,y1=bounds
    x,y=xy(x0,y1)
    rect(x,y,(x1-x0)*S,(y1-y0)*S,color)


def wall(axis,coord,lo,hi,color=INK):
    points=[xy(lo,coord),xy(hi,coord)] if axis=='h' else [xy(coord,lo),xy(coord,hi)]
    line(points,color,2.8)


def label(s,x,y,size=13,color=INK):
    px,py=xy(x,y)
    text(s,px,py-size/2,size,color,True)


def route(points,color=TEAL,dash=None):
    arrow([xy(*p) for p in points],color,2.3,dash,5)


def base_plan(ghost=False):
    rect(44,132,552,365,WHITE,9)
    slab(plan.PLATE,HexColor('#E7EAE3'))
    for key,name,bounds,color in plan.ROOMS:
        slab(bounds,HexColor('#F0F2EC') if ghost else HexColor('#'+color))
    slab(plan.MAINTENANCE,PURPLE_PALE)
    slab((3,38,10,43),PURPLE_PALE)
    for segment in plan.walls(): wall(*segment,LINE if ghost else INK)
    for segment in [('v',3,38,43),('v',10,38,43),('h',43,3,5),('h',43,8,10)]:
        wall(*segment,LINE if ghost else INK)
    for value,x,y in plan.LABELS: label(value,x,y,19,MUTED if ghost else INK)


def stairs():
    for x,y0,y1 in plan.STAIRS:
        slab((x-1.35,min(y0,y1),x+1.35,max(y0,y1)),TEAL_PALE)
        for i in range(13):
            yy=y0+(y1-y0)*i/12
            line([xy(x-1.35,yy),xy(x+1.35,yy)],TEAL,.5)


def upper():
    for name,bounds,kind in plan.DECKS:
        slab(bounds,GOLD_PALE if kind=='secret' else TEAL)
    stairs()
    for x,y,z in plan.PARKOUR:
        slab((x-1.25,y-1.25,x+1.25,y+1.25),HexColor('#D5BA95'))


def marker(value,x,y,color=ORANGE):
    px,py=xy(x,y)
    circle(px,py,7.5,WHITE)
    text(value,px,py-5.5,12,color,True)


new('Acht Räume zum Erkunden.',
    'Eine größere Grundplatte mit Treppen, Brücken und Aufgaben in mehreren Räumen.')
cropped_image(ART/'raeumlicher_entwurf.png',(120,110,1890,1270),35,136,595,346)
note('Etwa dreimal so viel Platz',
     'Die Grundplatte wächst von 46 x 34 auf 82 x 58 Meter. Acht Hauptbereiche ersetzen die vier Höfe.',646,145,270,size=15.5)
note('Drei Aufgaben führen zum Ziel',
     'In Werkhalle, Gewächshaus und Archiv gibt es je eine Energiezelle. Alle drei versorgen den Generator.',646,259,270,size=15.5)
note('Abseits des Hauptwegs',
     '24 Münzen, sechs Baupläne und drei versteckte Verbindungen belohnen das Erkunden.',646,373,270,size=15.5)
text('Planungsmodell. Dächer fehlen in der Ansicht; die vordere Außenwand ist gekürzt.',44,493,10.5,MUTED)


new('Jeder Raum bekommt eine eigene Aufgabe.',
    'Die zentrale Halle verbindet die Räume. Die Aufgaben lassen sich in beliebiger Reihenfolge lösen.')
base_plan()
stairs()
for x,y,z in plan.PARKOUR: slab((x-1.25,y-1.25,x+1.25,y+1.25),HexColor('#D5BA95'))
for x,y,z in plan.CELLS: marker('Z',x,y,TEAL)
for x,y,z in plan.GEARS: marker('T',x,y,ORANGE)
circle(*xy(7,5),5,TEAL)
label('G1',34,5,11,PURPLE)
label('G3',6.5,40.8,9,PURPLE)
rows=[
 ('A · Eingang · 18 x 18 m','Auftrag und sichtbarer Startpunkt.'),
 ('B · Werkhalle · 18 x 20 m','Bauteile sammeln und eine Zelle bauen.'),
 ('C · Zentrale Halle · 32 x 28 m','Treppen, Orientierung und Sprungparcours.'),
 ('D · Lagerhalle · 28 x 20 m','Regal verschieben und Verstecke finden.'),
 ('E · Gewächshaus · 28 x 16 m','Bewässerung mit zwei Ventilen starten.'),
 ('F · Archiv · 24 x 16 m','Drei markierte Tafeln scannen.'),
 ('G · Übungsraum · 26 x 16 m','Ziele treffen und eine Truhe öffnen.'),
 ('H · Generatorraum · 28 x 18 m','Drei Zellen einsetzen und Signal starten.'),
]
for i,(title,body) in enumerate(rows):
    y=139+i*44
    text(title,624,y,15.5,TEAL)
    text(body,624,y+21,13.5,MUTED)
text('Bodenplan · Z steht für eine Aufgabenbelohnung, T für ein Bauteil.',61,497,9.5,MUTED)


new('Oben entstehen zusätzliche Verbindungen.',
    'Sechs Treppen verbinden den Boden mit den Galerien in drei Metern Höhe.')
base_plan(ghost=True)
upper()
route([(12,24),(12,33.5),(27.5,33.5)],INK)
route([(37,34),(52,34),(56,34),(56,41)],WHITE)
route([(8,49),(8,42),(19.5,42),(19.5,45.8),(49.5,45.8)],GOLD)
route([(69,4),(69,12.5),(73,12.5)],INK)
label('G2',21,48.1,10,GOLD)
label('Ziel',73,14.1,10,WHITE)
note('Werkhalle bis Gewächshaus',
     'Die türkisen Wege verbinden B, C, D und E. Unten bleiben die Türen und Laufwege offen.',625,145,286,size=15.5)
note('Versteckter Steg vom Archiv',
     'G2 liegt hinter einem Regal auf der Archivgalerie. Der goldene Steg führt über den Übungsraum direkt nach E.',625,261,286,GOLD,15.5)
note('Kurzer Parcours in der Halle',
     'Fünf Sprungplattformen führen zur Galerie mit einem Bauplan. Daneben liegt eine Treppe. Bei einem Fehltritt landet man auf dem Boden.',625,377,286,size=15.5)
text('Obere Ebene · Türkis zeigt Galerien und Brücken · Gold zeigt die Abkürzung G2.',61,497,9.5,MUTED)


new('Sammeln verändert den Fortschritt.',
    'Drei unterschiedliche Aufgaben liefern je eine Energiezelle. Die Reihenfolge bleibt frei.')
# The three cards describe the actual three energy-cell rewards.
cards=[
 ('B + D','Eine Zelle zusammenbauen',
  ['Ein Bauteil liegt in der Werkhalle.',
   'Das zweite liegt hinter dem verschiebbaren Regal in der Lagerhalle.',
   'Beide an der Werkbank einsetzen.'],TEAL),
 ('E','Die Bewässerung starten',
  ['Zwei Ventile zwischen den Beeten finden und aufdrehen.',
   'Fließendes Wasser zeigt den Erfolg.',
   'Die Zelle am Verteiler abholen.'],HexColor('#5E8547')),
 ('F','Das Archiv durchsuchen',
  ['Drei markierte Tafeln in den Regalgängen scannen.',
   'Das Terminal zählt die gelesenen Tafeln.',
   'Nach dem dritten Scan die Zelle abholen.'],PURPLE),
]
for i,(room,title,steps,color) in enumerate(cards):
    x=44+i*298
    rect(x,141,278,252,WHITE,10)
    text(room,x+17,155,12,color)
    text(title,x+17,180,18,color)
    y=215
    for j,step in enumerate(steps):
        text(str(j+1)+'.',x+17,y,14,color)
        h=para(step,x+37,y,225,15,max_h=63)
        y+=h+12
    arrow([(x+139,397),(x+139,419)],color,2,head=6)
rect(44,424,874,73,TEAL_PALE,10)
text('H · Drei Zellen einsetzen, dann das Signal starten.',62,439,19,TEAL)
text('Alle Zellen passen ins Inventar. Münzen und Baupläne sind für den Abschluss freiwillig.',62,470,14,MUTED)


new('Nebenwege bekommen eigene Belohnungen.',
    '24 Münzen markieren Laufwege. Sechs Baupläne liegen in Verstecken oder hinter kleinen Aufgaben.')
base_plan(ghost=True)
# All pickups are shown together here; the room diagram on page three separates heights.
route([(16,4.5),(26,4.5),(42,4.5),(52,4.5)],PURPLE,[3,3])
route([(6.5,35),(6.5,38.5)],PURPLE,[3,3])
route([(23.5,46),(39,46),(52,46)],GOLD,[3,3])
for x,y,z in plan.COINS: circle(*xy(x,y),2.7,GOLD)
for i,(x,y,z) in enumerate(plan.BLUEPRINTS,1): marker(str(i),x,y,BLUE)
text('Drei Abkürzungen',625,140,19,TEAL)
para('G1 verbindet A und H durch einen Wartungsgang. G2 führt oben von F nach E. G3 verbindet B und F durch ein Nebenlager.',625,170,286,15.5,max_h=99)
text('Hier liegen die sechs Baupläne',625,291,18,BLUE)
for i,body in enumerate(['1  Im Wartungsgang G1', '2  Im oberen Nebenraum G2',
                         '3  Im Nebenlager G3', '4  Auf der Galerie über dem Parcours',
                         '5  Hinter den Kisten in der Lagerhalle', '6  In der Truhe nach drei Zieltreffern']):
    text(body,625,323+i*26,14.5,MUTED)
text('Sammelplan für beide Ebenen · Goldene Punkte sind Münzen · Blaue Zahlen sind Baupläne.',61,497,9.5,MUTED)


new('So greift das Spielen ineinander.',
    'Erkunden bringt Bauteile und Abkürzungen. Die Aufgaben füllen den Generator mit Energie.')
cropped_image(ART/'zentrale_halle.png',(80,55,1520,1040),40,140,568,349)
note('In der Nähe mit E bedienen',
     'Bauteil aufnehmen, Ventil drehen, Tafel scannen oder eine Zelle einsetzen. Der Hinweis nennt die passende Aktion.',625,145,286,size=15.5)
note('Sammelstand bleibt sichtbar',
     'Zellen 0/3, Münzen 0/24 und Baupläne 0/6 stehen im HUD. Ein Raumhinweis zeigt, wo noch etwas fehlt.',625,261,286,size=15.5)
note('Nach dem Ziel weiter erkunden',
     'Mit drei Zellen lässt sich das Signal starten. Danach bleiben Parcours, Zielübung und übrige Fundstücke erreichbar.',625,377,286,size=15.5)
text('Räumlicher Entwurf und geplante Spielregeln. Aufgaben und Sammelstände sind noch Konzept.',44,497,10.5,MUTED)
c.save()
print(OUT)
