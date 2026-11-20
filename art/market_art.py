"""Blender geometry for Mara’s evening stall and the residents’ working props."""
import math


def models(Model):
    stall = Model("Abendmarkt · Maras Suppentisch", "evening_market")
    glow = Model("Abendmarkt · Laternenkerzen", "evening_market_lights")
    for x in (-0.89, 0.89):
        for z in (-0.47, 0.47):
            stall.box(x, z, 0.09, 0.23, 0.16, 0.18, "a79a8d")
            stall.box(x, z, 1.016, 0.11, 0.105, 1.95, "94654a")
            stall.pole((x, z, 1.53), (x * 0.70, z, 1.87), 0.031, "be9267")
        stall.box(x, .45, 1.56, .19, .175, .22, "c79861", .026)
        for y in (1.43, 1.69):
            stall.box(x, .45, y, .247, .23, .061, "76523e", .022)
        for dx in (-.092, .092):
            for dz in (-.079, .079):
                stall.box(x+dx, .45+dz, 1.56, .023, .023, .237, "795844", .008)
        glow.box(x, .45, 1.56, .194, .179, .218, "ffce77", .025, mode="raw")

    # Cloth has a shallow ridge, rounded dips between supports and a scalloped valance.
    section = [(-0.65, 1.88), (-0.36, 2.024), (0.03, 2.09), (0.37, 1.98), (0.65, 1.84)]
    for i in range(10):
        x0 = -1.12 + i * 0.224
        x1 = x0 + 0.224
        color = ("f3c989", "ad796a", "edbf80", "939970", "e9bd7e")[i % 5]
        for (z0, y0), (z1, y1) in zip(section, section[1:]):
            stall.face([(x0, z0, y0), (x0, z1, y1), (x1, z1, y1), (x1, z0, y0)], color)
        z, y = section[-1]
        stall.face([(x0, z, y), (x1, z, y), (x1, z, y - .10),
                    ((x0+x1)*.5, z, y-.145), (x0, z, y-.10)], color)
        stall.box(x0 + 0.112, z + .005, y - .03, .217, .014, .031, "ead7a5", .004)
    for z in (-0.47, 0.47):
        stall.pole((-0.99, z, 1.84), (0.99, z, 1.84), 0.041, "9a724e")
    stall.box(0., 0.12, 0.69, 1.85, 0.65, 0.087, "c59c71")
    stall.box(0., 0.13, 0.28, 1.67, 0.49, 0.078, "a07855")
    for i in range(10):
        stall.box((i - 4.5) * 0.174, .36, 0.49, 0.163, 0.055, 0.30,
                  ("b68f64", "c0966b", "a8815e")[i % 3])
    stall.pole((-0.39, 0.046, 0.731), (-0.39, 0.046, 0.925), 0.162, "5a5756", 12, tip=.202)
    stall.pole((-0.39, 0.046, 0.927), (-0.39, 0.046, 0.949), 0.208, "817476", 12)
    stall.pole((-0.39, 0.046, 0.949), (-0.39, 0.046, 0.955), 0.177, "cfa575", 12)
    for sign in (-1, 1):
        stall.box(-0.39 + sign * .201, 0.044, .888, .108, .071, .049, "766b64")
    for x, z, tint in [(0.02, .23, "698c82"), (0.28, .12, "9c775f"), (0.55, .21, "7e9e8c")]:
        stall.pole((x, z, .73), (x, z, .802), .075, tint, 10, tip=.119)
        stall.pole((x, z, .803), (x, z, .81), .108, "d6b17e", 10)
        stall.ellipsoid(x - .028, z + .002, .812, .021, .017, .01, "9b9b68", 6, 3)
    stall.pole((.74, -.13, .73), (.74, -.13, .97), .087, "bd865c", 10, tip=.072)
    stall.pole((.74, -.13, .97), (.74, -.13, 1.012), .072, "b88969", 10, tip=.094)

    # Feather the pools of lantern light across the pavement.
    light = Model("Abendmarkt · Kerzenlicht auf dem Pflaster", "evening_market_pool")
    for center in (-.78, .78):
        rings = [[(center+r*math.cos(a), .43+r*.74*math.sin(a), .086)
                  for i in range(20) for a in [i*math.tau/20]]
                 for r in (.19, .63, 1.12)]
        for layer, (inside, outside) in enumerate(zip(rings, rings[1:])):
            inner, outer = [((1.0, .71, .36, .32), (.91, .65, .35, .14)),
                            ((.91, .65, .35, .14), (.91, .65, .35, 0.))][layer]
            for i in range(20):
                j = (i + 1) % 20
                light.gradient([inside[i], inside[j], outside[j], outside[i]],
                               [inner, inner, outer, outer])
        light.gradient(rings[0], [(1.0, .71, .36, .32)] * 20)

    spoon = Model("Mara · Kochlöffel aus Olivenholz", "resident_spoon")
    spoon.pole((0., 0.028, -.07), (0., 0.21, .20), .019, "b58a59", 8, tip=.023)
    spoon.ellipsoid(0., .232, .235, .054, .036, .084, "d1a370", 9, 4)
    parcel = Model("Miro · Kräuterpäckchen", "resident_parcel")
    parcel.box(.034, .072, -.045, .205, .119, .143, "b8a479", .016, angle=.21)
    parcel.box(.033, .072, -.042, .038, .133, .151, "6e875c", .006, angle=.21)
    parcel.box(.035, .073, -.041, .212, .032, .151, "6e875c", .006, angle=.21)
    for x in (0., .071):
        parcel.ellipsoid(x, .072, .043, .049, .022, .026, "a7b47c", 7, 4)
    book = Model("Edda · Aufgeschlagenes Notizbuch", "resident_notebook")
    book.box(0., .075, -.068, .247, .17, .035, "8d6b60", .012)
    for side in (-1, 1):
        book.box(side*.055, .075, -.048, .109, .158, .026, "ece1c3", .008)
        for i in range(4):
            book.box(side*.056, .030+i*.027, -.033, .071, .004, .004, "aa967e", .001)
    hammer = Model("Runa · Schmiedehammer", "resident_hammer")
    hammer.pole((0., .02, -.105), (0., .08, .194), .027, "a78561", 8)
    hammer.box(0., .074, .201, .204, .091, .096, "68686b", .015)
    hammer.box(.092, .074, .201, .043, .105, .108, "8f8e8b", .015)
    return [stall, glow, light, spoon, parcel, book, hammer]
