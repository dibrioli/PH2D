#!/usr/bin/env python3
"""Grava UMA fixture de oráculo da lente: lê a saída em bruto de 16 bits e escreve uma amostra em grelha.

`amostra.py <bruto> <endian be|le> <canais 1|3> <passo> <saida.txt> <cabecalho...>`

Cada linha de dados é `x y v...` com os valores em `[0,1]` a 6 casas — a resolução do oráculo é
16 bits (`1/65535 ≈ 1,5e-5`), e 6 casas a guardam sem inventar precisão.
"""
import struct
import sys

W, H = 192, 128

bruto, endian, canais, passo, saida = sys.argv[1:6]
canais, passo = int(canais), int(passo)
fmt = (">" if endian == "be" else "<") + "H"
dados = open(bruto, "rb").read()
assert len(dados) == W * H * canais * 2, (bruto, len(dados))

with open(saida, "w") as f:
    for linha in sys.argv[6:]:
        f.write(f"# {linha}\n")
    for y in range(0, H, passo):
        for x in range(0, W, passo):
            base = 2 * canais * (y * W + x)
            vals = [struct.unpack(fmt, dados[base + 2 * c : base + 2 * c + 2])[0] / 65535 for c in range(canais)]
            f.write(f"{x} {y} " + " ".join(f"{v:.6f}" for v in vals) + "\n")
