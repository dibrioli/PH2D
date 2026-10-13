#!/usr/bin/env python3
"""Pontas TEXTURADAS no Rebelle (13/10, plano 27 W0): a textura da ponta MULTIPLICA o depósito ou é cortada
por um limiar? Eventos Motion IO + a régua.

uso: ponta_textura_rebelle.py eventos <dir>          → <dir>/blocos.txt + <bloco>_<k>_{0,1}.json + sim.json
     ponta_textura_rebelle.py monta <dir> <fixtures>  → das telas <dir>/ponta_<nome>_{a,b}.png (1000×640) às
                                                        fixtures <fixtures>/ponta_textura_<nome>_<px>.png
     ponta_textura_rebelle.py regua <fixtures>        → a tabela, medida SÓ sobre as fixtures

O gesto: 4 traços horizontais rectos de 220 px (x 390 → 610, um ponto a cada 3 px, pressão 1,0) numa tela
1000×640, um por linha: y 110 = 20 px água baixa · 250 = 40 px baixa · 390 = 20 px alta · 530 = 40 px alta.
O SET_BRUSH vai COMPLETO antes de cada gesto (só vale para o 1.º traço — medido 07/10), com a MESMA opacidade
em todos os presets (70) para que a única diferença seja a ponta.
"""
import json
import os
import sys

AZUL_ESCURO = {"r": 20, "g": 30, "b": 120}
OPACIDADE = 70.0
AGUAS = {"baixa": 10.0, "alta": 50.0}
X0, X1 = 390.0, 610.0
LINHAS = [(110, 20, "baixa"), (250, 40, "baixa"), (390, 20, "alta"), (530, 40, "alta")]
# nome → (preset "Grupo/Nome", max_brush_size do preset, porquê)
PRESETS = {
    "round": ("Watercolor/Round", 300, "controlo: Shape_0 = disco liso (miolo 1,00, dp 0,00)"),
    "fan": ("Watercolor/Fan", 300, "Shape_0 = leque de cerdas (miolo: média 0,44, 50 % vazio)"),
    "gouachedry": ("Gouache/Gouache Dry", 300, "Shape_0 = manchas de pincel seco (miolo: média 0,12, 74 % vazio)"),
    "granulation": ("Granulation/Granulation", 700, "Shape_0 = grão/salpico (miolo: média 0,57, 28 % vazio)"),
    "flatdry": ("Watercolor/Flat Dry", 300, "Shape_0 cheio (miolo 0,91) + Texture_0 ruído de média 0,49 dentro"),
}


def set_brush(preset, mx, size_px, agua):
    return {"event_type": "SET_BRUSH", "tool": "WATERCOLOR", "preset": preset, "size_px": float(size_px),
            "size": round(100.0 * (size_px / mx) ** (2.0 / 3.0), 1), "pressure": 100,
            "color": AZUL_ESCURO, "water": agua, "opacity": OPACIDADE}


def ponto(tipo, x, y, pr):
    return {"event_type": tipo, "pos": {"x": round(x, 2), "y": round(y, 2)}, "pen_tilt": {"x": 0, "y": 0},
            "rotation": 0.0, "pressure": pr}


def tela():
    return [{"event_type": "NEW_ARTWORK", "width": 1000, "height": 640, "units": "px", "dpi": 72,
             "paper": {"preset": "Default/EX00 White Simple", "color": {"r": 255, "g": 255, "b": 255},
                       "deckled_edges": False, "paper_scale": 100, "visible": True}},
            {"event_type": "SET_RANDOM_SEED", "seed": 7},
            {"event_type": "SET_CANVAS_TILT", "tilt": {"x": 0, "y": 0}, "enabled": False}]


def traco(ev, sb, x0, y):
    """O SET_BRUSH completo + um recto de 220 px (74 pontos a 3 px) em y, de x0 a x0 + 219."""
    ev.append(sb)
    xs = [x0 + 3 * k for k in range(74)]
    ev.append(ponto("POINTER_PRESS", xs[0], y, 1.0))
    ev += [ponto("POINTER_MOVE", x, y, 1.0) for x in xs[1:]]
    ev.append(ponto("POINTER_RELEASE", xs[-1], y, 0.0))


