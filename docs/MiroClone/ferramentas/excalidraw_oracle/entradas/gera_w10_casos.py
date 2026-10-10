#!/usr/bin/env python3
"""W10 (as pontas da W9): gera as duas entradas do corre_fidelidade.mjs, à PARTE do w9_casos.json.

  w10_focus_casos.json   F_*  a lei do legado {focus, gap} no 0.18.1: UMA forma por caso, setas soltas
                              de 12 ângulos × 7 focos, presas pelo FIM (e outras tantas pelo INÍCIO);
                              o editor re-liga-as ao EMPURRAR a forma. Forma × vão × rotação × cantos ×
                              espessura × proporção.
  w10_realce_casos.json  R_*  o que o editor MOSTRA a meio do arrasto de uma ponta sobre uma forma
                              (passo `captura`: o PNG e o appState de ligação/realce antes de largar).
Uso: python3 gera_w10_casos.py <pasta_entradas>
"""
import json
import math
import sys
from pathlib import Path



# As mesmas peças do gera_w9_casos.py (que escreve o w9_casos.json ao ser importado).
def base(i, t, x, y, w, h, **kw):
    e = {
        "id": i, "type": t, "x": x, "y": y, "width": w, "height": h, "angle": 0,
        "strokeColor": "#1e1e1e", "backgroundColor": "transparent", "fillStyle": "solid",
        "strokeWidth": 2, "strokeStyle": "solid", "roughness": 0, "opacity": 100,
        "groupIds": [], "frameId": None, "roundness": None, "seed": sum(map(ord, i)) * 7919 % 2**31,
        "version": 1, "versionNonce": 1, "isDeleted": False, "boundElements": None,
        "updated": 1, "link": None, "locked": False,
    }
    e.update(kw)
    return e


SHAPES = [("R", "rectangle", 160), ("E", "ellipse", 380), ("D", "diamond", 600)]


def shapes():
    return [base(sid, t, 500, cy - 60, 200, 120) for sid, t, cy in SHAPES]


def arrow(i, p0, p1, sb=None, eb=None, rnd={"type": 2}):
    rel = [[0.0, 0.0], [p1[0] - p0[0], p1[1] - p0[1]]]
    return base(i, "arrow", p0[0], p0[1], abs(rel[1][0]), abs(rel[1][1]), roundness=rnd, points=rel,
                startBinding=sb, endBinding=eb, startArrowhead=None, endArrowhead="arrow", elbowed=False)


def link(shapes_, arrows):
    by = {s["id"]: s for s in shapes_}
    for a in arrows:
        for k in ("startBinding", "endBinding"):
            b = a.get(k)
            if b and b["elementId"] in by:
                s = by[b["elementId"]]
                s["boundElements"] = (s["boundElements"] or []) + [{"id": a["id"], "type": "arrow"}]

CX, CY, R = 640.0, 400.0, 260.0
FOCOS = [-0.9, -0.6, -0.3, 0.0, 0.3, 0.6, 0.9]
ANGULOS = [0, 25, 50, 80, 90, 120, 155, 180, 215, 250, 290, 330]


def caso_focus(nome, desc, tipo, gap, w=200, h=120, angle=0.0, rnd=None, sw=2, focos=FOCOS):
    forma = base("F", tipo, CX - w / 2, CY - h / 2, w, h, angle=angle, roundness=rnd, strokeWidth=sw)
    ars = []
    for th in ANGULOS:
        s = [CX + R * math.cos(math.radians(th)), CY + R * math.sin(math.radians(th))]
        for k, f in enumerate(focos):
            b = {"elementId": "F", "focus": f, "gap": gap}
            ars.append(arrow(f"e{th}_{k}", s, [CX, CY], eb=b, rnd=None))
            ars.append(arrow(f"s{th}_{k}", [CX, CY], s, sb=b, rnd=None))
    link([forma], ars)
    return {"nome": nome, "descricao": desc, "elementos": [forma] + ars,
            "passos": PASSOS_F, "compacto": True}


# Só o fim se guarda (a saída inteira de 34 casos pesava 14 MB): abrir e montar correm sem se escrever.
PASSOS_F = [{"tipo": "restore", "guardar": False}, {"tipo": "montar", "guardar": False},
            {"tipo": "empurrar", "ids": ["F"]}]
focus = []
for tipo in ("rectangle", "ellipse", "diamond"):
    for gap in (0, 1, 6, 15):
        focus.append(caso_focus(f"F_{tipo}_g{gap}", f"{tipo} 200×120, gap {gap}", tipo, gap))
    focus.append(caso_focus(f"F_{tipo}_rot_g6", f"{tipo} 200×120 rodado 0,5 rad, gap 6", tipo, 6, angle=0.5))
focus.append(caso_focus("F_rectangle_redondo_g6", "rectangle de cantos redondos, gap 6", "rectangle", 6, rnd={"type": 3}))
focus.append(caso_focus("F_diamond_redondo_g6", "diamond de cantos redondos, gap 6", "diamond", 6, rnd={"type": 2}))
focus.append(caso_focus("F_rectangle_sw4_g6", "rectangle strokeWidth 4, gap 6", "rectangle", 6, sw=4))
focus.append(caso_focus("F_rectangle_alto_g6", "rectangle 80×220, gap 6", "rectangle", 6, w=80, h=220))
focus.append(caso_focus("F_ellipse_alto_g6", "ellipse 80×220, gap 6", "ellipse", 6, w=80, h=220))
focus.append(caso_focus("F_rectangle_gneg", "rectangle 200×120, gap −10", "rectangle", -10))
# Rodada 2 (W10): o foco ALÉM de ±1 (a recta rasante que só toca a faixa) e a elipse «redonda».
focus.append(caso_focus("F_rectangle_alem_g15", "rectangle 200×120, gap 15, focus ±1,05 e ±1,2", "rectangle", 15,
                        focos=[-1.2, -1.05, 1.05, 1.2]))
