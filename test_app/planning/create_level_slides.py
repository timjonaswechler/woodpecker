"""Build the planning slides. Requires reportlab; optional PDF previews use PyMuPDF.

Run from any directory: python create_level_slides.py
Only writes the planning PDF, never changes the game.
"""
from pathlib import Path
from math import atan2, cos, sin, pi
from xml.sax.saxutils import escape

from reportlab.pdfgen import canvas
from reportlab.lib.colors import HexColor, Color
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import Paragraph
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.utils import ImageReader

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = Path(__file__).with_name("level_varianten.pdf")
pdfmetrics.registerFont(TTFont("Barlow", str(ROOT / "asset/fonts/Barlow-Regular.ttf")))
W, H = 960, 540
BG = HexColor("#F5F4ED")
WHITE = HexColor("#FFFFFF")
INK = HexColor("#173438")
MUTED = HexColor("#50696C")
TEAL = HexColor("#087D76")
PALE = HexColor("#DCEBE3")
GOLD = HexColor("#BD780E")
GOLD_PALE = HexColor("#F7EACD")
BLUE = HexColor("#DDEDF0")
BLUE_INK = HexColor("#357A90")
CORAL = HexColor("#BE593D")
STONE = HexColor("#D8D6C8")
LINE = HexColor("#CDD5CD")
DARK = HexColor("#153C40")

c = canvas.Canvas(str(OUTPUT), pagesize=(W,H), pageCompression=1)
c.setTitle("Signalinseln | Vorschläge für abwechslungsreichere Level")
c.setAuthor("Woodpecker Demo · Levelplanung")
c.setSubject("Zehn Planungsfolien mit drei Layoutvarianten, Spieleinstieg und Level-Komponenten")
c.setCreator("Lokale Planungspräsentation mit ReportLab")
page = 0


def rect(x,y,w,h,fill=WHITE,stroke=None,r=0):
    c.setFillColor(fill)
    c.setStrokeColor(stroke or fill)
    if r: c.roundRect(x,H-y-h,w,h,r,fill=1,stroke=int(stroke is not None))
    else: c.rect(x,H-y-h,w,h,fill=1,stroke=int(stroke is not None))


def line(points,color=TEAL,width=3,dash=None):
    c.saveState(); c.setStrokeColor(color); c.setLineWidth(width)
    c.setLineCap(1); c.setLineJoin(1)
    if dash: c.setDash(dash)
    p=c.beginPath(); p.moveTo(points[0][0],H-points[0][1])
    for x,y in points[1:]: p.lineTo(x,H-y)
    c.drawPath(p); c.restoreState()


def arrow(points,color=TEAL,width=3,dash=None,head=8):
    line(points,color,width,dash)
    x,y=points[-1]; px,py=points[-2]; a=atan2(y-py,x-px)
    line([(x-head*cos(a-pi/6),y-head*sin(a-pi/6)),(x,y),(x-head*cos(a+pi/6),y-head*sin(a+pi/6))],color,width)


def poly(points,fill=PALE,stroke=LINE):
    c.setFillColor(fill); c.setStrokeColor(stroke); c.setLineWidth(1)
    p=c.beginPath();p.moveTo(points[0][0],H-points[0][1])
    for x,y in points[1:]: p.lineTo(x,H-y)
    p.close();c.drawPath(p,fill=1,stroke=1)


def circle(x,y,r,fill=TEAL,stroke=None):
    c.setFillColor(fill);c.setStrokeColor(stroke or fill)
    c.circle(x,H-y,r,fill=1,stroke=int(stroke is not None))


def text(s,x,y,size=18,color=INK,align="left"):
    c.setFont("Barlow",size);c.setFillColor(color)
    if align=="center": c.drawCentredString(x,H-y-size*0.85,s)
    elif align=="right": c.drawRightString(x,H-y-size*0.85,s)
    else: c.drawString(x,H-y-size*0.85,s)


