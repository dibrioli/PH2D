#!/usr/bin/env python3
"""A MATRIZ de distinção do EXAME (00_MISSAO §3.3): cada pincel contra a biblioteca INTEIRA, pela bateria pintada.

Cada pincel vira um VECTOR DE IDENTIDADE tirado das suas fotos da bateria pela régua (`exame_regua.mede`), em cinco
GRUPOS — forma (largura ÷ tamanho, pontas, perfil transversal), tinta (cobertura, opacidade, cor; também secas),
textura (textura, var. local, rugosidade da borda, cv, período), dinâmica (resposta à pressão, rápido × lento,
vertical × horizontal, 5 e 200 px) e mistura (o vermelho em cima e depois das faixas molhada e seca). Cada traço
normaliza-se pelo espalhamento ROBUSTO da biblioteca (1,4826 · MAD, com um chão = o ruído da régua); a diferença em
cada traço satura a 6; a distância de um grupo é a RMS, e a do par é a MÉDIA dos cinco grupos (nenhum grupo manda
pelo número de traços que tem).

O LIMIAR «claramente longe» calibra-se pelos vereditos do dono (19/10, smoke do P53), cada pincel julgado contra o
seu vizinho mais perto ENTRE OS QUE FICARAM na fábrica: o Filbert «nada novo» ⇒ IGUAL (tem de cair abaixo); o Flat
Oily e o Knife Smooth «bizarros» ⇒ DIFERENTES de tudo (inúteis, mas não repetidos: acima). O limiar é a média
geométrica entre o maior igual e o menor diferente; se a régua não os separa, di-lo. ⚠️ O par do NASCE_DE (Filbert ×
Acrylic Round) NÃO é a âncora: medido 19/10, à bateria (tamanho 40, a mesma cor) a Acrylic Round pinta um feixe de
fios transparente (cobertura 0,55) e o Filbert um redondo liso (0,91) — d 1,90; o «nada novo» do dono é o redondo
liso que a biblioteca já tem (o vizinho dele é o Soft Pastel, d 0,74). Os 8 do P53 que ele aceitou NÃO são âncora de
«distinto»: «OK» foi «serve», e a matriz mostra-os quase iguais entre si à mesma largura e cor.

uso: exame_matriz.py <dir da biblioteca pintada> <saida-dir> [--bateria b.json] [--saiu "Nome,Nome"]
     (cada subpasta = um pincel, com `meta.json` e `<cena>_<A|B>.ppm` do `exame_pinta`)
"""
import argparse
import warnings
import csv
import json
import math
import os
import sys
from concurrent.futures import ProcessPoolExecutor

import numpy as np

warnings.filterwarnings("ignore", message="All-NaN slice")

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from exame_regua import BATERIA, RUIDO, acha_foto, mede  # noqa: E402

IGUAIS = ["Filbert"]  # o dono, 19/10: «Filbert = nada novo»
DIFERENTES = ["Flat Oily", "Knife Smooth"]  # o dono, 19/10: «bizarros e inúteis» — não repetem ninguém
NASCE_DE = [("Filbert", "Acrylic Round")]  # o par da receita (informativo)
GRUPOS = ("forma", "tinta", "textura", "dinamica", "mistura")
CHAO = {"largura": 0.05, "pontas": 0.12, "alem": 0.15, "perfil": 0.04, "cobertura": RUIDO["cobertura"],
        "opacidade": RUIDO["opacidade"], "cor": RUIDO["cor"] / 255, "textura": RUIDO["textura"],
        "var_local": RUIDO["var_local"], "rugosidade": RUIDO["rugosidade"], "cv": RUIDO["cv"],
        "periodo_fracao": 0.03, "pressao": 0.06, "vermelho": 0.03, "razao": 0.05}


def mede_pincel(args):
    d, bat = args
    meta = json.load(open(os.path.join(d, "meta.json")))
    fotos = {}
    for c in bat["cenas"]:
        for foto in ("A", "B"):
            p = acha_foto(d, c["nome"], foto)
            if p:
                fotos[(c["nome"], foto)] = mede(p, c["nome"], bat)["tracos"]
    return meta, fotos