focus.append(caso_focus("F_ellipse_redondo_g6", "ellipse com roundness type 2, gap 6", "ellipse", 6, rnd={"type": 2}))
# Rodada 3: o menor vão que o escritor grava (o `gap 0` põe a ponta DENTRO, no foco).
focus.append(caso_focus("F_rectangle_g001", "rectangle 200×120, gap 0,01", "rectangle", 0.01))
focus.append(caso_focus("F_ellipse_g001", "ellipse 200×120, gap 0,01", "ellipse", 0.01))
# Rodada 4: o vão NEGATIVO na elipse (a ponta presa DENTRO dela saiu espelhada no nosso ficheiro) e no losango.
for g in (-10, -40):
    focus.append(caso_focus(f"F_ellipse_gneg{-g}", f"ellipse 200×120, gap {g}", "ellipse", g))
focus.append(caso_focus("F_diamond_gneg10", "diamond 200×120, gap −10", "diamond", -10))


# Rodada 4: as rectas PARALELAS aos eixos fora do centro (a seta vertical presa dentro da elipse saiu
# espelhada no nosso ficheiro): de cima, de baixo, da esquerda e da direita, a ±20 e ±50 do eixo, com o
# foco que a NOSSA lei dá a essa recta (rectângulo e elipse: d / hw ou d / hh).
def caso_eixos(nome, desc, tipo, gap):
    forma = base("F", tipo, CX - 100, CY - 60, 200, 120)
    ars = []
    for lado, (sx, sy) in (("cima", (0, -1)), ("baixo", (0, 1)), ("esq", (-1, 0)), ("dir", (1, 0))):
        for d in (-50, -20, 20, 50):
            if sx == 0:
                s, f = [CX + d, CY + 210 * sy], -(d / 100.0) * sy
            else:
                s, f = [CX + 210 * sx, CY + d], (d / 60.0) * sx
            ars.append(arrow(f"e{lado}{d}", s, [CX, CY], eb={"elementId": "F", "focus": f, "gap": gap}, rnd=None))
    link([forma], ars)
    return {"nome": nome, "descricao": desc, "elementos": [forma] + ars,
            "passos": PASSOS_F, "compacto": True}


for tipo in ("rectangle", "ellipse"):
    for g in (6, -10, -47):
        focus.append(caso_eixos(f"F_{tipo}_eixos_g{g}".replace("-", "neg"), f"{tipo}: rectas paralelas aos eixos, gap {g}", tipo, g))

# R — o realce: E é a elipse 200×120 de centro (600,380) (borda esquerda x=500); R o rectângulo de centro
# (600,160). Arrastar a PONTA de uma seta solta existente, ou desenhar uma nova com `a`.
realce = []
solta = arrow("aS", [330, 380], [460, 380])
for nome, alvo, espera, desc in [
    ("R_ponta_fora10", [490, 380], 50, "a ponta arrastada a 10 px FORA de E (na faixa)"),
    ("R_ponta_dentro16", [516, 380], 50, "a ponta arrastada a 16 px DENTRO de E"),
    ("R_ponta_dentro16_espera1500", [516, 380], 1500, "16 px dentro de E, parado 1500 ms"),
    ("R_ponta_miolo", [585, 400], 50, "a ponta no miolo de E, fora do centro"),
    ("R_ponta_miolo_espera1500", [585, 400], 1500, "no miolo de E, parado 1500 ms"),
    ("R_ponta_diag", [520, 120], 50, "a ponta a 20 px dentro do canto sup. esq. de R (a recta da seta é diagonal)"),
    ("R_ponta_banda_diag", [497, 130], 50, "a ponta a 3 px FORA da aresta esq. de R (na faixa), a recta diagonal"),
]:
    realce.append({"nome": nome, "descricao": desc, "elementos": shapes() + [solta],
                   "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                              {"tipo": "desenhar", "tecla": "v", "de": [400, 380], "a": [400, 380], "n": 1, "sem_escape": True},
                              {"tipo": "desenhar", "manter": True, "de": [460, 380], "a": alvo, "espera": espera, "captura": "meio"}]})
realce.append({"nome": "R_desenhar_dentro16", "descricao": "desenhar uma seta nova (tecla a) a 16 px dentro de E",
               "elementos": shapes(),
               "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                          {"tipo": "desenhar", "tecla": "a", "de": [330, 380], "a": [516, 380], "captura": "meio"}]})
realce.append({"nome": "R_desenhar_fora10", "descricao": "desenhar uma seta nova (tecla a) a 10 px fora de E",
               "elementos": shapes(),
               "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                          {"tipo": "desenhar", "tecla": "a", "de": [330, 380], "a": [490, 380], "captura": "meio"}]})
for c in realce:
    c["relogio"] = "livre"

out = Path(sys.argv[1])
for nome, casos in (("w10_focus_casos.json", focus), ("w10_realce_casos.json", realce)):
    # A varredura sem indentação (4,6 MB → ~2 MB); o realce legível.
    indent = None if nome.startswith("w10_focus") else 1
    (out / nome).write_text(json.dumps({"casos": casos}, indent=indent, ensure_ascii=False) + "\n")
    print(nome, len(casos))
