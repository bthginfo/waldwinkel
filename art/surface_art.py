"""Subtle, tileable finish textures for the Blender models."""

import math

import bpy
from mathutils.noise import noise


def finishes(base):
    def texture(name, index):
        side = 128
        image = bpy.data.images.new(name, width=side, height=side, alpha=False)
        pixel_data = []
        for y in range(side):
            v = y / side
            for x in range(side):
                a, b = math.tau * x / side, math.tau * v
                ca, sa, cb, sb = math.cos(a), math.sin(a), math.cos(b), math.sin(b)
                if index == 0:
                    # Timber grain stretches along the length of each board.
                    grain = .58 + .49*noise((.73*ca+3.7*cb, .73*sa+3.7*sb, .4*sa+.6*sb))
                    grain += .18*noise((1.15*ca+6.*cb, 1.15*sa+6.*sb, .74*ca+.85*sb))
                elif index == 3:
                    # Warp and weft cross at a regular thread count.
                    grain = .57 + .22*math.cos(18*a) + .18*math.cos(18*b)
                    grain += .08*math.cos(9*(a+b))
                elif index == 4:
                    grain = .63 + .19*math.cos(20*a + math.sin(b))
                    grain += .24*noise((ca+cb,sa+sb,-cb))
                else:
                    spread = 1.18 if index == 1 else 1.74
                    grain = .57 + .53*noise((spread*(ca+.56*cb), spread*(sa+.56*sb),
                                               spread*(cb+sb)))
                    grain += .30*noise((3.6*ca+1.7*cb, 3.6*sa+1.7*sb, 2.9*cb+2.4*sb))
                value = .81 + .19 * min(max(grain, 0.), 1.)
                pixel_data.extend((value, value, value, 1.))
        image.pixels.foreach_set(pixel_data)
        image.pack()
        return image

    materials = {}
    for i, (key, label) in enumerate([
        ("timber", "Längsmaserung"),
        ("plaster", "Kalkputz"),
        ("stone", "Steinporen"),
        ("weave", "Leinengewebe"),
        ("metal", "Gebürstetes Metall"),
        ("soil", "Krümelige Erde"),
    ]):
        material = base.copy()
        material.name = label
        nodes = material.node_tree.nodes
        vertex = next(node for node in nodes if node.type == "VERTEX_COLOR")
        sampler = nodes.new("ShaderNodeTexImage")
        sampler.image = texture(label, i)
        sampler.extension = "REPEAT"
        # Export glTF base colour texture. COLOR_0 still carries the baked lighting.
        material.node_tree.links.remove(nodes["Principled BSDF"].inputs["Base Color"].links[0])
        material.node_tree.links.new(sampler.outputs["Color"], nodes["Principled BSDF"].inputs["Base Color"])
        # Blender's active vertex colour export supplies COLOR_0 even without a shader link.
        nodes.remove(vertex)
        materials[key] = material
    return materials


def finish_for(asset, color, mode):
    """Keep authored materials and choose a finish for legacy opaque geometry."""
    if mode != "lit":
        return mode
    r, g, b = color[:3]
    wood = r > g+.037 and g > b+.032
    if asset in ("soil", "wet_soil", "fertilized_soil"):
        return "soil"
    if asset.startswith(("mine_floor","mine_wall","mine_deep","resource_","dusk_floor",
                         "dusk_wall","dusk_arch","dusk_seal","find_gorge","resonance_")):
        return "stone"
    if asset.startswith(("workshop_","gear_","resident_")) or asset in (
            "axe","hoe","pickaxe","watering_can","fishing_rod","mine_ladder","mine_support"):
        return "timber" if wood else "metal"
    if asset.startswith(("furniture_rug","furniture_cushion","furniture_sofa","furniture_armchair")):
        return "timber" if wood and r < .67 else "weave"
    if asset.startswith(("interior","cabin_annex","furniture_","pier_","farm_","watch_","wood_",
                         "frontier_","season_table","waybell")) or asset in (
            "workbench_old","bed","bedroom_bed","sale_crate","pattern_stall","card_binder"):
        return "timber" if wood else ("stone" if max(color[:3])-min(color[:3]) < .15 else "plaster")
    return mode


def uv(position, normal, finish):
    if abs(normal.z) > .54:
        x, y = position.x, position.y
    elif abs(normal.x) > abs(normal.y):
        x, y = position.y, position.z
    else:
        x, y = position.x, position.z
    scale = {"timber": (.63, 1.60), "plaster": (.57, .57), "stone": (.79, .79),
             "weave": (.66, .66), "metal": (1.09, .45), "soil": (1.9, 1.9)}
    sx, sy = scale[finish]
    return x*sx, y*sy
