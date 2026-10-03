# As entradas NOSSAS dos oraculos das camadas (GIMP e Krita leem a MESMA grelha): uma base e UMA
# camada por cima, W x H px. Coluna x -> alfa da camada = ALFAS[x]; linha y -> (alfa da base, cor da
# base, cor da camada) = produto BASE_ALFAS x BASES x TOPOS.

ALFAS = [0, 16, 32, 48, 64, 80, 96, 112, 128, 144, 160, 176, 192, 208, 224, 240, 255]
BASE_ALFAS = [255, 140]
BASES = [(0, 0, 0), (255, 255, 255), (128, 128, 128), (51, 51, 51), (230, 60, 30), (40, 90, 200)]
TOPOS = [(0, 0, 0), (255, 255, 255), (191, 191, 191), (70, 70, 70), (30, 200, 80), (240, 180, 20)]
W = len(ALFAS)
H = len(BASE_ALFAS) * len(BASES) * len(TOPOS)


def entradas():
    base = bytearray()
    topo = bytearray()
    for ba in BASE_ALFAS:
        for b in BASES:
            for t in TOPOS:
                for a in ALFAS:
                    base += bytes([b[0], b[1], b[2], ba])
                    topo += bytes([t[0], t[1], t[2], a])
    return bytes(base), bytes(topo)
