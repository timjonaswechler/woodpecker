"""Export the planning scene as the playable demo's static scenery and collision data.

blender --background planning/groundplate/groundplate_concept.blend \
  --python-exit-code 1 --python test_app/asset/blender/export_level.py
The saved planning scene is never modified.
"""
import sys
from pathlib import Path
import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from gltf_export import export_model

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT.parent / 'planning/groundplate'))
import layout as plan

OUT = ROOT / 'asset/meshes/environment'
OUT.mkdir(parents=True, exist_ok=True)

# Collections may be hidden for structural/decorative inspection in Blender.
# Always export the complete level; these changes only affect this process.
def include_collections(layer):
    layer.exclude = False
    layer.hide_viewport = False
    for child in layer.children:
        include_collections(child)

for collection in bpy.data.collections:
    collection.hide_viewport = False
    collection.hide_render = False
    collection.hide_select = False
include_collections(bpy.context.view_layer.layer_collection)
bpy.context.view_layer.update()
for obj in bpy.context.scene.objects:
    obj.hide_set(False)
    obj.hide_viewport = False
    obj.hide_render = False
    obj.hide_select = False
bpy.context.view_layer.update()
bpy.ops.object.select_all(action='DESELECT')

def point(x,y,z=0):
    return [round(x-39,5), round(z,5), round(27-y,5)]

def surface(bounds,height,bottom):
    x0,y0,x1,y1=bounds
    return dict(center=point((x0+x1)/2,(y0+y1)/2)[::2],
                half=[(x1-x0)/2,(y1-y0)/2],height=height,bottom=bottom)

surfaces=[surface(bounds,3,2.7) for _,bounds,_ in plan.DECKS]
for x,y0,y1 in plan.STAIRS:
    for i in range(12):
        a=y0+i*(y1-y0)/12
        b=y0+(i+1)*(y1-y0)/12
        surfaces.append(surface((x-1.35,min(a,b),x+1.35,max(a,b)),(i+1)*.25,0))
for x,y,z in plan.PARKOUR:
    surfaces.append(surface((x-1.25,y-1.25,x+1.25,y+1.25),z,0))

dynamic_prefixes=('Optional coin','Optional blueprint','Blueprint drawing',
 'Task reward energy cell','Cell luminous band','Assembly part',
 'Person legs','Person body','Practice target','Practice reward locker')
collidable_prefixes=('Room wall','G3 store wall','G2 upper room wall',
 'Gallery support','Gallery rail','Crate','Maintenance cabinet','Machine base',
 'Archive bookcase','Planter','Hall bench','Start bench','Cell assembly workbench',
 'Workbench controls','Hall map','Mission board','Storage rack lever',
 'Movable storage rack','Generator machine','Generator controls','Signal mast',
 'Archive reward terminal','Archive scan tablet','G2 concealing shelf',
 'Greenhouse reward stand')
obstacles=[]
for obj in list(bpy.data.objects):
    if obj.name.startswith(dynamic_prefixes) or obj.name=='Sphere':
        bpy.data.objects.remove(obj,do_unlink=True)
        continue
    if obj.type=='MESH' and obj.name.startswith(collidable_prefixes) and 'interaction ring' not in obj.name:
        corners=[obj.matrix_world @ Vector(v) for v in obj.bound_box]
        lo=[min(v[i] for v in corners) for i in range(3)]
        hi=[max(v[i] for v in corners) for i in range(3)]
        centre=[(a+b)/2 for a,b in zip(lo,hi)]
        half=[(b-a)/2 for a,b in zip(lo,hi)]
        obstacles.append(dict(name=obj.name,center=point(*centre),half=[half[0],half[2],half[1]]))
    if obj.type=='FONT':
        bpy.context.view_layer.objects.active=obj
        obj.select_set(True)
        bpy.ops.object.convert(target='MESH')
        obj.select_set(False)

data=dict(
    rooms=[dict(id=key,name=name,bounds=bounds) for key,name,bounds,_ in plan.ROOMS],
    surfaces=surfaces,obstacles=obstacles,
    coins=[point(*p) for p in plan.COINS],blueprints=[point(*p) for p in plan.BLUEPRINTS],
    gears=[point(x,y) for x,y,_ in plan.GEARS],
)
# Geometry and gameplay metadata share one level export. Cameras and lighting
# are controlled by the game; mesh buffers remain in the external BIN file.
bpy.ops.object.select_all(action='DESELECT')
for obj in bpy.data.objects:
    if obj.type=='MESH': obj.select_set(True)
export_model(OUT/'courtyard.gltf',use_selection=True,
    export_cameras=False,export_lights=False,export_apply=True,
    extras={"signalstation": {"layout": data}})
print('EXPORTED PLAYABLE SCENERY',len(surfaces),'surfaces',len(obstacles),'colliders')