def para(s,x,y,w,size=18,color=INK,max_h=100,leading=None):
    style=ParagraphStyle("p",fontName="Barlow",fontSize=size,leading=leading or size*1.23,textColor=color)
    p=Paragraph(escape(s).replace("\n","<br/>"),style)
    _,h=p.wrap(w,max_h)
    assert h<=max_h+0.2, f"Text overflow page {page}: {s[:70]} {h:.1f}>{max_h}"
    p.drawOn(c,x,H-y-h)
    return h


def tag(s,x,y,fill=PALE,color=TEAL):
    width=pdfmetrics.stringWidth(s,"Barlow",11)+20
    rect(x,y,width,24,fill,r=12);text(s,x+10,y+6,11,color)


def note(title,body,x,y,w=268,color=TEAL):
    text(title,x,y,17,color)
    return para(body,x,y+26,w,16,MUTED,80)


def new(title,kicker="LEVELPLANUNG",subtitle=None):
    global page
    if page: c.showPage()
    page+=1
    rect(0,0,W,H,BG)
    text(f"SIGNALINSELN  /  {kicker}",44,24,11,TEAL)
    text(title,44,52,32,INK)
    if subtitle: para(subtitle,44,99,872,16,MUTED,40)
    line([(44,505),(916,505)],LINE,0.8)
    text("Planungsentwurf · 12.10.2026 · test_app",44,517,10.5,MUTED)
    text(f"{page:02d} / 10",916,516,11,TEAL,"right")


def map_frame(kind="Draufsicht · schematisch"):
    rect(44,145,552,296,BLUE,r=14)
    text(kind,61,157,11,BLUE_INK)


def legend(y=458):
    line([(48,y+7),(79,y+7)],TEAL,3)
    text("Laufweg",87,y,12,MUTED)
    line([(177,y+7),(208,y+7)],GOLD,3,[5,4])
    text("Optional / Parcours",216,y,12,MUTED)
    line([(375,y+7),(406,y+7)],BLUE_INK,2,[2,4])
    text("Später geöffneter Rückweg",414,y,12,MUTED)


def node(x,y,n,label,dx=0,dy=16,color=TEAL):
    circle(x,y,11,WHITE,color)
    text(str(n),x,y-6.4,13,color,"center")
    text(label,x+dx,y+dy,14,INK,"center")


def arrow_chain(items,x,y,box_w=155,box_h=96,gap=17):
    for i,(title,body) in enumerate(items):
        bx=x+i*(box_w+gap)
        rect(bx,y,box_w,box_h,WHITE,r=10)
        text(f"0{i+1}",bx+14,y+12,12,TEAL)
        text(title,bx+14,y+33,18,INK)
        para(body,bx+14,y+58,box_w-26,14,MUTED,box_h-62)
        if i<len(items)-1: arrow([(bx+box_w+3,y+box_h/2),(bx+box_w+gap-3,y+box_h/2)],TEAL,1.8,head=5)


# 1. Actual evidence and a concrete design diagnosis.
new("Mehr Abwechslung beginnt beim Weg.","AUSGANGSPUNKT", "Das aktuelle Level hat neue Modelle, behält aber große Teile der alten Anordnung.")
rect(44,151,550,315,WHITE,r=12)
shot=ROOT / "artifacts/overview.png"
# A clipped crop focuses the existing screenshot on the playable terrain.
c.saveState()
p=c.beginPath();p.roundRect(56,H-438,526,257,8)
c.clipPath(p,stroke=0)
c.drawImage(ImageReader(str(shot)),56-280*0.56,H-181-900*0.56+200*0.56,1440*0.56,900*0.56,mask="auto")
c.restoreState()
text("Aktueller Spielstand · Screenshot aus der Demo",60,447,11,MUTED)
note("Ähnliche Grundform", "Viele rechte Winkel und ähnlich große Inseln machen die Wege vorhersehbar.",624,157)
note("Ähnliche Situationen", "Meist: offene Plattform betreten, etwas einsammeln, über die nächste Brücke laufen.",624,262)
note("Ziel für den nächsten Entwurf", "Enge und offene Räume, Sichtwechsel und Entscheidungen sollen den Weg prägen.",624,367)