def vector(fotos):
    """{(grupo, chave do traço): valor} — a identidade de um pincel."""
    v = {}

    def g(c, f, t, k):
        x = fotos.get((c, f), {}).get(t, {}).get(k)
        return float("nan") if x is None else x

    def put(grupo, chave, x, chao):
        if isinstance(x, (list, tuple)):
            for i, y in enumerate(x):
                put(grupo, f"{chave}[{i}]", y, chao)
            return
        v[(grupo, chave, chao)] = float("nan") if x is None else float(x)

    gestos = ["recto_lento", "recto_rapido", "curva_s", "espiral", "rampa_0_1", "rampa_afunila", "vertical",
              "vertical_rapido"]
    tam = {"recto_5": 5.0, "recto_40": 40.0, "recto_200": 200.0}
    for t in gestos + list(tam) + ["papel", "cruza_1", "cruza_2", "cruza_3_rapido"]:
        c = "tamanhos" if t in tam else ("sobre" if t in ("papel",) or t.startswith("cruza") else "gestos")
        size = tam.get(t, 40.0)
        put("forma", f"{t}.largura", g(c, "A", t, "largura") / size, CHAO["largura"])
        for k in ("pontas_ini", "pontas_fim"):
            put("forma", f"{t}.{k}", g(c, "A", t, k), CHAO["pontas"])
        for k in ("alem_ini", "alem_fim"):
            put("forma", f"{t}.{k}", g(c, "A", t, k), CHAO["alem"])
        for f in ("A", "B"):
            for k in ("cobertura", "opacidade"):
                put("tinta", f"{t}.{k}.{f}", g(c, f, t, k), CHAO[k])
            cor = fotos.get((c, f), {}).get(t, {}).get("cor")
            put("tinta", f"{t}.cor.{f}", [x / 255 for x in cor] if cor else [float("nan")] * 3, CHAO["cor"])
        for k in ("textura", "var_local", "rugosidade", "cv", "periodo_fracao"):
            put("textura", f"{t}.{k}", g(c, "A", t, k), CHAO[k])
    for t in ("recto_lento", "vertical", "espiral"):
        p = fotos.get(("gestos", "A"), {}).get(t, {}).get("perfil") or [float("nan")] * 33
        put("forma", f"{t}.perfil", p[::4], CHAO["perfil"])
    p = fotos.get(("tamanhos", "A"), {}).get("recto_200", {}).get("perfil") or [float("nan")] * 33
    put("forma", "recto_200.perfil", p[::4], CHAO["perfil"])
    for t in ("rampa_0_1", "rampa_afunila"):
        for k in ("pressao_largura", "pressao_cobertura"):
            put("dinamica", f"{t}.{k}", fotos.get(("gestos", "A"), {}).get(t, {}).get(k) or [float("nan")] * 10,
                CHAO["pressao"])

    def razao(a, b, k):
        x, y = a, b
        return x / y if x == x and y == y and y > 1e-6 else float("nan")

    for a, b in (("recto_rapido", "recto_lento"), ("vertical", "recto_lento"), ("vertical_rapido", "vertical"),
                 ("curva_s", "recto_lento"), ("espiral", "recto_lento")):
        for k in ("largura", "cobertura", "textura"):
            put("dinamica", f"{a}/{b}.{k}", razao(g("gestos", "A", a, k), g("gestos", "A", b, k), k), CHAO["razao"])
    for t, s in (("recto_5", 5.0), ("recto_200", 200.0)):
        for k in ("largura", "cobertura", "textura"):
            x, y = g("tamanhos", "A", t, k), g("tamanhos", "A", "recto_40", k)
            if k == "largura":
                x, y = x / s, y / 40.0
            put("dinamica", f"{t}/recto_40.{k}", razao(x, y, k), CHAO["razao"])
    for f in ("A", "B"):
        for t in ("cruza_1", "cruza_2", "cruza_3_rapido"):
            sob = fotos.get(("sobre", f), {}).get(t, {}).get("sobre", {})
            for base in ("faixa_molhada", "faixa_seca"):
                s = sob.get(base, {})
                for k in ("vermelho_em_cima", "vermelho_depois"):
                    put("mistura", f"{t}.{base}.{k}.{f}", s.get(k), CHAO["vermelho"])
                cor = s.get("cor_em_cima")
                put("mistura", f"{t}.{base}.cor.{f}", [x / 255 for x in cor] if cor else [float("nan")] * 3,
                    CHAO["cor"])
    return v


