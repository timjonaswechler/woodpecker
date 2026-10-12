"""Create the four-slide PDF for the ground-plate proposal.

Requires reportlab. Render create_model.py in Blender first.
This script only writes the planning PDF and does not modify the game.
"""
from pathlib import Path
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
c.setTitle("Eine Grundplatte, vier Höfe | Levelplan")
c.setAuthor("Woodpecker Demo")
c.setSubject("Räumlicher Entwurf mit Treppen, Brücken und versteckten Verbindungen")
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
    text("TEST_APP  /  LEVELPLAN", 44, 23, 11, TEAL)
    text(title, 44, 51, 31)
    text(subtitle, 44, 99, 16, MUTED)
    line([(44, 510), (916, 510)], LINE, .8)
    text("Planungsentwurf · 12.10.2026", 44, 519, 10, MUTED)
    text(f"{page:02d} / 04", 879, 519, 10, TEAL)


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


# Both floor plans use the same coordinates as create_model.py.
S, OX, OY = 10, 96, 476


def xy(x, y):
    return OX+x*S, OY-y*S


def slab(x0, y0, x1, y1, color):
    x, y = xy(x0, y1)
    rect(x, y, (x1-x0)*S, (y1-y0)*S, color)


def wall(a, b, color=INK):
    line([xy(*a), xy(*b)], color, 3.5)


def route(points, color=TEAL, dash=None):
    arrow([xy(*p) for p in points], color, 2.5, dash)


def label(s, x, y, size=13, color=INK):
    px, py = xy(x, y)
    text(s, px, py-size/2, size, color, True)


def stairs(x, y0, y1, color=TEAL_PALE):
    slab(x-1.35, y0, x+1.35, y1, color)
    for i in range(13):
        y = y0+(y1-y0)*i/12
        line([xy(x-1.35, y), xy(x+1.35, y)], TEAL, .55)


def base_plan(ghost=False):
    rect(44, 139, 552, 352, WHITE, 9)
    slab(-1, -1, 45, 33, STONE if not ghost else HexColor("#E9ECE6"))
    for i, (x0, y0, x1, y1) in enumerate([(1, 1, 19, 14), (1, 17, 19, 31),
                                         (25, 17, 43, 31), (25, 1, 43, 14)]):
        slab(x0, y0, x1, y1, FLOORS[i] if not ghost else HexColor("#F1F2EC"))
    segments = [
        ((.7,.7),(.7,14.3)), ((.7,.7),(19.3,.7)),
        ((.7,14.3),(9.5,14.3)), ((12.8,14.3),(19.3,14.3)),
        ((19.3,.7),(19.3,6.4)), ((19.3,9),(19.3,14.3)),
        ((.7,16.7),(.7,31.3)), ((.7,31.3),(19.3,31.3)),
        ((.7,16.7),(9.5,16.7)), ((12.8,16.7),(19.3,16.7)),
        ((19.3,16.7),(19.3,31.3)),
        ((43.3,16.7),(43.3,31.3)), ((24.7,31.3),(43.3,31.3)),
        ((24.7,16.7),(24.7,18.8)), ((24.7,21.5),(24.7,31.3)),
        ((24.7,16.7),(34,16.7)), ((37,16.7),(43.3,16.7)),
        ((24.7,14.3),(34,14.3)), ((37,14.3),(43.3,14.3)),
        ((43.3,.7),(43.3,14.3)), ((24.7,.7),(24.7,14.3)),
        ((24.7,.7),(43.3,.7)),
        ((19.4,5.1),(24.45,5.1)), ((19.3,21.5),(24.7,21.5)),
        ((3,19),(3,25)), ((3,25),(9,25)), ((9,19),(9,25)),
        ((3,19),(4.8,19)), ((7.4,19),(9,19)),
    ]
    segments += [((x,14.3),(x,16.7)) for x in [9.5,12.8,34,37]]
    for a, b in segments:
        wall(a, b, LINE if ghost else INK)
    for x, y in [(17.6,6),(18,10),(20.3,5.9),(30,22),(29.2,23.8),(41,19),(31,18.6)]:
        slab(x-.6,y-.6,x+.6,y+.6, LINE if ghost else HexColor("#BDA382"))
    for s, x, y in [("A",5,11),("B",4.5,28.5),("C",40,23),("D",28.5,4)]:
        label(s, x, y, 23, MUTED if ghost else INK)


def g1_floor():
    slab(19.6,5.2,23.8,10.2,PURPLE_PALE)
    slab(20.55,10.2,23.65,18.7,PURPLE_PALE)
    slab(22,18.7,24.8,21.3,PURPLE_PALE)


def upper_decks():
    for bounds in [(10,26,20,30.2),(20,27,30,30),(30,27,41.5,30.2),(30,9,42,12.5)]:
        slab(*bounds, TEAL)
    for bounds in [(16,21.8,20,26),(20,21.8,20.7,24.5),
                   (20.7,9,23.7,24.5),(23.7,9,30,12)]:
        slab(*bounds, GOLD_PALE)
    stairs(12,19,26)
    stairs(37,20,27)
    stairs(35,3,9)


