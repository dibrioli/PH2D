"""P11 T7: fixturas extra sobre trial_slices.py. uso: python3 -I trial_slices_tool_fixturas.py <dir>
odd.ase  = coordenadas impares (bounds, centro, pivot) para ver o arredondamento de Sprite Size 50 %.
trim.ase = arte opaca so em x,y em [8,56) para Trim ter o que cortar; uma slice atravessa a margem, outra fica toda fora."""
import os, struct, sys, zlib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_slices as t

W, H = t.W, t.H
ODD = {0: [
    {'name': 'odd', 'flags': 3, 'color': (0, 200, 0, 255), 'keys': [t.k(0, 5, 7, 15, 9, center=(3, 1, 9, 7), pivot=(7, 3))]},
    {'name': 'odd2', 'flags': 3, 'color': (0, 0, 255, 255), 'keys': [t.k(0, 33, 11, 13, 21, center=(1, 5, 11, 11), pivot=(-3, 25))]},
]}
TRIM = {0: [
    {'name': 'cross', 'flags': 3, 'color': (0, 200, 0, 255), 'keys': [t.k(0, 4, 4, 24, 24, center=(8, 8, 8, 8), pivot=(12, 12))]},
    {'name': 'outside', 'color': (0, 0, 255, 255), 'keys': [t.k(0, 58, 58, 4, 4)]},
]}


def arte_trim(frame):
    px = bytearray()
    for y in range(H):
        for x in range(W):
            px += bytes((70, 110, 160, 255)) if 8 <= x < 56 and 8 <= y < 56 else bytes(4)
    return bytes(px)


if __name__ == '__main__':
    d = sys.argv[1]
    os.makedirs(d, exist_ok=True)
    t.escreve(os.path.join(d, 'odd.ase'), 2, ODD)
    t.arte = arte_trim
    t.escreve(os.path.join(d, 'trim.ase'), 2, TRIM)
