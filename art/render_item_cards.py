"""Render catalogue thumbnails from Waldwinkel's own Blender model exports."""
from pathlib import Path
import re
import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets/models"
OUT = ROOT / "assets/ui/items"
furniture = list(dict.fromkeys(re.findall(r'"(furniture_[a-z_]+)"', (ROOT / "src/furniture.rs").read_text())))
models = furniture + ["workshop_forge", "workshop_loom", "farm_coop", "farm_barn"]

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.world = bpy.data.worlds.new("Catalogue lighting")
scene.world.use_nodes = True
scene.world.node_tree.nodes["Background"].inputs[0].default_value = (.49, .56, .53, 1.)
scene.world.node_tree.nodes["Background"].inputs[1].default_value = .46
scene.render.engine = "CYCLES"
scene.cycles.samples = 16
scene.cycles.use_denoising = False
scene.render.resolution_x = scene.render.resolution_y = 132
scene.render.resolution_percentage = 100
scene.render.film_transparent = True
scene.render.image_settings.color_mode = "RGBA"
scene.view_settings.view_transform = "AgX"

bpy.ops.object.camera_add()
camera = bpy.context.object
camera.data.type = "ORTHO"
scene.camera = camera
lights = []
for name, offset, base, color in [
    ("Key", (3.0, -3.2, 5.2), 255., (1., .83, .64)),
    ("Fill", (-4.0, -1.2, 3.4), 115., (.74, .89, 1.)),
    ("Rim", (1.2, 3.6, 3.8), 160., (1., .93, .75)),
]:
    light = bpy.data.lights.new(name, type="AREA")
    light.color = color
    lamp = bpy.data.objects.new(name, light)
    scene.collection.objects.link(lamp)
    lights.append((lamp, Vector(offset), base))

for name in models:
    before = set(scene.objects)
    bpy.ops.import_scene.gltf(filepath=str(SOURCE / f"{name}.glb"))
    imported = [obj for obj in scene.objects if obj not in before]
    points = [obj.matrix_world @ Vector(corner) for obj in imported if obj.type == "MESH"
              for corner in obj.bound_box]
    lo = Vector(tuple(min(p[i] for p in points) for i in range(3)))
    hi = Vector(tuple(max(p[i] for p in points) for i in range(3)))
    center = (lo + hi) * .5
    size = max(.5, (hi-lo).length)
    camera.location = center + Vector((2.3, -4.2, 2.7)).normalized() * (3.2 * size)
    camera.rotation_euler = (center - camera.location).to_track_quat("-Z", "Y").to_euler()
    camera.data.ortho_scale = 1.13 * size
    for lamp, offset, power in lights:
        lamp.location = center + offset.normalized() * (size * 1.8)
        lamp.rotation_euler = (center - lamp.location).to_track_quat("-Z", "Y").to_euler()
        lamp.data.energy = power * size ** 2 / 4
        lamp.data.size = size * 1.28
    scene.render.filepath = str(OUT / f"{name}.png")
    bpy.ops.render.render(write_still=True)
    for obj in imported:
        bpy.data.objects.remove(obj, do_unlink=True)
