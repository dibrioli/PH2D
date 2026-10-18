#!/usr/bin/env python3
"""A varredura do TILT do lápis no Rebelle (18/10): quanto a inclinação da caneta alarga o traço e lhe
muda a textura, ao LONGO do traço contra ATRAVÉS dele, e onde acaba a zona morta. Eventos Motion IO + a régua.

uso: tilt_lapis_rebelle.py eventos <dir>      → <dir>/blocos.txt + <bloco>_0_{0,1}.json
     tilt_lapis_rebelle.py regua <fixtures>   → a tabela (e <fixtures>/medidas.json), medida SÓ sobre as fixtures

Pencil › Graphite Pencil/8B, size_px 40, azul (23, 101, 179), semente 7, papel Default/EX00 White Simple — os
mesmos de `fixtures/rebelle/penciltilt.png`. Cada traço: recto horizontal de 500 px (x 60 → 560, um ponto a cada
3 px), UM pen_tilt constante em todos os eventos (salvo as rampas de C e E). O SET_BRUSH vai COMPLETO antes
de cada traço (só vale para o 1.º — medido 07/10). Cinco telas 1000×640:
  A — tilt AO LONGO do eixo do traço (x, 0), x ∈ {0, 20, 35, 45, 50, 55, 60, 63}, y = 45 + 75·i;
  B — tilt ATRAVÉS do traço (0, y): controlo (0, 0), 35, 40, 45; y = 80 + 160·i;
  C — rampa de pressão 0,15→1→0,15 (p = 0,15 + 0,85·sin πf) a tilt (55, 0) e (0, 0); o tilt em RAMPA linear
      0→63° ao longo do traço a pressão 1, ao longo (t, 0) e através (0, t);
  D — através 50, 55, 63 e a diagonal (39, 39);
  E — as rampas de DESCIDA 63→0 (ao longo, através), a rampa de pressão a (0, 55), e o PRESS a (0, 0) com
      todo o resto a (0, 55). O controlo (55, 0) mora em A (linha 55) e em C.
  F — as rampas de C/E enviadas a 20 ms por evento (A–E: 2 ms) e uma rampa através em degraus de 9°;
  G — degraus 0 → 55 e 55 → 0 a x 200, e o degrau e a rampa de descida a 6 px por evento.
`TILT_BLOCOS="B D E" tilt_lapis_rebelle.sh` repinta só esses blocos; a régua mede sempre os cinco.
"""
import json
import math
import os
import sys

AZUL = {"r": 23, "g": 101, "b": 179}
PRESET, MAX_PX, SIZE_PX = "Graphite Pencil/8B", 80.0, 40.0
X0, X1, PASSO = 60.0, 560.0, 3.0

cheia = lambda f: 1.0
rampa_p = lambda f: 0.15 + 0.85 * math.sin(math.pi * f)
const = lambda tx, ty: (lambda f: (tx, ty))

