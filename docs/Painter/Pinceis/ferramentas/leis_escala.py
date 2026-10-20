#!/usr/bin/env python3
"""AS LEIS DE ESCALA (03_MOTORES §7 c e b-resto, JANELA 2): o que muda num pincel do traço de 5 ao de 200 px — a cena
`tamanhos` da bateria (os rectos `recto_5`, `recto_40`, `recto_200`, horizontais), a foto A.

Por recto (as colunas de 10 % a 90 % do comprimento; em cada uma a cobertura c = máx |px − papel| / 255 na normal
±H, a largura a ¼ do máximo da coluna e o centro; o MIOLO = |y − centro| ≤ 0,35 · a largura mediana):

  larg     — a largura mediana (px)
  cob      — a cobertura média do miolo
  cv       — o desvio ÷ a média da cobertura do miolo coluna a coluna (o «colar de contas» AO LONGO)
  estrias  — a luminância p95 − p5 no miolo (o relevo/estria que se vê; a de `leis_oraculo.pontas` a 200 px)
  atraves  — o desvio-padrão da luminância ATRAVÉS do miolo (linha a linha), mediana das colunas
  grao     — o resíduo da média local 5×5 da cobertura no miolo ÷ a média (%) (a `textura` do EXAME)
  falhas   — a fracção das colunas com o pico abaixo de metade do pico mediano (o traço que ABRE: o recto de 5 px)

E a ponta do recto de 200 (`leis_oraculo.pontas`: a curva e o desfiado do fim).

uso: leis_escala.py oraculo <saida.json> <TOOL/prefixo>...       — fotos do Rebelle (`fixtures/rebelle_bateria/`)
     leis_escala.py nosso <bib-dir> <saida.json>                  — uma biblioteca pintada pela bancada
     leis_escala.py tabela <a.json> [<b.json>...]
"""
import json
import math
import os
import sys

import numpy as np

AQUI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, AQUI)
from exame_regua import acha_foto, box, le, papel_de  # noqa: E402
from leis_oraculo import limpo, pontas  # noqa: E402

REB = os.path.join(os.path.dirname(AQUI), "fixtures", "rebelle_bateria")
RECTOS = {"recto_5": (70, 5), "recto_40": (190, 40), "recto_200": (440, 200)}
X0, X1 = 60, 940
# ±H da janela de cada recto: o tamanho, sem chegar ao vizinho (5 a y 70, 40 a y 190, 200 a y 440).
JANELA = {"recto_5": 40, "recto_40": 70, "recto_200": 130}


def recto(a, papel, nome):
    y, _ = RECTOS[nome]
    H = JANELA[nome]
    c = np.abs(a - papel).max(2) / 255.0
    lum = a.mean(2)
    xs = np.arange(X0 + int(0.1 * (X1 - X0)), X0 + int(0.9 * (X1 - X0)))
    C = c[y - H:y + H + 1, xs]  # (linhas, colunas)
    Lm = lum[y - H:y + H + 1, xs]
    mx = C.max(0)
    nivel = np.percentile(mx, 90)
    if nivel < 0.06:
        return {"vazio": True}
    larg, cen = np.zeros(len(xs)), np.zeros(len(xs))
    o = np.arange(-H, H + 1.0)
    for j in range(len(xs)):
        col = C[:, j]
        if mx[j] < 0.08:
            cen[j] = 0.0
            continue
        dentro = np.where(col >= 0.25 * mx[j])[0]
        larg[j] = dentro[-1] - dentro[0] + 1
        cen[j] = float((col * o).sum() / max(col.sum(), 1e-9))
    L = float(np.median(larg))
    banda = np.abs(o[:, None] - cen[None, :]) <= max(0.35 * L, 0.5)
    v = C[banda]
    serie = np.array([C[banda[:, j], j].mean() if banda[:, j].any() else 0.0 for j in range(len(xs))])
    res = C - box(C, 2)
    lm = Lm[banda]
    p5, p95 = np.percentile(lm, [5, 95])
    atr = [Lm[banda[:, j], j].std() for j in range(len(xs)) if banda[:, j].sum() >= 3]
    return {"larg": L, "cob": float(v.mean()), "cv": float(serie.std() / max(serie.mean(), 1e-6)),
            "estrias": float(p95 - p5), "atraves": float(np.median(atr)) if atr else float("nan"),
            "grao": float(100 * res[banda].std() / max(v.mean(), 1e-6)),
            "falhas": float(np.mean(mx < 0.5 * np.median(mx)))}


