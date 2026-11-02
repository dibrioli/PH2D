#!/usr/bin/env python3
"""Oráculo FreeType do texto do Pixel Art (P8 item 8): que células acende cada glifo em MONO.

Corre a libfreetype do sistema (FTL OR GPL-2.0-or-later, `pacman -Qi freetype2`) por ctypes — a
mesma técnica de `docs/Pixel Art/app/tests/font_oracle.py` — e grava, por fonte × px × hinting × glifo,
o bitmap 1-bit, `bitmap_left`/`bitmap_top` e o avanço (26.6).  Nada do código da FreeType é lido.

Uso: python3 docs/PixelArt/ferramentas/oraculo_freetype_texto.py crates/ph2d-pixel-text/tests/fixtures/oracle_freetype_mono.json
"""
import ctypes as C
import json
import subprocess
import sys

FT_Long = FT_Pos = FT_Fixed = C.c_long


class FT_Generic(C.Structure):
    _fields_ = [('data', C.c_void_p), ('finalizer', C.c_void_p)]


class FT_BBox(C.Structure):
    _fields_ = [('xMin', FT_Pos), ('yMin', FT_Pos), ('xMax', FT_Pos), ('yMax', FT_Pos)]


class FT_Bitmap(C.Structure):
    _fields_ = [('rows', C.c_uint), ('width', C.c_uint), ('pitch', C.c_int), ('buffer', C.POINTER(C.c_ubyte)),
                ('num_grays', C.c_ushort), ('pixel_mode', C.c_ubyte), ('palette_mode', C.c_ubyte), ('palette', C.c_void_p)]


class FT_Glyph_Metrics(C.Structure):
    _fields_ = [(n, FT_Pos) for n in ('width', 'height', 'horiBearingX', 'horiBearingY', 'horiAdvance',
                                      'vertBearingX', 'vertBearingY', 'vertAdvance')]


class FT_Vector(C.Structure):
    _fields_ = [('x', FT_Pos), ('y', FT_Pos)]


class FT_GlyphSlotRec(C.Structure):
    _fields_ = [('library', C.c_void_p), ('face', C.c_void_p), ('next', C.c_void_p), ('glyph_index', C.c_uint),
                ('generic', FT_Generic), ('metrics', FT_Glyph_Metrics), ('linearHoriAdvance', FT_Fixed),
                ('linearVertAdvance', FT_Fixed), ('advance', FT_Vector), ('format', C.c_uint), ('bitmap', FT_Bitmap),
                ('bitmap_left', C.c_int), ('bitmap_top', C.c_int)]


class FT_FaceRec(C.Structure):
    _fields_ = [('num_faces', FT_Long), ('face_index', FT_Long), ('face_flags', FT_Long), ('style_flags', FT_Long),
                ('num_glyphs', FT_Long), ('family_name', C.c_char_p), ('style_name', C.c_char_p),
                ('num_fixed_sizes', C.c_int), ('available_sizes', C.c_void_p),
                ('num_charmaps', C.c_int), ('charmaps', C.c_void_p), ('generic', FT_Generic),
                ('bbox', FT_BBox), ('units_per_EM', C.c_ushort), ('ascender', C.c_short), ('descender', C.c_short),
                ('height', C.c_short), ('max_advance_width', C.c_short), ('max_advance_height', C.c_short),
                ('underline_position', C.c_short), ('underline_thickness', C.c_short),
                ('glyph', C.POINTER(FT_GlyphSlotRec)), ('size', C.c_void_p), ('charmap', C.c_void_p)]


ft = C.CDLL('libfreetype.so.6')
ft.FT_New_Face.argtypes = [C.c_void_p, C.c_char_p, C.c_long, C.POINTER(C.POINTER(FT_FaceRec))]
ft.FT_Done_Face.argtypes = [C.c_void_p]
ft.FT_Set_Pixel_Sizes.argtypes = [C.c_void_p, C.c_uint, C.c_uint]
ft.FT_Load_Char.argtypes = [C.c_void_p, C.c_ulong, C.c_int32]
FT_LOAD_NO_HINTING = 1 << 1
FT_LOAD_RENDER = 1 << 2
FT_LOAD_MONOCHROME = 1 << 12
FT_LOAD_TARGET_MONO = (2 & 15) << 16
LIB = C.c_void_p()
assert ft.FT_Init_FreeType(C.byref(LIB)) == 0
major, minor, patch = C.c_int(), C.c_int(), C.c_int()
ft.FT_Library_Version(LIB, C.byref(major), C.byref(minor), C.byref(patch))

FONTES = ['DejaVu Sans', 'DejaVu Serif', 'DejaVu Sans Mono']
TEXTOS = ['Ag1', 'PH2D']
PXS = [8, 12, 16]


def caminho(familia):
    return subprocess.check_output(['fc-match', familia, '--format=%{file}'], text=True)


def glifo(face_p, cp, hinting):
    flags = FT_LOAD_RENDER | FT_LOAD_MONOCHROME | FT_LOAD_TARGET_MONO | (0 if hinting else FT_LOAD_NO_HINTING)
    assert ft.FT_Load_Char(face_p, cp, flags) == 0
    s = face_p.contents.glyph.contents
    b = s.bitmap
    pitch = abs(b.pitch)
    buf = bytes(b.buffer[:pitch * b.rows]) if b.rows and pitch else b''
    rows = [''.join('#' if buf[y * pitch + (x >> 3)] & (0x80 >> (x & 7)) else '.' for x in range(b.width))
            for y in range(b.rows)]
    return {'cp': cp, 'left': s.bitmap_left, 'top': s.bitmap_top, 'advance_26_6': s.advance.x,
            'linear_advance_16_16': s.linearHoriAdvance, 'rows': rows}


def main():
    out = sys.argv[1]
    casos = []
    for fam in FONTES:
        path = caminho(fam)
        face_p = C.POINTER(FT_FaceRec)()
        assert ft.FT_New_Face(LIB, path.encode(), 0, C.byref(face_p)) == 0
        for px in PXS:
            assert ft.FT_Set_Pixel_Sizes(face_p, 0, px) == 0
            for hinting in (True, False):
                for texto in TEXTOS:
                    casos.append({'family': fam, 'file': path, 'px': px, 'hinting': hinting, 'text': texto,
                                  'glyphs': [glifo(face_p, ord(ch), hinting) for ch in texto]})
        ft.FT_Done_Face(face_p)
    doc = {
        'source': f'FreeType {major.value}.{minor.value}.{patch.value} (libfreetype.so.6 do sistema; FTL OR GPL-2.0-or-later)',
        'method': 'FT_Set_Pixel_Sizes(0, px); FT_Load_Char(RENDER|MONOCHROME|TARGET_MONO [|NO_HINTING]); '
                  'bitmap 1-bit (#=tinta), bitmap_left/top (da caneta e da linha de base, y para cima), advance 26.6',
        'command': 'python3 docs/PixelArt/ferramentas/oraculo_freetype_texto.py crates/ph2d-pixel-text/tests/fixtures/oracle_freetype_mono.json',
        'cases': casos,
    }
    with open(out, 'w') as f:
        json.dump(doc, f, indent=1)
        f.write('\n')
    print(f'{len(casos)} casos -> {out}')


main()
