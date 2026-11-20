"""Builds the static fonts embedded in the app from the variable fonts.

Sources (Google Fonts, SIL Open Font License):
  ofl/figtree/Figtree[wght].ttf                                  saved as Figtree-VF.ttf
  ofl/bricolagegrotesque/BricolageGrotesque[opsz,wdth,wght].ttf  saved as Bricolage-VF.ttf

Requires fonttools (pip install fonttools). Run it in the folder that holds
the variable fonts:

  python3 build-fonts.py <output-folder>

"Figtree Tabular" is Figtree with tabular figures (tnum) baked into the cmap,
so amounts line up in lists. iced does not apply OpenType features at runtime.
"""

import sys

from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

STYLE = {400: "Regular", 500: "Medium", 600: "SemiBold", 700: "Bold", 800: "ExtraBold"}


def freeze_tnum(font):
    gsub = font["GSUB"].table
    lookups = set()
    for record in gsub.FeatureList.FeatureRecord:
        if record.FeatureTag == "tnum":
            lookups.update(record.Feature.LookupListIndex)
    mapping = {}
    for index in lookups:
        lookup = gsub.LookupList.Lookup[index]
        for sub in lookup.SubTable:
            if lookup.LookupType == 7:
                sub = sub.ExtSubTable
            if hasattr(sub, "mapping"):
                mapping.update(sub.mapping)
    changed = 0
    for table in font["cmap"].tables:
        for codepoint, glyph in list(table.cmap.items()):
            if glyph in mapping:
                table.cmap[codepoint] = mapping[glyph]
                changed += 1
    return changed


def build(src, family, weight, out, axes, tnum):
    font = instancer.instantiateVariableFont(TTFont(src), {"wght": weight, **axes})
    if tnum:
        assert freeze_tnum(font) > 0, "font has no tnum feature"
    style = STYLE[weight]
    name = font["name"]
    for record in list(name.names):
        if record.nameID in (1, 2, 4, 6, 16, 17, 21, 22, 25):
            name.removeNames(nameID=record.nameID)
    full = f"{family} {style}"
    name.setName(family, 1, 3, 1, 0x409)
    name.setName("Regular", 2, 3, 1, 0x409)
    name.setName(full, 4, 3, 1, 0x409)
    name.setName(full.replace(" ", "-"), 6, 3, 1, 0x409)
    name.setName(family, 16, 3, 1, 0x409)
    name.setName(style, 17, 3, 1, 0x409)
    font["OS/2"].usWeightClass = weight
    font["OS/2"].fsSelection = (font["OS/2"].fsSelection & ~0b1100001) | 0b1000000
    if "STAT" in font:
        del font["STAT"]
    font.save(out)


if __name__ == "__main__":
    out = sys.argv[1]
    for w in (400, 500, 600, 700):
        build("Figtree-VF.ttf", "Figtree", w, f"{out}/Figtree-{STYLE[w]}.ttf", {}, False)
    for w in (600, 700):
        build("Figtree-VF.ttf", "Figtree Tabular", w, f"{out}/FigtreeTabular-{STYLE[w]}.ttf", {}, True)
    for w in (600, 800):
        build(
            "Bricolage-VF.ttf",
            "Bricolage Grotesque",
            w,
            f"{out}/BricolageGrotesque-{STYLE[w]}.ttf",
            {"opsz": 48, "wdth": 100},
            False,
        )
