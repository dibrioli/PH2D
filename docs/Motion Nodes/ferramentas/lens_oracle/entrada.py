#!/usr/bin/env python3
"""A ENTRADA NOSSA dos oráculos da lente (doc 129 §5.1) — PNG RGB 8 bits, só biblioteca padrão.

`entrada.py <cena|branco> <saida.png>`:
- `cena`: 192×128, um gradiente diagonal com três discos saturados e uma barra branca de arestas
  duras (as arestas são o que a aberração e o blur mostram);
- `branco`: 192×128 todo branco (a vinheta sobre ele É a máscara).

A mesma fórmula gera a imagem nos testes Rust (`ph2d-lens/tests/oraculos.rs::entrada_cena`), e o
gate `a_entrada_e_a_do_oraculo` confere-a contra a amostra gravada (`entrada_cena.txt`).
"""
import struct
import sys
import zlib

W, H = 192, 128
DISCOS = ((48, 40, 22, (230, 40, 30)), (120, 70, 28, (20, 180, 60)), (160, 30, 16, (40, 60, 230)))


def cena(x, y):
    r = (x * 255) // (W - 1)
    g = (y * 255) // (H - 1)
    b = ((x + y) * 255) // (W + H - 2)
    for cx, cy, raio, cor in DISCOS:
        if (x - cx) ** 2 + (y - cy) ** 2 <= raio * raio:
            r, g, b = cor
    if 20 <= x < 172 and 100 <= y < 108:
        r, g, b = 255, 255, 255
    return r, g, b


def png(caminho, pixel):
    linhas = b"".join(
        b"\x00" + b"".join(bytes(pixel(x, y)) for x in range(W)) for y in range(H)
    )

    def bloco(tipo, dados):
        corpo = tipo + dados
        return struct.pack(">I", len(dados)) + corpo + struct.pack(">I", zlib.crc32(corpo))

    with open(caminho, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(bloco(b"IHDR", struct.pack(">IIBBBBB", W, H, 8, 2, 0, 0, 0)))
        f.write(bloco(b"IDAT", zlib.compress(linhas, 9)))
        f.write(bloco(b"IEND", b""))


if __name__ == "__main__":
    tipo, saida = sys.argv[1], sys.argv[2]
    png(saida, cena if tipo == "cena" else (lambda x, y: (255, 255, 255)))
