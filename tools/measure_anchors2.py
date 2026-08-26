#!/usr/bin/env python3
"""Empirically measure isometric anchors (v2) for Resort Empire sprites.

JPEXS exports every frame of a sprite on a shared canvas, so frames of one
sprite folder are mutually aligned. For each booth we measure the top (N)
vertex of its `blok` ground-diamond frame directly from the alpha channel:
  anchor = (mean x of topmost opaque rows, topmost opaque y)

Projection used by the port: screen_x=(ty-tx)*24, screen_y=(tx+ty)*12 (y down),
matching the original rumus.findTileCoord_point (Flash y negated).

Output: assets/data/anchors2.json
"""
import json
import numpy as np
from PIL import Image

DATA = json.load(open("assets/data/gamedata.json"))
out = {"tile": None, "booth": {}, "wall": {}, "scenery": {}}


def alpha(path):
    return np.array(Image.open(path).convert("RGBA"))[:, :, 3]


def n_vertex(a, thresh=8):
    ys, xs = np.nonzero(a > thresh)
    y0 = int(ys.min())
    top = xs[ys <= y0 + 1]
    return [float((int(top.min()) + int(top.max())) / 2.0), float(y0)]


# --- tile canvas anchor (all tiles share the 51x27 canvas) ---
a = alpha("assets/sprites/tiles/9.png")  # full asphalt diamond
out["tile"] = {"anchor": n_vertex(a), "size": [a.shape[1], a.shape[0]]}

# --- booths: anchor from blok frame; extents recorded for validation ---
for name, b in DATA["Booth"].items():
    if not isinstance(b, dict) or "ROWS" not in b:
        continue
    a = alpha(f"assets/sprites/booths/{b['blok']}.png")
    ax, ay = n_vertex(a)
    ys, xs = np.nonzero(a > 8)
    out["booth"][name] = {
        "anchor": [ax, ay],
        "rows": b["ROWS"],
        "cols": b["COLS"],
        "fr": b["fr"],
        "blok": b["blok"],
        "left_tiles": round((ax - int(xs.min())) / 24.0, 2),
        "right_tiles": round((int(xs.max()) - ax) / 24.0, 2),
    }
    # walls share the booth position; anchor measured from wall frame S vertex
    for key in ("wall", "wall_alpha"):
        fr = b.get(key)
        if not fr:
            continue
        try:
            wa = alpha(f"assets/sprites/walls/{fr}.png")
        except FileNotFoundError:
            continue
        wys, wxs = np.nonzero(wa > 8)
        y1 = int(wys.max())
        bot = wxs[wys >= y1 - 1]
        out["wall"][str(fr)] = {
            "booth": name,
            "kind": key,
            "s_vertex": [float((int(bot.min()) + int(bot.max())) / 2.0), float(y1)],
        }

# --- scenery: bottom-center of alpha bbox (placed at tile center) ---
for name, s in DATA["Scenery"].items():
    if not isinstance(s, dict) or not s.get("fr"):
        continue
    try:
        a = alpha(f"assets/sprites/scenery/{s['fr']}.png")
    except FileNotFoundError:
        continue
    ys, xs = np.nonzero(a > 8)
    out["scenery"][name] = {
        "fr": s["fr"],
        "base": [float((int(xs.min()) + int(xs.max())) / 2.0), float(int(ys.max()))],
    }

json.dump(out, open("assets/data/anchors2.json", "w"), indent=1)
print("tile:", out["tile"])
for n in ("Lobby", "Cottage", "Lodge", "Pool", "Golf"):
    print(n, out["booth"][n])
print("scenery entries:", len(out["scenery"]), "wall entries:", len(out["wall"]))
