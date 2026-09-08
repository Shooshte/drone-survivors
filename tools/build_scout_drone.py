"""Build the scout's editable Blender source, game GLB and studio preview.

Run from any directory:
  blender --background --factory-startup --python tools/build_scout_drone.py
All design coordinates use Bevy axes: +Y up, -Z forward, in arena world units.
"""
from pathlib import Path
from math import cos, sin, pi
import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[1]
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)
for material in list(bpy.data.materials):
    bpy.data.materials.remove(material)


def point(p):
    """Bevy to Blender, reversed by glTF's Y-up export."""
    return Vector((p[0], -p[2], p[1]))


def material(name, color, metal=0.0, roughness=0.4, emission=0.0):
    mat = bpy.data.materials.new(name)
    mat.diffuse_color = (*color, 1)
    mat.use_nodes = True
    shader = mat.node_tree.nodes.get('Principled BSDF')
    shader.inputs['Base Color'].default_value = (*color, 1)
    shader.inputs['Metallic'].default_value = metal
    shader.inputs['Roughness'].default_value = roughness
    if emission:
        shader.inputs['Emission Color'].default_value = (*color, 1)
        shader.inputs['Emission Strength'].default_value = emission
    return mat


silver = material('01 • ceramic titanium', (0.57, 0.67, 0.73), 0.65, 0.3)
graphite = material('02 • graphite chassis', (0.025, 0.042, 0.055), 0.55, 0.36)
steel = material('03 • machined edges', (0.16, 0.23, 0.28), 0.8, 0.28)
cyan = material('04 • cyan levitation / telemetry', (0.015, 0.62, 0.82), 0.25, 0.25, 2.5)
glass = material('05 • optical sapphire', (0.006, 0.085, 0.14), 0.7, 0.12)
marking = material('06 • amber identification', (0.95, 0.38, 0.07), 0.35, 0.35)
parts = []


def finish(obj, name, mat, bevel=0.0, smooth=False):
    obj.name = name
    obj.data.materials.append(mat)
    if bevel:
        modifier = obj.modifiers.new('Manufactured edge radii', 'BEVEL')
        modifier.width = bevel
        modifier.segments = 3
    if smooth:
        for poly in obj.data.polygons:
            poly.use_smooth = True
    elif bevel:
        modifier = obj.modifiers.new('Weighted surface normals', 'WEIGHTED_NORMAL')
        modifier.keep_sharp = True
    parts.append(obj)
    return obj


def box(name, position, size, mat, bevel=0.12):
    bpy.ops.mesh.primitive_cube_add(size=1, location=point(position))
    obj = bpy.context.object
    obj.scale = (size[0], size[2], size[1])
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    return finish(obj, name, mat, bevel)


def cylinder(name, position, radius, depth, mat, axis=(0, 1, 0), vertices=32):
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth,
                                      location=point(position))
    obj = bpy.context.object
    obj.rotation_mode = 'QUATERNION'
    obj.rotation_quaternion = point(axis).to_track_quat('Z', 'Y')
    return finish(obj, name, mat, min(0.07, depth / 5), True)


def ring(name, position, radius, tube, mat, axis=(0, 1, 0)):
    bpy.ops.mesh.primitive_torus_add(major_segments=48, minor_segments=8,
                                    location=point(position), major_radius=radius,
                                    minor_radius=tube)
    obj = bpy.context.object
    obj.rotation_mode = 'QUATERNION'
    obj.rotation_quaternion = point(axis).to_track_quat('Z', 'Y')
    return finish(obj, name, mat, smooth=True)


def rail(name, a, b, radius, mat):
    a, b = Vector(a), Vector(b)
    return cylinder(name, (a + b) / 2, radius, (b - a).length, mat, b - a, 12)


