"""Render a planning model in Blender. Does not export or change game assets."""
from pathlib import Path
import bpy
from mathutils import Vector
from math import pi
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent))
import layout as plan
from layout import station_point
import passages
from scene_collections import organize_scene

ROOT = Path(__file__).resolve().parents[2] / 'test_app'
sys.path.insert(0, str(ROOT / 'asset/blender'))
from level_modules import build_stair

OUT = ROOT/'artifacts/groundplate-plan'
OUT.mkdir(parents=True,exist_ok=True)
bpy.ops.wm.read_factory_settings(use_empty=True)
scene=bpy.context.scene
scene.name='Eight-room exploration proposal'
bpy.context.preferences.filepaths.save_version=0


def mat(name,hex):
    srgb=tuple(int(hex[i:i+2],16)/255 for i in (0,2,4))
    rgb=tuple(v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in srgb)
    m=bpy.data.materials.new(name);m.diffuse_color=(*rgb,1)
    m.use_nodes=True;p=m.node_tree.nodes.get('Principled BSDF')
    p.inputs['Base Color'].default_value=(*rgb,1);p.inputs['Roughness'].default_value=.85
    return m

base=mat('Continuous base','9DAAA6')
wall=mat('Courtyard walls','D5D4C6')
A=mat('A start court','E9D7A8');B=mat('B workshop','BECBB3')
C=mat('C warehouse','DFB9A7');D=mat('D goal court','B4CED2')
teal=mat('Upper main route','277F78')
gold=mat('Upper hidden connection','DC9B39')
purple=mat('Ground hidden connection','8864AB')
dark=mat('Frames and railings','314B4F')
white=mat('Labels','F7F6ED');boxmat=mat('Storage crates','937D5C')
route=mat('Ground main route','EADFC4')


def box(name,x,y,z,w,d,h,m,bevel=.04):
    bpy.ops.mesh.primitive_cube_add(size=1,location=(x,y,z+h/2))
    obj=bpy.context.object;obj.name=name;obj.dimensions=(w,d,h)
    bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
    obj.data.materials.append(m)
    if bevel:
        mod=obj.modifiers.new('Edges','BEVEL');mod.width=bevel;mod.segments=1
        obj.modifiers.new('Normals','WEIGHTED_NORMAL')
    return obj


def wall_x(name,x0,x1,y,z=0,h=2.2):
    if x1>x0: return box(name,(x0+x1)/2,y,z,x1-x0,.45,h,wall)


def wall_y(name,x,y0,y1,z=0,h=2.2):
    if y1>y0: return box(name,x,(y0+y1)/2,z,.45,y1-y0,h,wall)


def deck(name,x0,y0,x1,y1,m):
    box(name,(x0+x1)/2,(y0+y1)/2,2.7,x1-x0,y1-y0,.3,m)


def rails_y(name,x,y0,y1):
    parts = [box(name+' top rail',x,(y0+y1)/2,3.7,.12,y1-y0,.12,dark)]
    n=max(1,int((y1-y0)/2.5))
    parts.extend(box(name+' post',x,y0+(y1-y0)*i/n,3,.12,.12,.75,dark) for i in range(n+1))
    return parts


def rails_x(name,y,x0,x1):
    parts = [box(name+' top rail',(x0+x1)/2,y,3.7,x1-x0,.12,.12,dark)]
    n=max(1,int((x1-x0)/2.5))
    parts.extend(box(name+' post',x0+(x1-x0)*i/n,y,3,.12,.12,.75,dark) for i in range(n+1))
    return parts


def label(txt,x,y,z,size=1.3,m=dark):
    curve=bpy.data.curves.new(txt,'FONT');curve.body=txt;curve.size=size
    curve.align_x='CENTER';curve.align_y='CENTER';curve.extrude=.004
    obj=bpy.data.objects.new(txt,curve);scene.collection.objects.link(obj)
    obj.location=(x,y,z);obj.data.materials.append(m)
    return obj


def disk(name,x,y,z,r,m):
    bpy.ops.mesh.primitive_cylinder_add(vertices=24,radius=r,depth=.04,location=(x,y,z+.02))
    obj=bpy.context.object;obj.name=name;obj.data.materials.append(m)
    return obj

def mark_cut(obj, secret, portal=None, solid=True):
    obj['signalstation_secret_cut'] = secret
    obj['signalstation_portal_cut'] = -1 if portal is None else secret * 2 + portal
    obj['signalstation_solid'] = solid
    return obj


def secret_wall_segment(segment, owner, z=0, name='Secret wall'):
    # Leave a visible plinth in overview; the full wall stays physical.
    base = wall_segment(*segment, z=z, h=.5, name=name+' base')
    base['signalstation_solid'] = True
    mark_cut(wall_segment(*segment, z=z+.5, h=2.2, name=name+' upper'), owner)