# bloco → [(rótulo, y, pressão(f), tilt(f))], f = fracção do comprimento ∈ [0, 1]
BLOCOS = {
    "A": [(f"ao_longo_{x}", 45 + 75 * i, cheia, const(float(x), 0.0))
          for i, x in enumerate((0, 20, 35, 45, 50, 55, 60, 63))],
    # ⚠️ 1.ª corrida (18/10) com B a 100 px de passo: o através a 55° e 63° sai com ~95 px e os dois
    # SOBREPUSERAM-SE ⇒ o através vai a 160 px de passo, em duas telas (B e D).
    "B": [(n, 80 + 160 * i, cheia, const(tx, ty)) for i, (n, tx, ty) in enumerate((
        ("controlo_0_0", 0.0, 0.0), ("atraves_35", 0.0, 35.0), ("atraves_40", 0.0, 40.0),
        ("atraves_45", 0.0, 45.0)))],
    "C": [("rampa_p_tilt_55_0", 80, rampa_p, const(55.0, 0.0)),
          ("rampa_p_tilt_0_0", 240, rampa_p, const(0.0, 0.0)),
          ("rampa_tilt_ao_longo_0_63", 400, cheia, lambda f: (round(63.0 * f, 2), 0.0)),
          ("rampa_tilt_atraves_0_63", 560, cheia, lambda f: (0.0, round(63.0 * f, 2)))],
    "D": [(n, 80 + 160 * i, cheia, const(tx, ty)) for i, (n, tx, ty) in enumerate((
        ("atraves_50", 0.0, 50.0), ("atraves_55", 0.0, 55.0), ("atraves_63", 0.0, 63.0),
        ("diagonal_39_39", 39.0, 39.0)))],
    # A rampa de SUBIDA (C) quase não alarga ⇒ o tilt é lido no PRESS? A rampa de DESCIDA e o salto
    # 0 → 55 depois do PRESS separam «preso no press» de «filtrado no tempo».
    "E": [("rampa_tilt_ao_longo_63_0", 80, cheia, lambda f: (round(63.0 * (1 - f), 2), 0.0)),
          ("rampa_tilt_atraves_63_0", 240, cheia, lambda f: (0.0, round(63.0 * (1 - f), 2))),
          ("rampa_p_tilt_0_55", 400, rampa_p, const(0.0, 55.0)),
          ("press_0_depois_0_55", 560, cheia, lambda f: (0.0, 0.0) if f == 0 else (0.0, 55.0))],
    # A rampa de subida fica estreita e a de descida fica larga até ~26° (C, E) — mas o salto 0 → 55 alarga
    # logo. F manda as rampas a 20 ms por evento (10× mais devagar que A–E) e uma rampa em DEGRAUS de 9°:
    # filtro no TEMPO, ou por evento?
    "F": [("rampa_tilt_atraves_0_63_lenta", 80, cheia, lambda f: (0.0, round(63.0 * f, 2))),
          ("rampa_tilt_atraves_63_0_lenta", 240, cheia, lambda f: (0.0, round(63.0 * (1 - f), 2))),
          ("degraus_9_atraves_0_63_lenta", 400, cheia, lambda f: (0.0, 9.0 * min(7, int(8 * f)))),
          ("rampa_tilt_ao_longo_0_63_lenta", 560, cheia, lambda f: (round(63.0 * f, 2), 0.0))],
    # F saiu BYTE A BYTE igual a E na descida (20 ms × 2 ms) ⇒ o filtro não é no tempo. Lido como uma média
    # exponencial por evento (α ≈ 1/54) que arranca no 1.º MOVE, ela prevê: degrau 0 → 55 a x 200 alarga só
    # ~250 px depois; degrau 55 → 0 estreita ~40 px depois (assimetria); a 6 px por evento, se o filtro é por
    # EVENTO, a demora em px dobra; se é por DISTÂNCIA, fica igual.
    "G": [("degrau_0_55_em_x200", 80, cheia, lambda f: (0.0, 0.0 if X0 + (X1 - X0) * f < 200 else 55.0)),
          ("degrau_55_0_em_x200", 240, cheia, lambda f: (0.0, 55.0 if X0 + (X1 - X0) * f < 200 else 0.0)),
          ("degrau_0_55_em_x200_6px", 400, cheia, lambda f: (0.0, 0.0 if X0 + (X1 - X0) * f < 200 else 55.0)),
          ("rampa_tilt_atraves_63_0_6px", 560, cheia, lambda f: (0.0, round(63.0 * (1 - f), 2)))],
}
# ms entre eventos ao enviar os traços de cada bloco (o ws_eventos.mjs)
INTERVALO = {"F": 20}
# px entre eventos dos traços que não usam o passo de rato (3 px)
PASSO_DE = {"degrau_0_55_em_x200_6px": 6.0, "rampa_tilt_atraves_63_0_6px": 6.0}
# meia altura do recorte de cada linha (não pode apanhar o vizinho); x 0–660 (o tilt ao longo estica as pontas
# para lá de 40–580)
MEIA = {"A": 37, "B": 75, "C": 75, "D": 75, "E": 75, "F": 75, "G": 75}
CROP_X, CROP_W = 0, 660


def set_brush():
    return {"event_type": "SET_BRUSH", "tool": "PENCIL", "preset": PRESET, "size_px": SIZE_PX,
            "size": round(100.0 * (SIZE_PX / MAX_PX) ** (2.0 / 3.0), 1), "pressure": 100, "color": AZUL}


def ponto(tipo, x, y, pr, tilt):
    return {"event_type": tipo, "pos": {"x": round(x, 2), "y": float(y)},
            "pen_tilt": {"x": tilt[0], "y": tilt[1]}, "rotation": 0.0, "pressure": round(pr, 3)}


def traco(ev, y, p, t, passo=PASSO):
    ev.append(set_brush())
    n = int(round((X1 - X0) / passo))
    fs = [k / n for k in range(n + 1)]
    xs = [X0 + (X1 - X0) * f for f in fs]
    ev.append(ponto("POINTER_PRESS", xs[0], y, p(0.0), t(0.0)))
    ev += [ponto("POINTER_MOVE", x, y, p(f), t(f)) for x, f in zip(xs[1:], fs[1:])]
    ev.append(ponto("POINTER_RELEASE", xs[-1], y, 0.0, t(1.0)))