def loft(name, sections, mat, x_offset=0):
    """Rounded superellipse cross-sections: z, center y, half width, half height."""
    count = 24
    vertices = []
    for z, y, width, height in sections:
        for i in range(count):
            angle = i * 2 * pi / count
            c, s = cos(angle), sin(angle)
            vertices.append(point((x_offset + width * abs(c)**0.8 * (1 if c >= 0 else -1),
                                   y + height * abs(s)**0.8 * (1 if s >= 0 else -1), z)))
    faces = []
    for row in range(len(sections) - 1):
        for i in range(count):
            a, b = row * count + i, row * count + (i + 1) % count
            faces.append((a, b, b + count, a + count))
    faces.extend((tuple(reversed(range(count))),
                  tuple((len(sections) - 1) * count + i for i in range(count))))
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    # Recalculate winding before bevel/export so all closed hulls face outward.
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.object.mode_set(mode='OBJECT')
    obj.select_set(False)
    return finish(obj, name, mat, 0.12, True)


def plate(name, outline, thickness, mat):
    """Closed wing or armor plate from a convex Bevy-space outline."""
    vertices = [point((x, y + side * thickness / 2, z))
                for side in [-1, 1] for x, y, z in outline]
    n = len(outline)
    faces = [tuple(reversed(range(n))), tuple(range(n, n * 2))]
    faces += [(i, (i + 1) % n, (i + 1) % n + n, i + n) for i in range(n)]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    return finish(obj, name, mat, 0.1)


# Long, cockpit-free keel. The twin upper plates leave an exposed central spine.
loft('Keel / sealed avionics', [(-15, 0, 1.6, 1.25), (-12, 0.25, 3.25, 2.1),
     (-7, 0.5, 4.25, 2.5), (0, 0.6, 4.6, 2.65), (7, 0.5, 4.15, 2.5),
     (12, 0.2, 2.8, 1.9), (15.5, 0.1, 1.1, 0.8)], graphite)
for side in [-1, 1]:
    loft('Split titanium carapace', [(-12.8, 1.0, 0.28, 0.4), (-10, 1.8, 1.0, 1.2),
         (-5, 2.35, 1.8, 1.85), (1.5, 2.5, 2.1, 2.1), (7, 2.3, 1.75, 1.8),
         (11.7, 1.5, 0.7, 1.0), (13, 1, 0.18, 0.3)], silver, side * 2.15)
    plate('Swept outrigger', [(side * 3.7, -0.5, -1.0), (side * 7.7, -0.5, 1.5),
          (side * 12.3, 0.8, 14.8), (side * 8.5, 0.8, 13.1)], 0.7, graphite)
    plate('Titanium outrigger fairing', [(side * 5.3, 0, 1.1), (side * 7.0, 0, 2.7),
          (side * 11.8, 1.25, 14.5), (side * 9.4, 1.25, 12.8)], 0.35, silver)
    plate('Raised tail stabilizer', [(side * 8.8, 1.3, 10.4), (side * 10.4, 5.7, 16.7),
          (side * 9.1, 5.5, 16.0), (side * 7.5, 1.3, 10.0)], 0.35, steel)
    rail('Tail light', (side * 9.6, 3.8, 14.25), (side * 10.2, 5.45, 16.3), 0.1, cyan)
    # Recessed equipment rather than prominent gun barrels.
    loft('Core module housing', [(-4.1, -0.35, 0.2, 0.35), (-2.8, -0.15, 1.0, 0.95),
         (1.8, -0.15, 1.3, 1.15), (4.0, -0.2, 0.65, 0.65)], steel, side * 5.65)
    box('Module service cover', (side * 5.65, 0.95, 0.2), (1.3, 0.2, 3.3), silver)
    for z in [-0.6, 0.1]:
        box('Module status light', (side * 5.65, 1.08, z), (0.9, 0.07, 0.18), cyan, 0.03)
    # Long lighting lines emphasize a narrow, fast silhouette at arena scale.
    rail('Forward running light', (side * 3.6, 2.55, -7.8), (side * 4.17, 2.8, -3.0), 0.12, cyan)
    rail('Aft running light', (side * 4.2, 2.95, 0.0), (side * 3.85, 2.6, 6.8), 0.12, cyan)
    for z in [5.4, 6.2, 7.0, 7.8, 8.6]:
        box('Heat exchanger louver', (side * 3.5, 3.6 - (z - 5.4) * 0.14, z),
            (0.7, 0.13, 0.25), graphite, 0.04)
    box('Amber flank identification', (side * 4.08, 3.35, 1.1), (0.27, 0.1, 1.4), marking, 0.04)