# 2. Reusable level components, grouped by gameplay purpose.
new("Welche Bausteine braucht ein Level?","BAUKASTEN", "Jeder Bereich bekommt eine klare Aufgabe im Spiel.")
cards=[
 ("Start & Orientierung","Sicherer Ankunftsort, erster Auftrag und ein sichtbares Fernziel.",TEAL),
 ("Unterschiedliche Räume","Offener Hof, enger Durchgang, Innenraum und exponierter Grat.",BLUE_INK),
 ("Wege & Höhen","Breiter Hauptweg, Rampen, Treppen, Abzweige und kurze Rundwege.",TEAL),
 ("Interaktive Hindernisse","Zelle, Schalter oder Zielscheibe öffnen eine sichtbare Tür oder Brücke.",CORAL),
 ("Freiwillige Entdeckungen","Nische, Aussicht und Parcours führen zu Belohnungen oder Abkürzungen.",GOLD),
 ("Sicherheit & Rückmeldung","Checkpoint vor Risiko; Licht, Bewegung und Ton zeigen den Erfolg.",BLUE_INK),
]
for i,(title,body,color) in enumerate(cards):
    x=44+(i%3)*296;y=151+(i//3)*163
    rect(x,y,280,145,WHITE,r=12)
    rect(x+16,y+18,28,4,color,r=2)
    text(title,x+16,y+37,19,color)
    para(body,x+16,y+70,246,18,MUTED,70)

# 3. Start without a wall of tutorial text.
new("Die ersten 90 Sekunden","SPIELEINSTIEG", "Zeitangaben sind Zielwerte für den Entwurf. Der Spieler bestimmt sein Tempo.")
items=[
 ("Ankommen","0–20 s: laufen und Kamera testen."),
 ("Ziel sehen","20–40 s: der Turm taucht hinter einer Felskante auf."),
 ("Etwas verändern","40–65 s: Zelle einsetzen; ein Tor öffnet sich."),
 ("Weg wählen","65–90 s: sicherer Pfad oder kleine Erkundungsschleife."),
]
arrow_chain(items,44,155,205,130,17)
# A simple eye-level framing diagram illustrates how the target is revealed.
rect(44,308,434,167,BLUE,r=12)
poly([(62,449),(62,327),(154,316),(200,352),(210,449)],STONE)
poly([(340,449),(359,329),(411,343),(459,312),(459,449)],STONE)
line([(90,450),(185,450),(256,410),(321,410),(420,454)],TEAL,5)
rect(283,343,12,65,WHITE,TEAL)
rect(272,338,35,7,TEAL)
line([(289,338),(289,324)],TEAL,2)
node(178,439,"S","Start",dy=16)
text("Das Fernziel wird beim Vorlaufen sichtbar.",62,314,13,BLUE_INK)
para("Direkt am Start",510,313,358,21,TEAL,35)
para("Ein ruhiger Platz, ein Ausgang und eine einfache Interaktion. Der Blaster folgt erst im nächsten Bereich.",510,350,390,18,INK,69)
para("Name und Einstellungen bleiben über das Menü erreichbar.",510,431,390,15,MUTED,43)

# 4. Option A: a loop with a goal physically near the start, reached by a long varied route.
new("A · Die Schlucht mit Rundweg","LAYOUTVARIANTE", "Eine gerichtete Expedition durch wechselnde Räume. Das Ziel ist anfangs sichtbar.")
map_frame()
poly([(80,205),(141,195),(187,232),(196,295),(267,331),(352,320),(432,288),(467,237),(461,203),(388,185),(302,207),(278,183),(325,169),(477,181),(551,218),(565,283),(515,365),(444,405),(322,425),(210,410),(118,356),(75,278)],PALE)
poly([(219,207),(280,230),(338,215),(419,226),(393,277),(290,303),(229,279)],STONE)
arrow([(111,241),(148,289),(213,355),(317,373),(400,354),(463,306),(508,250),(468,211),(369,208)],TEAL,4)
arrow([(374,197),(300,179),(229,178),(155,216)],BLUE_INK,2,[3,4])
text("Tor öffnet den kurzen Rückweg",268,157,12,BLUE_INK,"center")
arrow([(398,363),(499,391),(550,330),(520,260)],GOLD,3,[5,5])
circle(536,351,5,GOLD);text("Geheimnis",555,367,12,GOLD,"right")
arrow([(191,330),(131,386),(91,373)],GOLD,2,[5,4],head=6)
text("Versteckte Nische",115,401,12,GOLD,"center")
node(111,241,1,"Lager",dy=-34)
node(213,355,2,"Werkstatt",dy=20)
node(400,354,3,"Schießhof",dy=20)
node(508,250,4,"Grat",dx=35,dy=-5)
node(369,208,5,"Signalturm",dy=19)
note("Spielgefühl", "Ein klarer Weg mit Abzweigen. Kurven und Felsen geben den nächsten Bereich erst später frei.",624,157)
note("Was sich verändert", "Innenraum, breiter Hof und schmaler Höhenweg lösen die gleichförmigen Plattformen ab.",624,266)
note("Abwägung", "Gut für eine kompakte erste Demo. Die Reihenfolge ist stärker vorgegeben.",624,375)
legend()

# 5. Option B: asymmetrical hub and unequal districts on a continuous shore.
new("B · Der Hafen mit freien Bezirken","LAYOUTVARIANTE", "Ein zentraler Treffpunkt verbindet Bereiche mit verschiedener Größe und Nutzung.")
map_frame()
poly([(66,352),(98,290),(107,192),(171,177),(230,202),(251,267),(331,270),(364,200),(466,168),(539,186),(569,239),(512,282),(483,308),(559,342),(568,405),(473,428),(412,391),(341,364),(259,411),(179,420),(96,404)],PALE)
poly([(238,174),(308,178),(341,218),(329,259),(269,252)],STONE)
rect(132,199,65,42,DARK,r=4);text("Innenraum",164,213,12,WHITE,"center")
arrow([(111,382),(214,365),(305,328)],TEAL,4)
arrow([(305,328),(234,302),(175,249)],TEAL,4)
arrow([(318,326),(417,328),(489,380)],TEAL,4)
arrow([(321,308),(383,260),(481,213)],TEAL,4)
arrow([(481,213),(457,180),(357,178)],TEAL,3)
arrow([(184,204),(265,154),(356,170)],BLUE_INK,2,[3,4])
arrow([(504,363),(543,298),(513,243)],GOLD,3,[5,4])
node(111,382,1,"Anlegesteg",dy=19)
node(305,328,2,"Lagerplatz",dy=20)
node(175,249,3,"Werkstatt",dx=-37,dy=18)
node(489,380,4,"Vorratshof",dy=21)
node(481,213,5,"Klippenpfad",dx=0,dy=20)
node(350,178,6,"Finale",dy=20)
note("Spielgefühl", "Erkunde die Bezirke in freier Reihenfolge. Neue Türen verbinden sie später direkt.",624,157)
note("Was sich verändert", "Eine zusammenhängende Küste mit Gebäuden, Innenhöfen und einem felsigen Außenpfad.",624,266)
note("Abwägung", "Viel Freiheit. Rückwege zum Lager müssen kurz bleiben, sonst wird die Suche zur Laufarbeit.",624,375)
legend()

# 6. Option C: section rather than a misleading top-down map.
new("C · Die vertikale Turmruine","LAYOUTVARIANTE", "Ein kompakter Ort mit Innenräumen, Außenbalkonen und mehreren Etagen.")
map_frame("Seitenschnitt · schematisch")
poly([(123,434),(135,398),(192,359),(206,303),(226,247),(251,188),(461,183),(480,234),(457,291),(471,367),(530,406),(552,434)],STONE)
for x,y,w,h in [(183,362,277,49),(215,304,227,40),(239,245,196,38),(278,185,140,34)]:
    rect(x,y,w,h,WHITE,r=3)
    line([(x,y+h),(x+w,y+h)],DARK,5)
# Outside ramps and stairs link the floors, preserving visible spatial relationships.
arrow([(112,411),(272,411),(445,353),(249,344),(249,283),(405,283),(407,219),(322,219),(322,197)],TEAL,3.5)
arrow([(445,351),(498,316),(476,267),(437,257)],GOLD,3,[5,4])
line([(188,414),(162,327),(211,246),(276,216)],BLUE_INK,2,[3,4])
node(112,411,1,"Hof",dy=17)
text("Werkraum",302,378,14,INK,"center")
text("Galerie",324,315,14,INK,"center")
text("Treppen",221,293,11,TEAL,"right")
text("Dachgarten",316,255,14,INK,"center")
text("Signalraum",348,192,14,INK,"center")
for yy,s in [(391,"+0 m"),(326,"+3 m"),(264,"+6 m"),(199,"+9 m")]: text(s,67,yy,12,BLUE_INK)
text("Parcours außen",566,291,12,GOLD,"right")
note("Spielgefühl", "Innenräume und Außenbalkone wechseln sich ab. Dieselben Orte siehst du aus neuen Höhen.",624,157)
note("Was sich verändert", "Der Level wächst nach oben. Balkone zeigen früh erreichbare Ziele und noch verschlossene Räume.",624,266)
note("Abwägung", "Größter technischer Umbau: Für echte Etagen übereinander braucht die Navigation getrennte Ebenen.",624,375)
legend()

# 7. Honest comparison and a concrete recommended direction.
new("Mein Vorschlag: mit der Schlucht beginnen.","ENTSCHEIDUNG", "Variante A verändert die Wegführung deutlich und nutzt viele vorhandene Mechaniken.")
cols=[44,275,493,711]; widths=[231,218,218,205]
headers=["Vergleich","A · Schlucht","B · Hafen","C · Turmruine"]
for x,w,h in zip(cols,widths,headers):
    rect(x,150,w,43,TEAL if x==275 else DARK);text(h,x+15,162,17,WHITE)
rows=[
 ("Wie frei ist der Weg?","Geführt, mit Abzweigen","Bereiche frei wählbar","Etage für Etage"),
 ("Woraus entsteht Varianz?","Raumwechsel & Rundweg","Ungleiche Bezirke","Innen / außen / Höhe"),
 ("Größte Herausforderung","Sichtachsen & Tempo","Kurze Rückwege","Navigation pro Ebene"),
 ("Zusätzlicher Aufwand*","Mittel","Mittel","Hoch"),
]
for j,row in enumerate(rows):
    y=193+j*49
    for x,w,s in zip(cols,widths,row):
        rect(x,y,w,49,PALE if x==275 else (WHITE if j%2==0 else BG))
        para(s,x+15,y+12,w-26,16,INK,32)
rect(44,408,872,69,PALE,r=10)
para("Zielbild für A: eine Expedition von etwa 10–15 Minuten mit zwei Erkundungsabzweigen, einem freiwilligen Parcours und einem kurzen Rückweg.",61,422,836,20,INK,50)
text("* Relative Einschätzung für die aktuelle Demo; keine Zeitschätzung für die Umsetzung.",44,486,10.5,MUTED)

# 8. Mission dependencies with clear physical consequences.
new("Aufgaben sollen den Ort verändern.","SPIELABLAUF FÜR A", "Eine kurze Handlung öffnet jeweils den nächsten Raum. Sammelobjekte ergänzen die Erkundung.")
items=[
 ("Lager","Zelle in einer Nische entdecken."),
 ("Werkstatt","Zelle einsetzen. Das Hoftor öffnet sich."),
 ("Schießhof","Zielschalter treffen. Die Brücke fährt aus."),
 ("Felsgrat","Über Treppen zum Turm. Parcours als Abzweig."),
 ("Signalturm","Signal starten. Rückweg zum Lager öffnen."),
]
arrow_chain(items,44,157,160,144,18)
rect(44,325,416,149,WHITE,r=12)
text("Freiwillige Herausforderungen",62,344,21,GOLD)
para("Der Schießstand bietet Punkte und Rekorde. Ein versteckter Raum und der schwierigere Parcours belohnen neugierige Spieler.",62,382,376,18,INK,78)
rect(480,325,436,149,PALE,r=12)
text("Nach jedem Schritt sichtbar",498,344,21,TEAL)
para("Das Tor öffnet sich, eine Leitung leuchtet oder eine Brücke bewegt sich. Lange Lieferwege und bloßes Abhaken von Stationen entfallen.",498,382,396,18,INK,78)

# 9. Geometry, navigation and cameras must support both existing control modes.
new("Höhe braucht klare Verbindungen.","WEGE & STEUERUNG", "Der Hauptweg bleibt per Klick begehbar. Sprungabkürzungen sind freiwillig.")
rect(44,150,533,185,BLUE,r=12)
# Sideways offset terraces shown in section. The dashed jump route skips a detour.
poly([(63,315),(63,284),(169,284),(169,315)],STONE)
poly([(212,315),(212,253),(332,253),(332,315)],STONE)
poly([(386,315),(386,209),(560,209),(560,315)],STONE)
arrow([(83,282),(144,282),(209,253),(303,253),(384,209),(540,209)],TEAL,4)
arrow([(313,249),(347,200),(394,203)],GOLD,3,[5,4])
text("Lager",80,297,13,INK)
text("Werkstatt",243,275,13,INK)
text("Grat / Turm",429,231,13,INK)
text("Rampe / Treppe",112,228,12,TEAL)
text("Optionaler Sprung",338,169,12,GOLD)
text("Höhen schematisch; Sprungweiten erst im Prototyp festlegen.",62,319,10.5,MUTED)
note("Klicksteuerung", "Alle Pflichtziele bleiben über Laufwege erreichbar. Klicks wählen die sichere Route.",614,157,286)
note("FPV und Parcours", "Checkpoint vor der Sprungfolge; breite Landungen und kurze Rückwege nach einem Sturz.",614,252,286)
rect(44,359,872,117,WHITE,r=12)
text("Wichtig für die Planung",62,374,20,CORAL)
para("Aktuell kennt die Navigation pro Bodenposition nur die höchste Fläche. Für Brücken über begehbaren Räumen und echte Etagen brauchen wir getrennte Ebenen. In der ersten Schlucht-Version verlaufen die Höhenwege seitlich versetzt.",62,406,836,18,INK,70)

# 10. Asset inventory, minimal scope and criteria; no implementation implied.
new("Zuerst den neuen Weg ausprobieren.","VORSCHLAG ZUR UMSETZUNG", "Erst ein begehbares Modell aus einfachen Formen, dann die Ausgestaltung.")
rect(44,150,420,178,WHITE,r=12)
text("Schon vorhanden",62,167,21,TEAL)
para("Figur und Blaster, Zellen, Terminal, Turm, Brücken, Torrahmen, Checkpoints und Sprungsteuerung.",62,203,382,18,INK,75)
text("Weiterverwenden und passend verteilen.",62,294,15,MUTED)
rect(484,150,432,178,WHITE,r=12)
text("Für das neue Gelände ergänzen",502,167,21,CORAL)
para("Unregelmäßige Felswände, Ecken und Rampen, ein Werkstattraum sowie Wegweiser und markante Eingänge.",502,203,396,18,INK,75)
text("Neue Modelle erst nach dem Wegtest bauen.",502,294,15,MUTED)
rect(44,347,872,131,PALE,r=12)
text("Diesen kleinen Entwurf prüfen wir zuerst",62,363,21,TEAL)
para("Vom Start führt der Weg über Werkstatt, Schießhof und Grat zum Turm. Dazu kommen zwei Abzweige und ein kurzer Rückweg. Wir prüfen, ob sich die Bereiche unterschiedlich spielen und beide Steuerungsarten verständlich bleiben.",62,401,830,18,INK,69)
text("Diskussionsgrundlage: Grundform A, 10–15 Minuten als Ziel, Parcours freiwillig. Noch kein Level-Umbau.",44,487,10.5,MUTED)

assert page==10
c.save()
print(OUTPUT)
print(f"{page} slides, {OUTPUT.stat().st_size:,} bytes")