new("Eine Grundplatte, vier Höfe.",
    "Treppen führen auf Galerien. Brücken verbinden die oberen Wege.")
cropped_image(ART/"raeumlicher_entwurf.png", (140,145,1455,985), 35,140,590,333)
note("Ein gemeinsamer Boden", "Alle Höfe stehen auf derselben Platte. Unter den Brücken bleibt Platz zum Laufen.", 646,148,270)
note("Eigene kleine Bereiche", "Mauern und Tore trennen die Höfe. Jeder Hof hat einen eigenen Zweck und erkennbare Eingänge.",646,251,270)
note("Zwei versteckte Verbindungen", "Ein Nebenlager verbindet A mit C am Boden. Ein oberer Nebenraum verbindet B mit D.",646,358,270)
text("Räumlicher Entwurf. Vordere Wände sind für die Ansicht gekürzt.",44,486,11,MUTED)


new("Unten liegen vier abgegrenzte Bereiche.",
    "Der Hauptweg führt vom Starthof über Werkhof und Lagerhof zum Zielhof.")
base_plan()
g1_floor()
stairs(12,19,26)
stairs(37,20,27)
stairs(35,3,9)
slab(4.9,22.7,6.1,23.6,TEAL)
for x in [27.2,29.3,31.4]:
    circle(*xy(x,25.6),4,GOLD)
circle(*xy(10,6),8,TEAL)
label("Start",6.3,5.8,12)
label("G1",21.7,6.5,12,PURPLE)
route([(11,6),(11,17.7),(12,18.6)])
route([(37,20),(35.5,17.3),(35.5,15.4),(39,13),(39,4),(35,2),(35,2.8)])
route([(16,7.7),(22,7.7),(22,20),(26.2,20)],PURPLE,[3,3])
label("Treppe",14.1,18.2,11,TEAL)
label("Treppe",39.4,18.7,11,TEAL)
note("A · Starthof", "Platz zum Ankommen. Das Tor gegenüber führt in den Werkhof.",625,145,286,size=15.5)
note("B · Werkhof", "Werkstatt mit Konsole. Die Treppe daneben führt zur ersten Brücke.",625,228,286,size=15.5)
note("C · Lagerhof", "Kisten, Suchaufgabe und Schießstand. Vom Hof führt ein Tor weiter nach D.",625,311,286,size=15.5)
note("D · Zielhof", "Der Zielpunkt steht oben auf dem Balkon. Die Treppe beginnt vorne im Hof.",625,394,286,size=15.5)
text("Draufsicht · Bodenhöhe 0 m · Violett zeigt den versteckten Weg G1",61,493,10.5,MUTED)


new("Die oberen Wege verbinden die Höfe.",
    "Die Brücken liegen etwa drei Meter über dem Boden. Beide Ebenen sind begehbar.")
base_plan(ghost=True)
g1_floor()
upper_decks()
route([(12,20),(12,28.5),(19,28.5)],INK)
route([(31,28.5),(37,28.5),(37,21)],WHITE)
route([(35,4),(35,10.7),(39.1,10.7)],INK)
route([(18,24),(22.2,23),(22.2,11),(28.7,11)],GOLD)
label("Brücke 1",25,28.5,11,WHITE)
label("2",21.3,16.1,12,GOLD)
label("G2",18,22.8,11,GOLD)
label("G1",21.7,7.7,11,PURPLE)
label("Ziel",40,11,12,WHITE)
note("Hauptweg über Brücke 1", "In B die Treppe hinauf. Über die Brücke nach C, dort wieder hinunter. Weiter durch das Tor nach D.",625,147,286,size=15.5)
note("G1 · Nebenlager am Boden", "Ein Spalt zwischen Kisten in A führt ins Nebenlager. Der Gang unter Brücke 2 endet direkt in C.",625,263,286,PURPLE,15.5)
note("G2 · Nebenraum auf der Galerie", "Neben einem Regal in B liegt ein Seiteneingang. Dahinter führt Brücke 2 direkt zum Zielbalkon in D.",625,379,286,GOLD,15.5)
text("Gleiche Draufsicht · Türkis zeigt den Hauptweg oben · Gold zeigt G2",61,493,10.5,MUTED)


new("Hier läuft man oben und unten.",
    "Der goldene Steg liegt über dem violetten Gang. Beide Wege bleiben nutzbar.")
cropped_image(ART/"unter_der_bruecke.png", (65,60,1350,845), 44,143,558,335)
note("Eine Abkürzung muss auffallen", "Ein sichtbarer Spalt, Licht oder ein Gegenstand am Eingang gibt einen Hinweis auf den Nebenraum.",625,147,286,size=15.5)
note("Erkunden spart später einen Umweg", "G1 überspringt den Weg durch B. G2 überspringt C und führt direkt auf den Zielbalkon. Beide Wege bleiben offen.",625,263,286,size=15.5)
note("Beim Bau zuerst prüfen", "Boden und Brücken brauchen getrennte begehbare Ebenen. Figur und Klickwege müssen auf der richtigen Höhe bleiben.",625,379,286,size=15.5)
text("Planungsmodell in Blender. Farben markieren Wege, der Spielstil folgt später.",44,491,11,MUTED)

c.save()
print(OUT)
