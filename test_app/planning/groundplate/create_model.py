"""Render a planning model in Blender. Does not export or change game assets."""
from pathlib import Path
import bpy
from mathutils import Vector
from math import pi

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT/'artifacts/groundplate-plan'
OUT.mkdir(parents=True,exist_ok=True)
bpy.ops.wm.read_factory_settings(use_empty=True)
scene=bpy.context.scene
scene.name='Groundplate level proposal'


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
    if x1>x0: box(name,(x0+x1)/2,y,z,x1-x0,.45,h,wall)


def wall_y(name,x,y0,y1,z=0,h=2.2):
    if y1>y0: box(name,x,(y0+y1)/2,z,.45,y1-y0,h,wall)


def deck(name,x0,y0,x1,y1,m):
    box(name,(x0+x1)/2,(y0+y1)/2,2.7,x1-x0,y1-y0,.3,m)


def rails_y(name,x,y0,y1):
    box(name+' top rail',x,(y0+y1)/2,3.7,.12,y1-y0,.12,dark)
    n=max(1,int((y1-y0)/2.5))
    for i in range(n+1): box(name+' post',x,y0+(y1-y0)*i/n,3,.12,.12,.75,dark)


def rails_x(name,y,x0,x1):
    box(name+' top rail',(x0+x1)/2,y,3.7,x1-x0,.12,.12,dark)
    n=max(1,int((x1-x0)/2.5))
    for i in range(n+1): box(name+' post',x0+(x1-x0)*i/n,y,3,.12,.12,.75,dark)


def stairs(name,x,y0,y1,m):
    n=12;step=(y1-y0)/n
    for i in range(n):
        box(name+f' {i+1}',x,y0+(i+.5)*step,0,2.7,abs(step),(i+1)*.25,m,.02)
    for side in [-1,1]:
        # End posts make the stair head identifiable without hiding the steps.
        box(name+' top post',x+side*1.35,y1-.15,3,.13,.13,.8,dark)


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

# One slab supports every courtyard and all bridges.
box('ONE continuous ground plate',22,16,-.8,46,34,.8,base,.15)
for name,x0,y0,x1,y1,m in [('A',1,1,19,14,A),('B',1,17,19,31,B),('C',25,17,43,31,C),('D',25,1,43,14,D)]:
    box(name+' floor',(x0+x1)/2,(y0+y1)/2,0,x1-x0,y1-y0,.05,m)

# Boundaries with deliberate gates. Front walls are cut down for the drawing.
wall_y('A west',.7,.7,14.3)
wall_x('A front cutaway',.7,19.3,.7,h=.45)
wall_x('A north',.7,9.5,14.3);wall_x('A north',12.8,19.3,14.3)
wall_y('A east',19.3,.7,6.4);wall_y('A east',19.3,9.0,14.3)
wall_y('B west',.7,16.7,31.3);wall_x('B rear',.7,19.3,31.3)
wall_x('B entrance',.7,9.5,16.7);wall_x('B entrance',12.8,19.3,16.7)
wall_y('B east',19.3,16.7,31.3)
wall_y('C east',43.3,16.7,31.3);wall_x('C rear',24.7,43.3,31.3)
wall_y('C west',24.7,16.7,18.8);wall_y('C west',24.7,21.5,31.3)
wall_x('C south',24.7,34.0,16.7);wall_x('C south',37.0,43.3,16.7)
wall_x('D north',24.7,34,14.3);wall_x('D north',37,43.3,14.3)
wall_y('D east',43.3,.7,14.3);wall_y('D west',24.7,.7,14.3)
wall_x('D front cutaway',24.7,43.3,.7,h=.45)
# Enclose the two main doorways so the gap between courts is not a bypass.
for x in [9.5,12.8,34,37]: wall_y('Enclosed courtyard doorway',x,14.3,16.7)
wall_x('G1 passage end',19.3,24.7,21.5,h=2.2)

