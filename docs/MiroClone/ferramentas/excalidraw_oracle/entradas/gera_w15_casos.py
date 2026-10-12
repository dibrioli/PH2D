#!/usr/bin/env python3
"""W15 — entradas de IMAGEM para o oráculo Excalidraw 0.18.1 (moldura, SVG, filtro).

Gera, ao lado deste ficheiro:
  w15_moldura.excalidraw  quatro `image` de uma PNG 4×2 AZUL lisa (para o vermelho do contorno não
                          se confundir com a imagem): controlo (stroke transparente), tracejada
                          #e03131 4, a mesma com `roundness {type:3}`, e sólida #e03131 4. Afastadas
                          100 un. para medir o anel à volta de cada caixa no PNG.
  w15_svg.excalidraw      duas `image` com `files[..].mimeType = "image/svg+xml"` (círculo vermelho
                          r=50 num viewBox 100×100): com width/height no <svg> e só com viewBox.
  w15_sondas.json         sondas restoreElements para o `corre_esquema.mjs`: a imagem crua com os
                          campos de contorno, e uma com chaves CANDIDATAS a filtro (`filter`,
                          `grayscale`, `imageFilter`) — para ver se o restore as guarda ou as larga.
Sem PIL: o PNG escreve-se com zlib.
"""
import base64, json, struct, zlib
from pathlib import Path

HERE = Path(__file__).parent


def png(px):
    h, w = len(px), len(px[0])
    raw = b"".join(b"\0" + b"".join(bytes(p) for p in row) for row in px)

    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def img(i, x, y, w, h, fid, **kw):
    e = {"id": f"img{i}", "type": "image", "x": x, "y": y, "width": w, "height": h, "angle": 0,
         "strokeColor": "transparent", "backgroundColor": "transparent", "fillStyle": "solid",
         "strokeWidth": 2, "strokeStyle": "solid", "roughness": 1, "opacity": 100, "groupIds": [],
         "frameId": None, "roundness": None, "seed": 1500 + i, "version": 1, "versionNonce": i,
         "isDeleted": False, "boundElements": None, "updated": 1, "link": None, "locked": False,
         "status": "saved", "fileId": fid, "scale": [1, 1], "crop": None}
    e.update(kw)
    return e


def scene(els, files):
    return {"type": "excalidraw", "version": 2, "source": "PH2D W15 oracle", "elements": els,
            "appState": {"viewBackgroundColor": "#ffffff"}, "files": files}


def write(name, obj):
    p = HERE / name
    p.write_text(json.dumps(obj, indent=1) + "\n")
    print(p)


# --- moldura -------------------------------------------------------------------------------------
BLUE = (30, 60, 220, 255)
blue_url = "data:image/png;base64," + base64.b64encode(png([[BLUE] * 4] * 2)).decode()
FB = "azul4x2"
RED = dict(strokeColor="#e03131", strokeWidth=4)
mold = [
    img(1, 0, 0, 160, 80, FB),                                                    # controlo
    img(2, 260, 0, 160, 80, FB, strokeStyle="dashed", **RED),                     # tracejada
    img(3, 0, 180, 160, 80, FB, strokeStyle="dashed", roundness={"type": 3}, **RED),  # + cantos
    img(4, 260, 180, 160, 80, FB, strokeStyle="solid", **RED),                    # sólida
]
write("w15_moldura.excalidraw",
      scene(mold, {FB: {"mimeType": "image/png", "id": FB, "dataURL": blue_url, "created": 1}}))

# --- svg -----------------------------------------------------------------------------------------
SVG_WH = ('<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">'
          '<circle cx="50" cy="50" r="50" fill="#ff0000"/></svg>')
SVG_VB = ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">'
          '<circle cx="50" cy="50" r="50" fill="#ff0000"/></svg>')
u = lambda s: "data:image/svg+xml;base64," + base64.b64encode(s.encode()).decode()
svg_els = [
    img(1, 0, 0, 100, 100, "circ_wh"),
    img(2, 200, 0, 100, 100, "circ_vb"),
]
write("w15_svg.excalidraw", scene(svg_els, {
    "circ_wh": {"mimeType": "image/svg+xml", "id": "circ_wh", "dataURL": u(SVG_WH), "created": 1},
    "circ_vb": {"mimeType": "image/svg+xml", "id": "circ_vb", "dataURL": u(SVG_VB), "created": 1},
}))

# --- sondas (corre_esquema.mjs) --------------------------------------------------------------------
write("w15_sondas.json", {
    "sondas_elementos": [
        {"nome": "imagem_contorno_tracejado_redondo",
         "elementos": [img(9, 0, 0, 160, 80, FB, strokeStyle="dashed", roundness={"type": 3}, **RED)]},
        {"nome": "imagem_minima", "elementos": [{"type": "image", "x": 0, "y": 0, "width": 10, "height": 10, "fileId": FB}]},
        {"nome": "imagem_chaves_candidatas_a_filtro",
         "elementos": [img(10, 0, 0, 160, 80, FB, filter="grayscale", grayscale=True, imageFilter="grayscale")]},
    ],
    "sondas_appstate": [],
})
