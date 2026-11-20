"""Build the harvestable wild grass tuft at the same scale as the other finds."""
from pathlib import Path
import math
import random
import bpy

bpy.ops.wm.read_factory_settings(use_empty=True)
rng = random.Random(122026)
vertices, faces, shades = [], [], []
for blade in range(21):
    angle = math.tau * (blade / 21.) + rng.uniform(-.25, .25)
    length = rng.uniform(.19, .49)
    splay = rng.uniform(.14, .32)
    bend = (math.cos(angle), math.sin(angle))
    across = (-bend[1], bend[0])
    root = tuple(b * rng.uniform(.012, .11) for b in bend)
    width = rng.uniform(.021, .045)
    start = len(vertices)
    for t, taper in [(0., .72), (.38, 1.), (.72, .53), (1., 0.)]:
        center = (root[0] + bend[0] * splay * t*t, root[1] + bend[1] * splay * t*t)
        for edge in (-.5, .5):
            vertices.append((center[0] + across[0]*width*taper*edge,
                             center[1] + across[1]*width*taper*edge,
                             .012 + length * (t - .19*t**3)))
    for row in range(3):
        k = start + 2*row
        faces.append((k, k+1, k+3, k+2))
        shades.append((blade + int(row >= 2)) % 4)
mesh = bpy.data.meshes.new("Folded wild grass blades")
mesh.from_pydata(vertices, [], faces)
mesh.update()
clump = bpy.data.objects.new("Wild grass", mesh)
bpy.context.collection.objects.link(clump)
for name, color in [("Field green", (.24,.42,.10)), ("Fresh green", (.35,.55,.16)),
                    ("Sunlit blade", (.55,.64,.19)), ("Dry tips", (.66,.61,.22))]:
    material = bpy.data.materials.new(name)
    material.use_nodes = True
    shader = material.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Base Color"].default_value = (*color, 1.)
    shader.inputs["Roughness"].default_value = .85
    clump.data.materials.append(material)
for face, shade in zip(mesh.polygons, shades):
    face.material_index = shade
output = Path(__file__).resolve().parents[1] / "assets/models/find_grass.glb"
bpy.ops.export_scene.gltf(filepath=str(output), export_format="GLB")
