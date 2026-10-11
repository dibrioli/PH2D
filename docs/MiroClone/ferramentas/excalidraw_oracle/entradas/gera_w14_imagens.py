#!/usr/bin/env python3
"""W14 — entradas de IMAGEM para o oráculo Excalidraw 0.18.1 (gera w14_imagens.excalidraw).

Uma PNG 4×2 com oito cores distintas (cada píxel diz onde está), para ler recorte e espelho ao píxel
nas saídas: sem recorte, espelhada (`scale`), recortada (`crop`), redonda + opaca a metade + rodada,
e uma com `fileId` sem ficheiro (o «pending»). Sem PIL: o PNG escreve-se com zlib.
"""
import base64, json, struct, sys, zlib
from pathlib import Path

PX = [  # linha 0, linha 1 — RGBA
    [(255, 0, 0, 255), (0, 255, 0, 255), (0, 0, 255, 255), (255, 255, 0, 255)],
    [(0, 255, 255, 255), (255, 0, 255, 255), (0, 0, 0, 255), (255, 255, 255, 255)],
]

def png(px):
    h, w = len(px), len(px[0])
    raw = b"".join(b"\0" + b"".join(bytes(p) for p in row) for row in px)
    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))

data = png(PX)
url = "data:image/png;base64," + base64.b64encode(data).decode()
FID = "a4x2"

def img(i, x, y, w, h, **kw):
    e = {"id": f"img{i}", "type": "image", "x": x, "y": y, "width": w, "height": h, "angle": 0,
         "strokeColor": "transparent", "backgroundColor": "transparent", "fillStyle": "solid",
         "strokeWidth": 2, "strokeStyle": "solid", "roughness": 1, "opacity": 100, "groupIds": [],
         "frameId": None, "roundness": None, "seed": 1000 + i, "version": 1, "versionNonce": i,
         "isDeleted": False, "boundElements": None, "updated": 1, "link": None, "locked": False,
         "status": "saved", "fileId": FID, "scale": [1, 1], "crop": None}
    e.update(kw)
    return e

els = [
    img(1, 0, 0, 160, 80),
    img(2, 200, 0, 160, 80, scale=[-1, 1]),
    img(3, 400, 0, 80, 80, crop={"x": 2, "y": 0, "width": 2, "height": 2, "naturalWidth": 4, "naturalHeight": 2}),
    img(4, 0, 120, 160, 80, roundness={"type": 3}, opacity=50, angle=0.5),
    img(5, 200, 120, 160, 80, fileId="semficheiro", status="pending"),
    img(6, 400, 120, 160, 80, scale=[1, -1], crop={"x": 1, "y": 0, "width": 2, "height": 1, "naturalWidth": 4, "naturalHeight": 2}),
]
scene = {"type": "excalidraw", "version": 2, "source": "PH2D W14 oracle", "elements": els,
         "appState": {"viewBackgroundColor": "#ffffff"},
         "files": {FID: {"mimeType": "image/png", "id": FID, "dataURL": url, "created": 1}}}
out = Path(sys.argv[1] if len(sys.argv) > 1 else Path(__file__).with_name("w14_imagens.excalidraw"))
out.write_text(json.dumps(scene, indent=1) + "\n")
print(out, len(data), "bytes PNG")
