#!/usr/bin/env python3
"""W10 — a lei do legado {focus, gap} do 0.18.1, das saídas saidas/w10_F_*.0181.json.

Para cada seta presa (pelo fim `e…` ou pelo início `s…`): A = o ponto vizinho da ponta, T = a ponta depois
de o editor a re-ligar (passo `empurrar`), tudo no referencial LOCAL da forma (centro na origem, sem rotação).
Hipótese do foco: focus = d / h(n) — d a distância com sinal do centro à recta A→T (cruz(T−A, C−A)/|T−A|)
e h(n) a função de suporte na normal da recta: rectângulo E elipse hw|nx|+hh|ny| (a da CAIXA), losango
max(hw|nx|, hh|ny|) — ±1 = a recta que toca um canto da caixa (do losango).
Hipóteses do vão (a que distância da forma a ponta fica), medidas no ponto T: `cheb` (o rectângulo
alargado de g, cantos vivos), `verdadeira` (a distância euclidiana ao contorno), `semi` (a elipse de
semi-eixos + g), `suporte` (o losango cujas arestas se afastam g).
Uso: python3 mede_w10.py
"""
import json
import math
from pathlib import Path

S = Path(__file__).resolve().parent / "saidas"


def local(p, c, ang):
    x, y = p[0] - c[0], p[1] - c[1]
    co, si = math.cos(-ang), math.sin(-ang)
    return (x * co - y * si, x * si + y * co)


def support(tipo, hw, hh, n):
    nx, ny = abs(n[0]), abs(n[1])
    if tipo in ("rectangle", "ellipse"):  # ⚠️ a elipse normaliza pela CAIXA (medido), não pela elipse
        return hw * nx + hh * ny
    return max(hw * nx, hh * ny)


def dist_seg(p, a, b):
    ab = (b[0] - a[0], b[1] - a[1])
    t = max(0, min(1, ((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / (ab[0] ** 2 + ab[1] ** 2)))
    return math.hypot(p[0] - a[0] - t * ab[0], p[1] - a[1] - t * ab[1])


def gaps(tipo, hw, hh, t):
    x, y = abs(t[0]), abs(t[1])
    out = {}
    if tipo == "rectangle":
        out["cheb"] = max(x - hw, y - hh)
        dx, dy = max(x - hw, 0), max(y - hh, 0)
        out["verdadeira"] = math.hypot(dx, dy) if (dx > 0 and dy > 0) else max(x - hw, y - hh)
    elif tipo == "ellipse":
        # semi: a elipse (hw+g, hh+g) que passa por T
        lo, hi = -min(hw, hh) + 1e-9, 1e4
        for _ in range(200):
            g = (lo + hi) / 2
            if (x / (hw + g)) ** 2 + (y / (hh + g)) ** 2 > 1:
                lo = g
            else:
                hi = g
        out["semi"] = (lo + hi) / 2
        # verdadeira: amostragem fina do contorno
        best = min(math.hypot(x - hw * math.cos(a), y - hh * math.sin(a))
                   for a in (k * math.pi / 2 / 20000 for k in range(20001)))
        inside = (x / hw) ** 2 + (y / hh) ** 2 < 1
        out["verdadeira"] = -best if inside else best
    else:
        nrm = math.hypot(hh, hw)
        out["suporte"] = (x * hh + y * hw - hw * hh) / nrm
        d = dist_seg((x, y), (hw, 0), (0, hh))
        inside = x * hh + y * hw < hw * hh
        out["verdadeira"] = -d if inside else d
    return out


def main():
    print(f"{'caso':28} {'n':>4} {'fora':>4} {'|Δfocus|max':>12}  vão: hipótese → |Δ|max")
    for f in sorted(S.glob("w10_F_*.0181.json")):
        d = json.loads(f.read_text())
        st = {p["passo"]["tipo"]: p for p in d["passos"]}
        els = {e["id"]: e for e in st["empurrar"]["elementos"]}
        F = els["F"]
        hw, hh = F["width"] / 2, F["height"] / 2
        c = (F["x"] + hw, F["y"] + hh)
        n = fora = 0
        dfoc = 0.0
        dg = {}
        for a in els.values():
            if a["type"] != "arrow":
                continue
            pts = [(a["x"] + p[0], a["y"] + p[1]) for p in a["points"]]
            fim = a["id"][0] == "e"
            b = a["endBinding" if fim else "startBinding"]
            T, A = (pts[-1], pts[-2]) if fim else (pts[0], pts[1])
            T, A = local(T, c, F["angle"]), local(A, c, F["angle"])
            if math.hypot(*T) < 1e-6:  # não re-ligada: a recta do foco não toca a forma + vão (fica a ponta antiga, o centro)
                fora += 1
                continue
            u = (T[0] - A[0], T[1] - A[1])
            L = math.hypot(*u)
            sd = (u[0] * (0 - A[1]) - u[1] * (0 - A[0])) / L
            nrm = (-u[1] / L, u[0] / L)
            fp = sd / support(F["type"], hw, hh, nrm)
            dfoc = max(dfoc, abs(fp - b["focus"]))
            for k, v in gaps(F["type"], hw, hh, T).items():
                dg[k] = max(dg.get(k, 0), abs(v - b["gap"]))
            n += 1
        name = f.name[len("w10_"):-len(".0181.json")]
        print(f"{name:28} {n:4} {fora:4} {dfoc:12.5f}  " + "  ".join(f"{k} {v:.4f}" for k, v in dg.items()))


main()