# Two tandem anti-gravity pods, like hover-bike wheels tucked under the hull.
for index, z in enumerate([-7.8, 8.0], 1):
    cylinder(f'Hover {index} / central mount', (0, -2.9, z), 2.1, 1.6, steel)
    ring(f'Hover {index} / graphite duct', (0, -4.1, z), 6.3, 0.65, graphite)
    ring(f'Hover {index} / top machined lip', (0, -3.45, z), 6.32, 0.2, silver)
    ring(f'Hover {index} / bottom lip', (0, -5.45, z), 6.28, 0.25, steel)
    ring(f'Hover {index} / luminous levitation band', (0, -4.85, z), 6.36, 0.19, cyan)
    cylinder(f'Hover {index} / field emitter', (0, -4.5, z), 2.4, 1.4, graphite)
    cylinder(f'Hover {index} / underside coil', (0, -5.3, z), 1.9, 0.5, cyan)
    ring(f'Hover {index} / hub rim', (0, -3.8, z), 2.0, 0.18, silver)
    for i in range(6):
        angle = 2 * pi * i / 6
        a = (cos(angle) * 2.0, -4.05, z + sin(angle) * 2.0)
        b = (cos(angle + 0.24) * 5.8, -4.05, z + sin(angle + 0.24) * 5.8)
        rail(f'Hover {index} / field vane', a, b, 0.24, steel)
        cylinder(f'Hover {index} / rim fastener',
                 (cos(angle) * 6.3, -3.2, z + sin(angle) * 6.3), 0.16, 0.11, graphite, vertices=12)

# Optical cluster: one large camera and three smaller ranging / IR sensors.
for index, (x, y, radius, z) in enumerate([(-0.75, 0.25, 1.6, -15.3),
                                         (1.6, 1.55, 0.66, -14.9),
                                         (1.95, 0.0, 0.66, -15.0),
                                         (1.55, -1.45, 0.5, -14.8)], 1):
    axis = (0, 0, -1)
    cylinder(f'Sensor {index} / socket', (x, y, z), radius + 0.28, 1.3, steel, axis)
    cylinder(f'Sensor {index} / black bezel', (x, y, z - 0.66), radius + 0.08, 0.16, graphite, axis)
    ring(f'Sensor {index} / focus ring', (x, y, z - 0.78), radius * 0.82, 0.07, cyan, axis)
    cylinder(f'Sensor {index} / optical glass', (x, y, z - 0.79), radius * 0.68, 0.09, glass, axis)
    cylinder(f'Sensor {index} / iris', (x, y, z - 0.85), radius * 0.3, 0.025, cyan, axis)
    cylinder(f'Sensor {index} / lens glint', (x - radius * 0.2, y + radius * 0.23, z - 0.87),
             radius * 0.11, 0.025, silver, axis, 16)

# Small dorsal sensor and communication blade; no pilot canopy.
box('Dorsal processing spine', (0, 3.2, 3.0), (0.65, 1.2, 11.0), graphite)
cylinder('Dorsal lidar base', (0, 4.45, 0.6), 0.8, 0.3, steel)
cylinder('Dorsal lidar', (0, 4.72, 0.6), 0.56, 0.25, cyan)
plate('Communications blade', [(-0.18, 3.5, 7.0), (0.18, 3.5, 7.0),
      (0.18, 5.6, 11.5), (-0.18, 5.6, 11.5)], 0.25, graphite)
rail('Antenna tip', (0, 5.45, 11.1), (0, 5.75, 11.7), 0.1, cyan)
for z in [-3, -1.7, 3.0, 4.3]:
    box('Spine telemetry', (0, 3.88, z), (0.28, 0.05, 0.45), cyan, 0.015)

# Bake modifiers, preserve detailed named parts in the editable .blend.
bpy.ops.object.select_all(action='DESELECT')
for obj in parts:
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    for modifier in list(obj.modifiers):
        bpy.ops.object.modifier_apply(modifier=modifier.name)
    # Recalculate all final face normals, including mirrored plates.
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.object.mode_set(mode='OBJECT')
    obj.select_set(False)

