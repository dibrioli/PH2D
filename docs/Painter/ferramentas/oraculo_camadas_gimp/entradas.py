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


# ── A entrada dos efeitos de VIZINHANÇA (P4: desfoques, nitidez, bloom) ──
# Faixas horizontais de VW x FAIXA px, cada uma com listras verticais de LISTRA px (período 2·LISTRA):
# a linha do meio de cada faixa fica a FAIXA//2 px das outras, longe do alcance dos núcleos medidos.
#   0 preto | branco opacos      — luz × ecrã (a meia-sombra de um desfoque em luz clareia)
#   1 vermelho | azul opacos     — o mesmo, cromático
#   2 laranja opaco | TRANSPARENTE com cor escondida azul — pré-multiplicado × straight
#   3 branco a 140 | preto opaco — cobertura translúcida
#   4 preto opaco com UM píxel branco no centro — o impulso: o núcleo do programa, medido
VW = 48
FAIXA = 15
LISTRA = 6
VH = 5 * FAIXA
LISTRAS = [
    ((0, 0, 0, 255), (255, 255, 255, 255)),
    ((230, 60, 30, 255), (40, 90, 200, 255)),
    ((240, 180, 20, 255), (0, 0, 255, 0)),
    ((255, 255, 255, 140), (0, 0, 0, 255)),
]


def entrada_vizinhanca():
    px = bytearray()
    for f in range(5):
        for y in range(FAIXA):
            for x in range(VW):
                if f < 4:
                    px += bytes(LISTRAS[f][(x // LISTRA) % 2])
                elif y == FAIXA // 2 and x == VW // 2:
                    px += bytes((255, 255, 255, 255))
                else:
                    px += bytes((0, 0, 0, 255))
    return bytes(px)
