#!/usr/bin/env python3
"""Uma fonte MÍNIMA cujo par `AB` usa XPlacement — nenhuma fonte do repo usa (medido 11/10: Inter,
Noto Sans, Atkinson, Virgil), e sem ela a cláusula do `offset` do kerning não tem gate.

Da raiz da worktree:
    python3 -I "docs/Vector Module/ferramentas/vectorcraft_2026-10-11/fonte_kern_xplacement.py" \\
        crates/ph2d-vector-font/tests/fixtures/kern_xplacement.ttf

`A` e `B` são quadrados de avanço 500 (upem 1000); o `kern` do GPOS dá `A` XAdvance −50 e `B`
XPlacement +30: o pen de `B` anda 450, e o `B` desenha-se 30 adiante sem mover o pen.
"""
import sys
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.feaLib.builder import addOpenTypeFeaturesFromString


def square():
    pen = TTGlyphPen(None)
    pen.moveTo((50, 0)); pen.lineTo((50, 700)); pen.lineTo((450, 700)); pen.lineTo((450, 0)); pen.closePath()
    return pen.glyph()


fb = FontBuilder(1000, isTTF=True)
fb.setupGlyphOrder([".notdef", "A", "B"])
fb.setupCharacterMap({ord("A"): "A", ord("B"): "B"})
fb.setupGlyf({".notdef": square(), "A": square(), "B": square()})
fb.setupHorizontalMetrics({g: (500, 50) for g in [".notdef", "A", "B"]})
fb.setupHorizontalHeader(ascent=800, descent=-200)
fb.setupNameTable({"familyName": "PH2D Kern XPlacement", "styleName": "Regular"})
fb.setupOS2(sTypoAscender=800, sTypoDescender=-200, usWinAscent=800, usWinDescent=200)
fb.setupPost()
addOpenTypeFeaturesFromString(fb.font, "feature kern { pos A <0 0 -50 0> B <30 0 0 0>; } kern;")
fb.save(sys.argv[1])
