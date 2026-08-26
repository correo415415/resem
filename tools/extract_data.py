#!/usr/bin/env python3
"""Extract game database from decompiled AS3 (pack/serbi.as) into JSON.

Resort Empire (Little Giant World) was decompiled with JPEXS; all gameplay
data lives as AS3 object literals in pack/serbi.as. This script parses those
literals and produces assets/data/gamedata.json used by the Rust/Bevy port.
"""
import re, json, os

SRC = "/home/user/webapp/source/scripts/pack/serbi.as"
OUT = "/home/user/webapp/assets/data/gamedata.json"

text = open(SRC, encoding="utf-8", errors="replace").read().replace("\r\n", "\n")


def parse_as3_object(s, pos):
    """Parse an AS3 object/array literal starting at pos. Returns (value, newpos)."""
    while s[pos] in " \n\t":
        pos += 1
    c = s[pos]
    if c == '{':
        obj = {}
        pos += 1
        while True:
            while s[pos] in " \n\t,":
                pos += 1
            if s[pos] == '}':
                return obj, pos + 1
            m = re.match(r'"([^"]*)"\s*:', s[pos:])
            if not m:
                m = re.match(r'([A-Za-z_$][\w$]*)\s*:', s[pos:])
            if not m:
                raise ValueError(f"bad key at {pos}: {s[pos:pos+60]!r}")
            key = m.group(1)
            pos += m.end()
            val, pos = parse_as3_object(s, pos)
            obj[key] = val
    elif c == '[':
        arr = []
        pos += 1
        while True:
            while s[pos] in " \n\t,":
                pos += 1
            if s[pos] == ']':
                return arr, pos + 1
            val, pos = parse_as3_object(s, pos)
            arr.append(val)
    elif c == '"':
        m = re.match(r'"((?:\\.|[^"\\])*)"', s[pos:])
        raw = m.group(1).replace("\\'", "'").replace('\\"', '"')
        return raw, pos + m.end()
    else:
        m = re.match(r'-?\d+\.?\d*(?:[eE]-?\d+)?', s[pos:])
        if m:
            v = m.group(0)
            return (float(v) if ('.' in v or 'e' in v or 'E' in v) else int(v)), pos + m.end()
        m = re.match(r'(true|false|null)', s[pos:])
        if m:
            return {"true": True, "false": False, "null": None}[m.group(1)], pos + m.end()
        m = re.match(r'new\s+Point\((-?\d+\.?\d*),(-?\d+\.?\d*)\)', s[pos:])
        if m:
            return [float(m.group(1)), float(m.group(2))], pos + m.end()
        m = re.match(r'[A-Za-z_$][\w$.]*(\([^)]*\))?', s[pos:])
        if m:
            return {"__ref__": m.group(0)}, pos + m.end()
        raise ValueError(f"bad value at {pos}: {s[pos:pos+60]!r}")


def extract_assignments(prefix):
    result = {}
    for m in re.finditer(re.escape(prefix) + r'\.([\w$]+)\s*=\s*(?=[\{\[])', text):
        name = m.group(1)
        try:
            val, _ = parse_as3_object(text, m.end())
        except ValueError:
            continue
        if name not in result:
            result[name] = val
    return result


data = {}
data["Scenery"] = extract_assignments("dataOb.Scenery")
data["Tile"] = extract_assignments("dataOb.Tile")
data["Booth"] = extract_assignments("dataOb.Booth")
data["Visitor"] = extract_assignments("dataOb.Visitor")
data["Employee"] = extract_assignments("dataOb.Employee")
data["Smiley"] = extract_assignments("dataOb.Smiley")
data["Mission"] = extract_assignments("Mission")
data["Extra_Upgrade"] = extract_assignments("Extra_Upgrade")

m = re.search(r'dataOb\.listing\s*=\s*(\[[^\]]*\])', text)
data["listing"] = json.loads(m.group(1).replace("'", '"'))
m = re.search(r'EXPAND\["price"\]\s*=\s*(\[[^\]]*\])', text)
data["ExpandPrice"] = json.loads(m.group(1))
m = re.search(r'public static var EXPAND:Array = (\[.*?\]);', text, re.S)
val, _ = parse_as3_object(text, m.start(1))
data["Expand"] = val
m = re.search(r'public static var resortName:Array = (\[[^\]]*\]);', text)
data["resortName"] = json.loads(m.group(1))

ach = {}
for m in re.finditer(r'Achievements\["([\w$]+)"\]\s*=\s*(?=\{)', text):
    val, _ = parse_as3_object(text, m.end())
    ach[m.group(1)] = val
data["Achievements"] = ach


def canceled_mood(served, nested=True):
    out = []
    for v in served:
        if nested:
            out.append([v[0] / 4, v[1] / 4])
        else:
            out.append(v / 2)
    return out


for name, b in data["Booth"].items():
    if isinstance(b, dict) and "boostMood" in b and isinstance(b["boostMood"], dict):
        bm = b["boostMood"]
        if "served" in bm and (not bm.get("canceled")):
            served = bm["served"]
            nested = isinstance(served[0], list)
            bm["canceled"] = canceled_mood(served, nested)

listing = data["listing"]
for name, v in data["Visitor"].items():
    if isinstance(v, dict) and "likes" in v:
        v.pop("smiley", None)
        likes = v["likes"]
        fac = likes.get("Facility")
        fl = fac[0] if fac and isinstance(fac[0], list) else fac
        if fl:
            v["Other"] = [x for x in listing if x not in fl]

os.makedirs(os.path.dirname(OUT), exist_ok=True)
json.dump(data, open(OUT, "w"), indent=1)
print("wrote", OUT)
print("booths:", list(data["Booth"].keys()))
print("visitors:", len([k for k in data["Visitor"] if k.startswith("Visitor")]))
print("tiles:", len(data["Tile"]), "scenery:", len(data["Scenery"]), "missions:", len(data["Mission"]))