# Eight main rooms on a larger continuous slab. Colors identify their uses.
room_mats={key:mat(name,color) for key,name,_,color in plan.ROOMS}
green=mat('Plants','477B4D');leaf=mat('Leaves','81AA64')
blue=mat('Blueprints','428FC1');orange=mat('Interaction handles','D77438')


def floor(name,bounds,z,material,thickness=.035):
    x0,y0,x1,y1=bounds
    return box(name,(x0+x1)/2,(y0+y1)/2,z-thickness,x1-x0,y1-y0,thickness,material)


def wall_segment(axis,coord,lo,hi,z=0,h=2.5,name='Room wall'):
    if axis=='h': return wall_x(name,lo,hi,coord,z,h)
    return wall_y(name,coord,lo,hi,z,h)


def upright_disk(name,x,y,z,r,material):
    bpy.ops.mesh.primitive_cylinder_add(vertices=24,radius=r,depth=.12,
        location=(x,y,z),rotation=(pi/2,0,0))
    obj=bpy.context.object;obj.name=name;obj.data.materials.append(material)


def console(name,x,y,z=0):
    box(name,x,y,z,1.3,.9,1.1,dark)
    box(name+' screen',x,y-.48,z+.65,1,.06,.4,teal)
    disk(name+' interaction ring',x,y-1,z+.015,.55,orange)


def crate(x,y,z=0,w=1.5):
    box('Crate',x,y,z,w,w,w,boxmat)
    box('Crate strap',x,y-w/2-.01,z+.1,.13,.05,w-.2,dark)


floor('ONE continuous ground plate',plan.PLATE,0,base,.8)
for key,name,bounds,_ in plan.ROOMS:
    x0,y0,x1,y1=bounds
    floor(key+' '+name,(x0+.22,y0+.22,x1-.22,y1-.22),.045,room_mats[key])
floor('G1 maintenance passage',(18.22,.22,49.78,9.78),.05,purple)
main_walls = plan.walls()
for segment in main_walls:
    axis, coord, lo, hi = segment
    owner = next((i for i, p in enumerate(passages.PASSAGES)
                  if p['floor'] == 0 and passages.on_boundary(segment, p)), None)
    if owner is not None:
        secret_wall_segment(segment, owner, name='Room wall')
    else:
        wall_segment(*segment, h=.5 if axis=='h' and coord==0 else 2.5)

for name,bounds,kind in plan.DECKS:
    if name == 'F gallery':
        obj = mark_cut(floor('F gallery over store',(4,40,10,44),3,teal,.28), 2, solid=False)
        obj['signalstation_walk_bounds'] = [4,40,10,44]
        floor('F gallery aisle',(10,40,22,44),3,teal,.28)
    else:
        floor(name,bounds,3,gold if kind=='secret' else teal,.28)
for module in plan.STAIR_MODULES:
    obj = build_stair(module, teal, dark)
    obj.location = module.placement.origin
    obj.rotation_euler.z = module.placement.quarter_turns * pi / 2
for axis,coord,lo,hi in plan.gallery_rails():
    parts = (rails_x('Gallery rail',coord,lo,hi) if axis=='h'
             else rails_y('Gallery rail',coord,lo,hi))
    if ((axis=='h' and coord in (40,44) and 4<=lo<hi<=10)
            or (axis=='v' and coord==4 and 40<=lo<hi<=44)):
        for obj in parts: mark_cut(obj, 2)
for x,y in [(10,35),(17,35),(25,33),(30,36),(46,36),(51,33),
            (57,35),(74,35),(55,39),(52,46),(61,42),
            (5,41),(21,41),(26,46),(38,46),(48,46),(65,14),(75,14)]:
    box('Gallery support',x,y,0,.4,.4,2.72,dark)