def distancias(nomes, vecs):
    chaves = sorted(set().union(*[v.keys() for v in vecs]))
    X = np.array([[v.get(k, float("nan")) for k in chaves] for v in vecs])
    med = np.nanmedian(X, 0)
    mad = 1.4826 * np.nanmedian(np.abs(X - med), 0)
    chao = np.array([k[2] for k in chaves])
    esc = np.maximum(np.nan_to_num(mad, nan=0.0), chao)
    Z = X / esc
    grupos = np.array([k[0] for k in chaves])
    n = len(nomes)
    D = np.zeros((n, n))
    G = np.zeros((n, n, len(GRUPOS)))
    for i in range(n):
        dz = np.clip(np.abs(Z - Z[i]), 0, 6)
        for gi, gn in enumerate(GRUPOS):
            sel = grupos == gn
            blk = dz[:, sel]
            ok = np.isfinite(blk)
            cnt = ok.sum(1)
            G[i, :, gi] = np.sqrt(np.where(ok, blk ** 2, 0).sum(1) / np.maximum(cnt, 1))
            G[i, cnt == 0, gi] = np.nan
        D[i] = np.nanmean(G[i], 1)
    return D, G, chaves, esc


def calibra(nomes, D, saiu):
    """O limiar pelos vereditos do dono; `saiu` = os que já não estão na fábrica (não são vizinhos de ninguém)."""
    idx = {n: i for i, n in enumerate(nomes)}
    fica = [n for n in nomes if n not in saiu]

    def vizinho(n):
        d = [(float(D[idx[n], idx[m]]), m) for m in fica if m != n]
        return min(d) if d else (float("nan"), None)

    iguais = [(n, *vizinho(n)[::-1]) for n in IGUAIS if n in idx]
    difs = [(n, *vizinho(n)[::-1]) for n in DIFERENTES if n in idx]
    d_igual = max((d for _, _, d in iguais), default=float("nan"))
    d_dif = min((d for _, _, d in difs), default=float("nan"))
    if math.isfinite(d_igual) and math.isfinite(d_dif) and d_dif > d_igual:
        limiar, separa = math.sqrt(d_igual * d_dif), True
    elif math.isfinite(d_igual):
        limiar, separa = 1.25 * d_igual, False
    else:
        limiar, separa = float("nan"), False
    perto = sorted((round(vizinho(n)[0], 3), n, vizinho(n)[1]) for n in fica if vizinho(n)[0] < limiar)
    return {"iguais": iguais, "diferentes": difs, "d_igual_max": d_igual, "d_diferente_min": d_dif,
            "limiar": limiar, "separa": separa, "na_fabrica_abaixo_do_limiar": perto,
            "nasce_de": [(a, b, float(D[idx[a], idx[b]])) for a, b in NASCE_DE if a in idx and b in idx]}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("biblioteca"); ap.add_argument("saida"); ap.add_argument("--bateria", default=BATERIA)
    ap.add_argument("--saiu", default="", help="pincéis que já saíram da fábrica (marcados na tabela)")
    x = ap.parse_args()
    bat = json.load(open(x.bateria))
    dirs = sorted(os.path.join(x.biblioteca, d) for d in os.listdir(x.biblioteca)
                  if os.path.exists(os.path.join(x.biblioteca, d, "meta.json")))
    with ProcessPoolExecutor(max_workers=min(8, os.cpu_count() or 4)) as ex:
        res = list(ex.map(mede_pincel, [(d, bat) for d in dirs]))
    nomes = [m["nome"] for m, _ in res]
    vecs = [vector(f) for _, f in res]
    D, G, chaves, esc = distancias(nomes, vecs)
    saiu = set(filter(None, x.saiu.split(",")))
    cal = calibra(nomes, D, saiu)
    os.makedirs(x.saida, exist_ok=True)
    with open(os.path.join(x.saida, "matriz.csv"), "w", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["pincel"] + nomes)
        for n, row in zip(nomes, D):
            w.writerow([n] + [f"{v:.3f}" for v in row])
    lim = cal["limiar"]
    linhas = ["# Vizinhos mais perto — a matriz do EXAME (gerado por `exame_matriz.py`)", "",
              f"{len(nomes)} pincéis · {len(chaves)} traços de identidade · distância = média dos 5 grupos "
              f"({', '.join(GRUPOS)}) da RMS do desvio robusto (saturado a 6).", "",
              "**Calibração (os vereditos do dono, 19/10; cada um contra o vizinho entre os que ficaram):** "
              + "; ".join(f"{n} ~ {m} = {d:.3f} («nada novo»)" for n, m, d in cal["iguais"]) + "; "
              + "; ".join(f"{n} ~ {m} = {d:.3f} («bizarro»: diferente)" for n, m, d in cal["diferentes"])
              + f" ⇒ **limiar = {lim:.3f}** " + ("(média geométrica do maior igual e do menor diferente)"
                                                 if cal["separa"] else "(⚠️ a régua NÃO separa: 1,25 × o igual)")
              + "".join(f" · o par da receita {a} × {b} = {d:.3f}" for a, b, d in cal["nasce_de"]), ""]
    if cal["na_fabrica_abaixo_do_limiar"]:
        linhas.append(f"⚠️ Na fábrica, {len(cal['na_fabrica_abaixo_do_limiar'])} pincéis têm o vizinho abaixo do "
                      "limiar (candidatos a «nada novo» à mesma largura e cor): "
                      + "; ".join(f"{n} ~ {m} {d:.3f}" for d, n, m in cal["na_fabrica_abaixo_do_limiar"]))
        linhas.append("")
    linhas += ["| pincel | 1.º vizinho (d) | por grupo (forma · tinta · textura · dinâmica · mistura) | 2.º | 3.º | veredito |",
               "|---|---|---|---|---|---|"]
    top = {}
    for i, n in enumerate(nomes):
        ordem = [j for j in np.argsort(D[i]) if j != i and nomes[j] not in saiu]  # vizinho = quem FICOU
        top[n] = [(nomes[j], round(float(D[i, j]), 3)) for j in ordem[:3]]
        j = ordem[0]
        g = " · ".join(f"{v:.2f}" for v in G[i, j])
        marca = " (saiu)" if n in saiu else ""
        ver = "claramente longe" if D[i, j] >= lim else "PERTO — «nada novo»?"
        linhas.append(f"| {n}{marca} | {nomes[j]} ({D[i, j]:.3f}) | {g} | {nomes[ordem[1]]} ({D[i, ordem[1]]:.3f}) | "
                      f"{nomes[ordem[2]]} ({D[i, ordem[2]]:.3f}) | {ver} |")
    open(os.path.join(x.saida, "vizinhos.md"), "w").write("\n".join(linhas) + "\n")
    json.dump({"pinceis": nomes, "calibracao": cal, "top3": top,
               "dirs": {m["nome"]: d for (m, _), d in zip(res, dirs)},
               "grupos": {n: {nomes[j]: G[i, j].round(3).tolist() for j in range(len(nomes)) if j != i}
                          for i, n in enumerate(nomes)}},
              open(os.path.join(x.saida, "matriz.json"), "w"), indent=1, default=float)
    print("\n".join(linhas))


if __name__ == "__main__":
    main()
