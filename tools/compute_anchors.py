#!/usr/bin/env python3
"""Compute sprite anchors for booth/wall frames of Resort Empire.

Isometric model (SIZE_=24): tile (tx,ty) N-vertex at screen ((tx-ty)*24, (tx+ty)*12).
A booth footprint of r ROWS x c COLS occupies, relative to its base tile N-vertex,
the diamond spanning x in [-(r)*24, c*24] and y in [0, (r+c)*12].

Anchor = pixel position inside the PNG frame that corresponds to the N-vertex of
the footprint. Derived from the 'blok' frames (pure ground diamonds) measured
via alpha bounding boxes:
  anchor_x = bbox.x_max + 1 - c*24   (E vertex is at x = c*24 relative to N)
  anchor_y = bbox.y_min              (N vertex is topmost point)
Wall frames are measured relative to the same diamond bottom (S vertex).
"""
import json
from PIL import Image
import numpy as np

DATA = json.load(open("assets/data/gamedata.json"))
out = {"booth": {}, "wall": {}}


def bbox(path):
    a = np.array(Image.open(path))[:, :, 3]
    ys, xs = np.nonzero(a)
    return int(xs.min()), int(xs.max()), int(ys.min()), int(ys.max())


for name, b in DATA["Booth"].items():
    if not isinstance(b, dict) or "ROWS" not in b:
        continue
    r, c = b["ROWS"], b["COLS"]
    x0, x1, y0, y1 = bbox(f"assets/sprites/booths/{b['blok']}.png")
    ax = x1 + 1 - c * 24
    ay = y0
    out["booth"][name] = {"anchor": [ax, ay], "rows": r, "cols": c}
    for key in ("wall", "wall_alpha"):
        fr = b.get(key)
        if not fr:
            continue
        wx0, wx1, wy0, wy1 = bbox(f"assets/sprites/walls/{fr}.png")
        wax = wx0 + r * 24
        way = wy1 - (r + c) * 12
        out["wall"][str(fr)] = {"anchor": [wax, way], "rows": r, "cols": c, "booth": name, "kind": key}

json.dump(out, open("assets/data/anchors.json", "w"), indent=1)
print("booth anchors:", len(out["booth"]), "wall anchors:", len(out["wall"]))