# Workshop enclosure, with an open entrance and a visible ceiling edge.
wall_y('Workshop room west',3,19,25,h=3)
wall_x('Workshop room back',3,9,25,h=3)
wall_x('Workshop facade',3,4.8,19,h=3);wall_x('Workshop facade',7.4,9,19,h=3)
wall_y('Workshop room east',9,19,25,h=3)
box('Workshop sign',6,19,2.65,6,.2,.45,teal)
box('Workshop console',5.4,23.3,.05,1.2,.8,1.0,dark)
box('Workshop console screen',5.4,22.85,.85,.9,.1,.45,teal)

# Main route above the ground plate, 3m above the courtyards.
stairs('B stairs',12,19,26,teal)
deck('B gallery',10,26,20.0,30.2,teal)
deck('BRIDGE 1 above the ground corridor',20,27,30,30,teal)
deck('C gallery',30,27,41.5,30.2,teal)
stairs('C stairs',37,20,27,teal)
rails_x('B rear gallery',30.15,10,20)
rails_x('Bridge 1 north',30,20,30);rails_x('Bridge 1 south',27,20,30)
rails_x('C rear gallery',30.15,30,41.5)
rails_x('C front gallery',27,30,35.3)
rails_x('C front gallery',27,38.6,41.5)
for x,y in [(11,29),(18,29),(27,28.5),(32,29),(41,29)]: box('Gallery support',x,y,0,.45,.45,2.7,dark)

# Hidden ground route G1. A small storeroom leads into a passage below Bridge 2.
box('G1 storeroom floor',21.7,7.7,.05,4.2,5,.035,purple)
box('G1 passage floor',22.1,14.45,.05,3.1,8.5,.035,purple)
box('G1 exit to C',23.4,20,.05,2.8,2.6,.035,purple)
wall_x('G1 storeroom back',19.4,24.45,5.1,h=2.2)
# Crates partially screen the doorway without completely covering the clue.
for x,y in [(17.6,6.0),(18,10),(20.3,5.9)]: box('G1 crates',x,y,.1,1.1,1.1,1.2,boxmat)

# Hidden upper room G2, with a second bridge toward the goal balcony.
deck('G2 side room floor',16,21.8,20.0,26,gold)
wall_y('G2 room west',16,21.8,26,z=3,h=1.8)
wall_x('G2 room south',16,17.3,21.8,z=3,h=1.8)
wall_x('G2 room south',19.0,20.0,21.8,z=3,h=1.8)
box('G2 shelf',17.1,25.3,3,1.7,.6,1.3,boxmat)
deck('BRIDGE 2 secret gallery connector',20,21.8,20.7,24.5,gold)
deck('BRIDGE 2 long part above G1',20.7,9.0,23.7,24.5,gold)
deck('BRIDGE 2 into D balcony',23.7,9,30,12,gold)
rails_y('Bridge 2 west',20.7,9,21.3);rails_y('Bridge 2 east',23.7,12,24.5)
rails_x('Bridge 2 south',9,20.7,32);rails_x('Bridge 2 north',12,23.7,30)
for x,y in [(20.7,13),(23.7,18),(23.7,22),(28,10.5)]: box('Bridge 2 support',x,y,0,.25,.25,2.7,dark)
deck('D goal balcony',30,9,42,12.5,teal)
stairs('D stairs',35,3,9,teal)
rails_x('D rear balcony',12.5,30,42)
rails_y('D east balcony',42,9,12.5)
for x,y in [(31,11),(41,11)]: box('D balcony support',x,y,0,.45,.45,2.7,dark)
# A small landmark makes the final courtyard identifiable.
box('Signal base',40,11,3,1.8,1.5,.3,dark)
for x in [39.4,40.6]: box('Signal mast',x,11,3.3,.18,.18,3.0,dark)
box('Signal head',40,11,6.1,2,.65,.65,teal)
box('Signal light',40,10.6,6.3,.9,.1,.18,gold)

