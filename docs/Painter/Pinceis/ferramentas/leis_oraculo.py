#!/usr/bin/env python3
"""AS LEIS SISTÉMICAS (03_MOTORES §7, JANELA 2): o que o oráculo faz nos quatro sítios onde quase todo pincel nosso
diverge — a PRESSÃO, o traço FINO, as PONTAS e a tinta por cima da faixa MOLHADA — medido com a régua do EXAME
(`exame_regua.py`) nas fotos da bateria (`fixtures/rebelle_bateria/`, a foto A), e o mesmo do NOSSO lado.

  pressão  — `rampa_0_1` e `rampa_afunila` (cena gestos): largura e cobertura por decil de pressão ÷ as de p ≥ 0,9
             (`pressao_largura`/`pressao_cobertura`), e a resposta na FAIXA DA MÃO DO DONO (0,24–0,86, doc 47 §2):
             (largura·cobertura) no decil 8 ÷ no decil 2
  fino     — `recto_5` e `recto_40` (cena tamanhos): largura, cobertura, cv; a razão cobertura(5) ÷ cobertura(40)
  pontas   — `recto_200` (cena tamanhos): em cada linha ao longo do traço (o miolo, ±0,8 da meia-largura), a
             coluna onde a tinta acaba; a CURVA (o fim do eixo − o das bordas, ÷ a meia-largura: a cápsula é > 0,
             o corte recto ≈ 0) e o DESFIADO (o desvio do fim de linha para linha, px, sem a curva)
  molhada  — `cruza_*` (cena sobre): a largura do azul EM CIMA da faixa molhada ÷ em cima da seca (a lente), e a
             cor em cima da molhada

uso: leis_oraculo.py oraculo <saida.json>        — as origens de fábrica no Rebelle (`exame_matriz.ORIGEM`)
     leis_oraculo.py nosso <bib-dir> <saida.json> — uma biblioteca pintada pela bancada (`exame.sh`/`exame_pinta`)
     leis_oraculo.py tabela <oraculo.json> <nosso.json>
"""
import json
import math
import os
import sys
from multiprocessing import Pool

import numpy as np

AQUI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, AQUI)
from exame_regua import BATERIA, acha_foto, le, mede, papel_de, tracos  # noqa: E402
from exame_matriz import ORIGEM  # noqa: E402

REB = os.path.join(os.path.dirname(AQUI), "fixtures", "rebelle_bateria")
BAT = json.load(open(BATERIA))
MAO = (2, 8)  # os decis da faixa da mão do dono: 0,2–0,3 e 0,8–0,9


def prefixo(origem):
    tool, preset = origem.split(":", 1)
    return os.path.join(REB, tool, "".join(c.lower() if c.isalnum() else "_" for c in preset))


def mao(m):
    L, C = m.get("pressao_largura"), m.get("pressao_cobertura")
    if not L or not C:
        return None
    a, b = L[MAO[0]] * C[MAO[0]], L[MAO[1]] * C[MAO[1]]
    if not (math.isfinite(a) and math.isfinite(b)):
        return None
    return b / a if a > 1e-3 else 99.0


def pontas(png):
    """O fim do recto de 200 px: a CURVA e o DESFIADO das duas pontas (ver o cabeçalho)."""
    a = le(png)
    papel = papel_de(a)
    c = np.abs(a - papel).max(2) / 255.0
    t = next(t for t in tracos(BAT, "tamanhos") if t["nome"] == "recto_200")
    (x0, y0), (x1, _) = t["pts"][0][:2], t["pts"][-1][:2]
    banda = c[int(y0) - 130:int(y0) + 130]
    nivel = np.percentile(banda[:, int(x0) + 100:int(x1) - 100].max(0), 90)
    lin = banda[:, int(x0) + 100:int(x1) - 100].mean(1)
    rows = np.where(lin > 0.25 * lin.max())[0]
    if len(rows) < 10 or nivel < 0.06:
        return None
    meio, hw = 0.5 * (rows[0] + rows[-1]), 0.5 * (rows[-1] - rows[0] + 1)
    out = {}
    for lado in ("ini", "fim"):
        fins = []
        for r in rows:
            v = (r - meio) / hw
            if abs(v) > 0.8:
                continue
            row = banda[r]
            tem = np.where(row >= 0.25 * nivel)[0]
            if not len(tem):
                continue
            fins.append((v, (x0 - tem[0]) if lado == "ini" else (tem[-1] - x1)))
        if len(fins) < 10:
            continue
        v, e = np.array(fins).T
        eixo = np.median(e[np.abs(v) < 0.25])
        bordas = np.median(e[np.abs(v) > 0.6])
        k = np.polyfit(v, e, 2)
        res = e - np.polyval(k, v)
        out[lado] = {"curva": float((eixo - bordas) / hw), "desfiado_px": float(res.std()),
                     "alem_px": float(eixo)}
    # O TUBO: a amplitude da luminância através do traço (10/30/50/70/90 % da largura, colunas 300..700).
    lum = a[int(y0) - 130:int(y0) + 130, 300:700].mean(2).mean(1)
    esc = np.where(lum < 240)[0]
    if len(esc) > 5:
        perfil = lum[esc[0]:esc[-1] + 1]
        q = [perfil[int(len(perfil) * f)] for f in (0.1, 0.3, 0.5, 0.7, 0.9)]
        out["tubo"] = float(max(q) - min(q))
    return out