# Validate world-space bounds before export, independently of the Bevy test.
for obj in parts:
    for vertex in obj.data.vertices:
        b = obj.matrix_world @ vertex.co
        assert abs(b.x) <= 18.001 and abs(b.y) <= 18.001 and abs(b.z) <= 6.001, (obj.name, b)

source = ROOT / 'art/scout/scout_drone.blend'
source.parent.mkdir(parents=True, exist_ok=True)
# Studio setup lives only in the source; only mesh parts are selected for export.
scene = bpy.context.scene
scene.render.engine = 'CYCLES'
scene.cycles.samples = 48
scene.cycles.use_denoising = True
scene.render.resolution_x = 1400
scene.render.resolution_y = 1050
scene.render.resolution_percentage = 100
scene.world.color = (0.17, 0.17, 0.17)
scene.view_settings.view_transform = 'AgX'

floor_mat = material('Studio floor — not exported', (0.035, 0.052, 0.066), 0.15, 0.55)
bpy.ops.mesh.primitive_plane_add(size=200, location=(0, 0, -7.5))
bpy.context.object.name = 'Studio floor (not exported)'
bpy.context.object.data.materials.append(floor_mat)


def area(name, location, energy, size, color):
    bpy.ops.object.light_add(type='AREA', location=point(location))
    light = bpy.context.object
    light.name = name
    light.data.energy = energy
    light.data.shape = 'DISK'
    light.data.size = size
    light.data.color = color
    light.rotation_euler = (-light.location).to_track_quat('-Z', 'Y').to_euler()


area('Key softbox', (-25, 42, -25), 32000, 30, (0.78, 0.89, 1))
area('Warm fill', (32, 20, -10), 21000, 25, (1, 0.88, 0.72))
area('Cyan rim', (0, 25, 32), 45000, 22, (0.48, 0.78, 1))
bpy.ops.object.camera_add(location=point((36, 29, -46)))
camera = bpy.context.object
camera.name = 'Scout three-quarter preview'
camera.rotation_euler = (point((0, 0, 0)) - camera.location).to_track_quat('-Z', 'Y').to_euler()
camera.data.type = 'ORTHO'
camera.data.ortho_scale = 48
scene.camera = camera
# Save an immediately useful material-preview viewport in the editable source.
for screen in bpy.data.screens:
    for space_area in screen.areas:
        if space_area.type == 'VIEW_3D':
            space_area.spaces.active.region_3d.view_perspective = 'CAMERA'
bpy.ops.wm.save_as_mainfile(filepath=str(source))

# Consolidate export copies by material: six draw primitives, independent of part count.
bpy.ops.object.select_all(action='DESELECT')
export_parts = []
for mat in [silver, graphite, steel, cyan, glass, marking]:
    copies = []
    for original in parts:
        if original.data.materials[0] == mat:
            obj = original.copy()
            obj.data = original.data.copy()
            bpy.context.collection.objects.link(obj)
            obj.select_set(True)
            copies.append(obj)
    bpy.context.view_layer.objects.active = copies[0]
    bpy.ops.object.join()
    joined = bpy.context.object
    joined.name = 'Scout / ' + mat.name
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    export_parts.append(joined)
    joined.select_set(False)
for obj in export_parts:
    obj.select_set(True)
out = ROOT / 'assets/models/scout_drone.glb'
out.parent.mkdir(parents=True, exist_ok=True)
bpy.ops.export_scene.gltf(filepath=str(out), export_format='GLB', use_selection=True,
                          export_yup=True, export_animations=False, export_cameras=False,
                          export_lights=False, export_apply=True)
for obj in export_parts:
    bpy.data.objects.remove(obj, do_unlink=True)
scene.render.filepath = str(ROOT / 'docs/images/scout-drone.png')
Path(scene.render.filepath).parent.mkdir(parents=True, exist_ok=True)
bpy.ops.render.render(write_still=True)
print(f'Scout: {len(parts)} editable parts, 6 exported material meshes; {out}')
