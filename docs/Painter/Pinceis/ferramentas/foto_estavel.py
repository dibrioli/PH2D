#!/usr/bin/env python3
"""A guarda da FOTO VELHA do `fotos_bateria.sh`: a captura só vale se a tela PAROU e se NÃO é a cena anterior.

Achado de 2026-10-20 (06_LEIS_DAS_LACUNAS §7): 31 fotos do lote de 19/10 eram a cena ANTERIOR (máxΔ 0–14) —
o BOOKMARK responde antes de o Rebelle repintar a janela, e a foto A saía da tela antiga (a B, segundos depois,
já era a nova em 24 dos 31 casos; nos outros 7 a janela ficou parada a cena toda). Nenhum prazo fixo é seguro: a
foto espera a tela ESTÁVEL (duas capturas seguidas iguais) e DIFERENTE das fotos da cena anterior.

uso: foto_estavel.py <captura> <captura anterior> [foto da cena anterior …]
imprime: `ok` · `instavel <máxΔ>` (a tela ainda muda) · `igual <ficheiro> <máxΔ>` (é a cena anterior)
"""
import sys

import numpy as np
from PIL import Image

ESTAVEL = 2  # máxΔ entre duas capturas seguidas da tela PARADA (medido: 0 sob o Wine; 2 de folga)
IGUAL = 16   # o mesmo limiar do `leis_lacunas.py velhas` (as velhas de 19/10 tinham máxΔ 0–15)


def le(p):
    return np.asarray(Image.open(p).convert("RGB"), np.int16)


def main():
    cur, prev, antes = sys.argv[1], sys.argv[2], sys.argv[3:]
    a = le(cur)
    m = int(np.abs(a - le(prev)).max())
    if m > ESTAVEL:
        print(f"instavel {m}")
        return
    for f in antes:
        try:
            b = le(f)
        except OSError:
            continue
        m = int(np.abs(a - b).max())
        if m <= IGUAL:
            print(f"igual {f} {m}")
            return
    print("ok")


if __name__ == "__main__":
    main()
