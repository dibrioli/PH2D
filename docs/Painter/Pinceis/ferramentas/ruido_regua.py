#!/usr/bin/env python3
"""O RUÍDO de cada métrica da RÉGUA (`exame_regua.RUIDO`), medido no oráculo (04_BANCADA §3.1).

O método do 04 §4(c): o MESMO gesto pintado duas vezes pelo mesmo preset do Rebelle, |Δ| de cada métrica, o chão =
o p75 sobre os 228 presets. Os pares da bateria (os pontos são iguais ao byte, só a posição muda):

  R1  gestos.recto_lento × tamanhos.recto_40   (o par da J1; 500 × 880 px — a carga que acaba entra aqui)
  R2  tamanhos.recto_40  × sobre.papel         (880 × 880 px: os MESMOS pontos, outra cena)
  V1  sobre.cruza_1      × sobre.cruza_2       (o vertical que cruza as faixas, duas vezes: também a MISTURA)
  P   a resposta à pressão: rampa_afunila SUBIDA × DESCIDA (a mesma pressão, decil a decil) · rampa_0_1 × a subida
      da afunila (as duas sobem; a afunila 1,7× mais depressa) · rampa_0_1 × afunila inteiras (o que o `compara` lê)

A foto B entra quando difere da A (as A = B são o controlo de ZERO: a régua é determinística — conferido aqui).
As RAZÕES da matriz (rápido/lento, 5/40…) têm o ruído de |r − 1| nos mesmos pares.

uso: ruido_regua.py [--json matriz/ruido_regua.json] [--cache dir]
"""
import argparse
import hashlib
import json
import math
import os
import sys
from collections import defaultdict
from concurrent.futures import ProcessPoolExecutor

import numpy as np

AQUI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, AQUI)
import exame_matriz as M  # noqa: E402
import exame_regua as R  # noqa: E402

PARES = {"R1": (("gestos", "recto_lento"), ("tamanhos", "recto_40")),
         "R2": (("tamanhos", "recto_40"), ("sobre", "papel")),
         "V1": (("sobre", "cruza_1"), ("sobre", "cruza_2"))}
METRICAS = list(dict.fromkeys(R.ESCALARES + ["periodo", "periodo_fracao", "perfil", "cor"]))
MISTURA = ["vermelho_em_cima", "vermelho_depois", "cor_em_cima"]
RAZOES = ["largura", "cobertura", "textura"]
# o RUÍDO provisório da J1 (antes desta medição), para a tabela
PROVISORIO = {"largura": 2.0, "cobertura": 0.012, "opacidade": 0.02, "rugosidade": 0.4, "textura": 1.0,
              "var_local": 0.01, "cv": 0.012, "cor": 6.0, "pontas_ini": 0.12, "pontas_fim": 0.12, "alem_ini": 0.15,
              "alem_fim": 0.15, "perfil": 0.04, "pressao_largura": 0.08, "pressao_cobertura": 0.06,
              "vermelho_em_cima": 0.05, "vermelho_depois": 0.03, "cor_em_cima": 8.0}


def sha(p):
    return hashlib.sha1(open(p, "rb").read()).hexdigest()


def pressao_partida(args):
    """As séries por decil da rampa_0_1 e da rampa_afunila (inteira, subida, descida) numa foto da cena gestos."""
    png, bat = args
    a = R.le(png)
    papel = R.papel_de(a)
    c = np.abs(a - papel).max(2) / 255.0
    ts = R.tracos(bat, "gestos")
    out = {}
    for nome in ("rampa_0_1", "rampa_afunila"):
        tr = next(t for t in ts if t["nome"] == nome)
        m = R.mede_traco(c, a, papel, tr, ts, ([23, 101, 179], [191, 0, 18]), amostras=True)
        am = m.get("_amostras")
        if not am:
            continue
        pr, lg, cb, lp, wl, cl = (am[k] for k in ("pr", "larg25", "cob", "limpa", "wl", "cl"))
        topo = int(np.argmax(pr))
        partes = {"inteira": np.ones(len(pr), bool)}
        if nome == "rampa_afunila":
            idx = np.arange(len(pr))
            partes["sobe"], partes["desce"] = idx <= topo, idx >= topo
        for parte, msk in partes.items():
            pl, pc = [], []
            for k in range(10):
                sel = lp & msk & (pr >= k / 10) & (pr < (k + 1) / 10 + (k == 9))
                ok = sel.sum() > 3
                pl.append(float(np.nanmedian(lg[sel]) / wl) if ok and wl > 0 else float("nan"))
                pc.append(float(np.nanmean(cb[sel]) / cl) if ok and cl > 0 else float("nan"))
            out[f"{nome}.{parte}"] = {"pressao_largura": pl, "pressao_cobertura": pc}
    return out