def ponta(a, papel, nome):
    """O fim de um recto (`leis_oraculo.pontas` a qualquer tamanho): a CURVA (o fim no eixo − o das bordas ÷ a
    meia-largura; a cápsula > 0, o corte recto ≈ 0), o DESFIADO (o desvio do fim de linha para linha, px, sem a
    curva) e o ALÉM (quanto o eixo passa do último ponto, px)."""
    y, _ = RECTOS[nome]
    H = JANELA[nome]
    c = np.abs(a - papel).max(2) / 255.0
    banda = c[y - H:y + H + 1]
    m = max(10, (X1 - X0) // 8)
    nivel = np.percentile(banda[:, X0 + m:X1 - m].max(0), 90)
    lin = banda[:, X0 + m:X1 - m].mean(1)
    rows = np.where(lin > 0.25 * lin.max())[0]
    if len(rows) < 6 or nivel < 0.06:
        return None
    meio, hw = 0.5 * (rows[0] + rows[-1]), 0.5 * (rows[-1] - rows[0] + 1)
    out = {}
    for lado in ("ini", "fim"):
        fins = []
        for r in rows:
            v = (r - meio) / hw
            if abs(v) > 0.8:
                continue
            tem = np.where(banda[r] >= 0.25 * nivel)[0]
            if len(tem):
                fins.append((v, (X0 - tem[0]) if lado == "ini" else (tem[-1] - X1)))
        if len(fins) < 6:
            continue
        v, e = np.array(fins, float).T
        eixo, bordas = np.median(e[np.abs(v) < 0.25]), np.median(e[np.abs(v) > 0.6])
        res = e - np.polyval(np.polyfit(v, e, 2), v)
        out[lado] = {"curva": float((eixo - bordas) / hw), "desfiado_px": float(res.std()), "alem_px": float(eixo)}
    return out


def escala(png):
    a = le(png)
    papel = papel_de(a)
    out = {k: recto(a, papel, k) for k in RECTOS}
    out["pontas"] = pontas(png)
    out["pontas_40"] = ponta(a, papel, "recto_40")
    out["pontas_200"] = ponta(a, papel, "recto_200")
    return out


def main():
    cmd = sys.argv[1]
    if cmd == "oraculo":
        r = {}
        for pre in sys.argv[3:]:
            f = acha_foto(os.path.join(REB, pre), "tamanhos", "A")
            if f:
                r[pre] = limpo(escala(f))
        json.dump({"cabecalho": {"fonte": "fixtures/rebelle_bateria (foto A da cena tamanhos)",
                                 "ferramenta": "ferramentas/leis_escala.py"}, "escala": r},
                  open(sys.argv[2], "w"), indent=1)
    elif cmd == "nosso":
        bib, r = sys.argv[2], {}
        for s in sorted(os.listdir(bib)):
            mp = os.path.join(bib, s, "meta.json")
            f = acha_foto(os.path.join(bib, s), "tamanhos", "A")
            if os.path.exists(mp) and f:
                r[json.load(open(mp))["nome"]] = limpo(escala(f))
        json.dump({"cabecalho": {"fonte": bib}, "escala": r}, open(sys.argv[3], "w"), indent=1)
    elif cmd == "tabela":
        print("| pincel | larg 5·40·200 | cob 5·40·200 | cv 5·40·200 | estrias 5·40·200 | através 5·40·200 "
              "| grão 5·40·200 | curva·desfiado fim | 40: ini · fim | 200: ini · fim |")
        print("|---|---|---|---|---|---|---|---|---|---|")
        for f in sys.argv[2:]:
            for nome, e in json.load(open(f))["escala"].items():
                def g(k, q):
                    v = e.get(k, {}).get(q)
                    return "—" if v is None or not math.isfinite(v) else f"{v:.2f}" if abs(v) < 10 else f"{v:.0f}"
                cel = [" · ".join(g(k, q) for k in RECTOS) for q in ("larg", "cob", "cv", "estrias", "atraves", "grao")]
                fim = (e.get("pontas") or {}).get("fim") or {}
                pf = f"{fim.get('curva') or float('nan'):.2f} · {fim.get('desfiado_px') or float('nan'):.1f}"
                def pt(k):
                    p = e.get(k) or {}
                    return " · ".join(f"{(p.get(l) or {}).get('curva', float('nan')):.2f}/"
                                      f"{(p.get(l) or {}).get('desfiado_px', float('nan')):.1f}/"
                                      f"{(p.get(l) or {}).get('alem_px', float('nan')):.0f}" for l in ("ini", "fim"))
                print(f"| {nome} | " + " | ".join(cel) + f" | {pf} | {pt('pontas_40')} | {pt('pontas_200')} |")


if __name__ == "__main__":
    main()