def tela():
    return [{"event_type": "NEW_ARTWORK", "width": 1000, "height": 640, "units": "px", "dpi": 72,
             "paper": {"preset": "Default/EX00 White Simple", "color": {"r": 255, "g": 255, "b": 255},
                       "deckled_edges": False, "paper_scale": 100, "visible": True}},
            {"event_type": "SET_RANDOM_SEED", "seed": 7},
            {"event_type": "SET_CANVAS_TILT", "tilt": {"x": 0, "y": 0}, "enabled": False}]


def eventos(d):
    os.makedirs(d, exist_ok=True)
    with open(os.path.join(d, "blocos.txt"), "w") as fb:
        for b, linhas in BLOCOS.items():
            ev = []
            for nome, y, p, t in linhas:
                traco(ev, y, p, t, PASSO_DE.get(nome, PASSO))
            json.dump({"frames": [{"events": tela() + [set_brush()]}]}, open(os.path.join(d, f"{b}_0_0.json"), "w"))
            json.dump({"frames": [{"events": ev}]}, open(os.path.join(d, f"{b}_0_1.json"), "w"))
            fb.write(f"{b} 1 0 {INTERVALO.get(b, 2)}\n")


def perfil_largura(png, y, meia):
    """Largura a meia altura (a da régua mede_traco) a cada 25 px de x, mediana numa janela de ±6 colunas."""
    import numpy as np
    from PIL import Image
    a = np.asarray(Image.open(png).convert("RGB"), dtype=np.float32)[y - meia:y + meia]
    bg = np.median(np.concatenate([a[:2].reshape(-1, 3), a[-2:].reshape(-1, 3)]), axis=0)
    c = np.abs(a - bg).max(axis=2) / 255.0
    out = []
    for x in range(int(X0), int(X1) + 1, 25):
        ws = []
        for cx in range(max(0, x - 6), x + 7):
            col = c[:, cx]
            m = col.max()
            if m < 0.08:
                ws.append(0.0); continue
            ys = np.where(col >= 0.5 * m)[0]
            ws.append(float(ys[-1] - ys[0] + 1))
        out.append((x, float(np.median(ws))))
    return out


def regua(fx):
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    from mede_traco import mede
    res = {}
    print("bloco | traço | y | larg. mediana | larg. 5/50/95 | textura % | borda esc. | cob. miolo | orla 10–90 | comprimento")
    for b, linhas in BLOCOS.items():
        png = os.path.join(fx, f"tilt_lapis_{b}.png")
        for nome, y, _, _ in linhas:
            m = MEIA[b]
            r = mede(png, (CROP_X, y - m, CROP_W, 2 * m))
            res[f"{b}_{nome}"] = dict(r, bloco=b, y=y, crop=[CROP_X, y - m, CROP_W, 2 * m])
            if "erro" in r:
                print(f"{b} | {nome} | {y} | {r['erro']}"); continue
            l = r["largura_5_50_95"]
            print(f"{b} | {nome} | {y} | {r['largura_mediana_px']:.1f} | {l[0]:.0f}/{l[1]:.0f}/{l[2]:.0f} | "
                  f"{r['textura_pct']:.1f} | {r['borda_escura']:.2f} | {r['cobertura_miolo']:.3f} | "
                  f"{r['orla_10_90_px']:.1f} | {r['comprimento_px']}")
    print("\nperfil da largura (px a meia altura, a cada 25 px de x, x 60 → 560) — blocos C, E, F e G")
    for b in ("C", "E", "F", "G"):
        for nome, y, p, t in BLOCOS[b]:
            pf = perfil_largura(os.path.join(fx, f"tilt_lapis_{b}.png"), y, MEIA[b])
            res[f"{b}_{nome}"]["perfil_largura"] = [
                {"x": x, "f": round((x - X0) / (X1 - X0), 3), "pressao": round(p((x - X0) / (X1 - X0)), 3),
                 "tilt": list(t((x - X0) / (X1 - X0))), "largura_px": w} for x, w in pf]
            print(f"{b} {nome}: " + " ".join(f"{w:.0f}" for _, w in pf))
    json.dump(res, open(os.path.join(fx, "medidas.json"), "w"), indent=1, ensure_ascii=False)
    return res


if __name__ == "__main__":
    {"eventos": eventos, "regua": regua}[sys.argv[1]](sys.argv[2])