def pct(v, ps=(50, 75, 90)):
    v = np.asarray([x for x in v if x is not None and math.isfinite(x)])
    return [float(np.percentile(v, p)) for p in ps] if len(v) else [float("nan")] * len(ps)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", default=os.path.join(M.PINCEIS, "matriz", "ruido_regua.json"))
    ap.add_argument("--cache", default=os.path.join(os.environ.get("TMPDIR", "/tmp"), "ph2d_exame", "regua_cache"))
    x = ap.parse_args()
    bat = json.load(open(R.BATERIA))
    pop = [e for e in M.populacao(os.devnull, M.REBELLE, None) if e["fonte"] == "rebelle"]
    fotos = M.mede_todos(pop, bat, x.cache)

    # quais fotos B são OUTRA imagem (A ≠ B ao byte); as iguais são o controlo de zero
    distintas, zero_ok, zero_n = [], 0, 0
    for e, fs in zip(pop, fotos):
        d = set()
        for cena in ("gestos", "tamanhos", "sobre"):
            pa, pb = R.acha_foto(e["caminho"], cena, "A"), R.acha_foto(e["caminho"], cena, "B")
            if pa and pb and sha(pa) != sha(pb):
                d.add(cena)
            elif pa and pb:
                zero_n += 1
                zero_ok += json.dumps(fs.get((cena, "A")), sort_keys=True, default=float) == \
                    json.dumps(fs.get((cena, "B")), sort_keys=True, default=float)
        distintas.append(d)

    def fotos_de(i, *cenas):
        return ["A"] + (["B"] if all(c in distintas[i] for c in cenas) else [])

    # os escalares, o perfil e a cor, por par: (nível, |Δ|) — o nível é a média dos dois lados (0 nos vectores)
    dif = defaultdict(lambda: defaultdict(list))  # dif[metrica][par] = [(nível, |Δ|) …]
    razao = defaultdict(list)

    def nivel(u, v):
        return (abs(u) + abs(v)) / 2 if isinstance(u, (int, float)) and isinstance(v, (int, float)) else 0.0

    for i, fs in enumerate(fotos):
        for par, ((ca, ta), (cb, tb)) in PARES.items():
            for f in fotos_de(i, ca, cb):
                a, b = fs.get((ca, f), {}).get(ta, {}), fs.get((cb, f), {}).get(tb, {})
                if not a or not b or a.get("vazio") or b.get("vazio"):
                    continue
                for k in METRICAS:
                    d = R.dist_metrica(k, a.get(k), b.get(k))
                    if math.isfinite(d):
                        dif[k][par].append((nivel(a.get(k), b.get(k)), abs(d)))
                for k in RAZOES:
                    if a.get(k) and b.get(k) and a[k] > 1e-6:
                        razao[par].append(abs(b[k] / a[k] - 1))
                if par == "V1":  # a mistura: o mesmo cruzamento duas vezes, sobre cada faixa
                    for base in ("faixa_molhada", "faixa_seca"):
                        sa, sb = a.get("sobre", {}).get(base, {}), b.get("sobre", {}).get(base, {})
                        for k in MISTURA:
                            d = R.dist_metrica(k, sa.get(k), sb.get(k))
                            if math.isfinite(d):
                                dif[k][f"V1:{base}"].append((nivel(sa.get(k), sb.get(k)), abs(d)))

    # a pressão: as rampas partidas (uma passagem própria — as séries por amostra não vão para a cache)
    tarefas = []
    for i, e in enumerate(pop):
        for f in fotos_de(i, "gestos"):
            p = R.acha_foto(e["caminho"], "gestos", f)
            if p:
                tarefas.append((p, bat))
    with ProcessPoolExecutor(max_workers=min(12, os.cpu_count() or 4)) as ex:
        rampas = list(ex.map(pressao_partida, tarefas, chunksize=4))
    por_decil = defaultdict(lambda: defaultdict(list))
    pares_p = {"P_sobe×desce": ("rampa_afunila.sobe", "rampa_afunila.desce"),
               "P_0_1×sobe": ("rampa_0_1.inteira", "rampa_afunila.sobe"),
               "P_0_1×afunila": ("rampa_0_1.inteira", "rampa_afunila.inteira")}
    for r in rampas:
        for par, (u, v) in pares_p.items():
            if u not in r or v not in r:
                continue
            for k in ("pressao_largura", "pressao_cobertura"):
                d = R.dist_metrica(k, r[u][k], r[v][k])
                if math.isfinite(d):
                    dif[k][par].append((0.0, abs(d)))
                xa, xb = np.asarray(r[u][k]), np.asarray(r[v][k])
                ok = np.isfinite(xa) & np.isfinite(xb)
                por_decil[k][par].extend(np.abs(xa[ok] - xb[ok]).tolist())

    # O MODELO de cada métrica. Os pares usados: R1 · R2 · V1 (a mistura: V1 sobre as duas faixas; a pressão: a
    # rampa_0_1 × a SUBIDA da afunila — as duas sobem a partir do início do traço; a descida é OUTRO gesto, a
    # pressão a cair na 2.ª metade da carga: a histerese do pincel, não o ruído). Se o p75 do quartil de CIMA do
    # nível passa 3× o do quartil de BAIXO (3 = a tolerância de 3 ruídos: uma constante só aprovaria em baixo o
    # que reprova em cima), o ruído é máx(a, b·|nível do oráculo|) (a = p75 do quartil de baixo, b = p75 de |Δ|/nível
    # no resto); senão, a constante = o p75 de todos (o chão do 04 §4(c)).
    res = {"populacao": len(pop), "controlo_zero": {"cenas_A_igual_B": zero_n, "medidas_iguais": zero_ok},
           "metricas": {}}
    for k in sorted(dif):
        pares = dif[k]
        if k.startswith("pressao"):
            usados = ["P_0_1×sobe"]
        elif k in MISTURA:
            usados = [p for p in pares if p.startswith("V1:")]
        else:
            usados = ["R1", "R2", "V1"]
        tudo = np.asarray([t for p in usados for t in pares.get(p, [])])
        L, D = tudo[:, 0], tudo[:, 1]
        modelo = {"tipo": "constante", "ruido": pct(D)[1]}
        if L.any():
            ordem = np.argsort(L, kind="stable")
            q = max(len(ordem) // 4, 1)
            baixo, cima = ordem[:q], ordem[-q:]
            a, c = float(np.percentile(D[baixo], 75)), float(np.percentile(D[cima], 75))
            modelo["p75_baixo_cima"] = [a, c, float(np.median(L[baixo])), float(np.median(L[cima]))]
            if c > 3 * a:
                resto = ordem[q:]
                resto = resto[L[resto] > 0]
                modelo.update(tipo="máx(a, b·nível)", a=a, b=float(np.percentile(D[resto] / L[resto], 75)))
        res["metricas"][k] = {"por_par": {p: {"n": len(v), "p50_p75_p90": pct([d for _, d in v]),
                                              "zeros": float(np.mean([d == 0 for _, d in v])) if v else None}
                                          for p, v in sorted(pares.items())},
                              "usados": usados, "n": int(len(D)), "p50_p75_p90": pct(D), "modelo": modelo}
    res["chao_matriz"] = {
        "pressao": pct(por_decil["pressao_largura"]["P_0_1×sobe"] + por_decil["pressao_cobertura"]["P_0_1×sobe"])[1],
        "pressao_por_par": {p: pct(por_decil["pressao_largura"][p] + por_decil["pressao_cobertura"][p])
                            for p in pares_p},
        "vermelho": pct([d for k in ("vermelho_em_cima", "vermelho_depois") for p in ("V1:faixa_molhada",
                                                                                     "V1:faixa_seca")
                         for _, d in dif[k][p]])[1],
        "razao": pct([d for p in ("R1", "R2", "V1") for d in razao[p]])[1],
        "razao_por_par": {p: pct(v) for p, v in razao.items()}}
    os.makedirs(os.path.dirname(x.json), exist_ok=True)
    json.dump(res, open(x.json, "w"), indent=1, default=float)

    print(f"população {len(pop)} · controlo de zero: {zero_ok}/{zero_n} cenas A = B ao byte dão medidas iguais")
    print("| métrica | R1 · R2 · V1 (p75) | outros pares (p75) | p50 · p75 · p90 dos usados | p75 em baixo → em cima "
          "do nível | **o ruído** | o provisório | a régua hoje |")
    print("|---|---|---|---|---|---|---|---|")
    deriva = 0
    for k, v in res["metricas"].items():
        pp, m = v["por_par"], v["modelo"]
        cel = " · ".join(f"{pp[p]['p50_p75_p90'][1]:.3g}" if p in pp else "—" for p in ("R1", "R2", "V1"))
        ext = " · ".join(f"{p} {pp[p]['p50_p75_p90'][1]:.3g}" for p in pp if p not in ("R1", "R2", "V1"))
        a, b, c = v["p50_p75_p90"]
        bc = m.get("p75_baixo_cima")
        bc = f"{bc[0]:.3g} @{bc[2]:.3g} → {bc[1]:.3g} @{bc[3]:.3g}" if bc else "—"
        r = f"máx({m['a']:.3g}, {m['b']:.3g}·nível)" if m["tipo"] != "constante" else f"{m['ruido']:.3g}"
        hoje = R.RUIDO.get(k)
        alvo = (m["a"], m["b"]) if m["tipo"] != "constante" else m["ruido"]
        igual = hoje is not None and np.allclose(np.atleast_1d(hoje), np.atleast_1d(alvo), rtol=0.05)
        deriva += hoje is not None and not igual
        print(f"| {k} | {cel} | {ext or '—'} | {a:.3g} · {b:.3g} · {c:.3g} | {bc} | **{r}** | "
              f"{PROVISORIO.get(k, '—')} | {hoje if igual or hoje is None else f'{hoje} ≠'} |")
    print("chão da matriz:", json.dumps(res["chao_matriz"], default=float))
    if deriva:
        print(f"⚠️ {deriva} métricas do `exame_regua.RUIDO` diferem do medido (> 5 %)")

if __name__ == "__main__":
    main()
