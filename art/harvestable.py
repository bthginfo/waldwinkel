"""Extract individual resource meshes and their world positions from the diorama."""
from contextlib import contextmanager
import math
import struct



LEAF = 6
MOOR_LEAF = 7
ROCK = 8


def append_geometry(model, source, mark, x, z, height, scale):
    v, f = mark
    start = len(model.vertices) - v
    model.vertices.extend(((a-x)/scale, (b+z)/scale, (c-height)/scale)
                          for a,b,c in source.vertices[v:])
    model.faces.extend(tuple(i+start for i in face) for face in source.faces[f:])
    model.colors.extend(source.colors[f:])


def mesh_mark(model):
    return len(model.vertices), len(model.faces)


def truncate(model, mark):
    v, f = mark
    del model.vertices[v:], model.faces[f:], model.colors[f:]


class Harvestables:
    def __init__(self, model, seasons):
        self.model = model
        self.seasons = seasons
        self.variants = {}
        self.shadows = model("Rohstoffe · Schatten beim Backen")
        self.nodes = []

    @contextmanager
    def capture(self, source, x, z, height, scale, variant):
        import landscape

        ox, oz = landscape.ORIGINS.get(source.asset, (0, 0))
        wx, wz = ox+x, oz+z
        mark = mesh_mark(source)
        layers = [(season, mesh, mesh_mark(mesh)) for season, mesh in self.seasons.items()]
        blockers = len(landscape.BLOCKERS)
        yield
        # Render the trunk and crown together at runtime so they fall as one piece.
        if variant not in self.variants:
            names = ("spring", "summer", "autumn", "winter") if variant in (LEAF, MOOR_LEAF) else ("summer", "winter")
            meshes = []
            for season in names:
                mesh = self.model(f"Rohstoff {variant} · {season}", f"forage_{variant}_{season}")
                append_geometry(mesh, source, mark, x, z, height, scale)
                for name, layer, previous in layers:
                    if name == season:
                        append_geometry(mesh, layer, previous, wx, wz, height, scale)
                meshes.append(mesh)
            self.variants[variant] = meshes
        append_geometry(self.shadows, source, mark, -ox, -oz, 0, 1)
        for season, layer, previous in layers:
            if season == "summer":
                append_geometry(self.shadows, layer, previous, 0, 0, 0, 1)
            truncate(layer, previous)
        truncate(source, mark)
        del landscape.BLOCKERS[blockers:]
        angle = math.tau * (int(abs(wx*71 + wz*193)) % 73) / 73
        self.nodes.append((wx, wz, height, scale, angle, variant))

    def bake(self, root):
        with (root/"assets"/"world"/"harvest.bin").open("wb") as output:
            output.write(struct.pack("<4sI", b"WHV1", len(self.nodes)))
            for node in self.nodes:
                output.write(struct.pack("<fffffB", *node))
        return [mesh for variant in self.variants.values() for mesh in variant]
