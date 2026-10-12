#!/usr/bin/env python3
"""W17 — entradas ESTRAGADAS para a sonda das imagens ilegíveis no Excalidraw 0.18.1.

Todas partem da `w16_quadrantes.png` (400×300) e têm o CABEÇALHO bom (o tamanho lê-se), com os dados
estragados — o caso em que a nossa porta de entrada aceita o ficheiro e só o desenho o descobre:
- `w17_png_truncado.png` — os primeiros 60 % dos bytes (o IHDR inteiro, o IDAT cortado a meio).
- `w17_png_idat_lixo.png` — o IDAT trocado por bytes pseudo-aleatórios do mesmo tamanho, CRC refeito
  (o contentor é válido; o zlib não).
- `w17_jpeg_lixo.jpg` — o JPEG (qualidade 90) com os dados de entropia depois do SOS trocados por
  lixo (sem `0xFF`, para não fabricar marcadores), o EOI mantido.
- `w17_jpeg_truncado.jpg` — o mesmo JPEG bom cortado a 50 % (sem EOI).

O lixo vem de um gerador semeado (`random.Random(17)`): as entradas são estáveis entre corridas.
Uso: python3 -I gera_w17_imagens.py [pasta]
"""
import io
import random
import struct
import sys
import zlib
from pathlib import Path

from PIL import Image

aqui = Path(__file__).resolve().parent
out = Path(sys.argv[1]) if len(sys.argv) > 1 else aqui
png = (aqui / "w16_quadrantes.png").read_bytes()
rng = random.Random(17)

(out / "w17_png_truncado.png").write_bytes(png[: len(png) * 6 // 10])

# Os pedaços do PNG: o IDAT recebe lixo do mesmo comprimento e um CRC certo.
partes, i = [png[:8]], 8
while i < len(png):
    (n,) = struct.unpack(">I", png[i : i + 4])
    tipo = png[i + 4 : i + 8]
    dados = png[i + 8 : i + 8 + n]
    if tipo == b"IDAT":
        dados = bytes(rng.randrange(256) for _ in range(n))
    crc = zlib.crc32(tipo + dados) & 0xFFFFFFFF
    partes.append(struct.pack(">I", n) + tipo + dados + struct.pack(">I", crc))
    i += 12 + n
(out / "w17_png_idat_lixo.png").write_bytes(b"".join(partes))

buf = io.BytesIO()
Image.open(io.BytesIO(png)).convert("RGB").save(buf, "JPEG", quality=90)
jpg = buf.getvalue()
sos = jpg.index(b"\xff\xda")
(n,) = struct.unpack(">H", jpg[sos + 2 : sos + 4])
inicio = sos + 2 + n
lixo = bytes(rng.randrange(255) for _ in range(len(jpg) - 2 - inicio))
(out / "w17_jpeg_lixo.jpg").write_bytes(jpg[:inicio] + lixo + jpg[-2:])
(out / "w17_jpeg_truncado.jpg").write_bytes(jpg[: len(jpg) // 2])
print("ok", sorted(p.name for p in out.glob("w17_*")))