def eventos(d):
    """Cada BLOCO é uma tela; cada PARTE dele começa por um SET_BRUSH de aquecimento num ficheiro à parte
    (`<bloco>_<k>_0.json`, com a tela na parte 0) e o shell espera 2,5 s antes dos traços (`_1.json`).
    ⚠️ Medido 13/10 (1.ª corrida, sem aquecimento): o 1.º traço depois de TROCAR de preset saiu às vezes com
    o preset ANTERIOR (o Fan a 20 px com Σ 1,56× do controlo e cobertura 0,94; a Granulation a 20 px com
    1,96×; o Round a 100 % com um fio de 4 px) — a troca de preset carrega devagar e os traços chegam antes."""
    os.makedirs(d, exist_ok=True)
    blocos = []
    for nome, (preset, mx, _) in PRESETS.items():
        ev = []
        for y, size, agua in LINHAS:
            traco(ev, set_brush(preset, mx, size, AGUAS[agua]), X0, y)
        blocos.append((nome, [(set_brush(preset, mx, LINHAS[0][1], AGUAS[LINHAS[0][2]]), ev)], 1))
    # A varredura da OPACIDADE (40 px, água 10): Granulation sozinha numa tela; Gouache Dry · Fan · Round
    # em três colunas noutra (x 60 · 390 · 720). Uma linha por opacidade, y 80 · 200 · 320 · 440 · 560.
    for nome, colunas in (("op_granulation", [(390, "granulation")]),
                          ("op_tres", [(60, "gouachedry"), (390, "fan"), (720, "round")])):
        partes = []
        for x0, chave in colunas:
            preset, mx, _ = PRESETS[chave]
            ev = []
            for y, op in zip(OP_LINHAS, OPACIDADES):
                sb = set_brush(preset, mx, 40, AGUAS["baixa"]); sb["opacity"] = op
                traco(ev, sb, x0, y)
            partes.append((ev[0], ev))
        blocos.append((nome, partes, 0))
    with open(os.path.join(d, "blocos.txt"), "w") as f:
        for nome, partes, com_sim in blocos:
            for k, (aquece, ev) in enumerate(partes):
                pre = (tela() if k == 0 else []) + [aquece]
                json.dump({"frames": [{"events": pre}]}, open(os.path.join(d, f"{nome}_{k}_0.json"), "w"))
                json.dump({"frames": [{"events": ev}]}, open(os.path.join(d, f"{nome}_{k}_1.json"), "w"))
            f.write(f"{nome} {len(partes)} {com_sim}\n")
    json.dump({"frames": [{"events": [{"event_type": "SIMULATION", "repeats": 400}, {"event_type": "DRY"}]}]},
              open(os.path.join(d, "sim.json"), "w"))


OP_LINHAS = (80, 200, 320, 440, 560)
OPACIDADES = (100.0, 85.0, 70.0, 55.0, 40.0)


def regua_opacidade(dest):
    """Sobre as fixtures inteiras (1000×640) da varredura: Σ tinta, cobertura e miolo vazio por opacidade."""
    import numpy as np
    from PIL import Image
    print("varredura da opacidade (40 px, água 10) | opacidade | Σ tinta | máx | cobertura | miolo vazio | miolo média")
    for fx, colunas in (("ponta_textura_opacidade_granulation_40.png", [(390, "granulation")]),
                        ("ponta_textura_opacidade_gouachedry_fan_round_40.png",
                         [(60, "gouachedry"), (390, "fan"), (720, "round")])):
        t = 255 - np.asarray(Image.open(os.path.join(dest, fx)).convert("RGB")).astype(int).min(axis=2)
        for x0, chave in colunas:
            for y, op in zip(OP_LINHAS, OPACIDADES):
                faixa = t[y - 50:y + 50, max(0, x0 - 30):x0 + 250]
                corpo = t[y - 20:y + 21, x0 + 10:x0 + 211]
                miolo = t[y - 9:y + 10, x0 + 10:x0 + 211]
                print(f"{chave} | {op:.0f} | {int(faixa.sum())} | {int(faixa.max())} | {(corpo > 8).mean():.3f} | "
                      f"{(miolo < 8).mean():.3f} | {miolo.mean():.1f}")