def lente(png, m):
    """A lente: a largura do azul em cima da faixa molhada ÷ em cima da seca, por cruzamento."""
    a = le(png)
    out = {}
    ts = {t["nome"]: t for t in tracos(BAT, "sobre")}
    for nome in ("cruza_1", "cruza_2", "cruza_3_rapido"):
        x = int(ts[nome]["pts"][0][0])
        w = {}
        for faixa in ("faixa_molhada", "faixa_seca"):
            y = int(ts[faixa]["pts"][0][1])
            larg = []
            for yy in range(y - 20, y + 21):
                row = a[yy, x - 60:x + 61]
                azul = row[:, 2] > row[:, 0] + 30
                xs = np.where(azul)[0]
                larg.append(xs[-1] - xs[0] + 1 if len(xs) else 0)
            w[faixa] = float(np.median(larg))
        cor = m["tracos"][nome].get("sobre", {}).get("faixa_molhada", {}).get("cor_em_cima")
        out[nome] = {"molhada_px": w["faixa_molhada"], "seca_px": w["faixa_seca"],
                     "lente": w["faixa_molhada"] / max(w["faixa_seca"], 1.0), "cor_em_cima_molhada": cor}
    return out


def leis(d):
    """As quatro leis de UM pincel (`d` = pasta ou prefixo das fotos, ver `acha_foto`)."""
    out = {}
    g, ta, so = (acha_foto(d, c, "A") for c in ("gestos", "tamanhos", "sobre"))
    if g:
        m = mede(g, "gestos", BAT)["tracos"]
        out["pressao"] = {r: {"largura": m[r].get("pressao_largura"), "cobertura": m[r].get("pressao_cobertura"),
                              "mao": mao(m[r])} for r in ("rampa_0_1", "rampa_afunila")}
    if ta:
        m = mede(ta, "tamanhos", BAT)["tracos"]
        f = {k: {q: m[k].get(q) for q in ("largura", "cobertura", "cv", "textura")} for k in ("recto_5", "recto_40")}
        c5, c40 = f["recto_5"]["cobertura"], f["recto_40"]["cobertura"]
        f["razao_5_40"] = c5 / c40 if c5 is not None and c40 else None
        out["fino"] = f
        out["pontas"] = pontas(ta)
    if so:
        out["molhada"] = lente(so, mede(so, "sobre", BAT))
    return out


def limpo(v):
    """O JSON estrito (o `serde_json` do portão recusa `NaN`): um número não finito vira `null`."""
    if isinstance(v, dict):
        return {k: limpo(x) for k, x in v.items()}
    if isinstance(v, (list, tuple)):
        return [limpo(x) for x in v]
    if isinstance(v, (float, np.floating)):
        return float(v) if math.isfinite(v) else None
    if isinstance(v, np.integer):
        return int(v)
    return v


def _um(job):
    nome, d = job
    return nome, limpo(leis(d))


def main():
    cmd = sys.argv[1]
    if cmd == "oraculo":
        jobs = [(n, prefixo(o)) for n, o in ORIGEM.items() if acha_foto(prefixo(o), "gestos", "A")]
        with Pool(12) as p:
            r = dict(p.map(_um, jobs))
        cab = {"fonte": "fixtures/rebelle_bateria (as fotos A da bateria, Rebelle 8 sob Wine, J1)",
               "origens": {n: ORIGEM[n] for n in r}, "ferramenta": "ferramentas/leis_oraculo.py oraculo",
               "regua": "exame_regua.py (a do EXAME)"}
        json.dump({"cabecalho": cab, "leis": r}, open(sys.argv[2], "w"), indent=1, default=float)
    elif cmd == "nosso":
        bib = sys.argv[2]
        jobs = []
        for s in sorted(os.listdir(bib)):
            mp = os.path.join(bib, s, "meta.json")
            if os.path.exists(mp):
                jobs.append((json.load(open(mp))["nome"], os.path.join(bib, s)))
        with Pool(12) as p:
            r = dict(p.map(_um, jobs))
        json.dump({"cabecalho": {"fonte": bib}, "leis": r}, open(sys.argv[3], "w"), indent=1, default=float)
    elif cmd == "tabela":
        o, n = (json.load(open(f))["leis"] for f in sys.argv[2:4])
        tabela(o, n)


def f2(x):
    return "—" if x is None or (isinstance(x, float) and not math.isfinite(x)) else f"{x:.2f}"


def tabela(o, n):
    print("| pincel | mão o→n | cob 5/40 o→n | larg 5 o→n | curva fim o→n | desfiado fim o→n | lente o→n |")
    print("|---|---|---|---|---|---|---|")
    for nome in sorted(n):
        a, b = o.get(nome, {}), n[nome]
        g = lambda d, *k: (lambda v: v)(_get(d, k))  # noqa: E731
        print(f"| {nome} | {f2(g(a, 'pressao', 'rampa_0_1', 'mao'))}→{f2(g(b, 'pressao', 'rampa_0_1', 'mao'))} "
              f"| {f2(g(a, 'fino', 'razao_5_40'))}→{f2(g(b, 'fino', 'razao_5_40'))} "
              f"| {f2(g(a, 'fino', 'recto_5', 'largura'))}→{f2(g(b, 'fino', 'recto_5', 'largura'))} "
              f"| {f2(g(a, 'pontas', 'fim', 'curva'))}→{f2(g(b, 'pontas', 'fim', 'curva'))} "
              f"| {f2(g(a, 'pontas', 'fim', 'desfiado_px'))}→{f2(g(b, 'pontas', 'fim', 'desfiado_px'))} "
              f"| {f2(g(a, 'molhada', 'cruza_1', 'lente'))}→{f2(g(b, 'molhada', 'cruza_1', 'lente'))} |")


def _get(d, k):
    for x in k:
        if not isinstance(d, dict):
            return None
        d = d.get(x)
    return d


if __name__ == "__main__":
    main()