# Local courtyard props show different uses at a glance.
for x,y,h in [(30,22,1.4),(29.2,23.8,.8),(41,19,1.2),(31,18.6,1.0)]: box('C supply crate',x,y,.05,1.5,1.5,h,boxmat)
for x in [27.2,29.3,31.4]:
    box('Target support',x,25.6,.05,.1,.15,1.8,dark)
    bpy.ops.mesh.primitive_cylinder_add(vertices=24,radius=.6,depth=.12,location=(x,25.55,1.75),rotation=(pi/2,0,0))
    bpy.context.object.data.materials.append(gold)
for x,y in [(5,5),(5,8)]: box('A low bench',x,y,.05,3.1,.8,.5,boxmat)
disk('Start',10,6,.06,1.5,teal)
disk('Checkpoint B stairs',12,18.0,.06,.6,gold)
disk('Checkpoint D stairs',35,2.0,.06,.6,gold)
# Person under the bridge gives a scale and makes the two levels visible.
box('Person legs',22,16,.1,.45,.4,.65,dark)
box('Person body',22,16,.75,.65,.45,.65,gold)
bpy.ops.mesh.primitive_uv_sphere_add(segments=12,ring_count=8,radius=.28,location=(22,16,1.67));bpy.context.object.data.materials.append(white)

# Floor labels are part of the proposal illustration.
label('A',7.8,10.5,.075,2.7)
label('B',5,28.4,.075,2.7)
label('C',39.5,23.6,.075,2.7)
label('D',29.2,4.6,.075,2.7)
label('G1',21.8,7.7,.105,.8,white)
label('G2',18,23.5,3.03,.85,dark)
label('1',24.7,28.5,3.03,1.2,white)
label('2',22.2,18.5,3.03,1.2,dark)

world=bpy.data.worlds.new('Warm background');scene.world=world;world.use_nodes=True
world.node_tree.nodes['Background'].inputs[0].default_value=(.77,.79,.75,1)
world.node_tree.nodes['Background'].inputs[1].default_value=.65
bpy.ops.object.light_add(type='AREA',location=(8,-14,48));bpy.context.object.data.energy=19000;bpy.context.object.data.shape='DISK';bpy.context.object.data.size=28
bpy.context.object.rotation_euler=(Vector((22,16,0))-bpy.context.object.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.light_add(type='AREA',location=(42,32,35));bpy.context.object.data.energy=12000;bpy.context.object.data.size=25
bpy.context.object.rotation_euler=(Vector((22,16,0))-bpy.context.object.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(63,-60,82));camera=bpy.context.object;scene.camera=camera
camera.rotation_euler=(Vector((22,16,1))-camera.location).to_track_quat('-Z','Y').to_euler()
camera.data.type='ORTHO';camera.data.ortho_scale=72
scene.render.engine='CYCLES';scene.cycles.samples=32;scene.cycles.use_denoising=True
scene.render.resolution_x=1600;scene.render.resolution_y=1100;scene.render.resolution_percentage=100
scene.render.image_settings.file_format='PNG';scene.render.film_transparent=False
scene.view_settings.view_transform='AgX'
scene.render.filepath=str(OUT/'raeumlicher_entwurf.png')
bpy.ops.wm.save_as_mainfile(filepath=str(Path(__file__).with_name('groundplate_concept.blend')))
bpy.ops.render.render(write_still=True)
# Second view makes passing below the elevated route easier to understand.
camera.location=(46,-14,21)
camera.rotation_euler=(Vector((22,15,1.6))-camera.location).to_track_quat('-Z','Y').to_euler()
camera.data.ortho_scale=26
scene.render.resolution_x=1400;scene.render.resolution_y=900
scene.render.filepath=str(OUT/'unter_der_bruecke.png')
bpy.ops.render.render(write_still=True)
print('PLANNING MODEL RENDERED',OUT)