# A fixture: por preset × tamanho, uma grelha 2×2 de células 320×100 recortadas da tela (x 340–660,
# y = eixo ± 50): colunas = estado a (molhado, ~2 s depois do último Up) | b (SIMULATION 400 + DRY);
# linhas = água baixa | alta. O traço fica em x 50–270, y 50 de cada célula.
CEL_X0, CEL_W, CEL_H = 340, 320, 100


def monta(d, dest):
    import numpy as np
    from PIL import Image
    for nome in PRESETS:
        fotos = {e: np.asarray(Image.open(os.path.join(d, f"ponta_{nome}_{e}.png")).convert("RGB")) for e in "ab"}
        for size in (20, 40):
            g = np.full((2 * CEL_H, 2 * CEL_W, 3), 255, np.uint8)
            for li, (y, sz, agua) in enumerate(l for l in LINHAS if l[1] == size):
                for ci, e in enumerate("ab"):
                    g[li * CEL_H:(li + 1) * CEL_H, ci * CEL_W:(ci + 1) * CEL_W] = \
                        fotos[e][y - CEL_H // 2:y + CEL_H // 2, CEL_X0:CEL_X0 + CEL_W]
            Image.fromarray(g).save(os.path.join(dest, f"ponta_textura_{nome}_{size}.png"))


def mede(tinta, size):
    """`tinta` = 255 − min(rgb) de UMA célula; o traço vai de x 50 a 270 com o eixo perto de y 50."""
    import numpy as np
    x0, x1, y0 = X0 - CEL_X0, X1 - CEL_X0, CEL_H // 2
    r = size / 2.0
    ys = np.arange(int(y0 - 2 * r), int(y0 + 2 * r) + 1)
    peso = tinta[ys][:, int(x0) + 10:int(x1) - 10].sum(axis=1)
    yc = float((ys * peso).sum() / peso.sum()) if peso.sum() > 0 else float(y0)
    corpo = tinta[int(round(yc - r)):int(round(yc + r)) + 1, int(x0) + 10:int(x1) - 9]
    miolo = tinta[int(round(yc - r / 2)) + 1:int(round(yc + r / 2)), int(x0) + 10:int(x1) - 9]
    eixo = tinta[int(round(yc)), int(x0) + 10:int(x1) - 9]
    return {"y_eixo": round(yc - y0, 1), "soma_tinta": int(tinta.sum()),
            "cobertura": round(float((corpo > 8).mean()), 3),
            "miolo_vazio": round(float((miolo < 8).mean()), 3), "miolo_media": round(float(miolo.mean()), 1),
            "eixo_media": round(float(eixo.mean()), 1), "eixo_dp": round(float(eixo.std()), 1),
            "eixo_cv": round(float(eixo.std() / max(eixo.mean(), 1e-9)), 3)}


def regua(dest):
    import numpy as np
    from PIL import Image
    res = {}
    for nome in PRESETS:
        for size in (20, 40):
            png = os.path.join(dest, f"ponta_textura_{nome}_{size}.png")
            if not os.path.exists(png):
                continue
            t = 255 - np.asarray(Image.open(png).convert("RGB")).astype(int).min(axis=2)
            for li, agua in enumerate(("baixa", "alta")):
                for ci, e in enumerate("ab"):
                    cel = t[li * CEL_H:(li + 1) * CEL_H, ci * CEL_W:(ci + 1) * CEL_W]
                    res[f"{nome}_{size}_{agua}_{e}"] = mede(cel, size)
    for k, m in res.items():
        nome, rest = k.split("_", 1)
        c = res.get(f"round_{rest}")
        m["razao_controlo"] = round(m["soma_tinta"] / c["soma_tinta"], 3) if c and c["soma_tinta"] else None
    print("traço (preset_px_água_estado) | Σ tinta | razão/controlo | cobertura | miolo vazio | miolo média | "
          "eixo média | eixo dp | eixo cv | Δy eixo")
    for k, m in res.items():
        print(f"{k} | {m['soma_tinta']} | {m['razao_controlo']} | {m['cobertura']} | {m['miolo_vazio']} | "
              f"{m['miolo_media']} | {m['eixo_media']} | {m['eixo_dp']} | {m['eixo_cv']} | {m['y_eixo']}")
    return res


def cabecalhos(d, dest):
    """Acrescenta (ou reescreve) a entrada de cada fixture `ponta_textura_*` no `cabecalhos.json`."""
    import collections
    import hashlib
    cab_path = os.path.join(dest, "cabecalhos.json")
    cab = json.load(open(cab_path))
    comum = {"tool": "WATERCOLOR", "pressure": 100, "seed": 7, "paper": "Default/EX00 White Simple",
             "canvas": [1000, 640], "cor": [AZUL_ESCURO[c] for c in "rgb"], "data": "2026-10-13",
             "rebelle": "8.3.4 (teste, sob Wine 11, porta WebSocket)",
             "comando": "bash docs/Painter/NewFeaturesWetPaint/ferramentas/ponta_textura_rebelle.sh"}

    def conta(bloco, partes):
        n = collections.Counter()
        for k in range(partes):
            for suf in "01":
                for e in json.load(open(os.path.join(d, f"{bloco}_{k}_{suf}.json")))["frames"][0]["events"]:
                    n[e["event_type"]] += 1
        return dict(sorted(n.items()))

    for nome, (preset, mx, porque) in PRESETS.items():
        for size in (20, 40):
            fx = f"ponta_textura_{nome}_{size}"
            ev = conta(nome, 1)
            ev["SIMULATION"] = 1; ev["DRY"] = 1
            cab[fx] = dict(comum, preset=preset, size_px=float(size),
                           size_slider=round(100.0 * (size / mx) ** (2.0 / 3.0), 1),
                           extras={"opacity": OPACIDADE, "water": {"baixa": AGUAS["baixa"], "alta": AGUAS["alta"]}},
                           ponta=porque,
                           gesto="recto horizontal de 220 px (x 390→610, ponto a cada 3 px, pressão 1,0); "
                                 "SET_BRUSH completo antes de cada gesto, aquecimento de 2,5 s depois da troca de preset",
                           layout="grelha 2×2 de células 320×100 recortadas da tela a zoom 100 % (x 340–660, eixo ± 50): "
                                  "colunas a = molhado ~2 s depois do último Up | b = SIMULATION 400 + DRY; "
                                  "linhas água baixa (10) | alta (50); o traço fica em x 50–270, y 50 de cada célula",
                           simulation=[400], eventos=ev,
                           sha256=hashlib.sha256(open(os.path.join(dest, fx + ".png"), "rb").read()).hexdigest())
    for fx, bloco, partes, chaves in (
            ("ponta_textura_opacidade_granulation_40", "op_granulation", 1, ["granulation"]),
            ("ponta_textura_opacidade_gouachedry_fan_round_40", "op_tres", 3, ["gouachedry", "fan", "round"])):
        cab[fx] = dict(comum, preset=[PRESETS[c][0] for c in chaves], size_px=40.0,
                       size_slider=[round(100.0 * (40 / PRESETS[c][1]) ** (2.0 / 3.0), 1) for c in chaves],
                       extras={"water": AGUAS["baixa"], "opacity": list(OPACIDADES)},
                       gesto="recto horizontal de 220 px (74 pontos a 3 px), pressão 1,0, um por opacidade",
                       layout="a tela inteira 1000×640 a zoom 100 %, molhado (~2 s depois do último Up, sem simulação); "
                              f"linhas y {list(OP_LINHAS)} = opacidade {list(OPACIDADES)}; colunas x0 "
                              f"{[60, 390, 720][:len(chaves)] if len(chaves) > 1 else [390]} = {chaves}",
                       simulation=[], eventos=conta(bloco, partes),
                       sha256=hashlib.sha256(open(os.path.join(dest, fx + ".png"), "rb").read()).hexdigest())
    json.dump(cab, open(cab_path, "w", encoding="utf-8"), indent=1, ensure_ascii=False)
    open(cab_path, "a").write("\n")


if __name__ == "__main__":
    if sys.argv[1] == "eventos":
        eventos(sys.argv[2])
    elif sys.argv[1] == "monta":
        monta(sys.argv[2], sys.argv[3])
    elif sys.argv[1] == "cabecalhos":
        cabecalhos(sys.argv[2], sys.argv[3])
    elif sys.argv[1] == "regua":
        regua(sys.argv[2])
        regua_opacidade(sys.argv[2])