# Physical roofs and walls stay collidable even when the overview cuts them away.
for i, secret in enumerate(passages.PASSAGES):
    if secret['id'] == 'G3': floor('Store floor',(3,38,10,43),.08,purple)
    for segment in list(passages.shells(secret)) + secret['partitions']:
        axis, coord, lo, hi = segment
        shared = secret['floor'] == 0 and all(any(
            a == axis and c == coord and l <= v+.5 <= h for a,c,l,h in main_walls)
            for v in range(int(lo), int(hi)))
        if shared: continue
        for start in range(int(lo), int(hi), 3):
            secret_wall_segment((axis,coord,start,min(hi,start+3)), i, z=secret['floor'])
    for r, (bounds, entry) in enumerate(passages.roofs(secret)):
        pieces = [bounds]
        if secret['floor'] == 0:
            for _, upper_bounds, kind in plan.DECKS:
                if kind == 'main':
                    pieces = [part for b in pieces for part in passages.subtract_rect(b, upper_bounds)]
        for part, bounds in enumerate(pieces):
            mark_cut(floor(f'Secret cover {i}/{r}/{part}', bounds,
                           secret['floor']+3,wall,.3), i, entry)
    for j, portal in enumerate(secret['portals']):
        axis, coord, lo, hi, _, _, _ = portal
        obj = wall_segment(axis,coord,lo,hi,z=secret['floor']+2.5,h=.2,name='Secret frame')
        mark_cut(obj, i)
        x,y = (coord,(lo+hi)/2) if axis == 'v' else ((lo+hi)/2,coord)
        obj = box(f'Portal preview {i}/{j}',x,y,secret['floor'],
                  .7 if axis=='v' else hi-lo, hi-lo if axis=='v' else .7,2.5,
                  boxmat if i < 2 else wall)
        obj['signalstation_dynamic_portal'] = True

# A: the visible starting point and the mission board.
disk('Start',7,5,.06,1.4,teal)
console('Mission board',*station_point('board'))
box('Start bench',4,8,.05,4,.8,.55,boxmat)
for x,y in [(16,2.3),(16.3,7.1)]: crate(x,y)

# B: large work tables, a two-part assembly task and a hidden rear door.
output_x, output_y, _ = plan.WORKSHOP.point("output")
craft_x, craft_y, _ = plan.WORKSHOP.point("craft")
box('Cell assembly workbench',output_x,output_y,0,2,1.5,1,boxmat)
console('Workbench controls',craft_x,craft_y)
for x,y in [(1.75,24),(1.75,27)]: box('Machine base',x,y,0,2.6,2,1.1,dark)
for x,y in [(3.5,36),(9.7,36),(15.5,24)]: crate(x,y)

# C: a broad central hall with an optional jump route and stairs beside it.
for x,y,z in plan.PARKOUR: box('Optional parkour block',x,y,0,2.5,2.5,z,boxmat)
disk('Hall checkpoint',*station_point('checkpoint','approach')[:2],.06,.9,teal)
console('Hall map',21,34)
for x,y in [(23,30),(46,19)]: box('Hall bench',x,y,0,3.5,1,.65,boxmat)
box('Person legs',37,33,.05,.45,.4,.65,dark)
box('Person body',37,33,.7,.65,.4,.65,orange)
bpy.ops.mesh.primitive_uv_sphere_add(segments=12,ring_count=8,radius=.28,location=(37,33,1.62))
bpy.context.object.data.materials.append(white)

# D: movable storage, a switch and an extra reward behind the crates.
for x,y in [(58,23),(61,23),(74,24),(75,25.5),(68,35),(62,28)]: crate(x,y)
px, py, pz = station_point('part1')
for x in [px-2,px+2]: box('Storage rack rail',x,py,pz+.05,.12,6,.07,dark)
box('Movable storage rack',px,py+1,pz,3,1.4,1.3,boxmat)
console('Storage rack lever',*station_point('rack'))
rx, ry, _ = station_point('rack')
box('Lever handle',rx,ry,1.1,.15,.15,.8,orange)

# E: two valves feed planted beds; the task reward stands by the manifold.
for x,y in [(66,46),(72,46),(67,51)]:
    box('Planter',x,y,0,3.8,2.1,.65,boxmat)
    for dx in [-1.2,0,1.2]:
        box('Plant stem',x+dx,y,.65,.12,.12,.8,green)
        bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1,radius=.65,location=(x+dx,y,1.4))
        bpy.context.object.data.materials.append(leaf)
for x,y,_ in [station_point('valve0'), station_point('valve1')]:
    box('Valve pipe',x,y,0,.25,.25,1.2,dark)
    upright_disk('Valve handwheel',x,y-.18,1.2,.42,orange)
vx0, vy0, _ = station_point('valve0')
vx1, vy1, _ = station_point('valve1')
box('Irrigation main pipe',(vx0+vx1)/2,vy0,.15,abs(vx1-vx0),.2,.2,dark)
box('Irrigation return pipe',vx1,(vy0+vy1)/2,.15,.2,abs(vy1-vy0),.2,dark)
box('Greenhouse reward stand',*station_point('cell1'),1.8,1.3,1,boxmat)

# F: short aisles, three scan tablets and a terminal.
for x,y in [(3,45),(11.5,51),(19,53)]:
    box('Archive bookcase',x,y,0,3,.8,1.7,boxmat)
    for dx in [-1,-.5,0,.5,1]: box('Archive folder',x+dx,y-.43,.7,.28,.2,.7,blue)
