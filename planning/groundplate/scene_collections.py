"""Group the level for independent structural and decorative views in Blender."""
import bpy
import layout as plan


ESSENTIAL = '01 Notwendig'
DECORATION = '02 Deko'
PLANNING = '03 Planung'
RENDER = '04 Kamera und Licht'

# Keep complete functional assemblies together, including their small details.
# Furniture in Deko can still have collision in the game; this is a viewing aid.
GROUPS = {
    (ESSENTIAL, 'Baukoerper und Wege'): {
        'ONE continuous ground plate', 'G1 maintenance passage', 'G3 store floor',
        'Room wall', 'G3 store wall', 'G2 upper room wall', 'Gallery support',
        'Gallery rail top rail', 'Gallery rail post', 'Stair top post',
        'Optional parkour block', 'G2 concealing shelf',
        *(f'{key} {name}' for key, name, _, _ in plan.ROOMS),
        *(name for name, _, _ in plan.DECKS),
        *(f'Stair {i}' for i in range(1, 13)),
    },
    (ESSENTIAL, 'Stationen und Aufgaben'): {
        'Start', 'Hall checkpoint', 'Mission board', 'Cell assembly workbench',
        'Workbench controls', 'Hall map', 'Storage rack rail',
        'Movable storage rack', 'Storage rack lever', 'Lever handle',
        'Valve pipe', 'Valve handwheel', 'Irrigation main pipe',
        'Irrigation return pipe', 'Greenhouse reward stand',
        'Archive scan tablet', 'Archive reward terminal', 'Practice lane',
        'Practice target', 'Practice target centre', 'Practice target post',
        'Practice reward locker', 'Generator machine', 'Empty cell socket',
        'Generator controls', 'Signal mast', 'Signal head', 'Signal lamp',
        'Secret entrance light',
    },
    (ESSENTIAL, 'Sammelobjekte'): {
        'Optional coin', 'Optional blueprint', 'Blueprint drawing',
        'Assembly part', 'Task reward energy cell', 'Cell luminous band',
    },
    (DECORATION, 'Einrichtung'): {
        'Start bench', 'Hall bench', 'Crate', 'Crate strap', 'Machine base',
        'Archive bookcase', 'Archive folder', 'Maintenance cabinet',
    },
    (DECORATION, 'Pflanzen'): {'Planter', 'Plant stem', 'Icosphere'},
    (PLANNING, 'Platzhalter'): {'Person legs', 'Person body', 'Sphere', 'Practice start'},
}


def object_group(obj):
    if obj.type in {'CAMERA', 'LIGHT'}:
        return (RENDER,)
    if obj.type == 'FONT':
        return (PLANNING, 'Beschriftungen')
    # Blender appends .001 etc. to repeated names. Console details belong to
    # their station, including screens and interaction rings.
    name, separator, suffix = obj.name.rpartition('.')
    name = name if separator and suffix.isdigit() else obj.name
    for detail in (' screen', ' interaction ring'):
        name = name.removesuffix(detail)
    for path, names in GROUPS.items():
        if name in names:
            return path
    raise ValueError(f'Assign the new level object to a group: {obj.name}')


def organize_scene(scene):
    """Move each object into one collection without changing its geometry."""
    # Resolve all names before making changes, so an unknown object cannot
    # leave a scene partially organized or silently disappear into Deko.
    assignments = [(obj, object_group(obj)) for obj in scene.objects]
    colors = {ESSENTIAL: 'COLOR_04', DECORATION: 'COLOR_05',
              PLANNING: 'COLOR_03', RENDER: 'COLOR_02'}
    collections = {}
    for obj, path in assignments:
        parent = scene.collection
        for index, name in enumerate(path):
            key = path[:index + 1]
            if key not in collections:
                collection = parent.children.get(name)
                if collection is None:
                    collection = bpy.data.collections.new(name)
                    parent.children.link(collection)
                collection.color_tag = colors[path[0]]
                collections[key] = collection
            parent = collections[key]
        if obj.name not in parent.objects:
            parent.objects.link(obj)
        for previous in tuple(obj.users_collection):
            if previous != parent:
                previous.objects.unlink(obj)

    # Show both viewport and render switches in the Outliner.
    for screen in bpy.data.screens:
        for area in screen.areas:
            if area.type == 'OUTLINER':
                area.spaces.active.show_restrict_column_hide = True
                area.spaces.active.show_restrict_column_render = True
    return {name: len(collections[(name,)].all_objects) for name in colors}