for key in ('scan0','scan1','scan2'): console('Archive scan tablet',*station_point(key))
console('Archive reward terminal',*station_point('cell2'))

# G: shooting is optional and awards a blueprint instead of a main cell.
for x in [29,35,41]:
    box('Practice lane',x,45,.05,.09,12,.015,white)
    box('Practice target post',x,51,0,.15,.2,1.8,dark)
    upright_disk('Practice target',x,51,1.85,.8,gold)
    upright_disk('Practice target centre',x,50.9,1.85,.24,orange)
console('Practice start',28,42)
box('Practice reward locker',46,51,0,2,1.5,.3,dark)

# H: three sockets power the signal; the other rooms stay accessible.
gx, gy, gz = station_point('generator')
box('Generator machine',gx-3,gy,gz,3.8,2.5,1.4,dark)
for dx in [-1.15,0,1.15]: box('Empty cell socket',gx-3+dx,gy-.3,gz+1.4,.65,.9,.12,white)
console('Generator controls',gx,gy,gz)
sx, sy, sz = station_point('signal')
for x in [sx-1,sx+1]: box('Signal mast',x,sy,sz,.2,.2,3.6,dark)
box('Signal head',sx,sy,sz+3.3,3,.8,.7,teal)
box('Signal lamp',sx,sy-.45,sz+3.5,1.4,.1,.22,gold)

# G1 contains a reward alcove and clues at both entrances.
for x in [24,43]: box('Maintenance cabinet',x,8,0,3,1.4,1.6,dark)
for x,y in [(21,2),(39,2.2),(47,7.6)]: crate(x,y)
# Quiet architectural hints rather than floating secret labels or gold beacons.
box('Maintenance cable',16,4.5,.06,4,.08,.05,dark)
box('Maintenance cable return',52,4.5,.06,4,.08,.05,dark)

for x,y,z in plan.COINS: upright_disk('Optional coin',x,y,z+.8,.35,gold)
for x,y,z in plan.BLUEPRINTS:
    box('Optional blueprint',x,y,z+.4,.95,.7,.08,blue)
    box('Blueprint drawing',x,y,z+.485,.65,.05,.015,white)
for x,y,z in plan.GEARS: disk('Assembly part',x,y,z+1.3,.48,orange)
for x,y,z in plan.CELLS:
    box('Task reward energy cell',x,y,z,.65,.65,1,teal)
    box('Cell luminous band',x,y-.34,z+.35,.5,.06,.2,gold)
for value,x,y in plan.LABELS: label(value,x,y,.085,3.2)
label('START',7,3,.09,.75)

world=bpy.data.worlds.new('Warm background');scene.world=world;world.use_nodes=True
world.node_tree.nodes['Background'].inputs[0].default_value=(.77,.79,.75,1)
world.node_tree.nodes['Background'].inputs[1].default_value=.65
bpy.ops.object.light_add(type='AREA',location=(18,-25,78));bpy.context.object.data.energy=40000;bpy.context.object.data.shape='DISK';bpy.context.object.data.size=45
bpy.context.object.rotation_euler=(Vector((39,27,0))-bpy.context.object.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.light_add(type='AREA',location=(70,50,60));bpy.context.object.data.energy=28000;bpy.context.object.data.size=35
bpy.context.object.rotation_euler=(Vector((39,27,0))-bpy.context.object.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(114,-98,137));camera=bpy.context.object;scene.camera=camera
camera.rotation_euler=(Vector((39,27,1))-camera.location).to_track_quat('-Z','Y').to_euler()
camera.data.type='ORTHO';camera.data.ortho_scale=118
scene.render.engine='CYCLES';scene.cycles.samples=40;scene.cycles.use_denoising=True
scene.render.resolution_x=2000;scene.render.resolution_y=1350;scene.render.resolution_percentage=100
scene.render.image_settings.file_format='PNG';scene.render.film_transparent=False
scene.view_settings.view_transform='AgX'
scene.render.filepath=str(OUT/'raeumlicher_entwurf.png')
organize_scene(scene)
bpy.ops.wm.save_as_mainfile(filepath=str(Path(__file__).with_name('groundplate_concept.blend')))
if '--no-render' not in sys.argv:
    bpy.ops.render.render(write_still=True)
# Second view makes passing below the elevated route easier to understand.
camera.location=(78,-18,43)
camera.rotation_euler=(Vector((35,28,1))-camera.location).to_track_quat('-Z','Y').to_euler()
camera.data.ortho_scale=57
scene.render.resolution_x=1600;scene.render.resolution_y=1100
scene.render.filepath=str(OUT/'zentrale_halle.png')
if '--no-render' not in sys.argv:
    bpy.ops.render.render(write_still=True)
print('PLANNING MODEL SAVED',OUT)
