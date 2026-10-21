#!/usr/bin/env python3
"""A MATRIZ de distinção do EXAME (00_MISSAO §3.3) — TRIAGEM, nunca veredito: o veredito é a prancha + a revisão cega.

Cada pincel vira um VECTOR DE IDENTIDADE tirado das suas fotos da bateria pela régua (`exame_regua.mede`), em cinco
GRUPOS — forma (largura relativa ao próprio recto e a absoluta, pontas, perfil transversal), tinta (cobertura,
opacidade, cor; molhada e seca), textura (textura, var. local, rugosidade da borda, cv, período), dinâmica (pressão,
rápido × lento, vertical × horizontal, 5 e 200 px) e mistura (o vermelho em cima e depois das faixas). A distância de
um par: cada traço ÷ a ESCALA CONGELADA, a diferença saturada a 6, a RMS por grupo, a média dos cinco grupos.

A população: os NOSSOS (as pastas do `exame_pinta`), os 228 presets do Rebelle que pintam as três cenas
(`fixtures/rebelle_bateria/indice.csv`) e os 34 do ArtRage (`fixtures/artrage_bateria`, só a foto A: sem água, B ≡
A). A ESCALA é a da biblioteca do Rebelle (1,4826 · MAD de cada traço sobre os 228), com o CHÃO = o ruído do próprio
oráculo (o mesmo recto pintado duas vezes: recto_lento da cena gestos × recto_40 da cena tamanhos, p75 sobre os 228) —
congelada em `matriz/escala.json` (`--recalibra` refá-la): pincel novo não mexe na régua dos outros. A LEI (os números
e o porquê) está no `04_BANCADA.md` §4; `--recalibra` imprime as medições de onde ela sai (`matriz/oraculos.md`).

uso: exame_matriz.py <dir da biblioteca pintada> <saida-dir> [--bateria b.json] [--saiu "Nome,…"] [--recalibra]
     [--rebelle dir] [--artrage dir] [--cache dir]
"""
import argparse
import csv
import glob
import hashlib
import json
import math
import os
import re
import sys
import warnings
from collections import defaultdict
from concurrent.futures import ProcessPoolExecutor

import numpy as np

warnings.filterwarnings("ignore", message="All-NaN slice")
warnings.filterwarnings("ignore", message="Mean of empty slice")

AQUI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, AQUI)
from bateria_rebelle import max_do_preset  # noqa: E402
from exame_regua import BATERIA, RUIDO, acha_foto, mede  # noqa: E402

PINCEIS = os.path.dirname(AQUI)
REBELLE = os.path.join(PINCEIS, "fixtures", "rebelle_bateria")
ARTRAGE = os.path.join(PINCEIS, "fixtures", "artrage_bateria")
ESCALA = os.path.join(PINCEIS, "matriz", "escala.json")
# O 6.º grupo, «ferramenta» (J4 `mistura`, `matriz_ferramenta.py`), só existe nas FERRAMENTAS: nos pincéis que pintam é
# NaN, e a média dos grupos de um par deles é a de antes.
GRUPOS = ("forma", "tinta", "textura", "dinamica", "mistura", "ferramenta")
# O chão das famílias que o recto repetido não mede, MEDIDO no Rebelle pelo `ruido_regua.py` (04 §3.1, o p75): a
# pressão por decil (rampa_0_1 × a subida da afunila), o vermelho (cruza_1 × cruza_2, em cima e depois, sobre as
# duas faixas), as razões (|r − 1| nos pares repetidos). Eram 0,06 · 0,03 · 0,05 (provisórios, 19/10).
CHAO_MEDIDO = {"pressao": 0.0679, "vermelho": 0.0155, "razao": 0.0704, "cor": RUIDO["cor"] / 255}
FAMILIAS_DO_RECTO = ["largura", "cobertura", "opacidade", "textura", "var_local", "rugosidade", "cv",
                     "periodo_fracao", "pontas", "alem", "perfil", "cor"]
# Os pincéis de fábrica e o preset do Rebelle de que cada um nasceu (`midias_de_fabrica.rs` MIDIAS → a foto → o
# `fixtures/rebelle/README.md`). Wet Drip e Wet Dry Brush não têm foto de origem.
ORIGEM = {"Wet Liner": "WATERCOLOR:Watercolor/Liner", "Oil Bristle": "OIL_AND_ACRYLIC:Bristle/Bristle Flat",
          "Acrylic Round": "OIL_AND_ACRYLIC:Round/Round Hard", "Express Oil": "EXPRESS_OIL:Round/Hairy",
          "Graphite 8B": "PENCIL:Graphite Pencil/8B", "Charcoal": "PENCIL:Charcoal/Charcoal",
          "Soft Pastel": "PASTEL:Pastel Dry/Dry Thin", "Oil Pastel": "PASTEL:Pastel Oil/Oil Pastel 2",
          "Ink Bullet": "INK_PEN:Pen/Bullet", "Dip Pen": "INK_PEN:Pen/Dip Pen", "Ink Brush": "INK_PEN:Brush/Brush",
          "Marker Bullet": "MARKER:Marker/Bullet", "Marker Liner": "MARKER:Marker/Liner",
          "Airbrush": "AIRBRUSH:Default/Spray 2", "Wet Flat": "WATERCOLOR:Watercolor/Flat",
          "Gouache Flat": "WATERCOLOR:Gouache/Gouache Flat", "Graphite 2H": "PENCIL:Graphite Pencil/2H",
          "Graphite HB": "PENCIL:Graphite Pencil/HB", "Graphite 2B": "PENCIL:Graphite Pencil/2B",
          "Colored 4B": "PENCIL:Colored Pencil/4B Color", "Scratch": "PENCIL:Scratches/Scratch",
          "Square Hard": "PASTEL:Pastel Dry/Square Hard", "Wax Scrape": "PASTEL:Scrape/Wax Scrape",
          "Dabber": "OIL_AND_ACRYLIC:Dabber/Dabber", "Wet Round": "WATERCOLOR:Watercolor/Round",
          "Wet Wash": "WATERCOLOR:Watercolor/Wash Round", "Filbert": "OIL_AND_ACRYLIC:Flat/Filbert",
          "Flat Oily": "OIL_AND_ACRYLIC:Flat/Flat Oily", "Knife Smooth": "OIL_AND_ACRYLIC:Knife/Knife Smooth"}


# ---------------------------------------------------------------- a população e as medidas (com cache)

def populacao(biblioteca, rebelle, artrage):
    out = []
    for d in sorted(glob.glob(os.path.join(biblioteca, "*", "meta.json"))):
        m = json.load(open(d))
        out.append({"id": f"ph2d:{m['nome']}", "nome": m["nome"], "fonte": "ph2d", "ferr": m.get("meio", ""),
                    "caminho": os.path.dirname(d), "relativo": m.get("relativo", ""),
                    "ferramenta": bool(m.get("ferramenta"))})
    if rebelle and os.path.exists(os.path.join(rebelle, "indice.csv")):
        ok = defaultdict(set)
        for r in csv.DictReader(open(os.path.join(rebelle, "indice.csv"))):
            if r["estado"] == "ok":
                ok[(r["TOOL"], r["nome"], r["preset"])].add(r["cena"])
        for (tool, nome, preset), cenas in sorted(ok.items()):
            if {"gestos", "tamanhos", "sobre"} <= cenas:
                out.append({"id": f"rebelle:{tool}:{preset}", "nome": preset, "fonte": "rebelle", "ferr": tool,
                            "caminho": os.path.join(rebelle, tool, nome), "teto": max_do_preset(tool, preset)})
    if artrage and os.path.isdir(artrage):
        for f in sorted(glob.glob(os.path.join(artrage, "*", "*_gestos_A.png"))):
            pre = f[: -len("_gestos_A.png")]
            ferr = os.path.basename(os.path.dirname(f))
            out.append({"id": f"artrage:{ferr}/{os.path.basename(pre)}", "nome": f"{ferr}/{os.path.basename(pre)}",
                        "fonte": "artrage", "ferr": ferr, "caminho": pre})
    return out


def _versao_da_regua():
    """A versão das MEDIDAS (o código que mede), não do ficheiro: trocar o `RUIDO` não invalida a cache."""
    import inspect
    import exame_regua as R
    fontes = [inspect.getsource(f) for f in (R.le, R.papel_de, R.tracos, R.densifica, R.bilinear, R.dist_a,
                                               R.meia_janela, R.faixa, R.box, R.extensao, R.desmistura,
                                               R.mede_traco, R.mede)]
    return hashlib.sha1("".join(fontes).encode()).hexdigest()[:12]


def mede_com_cache(args):
    png, cena, bat, cache, versao = args
    st = os.stat(png)
    chave = hashlib.sha1(f"{os.path.abspath(png)}|{st.st_mtime_ns}|{st.st_size}|{versao}".encode()).hexdigest()
    f = os.path.join(cache, chave + ".json") if cache else None
    if f and os.path.exists(f):
        return json.load(open(f))
    t = mede(png, cena, bat)["tracos"]
    if f:
        json.dump(t, open(f, "w"), default=float)
    return t


def mede_todos(pop, bat, cache):
    if cache:
        os.makedirs(cache, exist_ok=True)
    versao = _versao_da_regua()
    tarefas, onde = [], []
    for i, e in enumerate(pop):
        for c in bat["cenas"]:
            for foto in ("A", "B"):
                p = acha_foto(e["caminho"], c["nome"], foto)
                if p and not (foto == "B" and p == acha_foto(e["caminho"], c["nome"], "A")):
                    tarefas.append((p, c["nome"], bat, cache, versao))
                    onde.append((i, c["nome"], foto))
    with ProcessPoolExecutor(max_workers=min(12, os.cpu_count() or 4)) as ex:
        res = list(ex.map(mede_com_cache, tarefas, chunksize=4))
    fotos = [dict() for _ in pop]
    for (i, c, f), r in zip(onde, res):
        fotos[i][(c, f)] = r
    for fs in fotos:  # oráculo sem água (ArtRage; a foto única): B ≡ A
        for (c, f) in list(fs):
            if f == "A" and (c, "B") not in fs:
                fs[(c, "B")] = fs[(c, "A")]
    return fotos


# ---------------------------------------------------------------- o vector

def vector(fotos):
    """{(grupo, chave, família): valor} — a identidade de um pincel."""
    v = {}

    def g(c, f, t, k):
        x = fotos.get((c, f), {}).get(t, {}).get(k)
        return float("nan") if x is None else x

    def put(grupo, chave, x, fam):
        if isinstance(x, (list, tuple)):
            for i, y in enumerate(x):
                put(grupo, f"{chave}[{i}]", y, fam)
            return
        v[(grupo, chave, fam)] = float("nan") if x is None else float(x)

    gestos = ["recto_lento", "recto_rapido", "curva_s", "espiral", "rampa_0_1", "rampa_afunila", "vertical",
              "vertical_rapido"]
    tam = {"recto_5": 5.0, "recto_40": 40.0, "recto_200": 200.0}
    w0 = g("gestos", "A", "recto_lento", "largura") / 40.0
    put("forma", "recto_lento.largura_absoluta", w0, "largura")
    for t in gestos + list(tam) + ["papel", "cruza_1", "cruza_2", "cruza_3_rapido"]:
        c = "tamanhos" if t in tam else ("sobre" if t in ("papel",) or t.startswith("cruza") else "gestos")
        size = tam.get(t, 40.0)
        if t != "recto_lento":
            put("forma", f"{t}.largura", g(c, "A", t, "largura") / size / w0 if w0 > 0 else float("nan"), "largura")
        for k in ("pontas_ini", "pontas_fim"):
            put("forma", f"{t}.{k}", g(c, "A", t, k), "pontas")
        for k in ("alem_ini", "alem_fim"):
            put("forma", f"{t}.{k}", g(c, "A", t, k), "alem")
        for f in ("A", "B"):
            for k in ("cobertura", "opacidade"):
                put("tinta", f"{t}.{k}.{f}", g(c, f, t, k), k)
            cor = fotos.get((c, f), {}).get(t, {}).get("cor")
            put("tinta", f"{t}.cor.{f}", [x / 255 for x in cor] if cor else [float("nan")] * 3, "cor")
        for k in ("textura", "var_local", "rugosidade", "cv", "periodo_fracao"):
            put("textura", f"{t}.{k}", g(c, "A", t, k), k)
    for t in ("recto_lento", "vertical", "espiral"):
        p = fotos.get(("gestos", "A"), {}).get(t, {}).get("perfil") or [float("nan")] * 33
        put("forma", f"{t}.perfil", p[::4], "perfil")
    p = fotos.get(("tamanhos", "A"), {}).get("recto_200", {}).get("perfil") or [float("nan")] * 33
    put("forma", "recto_200.perfil", p[::4], "perfil")
    for t in ("rampa_0_1", "rampa_afunila"):
        for k in ("pressao_largura", "pressao_cobertura"):
            put("dinamica", f"{t}.{k}", fotos.get(("gestos", "A"), {}).get(t, {}).get(k) or [float("nan")] * 10,
                "pressao")

    def razao(x, y):
        return x / y if x == x and y == y and y > 1e-6 else float("nan")

    for a, b in (("recto_rapido", "recto_lento"), ("vertical", "recto_lento"), ("vertical_rapido", "vertical"),
                 ("curva_s", "recto_lento"), ("espiral", "recto_lento")):
        for k in ("largura", "cobertura", "textura"):
            put("dinamica", f"{a}/{b}.{k}", razao(g("gestos", "A", a, k), g("gestos", "A", b, k)), "razao")
    for t, s in (("recto_5", 5.0), ("recto_200", 200.0)):
        for k in ("largura", "cobertura", "textura"):
            x, y = g("tamanhos", "A", t, k), g("tamanhos", "A", "recto_40", k)
            if k == "largura":
                x, y = x / s, y / 40.0
            put("dinamica", f"{t}/recto_40.{k}", razao(x, y), "razao")
    for f in ("A", "B"):
        for t in ("cruza_1", "cruza_2", "cruza_3_rapido"):
            sob = fotos.get(("sobre", f), {}).get(t, {}).get("sobre", {})
            for base in ("faixa_molhada", "faixa_seca"):
                s = sob.get(base, {})
                for k in ("vermelho_em_cima", "vermelho_depois"):
                    put("mistura", f"{t}.{base}.{k}.{f}", s.get(k), "vermelho")
                cor = s.get("cor_em_cima")
                put("mistura", f"{t}.{base}.cor.{f}", [x / 255 for x in cor] if cor else [float("nan")] * 3, "cor")
    return v


# ---------------------------------------------------------------- o ruído do oráculo e a escala

def ruido_do_recto(fotos_oraculo):
    """O mesmo recto a 40 pintado duas vezes (recto_lento da cena gestos × recto_40 da cena tamanhos, foto A): o
    |Δ| de cada família em cada preset. Devolve {família: [Δ…]}."""
    d = defaultdict(list)
    for fs in fotos_oraculo:
        a = fs.get(("gestos", "A"), {}).get("recto_lento", {})
        b = fs.get(("tamanhos", "A"), {}).get("recto_40", {})
        if not a or not b or a.get("vazio") or b.get("vazio"):
            continue
        for k in ("cobertura", "opacidade", "textura", "var_local", "rugosidade", "cv", "periodo_fracao"):
            if a.get(k) is not None and b.get(k) is not None:
                d[k].append(abs(a[k] - b[k]))
        if a.get("largura") and b.get("largura"):
            d["largura"].append(abs(a["largura"] - b["largura"]) / 40.0)
        for k, fam in (("pontas_ini", "pontas"), ("pontas_fim", "pontas"), ("alem_ini", "alem"), ("alem_fim", "alem")):
            if a.get(k) is not None and b.get(k) is not None:
                d[fam].append(abs(a[k] - b[k]))
        if a.get("perfil") and b.get("perfil"):
            d["perfil"].extend(np.abs(np.asarray(a["perfil"][::4]) - np.asarray(b["perfil"][::4])).tolist())
        if a.get("cor") and b.get("cor"):
            d["cor"].extend((np.abs(np.asarray(a["cor"]) - np.asarray(b["cor"])) / 255).tolist())
    return {k: [x for x in v if math.isfinite(x)] for k, v in d.items()}


def escala_de(vecs_oraculo, ruido):
    chaves = sorted(set().union(*[v.keys() for v in vecs_oraculo]))
    X = np.array([[v.get(k, float("nan")) for k in chaves] for v in vecs_oraculo])
    med = np.nanmedian(X, 0)
    mad = np.nan_to_num(1.4826 * np.nanmedian(np.abs(X - med), 0), nan=0.0)
    chao = {f: float(np.percentile(ruido[f], 75)) for f in FAMILIAS_DO_RECTO if ruido.get(f)}
    for f, x in CHAO_MEDIDO.items():
        chao.setdefault(f, x)
    esc = [max(float(m), chao.get(k[2], 1e-3), 1e-6) for k, m in zip(chaves, mad)]
    return {"tracos": [[k[0], k[1], k[2], e] for k, e in zip(chaves, esc)], "chao": chao}


class Regua:
    def __init__(self, escala, escala_ferramenta=None):
        self.chaves = [tuple(t[:3]) for t in escala["tracos"]]
        esc = [t[3] for t in escala["tracos"]]
        # O grupo «ferramenta» com a escala PRÓPRIA dele (a da pintura, congelada, não muda).
        for k, e in sorted((escala_ferramenta or {}).get("escala", {}).items()):
            self.chaves.append(("ferramenta", k, k.split(".")[2]))
            esc.append(e)
        self.esc = np.array(esc)
        self.grupos = np.array([k[0] for k in self.chaves])

    def Z(self, vecs):
        return np.array([[v.get(k, float("nan")) for k in self.chaves] for v in vecs]) / self.esc

    def dist(self, Za, Zb, mascara=None):
        """Distâncias (|A| × |B|) e por grupo (|A| × |B| × 5)."""
        G = np.full((len(Za), len(Zb), len(GRUPOS)), np.nan)
        for gi, gn in enumerate(GRUPOS):
            sel = self.grupos == gn
            if mascara is not None:
                sel &= mascara
            if not sel.any():
                continue
            a, b = Za[:, sel], Zb[:, sel]
            for i in range(len(a)):
                dz = np.clip(np.abs(b - a[i]), 0, 6)
                ok = np.isfinite(dz)
                cnt = ok.sum(1)
                with np.errstate(invalid="ignore"):
                    G[i, :, gi] = np.where(cnt > 0, np.sqrt(np.where(ok, dz ** 2, 0).sum(1) / np.maximum(cnt, 1)),
                                           np.nan)
        return np.nanmean(G, 2), G


# ---------------------------------------------------------------- as séries graduadas do Rebelle

DUREZA = ["9H", "8H", "7H", "6H", "5H", "4H", "3H", "2H", "H", "F", "HB", "B", "2B", "3B", "4B", "5B", "6B", "7B",
          "8B", "9B"]


# As formas de PONTA (o vocabulário do pincel: Liner/Round/Flat de um mesmo grupo são a mesma família).
PONTAS = {"round", "flat", "filbert", "liner", "bullet", "chisel", "fan", "wash", "mop", "bright", "oval", "square",
          "wide"}


def chave_de_familia(ferr, grupo, nome):
    """A REGRA (04 §4, J2): o nome sem a dureza (2H…8B), sem o número do fim («Charcoal 2») e sem a forma da ponta
    (Liner · Round · Flat…) é a BASE; a família é (ferramenta, grupo, base). Devolve (chave, ordem, o que gradua)."""
    toks = nome.split()
    dur = [DUREZA.index(x) for x in toks if x in DUREZA]
    num = int(toks[-1]) if len(toks) > 1 and toks[-1].isdigit() else 1
    if len(toks) > 1 and toks[-1].isdigit():
        toks = toks[:-1]
    forma = tuple(sorted(x.lower() for x in toks if x.lower() in PONTAS))
    base = " ".join(x.lower() for x in toks if x not in DUREZA and x.lower() not in PONTAS)
    return (ferr, grupo, base), (forma, dur[0] if dur else -1, num), {"dureza": dur[:1], "numero": num,
                                                                      "forma": forma}


def series(itens):
    """As FAMÍLIAS pela regra: `itens` = [(i, ferramenta, grupo, nome)]. {chave+(tipo,): [(ordem, nome, i)…]}, só as de
    ≥ 2 membros; o tipo diz o que gradua (dureza · numero · forma, e as misturas)."""
    fam = defaultdict(list)
    for i, ferr, grupo, nome in itens:
        k, ordem, g = chave_de_familia(ferr, grupo, nome)
        fam[k].append((ordem, nome, i, g))
    out = {}
    for k, ms in fam.items():
        if len(ms) < 2:
            continue
        tipo = "+".join(n for n in ("dureza", "numero", "forma") if len({repr(m[3][n]) for m in ms}) > 1)
        out[k + (tipo,)] = sorted((o, n, i) for o, n, i, _ in ms)
    return out


def vizinhos_na_familia(membros, i):
    """Com quem o membro `i` tem de dar um PASSO visível: os adjacentes na mesma ponta (o eixo ordenado) e todos os
    de outra ponta (a forma não tem ordem)."""
    ordem = {m[2]: m[0] for m in membros}
    mesma = [m[2] for m in membros if m[0][0] == ordem[i][0]]
    k = mesma.index(i)
    return mesma[max(k - 1, 0):k] + mesma[k + 1:k + 2] + [m[2] for m in membros if m[0][0] != ordem[i][0]]


# ---------------------------------------------------------------- a lei

# O passo VISÍVEL dentro de uma família = o menor passo da série do Rebelle que a prancha MOSTRA: HB → 2B do
# Graphite Pencil (04 §4(b); 2H → H → HB não se distinguem a olho). Conta-se na escala de hoje (o 0,40 é o de 19/10).
PASSO_VISIVEL_PAR = ("PENCIL:Graphite Pencil/HB", "PENCIL:Graphite Pencil/2B")
DUPLICADO = 0.05  # fotos iguais ao byte dão 0,000; abaixo disto o par é o MESMO pincel com outro nome


def lei(nn_reb):
    """«Claramente longe» = acima do p10 da distância ao vizinho mais perto DENTRO do Rebelle, fora da própria série e
    fora dos duplicados ao byte: o menor afastamento que um fabricante profissional dá a dois pincéis que vende como
    diferentes (o porquê e as recusas no 04 §4)."""
    return float(np.percentile(nn_reb, 10))


def q(v, ps=(10, 25, 50, 75, 90)):
    v = [x for x in v if math.isfinite(x)]
    return " · ".join(f"{np.percentile(v, p):.2f}" for p in ps) if v else "—"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("biblioteca"); ap.add_argument("saida"); ap.add_argument("--bateria", default=BATERIA)
    ap.add_argument("--saiu", default=""); ap.add_argument("--recalibra", action="store_true")
    ap.add_argument("--rebelle", default=REBELLE); ap.add_argument("--artrage", default=ARTRAGE)
    ap.add_argument("--cache", default=os.path.join(os.environ.get("TMPDIR", "/tmp"), "ph2d_exame", "regua_cache"))
    ap.add_argument("--escala", default=ESCALA)
    x = ap.parse_args()
    bat = json.load(open(x.bateria))
    saiu = set(filter(None, x.saiu.split(",")))
    pop = populacao(x.biblioteca, x.rebelle, x.artrage)
    fotos = mede_todos(pop, bat, x.cache)
    # O recto de 200 px de um preset cujo Max Size é menor saiu AO MAX (o README das fotos): não é o pincel a 200, é o
    # tecto do Rebelle — esse traço fica fora da identidade dele (31 presets).
    for e, fs in zip(pop, fotos):
        if e.get("teto", 1e9) < 200:
            for f in ("A", "B"):
                if ("tamanhos", f) in fs:
                    fs[("tamanhos", f)] = {k: v for k, v in fs[("tamanhos", f)].items() if k != "recto_200"}
    vecs = [vector(f) for f in fotos]
    # As FERRAMENTAS (o `meta.json` da bancada): o grupo «ferramenta» da cena «sobre».
    import matriz_ferramenta
    for e, v in zip(pop, vecs):
        if e.get("ferramenta"):
            for k, val in matriz_ferramenta.vector(e["caminho"]).items():
                v[("ferramenta", k, k.split(".")[2])] = val
    idx = {f: [i for i, e in enumerate(pop) if e["fonte"] == f] for f in ("ph2d", "rebelle", "artrage")}
    ruido = ruido_do_recto([fotos[i] for i in idx["rebelle"]])
    if x.recalibra or not os.path.exists(x.escala):
        esc = escala_de([vecs[i] for i in idx["rebelle"]], ruido)
        os.makedirs(os.path.dirname(x.escala), exist_ok=True)
        json.dump(esc, open(x.escala, "w"), indent=0)
    esc_f = os.path.join(os.path.dirname(x.escala), "escala_ferramenta.json")
    regua = Regua(json.load(open(x.escala)), json.load(open(esc_f)) if os.path.exists(esc_f) else None)
    Z = regua.Z(vecs)
    D, G = regua.dist(Z, Z)
    np.fill_diagonal(D, np.inf)

    # o ruído de DISTÂNCIA: o vector do recto pintado duas vezes (as famílias do recto, só os traços recto_lento)
    mrecto = np.array([k[1].startswith("recto_lento.") and k[2] in FAMILIAS_DO_RECTO for k in regua.chaves])
    vb = []
    for i in idx["rebelle"]:
        f2 = dict(fotos[i])
        g2 = dict(f2.get(("gestos", "A"), {}))
        g2["recto_lento"] = f2.get(("tamanhos", "A"), {}).get("recto_40", {})
        f2[("gestos", "A")] = g2
        vb.append(vector(f2))
    Zb = regua.Z(vb)
    Zr = Z[idx["rebelle"]]
    d_ruido = [float(regua.dist(Zr[k:k + 1], Zb[k:k + 1], mrecto)[0][0, 0]) for k in range(len(Zr))]
    d_ruido = [d for d in d_ruido if math.isfinite(d)]
    duplicado = DUPLICADO
    d_recto = []  # a mesma máscara entre o preset e o seu vizinho: a escala que o ruído tem de ter ao lado
    for a, i in enumerate(idx["rebelle"]):
        j = idx["rebelle"][int(np.argmin([D[i, k] if k != i else np.inf for k in idx["rebelle"]]))]
        d_recto.append(float(regua.dist(Z[i:i + 1], Z[j:j + 1], mrecto)[0][0, 0]))

    # (a) e (b): o vizinho mais perto dentro de cada biblioteca, e as séries
    reb = idx["rebelle"]
    ser = series([(i, pop[i]["ferr"], *pop[i]["nome"].partition("/")[::2]) for i in reb])
    mesma_serie = {}
    for k, membros in ser.items():
        for _, _, i in membros:
            mesma_serie[i] = {j for _, _, j in membros}

    def nn(i, dentro, exclui=()):
        cand = [j for j in dentro if j != i and j not in exclui]
        j = min(cand, key=lambda j: D[i, j])
        return j, float(D[i, j])

    nn_reb = {i: nn(i, reb) for i in reb}
    dups = {i: {j for j in reb if j != i and D[i, j] < duplicado} for i in reb}
    # A FAMÍLIA = a série unida aos duplicados ao byte (o «2H Color» do Rebelle é o «2H» da série Graphite).
    pai = {i: i for i in reb}

    def raiz(i):
        while pai[i] != i:
            pai[i] = pai[pai[i]]
            i = pai[i]
        return i

    for i in reb:
        for j in mesma_serie.get(i, set()) | dups[i]:
            pai[raiz(i)] = raiz(j)
    familia = defaultdict(set)
    for i in reb:
        familia[raiz(i)].add(i)
    nn_reb_fora = {i: nn(i, reb, familia[raiz(i)]) for i in reb}
    nn_art = {i: nn(i, idx["artrage"]) for i in idx["artrage"]} if len(idx["artrage"]) > 1 else {}
    limiar = p10 = lei([d for _, d in nn_reb_fora.values()])
    piso = float(np.percentile(d_ruido, 50))
    passos = []
    for k, membros in ser.items():
        for (_, a, i), (_, b, j) in zip(membros, membros[1:]):
            passos.append((k, a, b, float(D[i, j]), GRUPOS[int(np.nanargmax(G[i, j]))]))

    L = ["# Os oráculos pela matriz — de onde sai a lei (gerado por `exame_matriz.py --recalibra`)", "",
         f"População: {len(reb)} presets do Rebelle (os que pintam as 3 cenas) · {len(idx['artrage'])} do ArtRage · "
         f"{len(idx['ph2d'])} nossos. Escala congelada em `matriz/escala.json` (o MAD de cada traço sobre o Rebelle, "
         "com o chão = o ruído do recto repetido).", "",
         "## (c) O ruído do oráculo — o mesmo recto a 40 pintado duas vezes (recto_lento × recto_40)", "",
         "| família | p50 | p75 (o chão da escala) | p90 | zeros |", "|---|---|---|---|---|"]
    for f in FAMILIAS_DO_RECTO:
        if ruido.get(f):
            v = np.asarray(ruido[f])
            L.append(f"| {f} | {np.percentile(v, 50):.4f} | {np.percentile(v, 75):.4f} | {np.percentile(v, 90):.4f} | "
                     f"{(v == 0).mean():.0%} |")
    L += ["", f"Em DISTÂNCIA (só os traços do recto, as famílias acima): o mesmo preset repintado dá "
              f"p10 · p25 · p50 · p75 · p90 = **{q(d_ruido)}**; o mesmo recto entre um preset e o seu vizinho "
              f"mais perto: {q(d_recto)}.", "",
          "## (a) A distância ao vizinho mais perto dentro do Rebelle — p10 · p25 · p50 · p75 · p90", "",
          "| ferramenta | n | qualquer vizinho | fora da própria série |", "|---|---|---|---|"]
    por = defaultdict(list)
    for i in reb:
        por[pop[i]["ferr"]].append(i)
    for ferr, ii in sorted(por.items()) + [("TOTAL", reb)]:
        L.append(f"| {ferr} | {len(ii)} | {q([nn_reb[i][1] for i in ii])} | {q([nn_reb_fora[i][1] for i in ii])} |")
    if nn_art:
        L.append(f"| ArtRage (dentro dele) | {len(nn_art)} | {q([d for _, d in nn_art.values()])} | — |")
    pares_dup = sorted({tuple(sorted((i, j))) for i in reb for j in dups[i]})
    L += ["", f"**Duplicados** (d < {duplicado:.2f}): {len(pares_dup)} pares — "
              + "; ".join(f"{pop[i]['nome']} = {pop[j]['nome']} ({D[i, j]:.3f})" for i, j in pares_dup[:12])
              + ". ⚠️ Os «X Color» e os «Graphite Pencil/X» do Rebelle dão fotos IGUAIS AO BYTE com a cor da bateria "
                "(sha1 conferido): só a cor de omissão os separa — o Rebelle vende duplicados."]
    L += ["", "Os 15 pares mais perto do Rebelle (fora da família: a série unida aos duplicados ao byte):", ""]
    pares = sorted({tuple(sorted((i, j))): d for i, (j, d) in nn_reb_fora.items()}.items(), key=lambda t: t[1])
    for (i, j), d in pares[:15]:
        L.append(f"- {pop[i]['ferr']} {pop[i]['nome']} ~ {pop[j]['ferr']} {pop[j]['nome']}: {d:.3f}")
    L += ["", "## (b) As séries graduadas do Rebelle — a distância entre membros VIZINHOS da série", "",
          "| série | passos (membro → membro: d (o grupo que mais muda)) |", "|---|---|"]
    for k, membros in sorted(ser.items()):
        ps = [f"{a} → {b}: {d:.2f} ({gr})" for kk, a, b, d, gr in passos if kk == k]
        L.append(f"| {k[0]} {k[1]}/{k[2] or '·'} ({k[3]}) | {' · '.join(ps)} |")
    dp = [d for *_, d, _ in passos]
    L += ["", f"Passos de série: {q(dp)} (n = {len(dp)}).", "",
          f"## A lei (a proposta está no 04 §4)", "",
          f"limiar = p10 do vizinho mais perto no Rebelle, fora da série e dos duplicados = **{limiar:.3f}** "
          f"(o recto repintado, só os traços do recto: mediana {piso:.3f})"]
    os.makedirs(x.saida, exist_ok=True)
    open(os.path.join(x.saida, "oraculos.md"), "w").write("\n".join(L) + "\n")

    # --- os nossos: a matriz entre eles, e os vizinhos de oráculo
    nos = idx["ph2d"]
    fica = [i for i in nos if pop[i]["nome"] not in saiu]
    nomes = [pop[i]["nome"] for i in nos]
    with open(os.path.join(x.saida, "matriz.csv"), "w", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["pincel"] + nomes)
        for i in nos:
            w.writerow([pop[i]["nome"]] + [("" if i == j else f"{D[i, j]:.3f}") for j in nos])
    with open(os.path.join(x.saida, "matriz_oraculos.csv"), "w", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        orac = reb + idx["artrage"]
        w.writerow(["pincel"] + [pop[j]["id"] for j in orac])
        for i in nos:
            w.writerow([pop[i]["nome"]] + [f"{D[i, j]:.3f}" for j in orac])
    id_reb = {f"{pop[j]['ferr']}:{pop[j]['nome']}": j for j in reb}
    top, rank_origem = {}, {}
    # As FAMÍLIAS da nossa fábrica pela MESMA regra (a pasta = meio/grupo) unidas pela ORIGEM: dois nossos cujas
    # origens no Rebelle são da mesma família (série ∪ duplicados ao byte) são da mesma família.
    a_, b_ = PASSO_VISIVEL_PAR
    passo_visivel = float(D[id_reb[a_], id_reb[b_]]) if a_ in id_reb and b_ in id_reb else 0.40
    pai_n = {i: i for i in nos}

    def raiz_n(i):
        while pai_n[i] != i:
            i = pai_n[i]
        return i

    def pasta(i):
        partes = pop[i].get("relativo", "").split("/")
        return (partes[-3], partes[-2]) if len(partes) >= 3 else ("", "")

    for membros in series([(i, *pasta(i), pop[i]["nome"]) for i in nos]).values():
        for _, _, j in membros[1:]:
            pai_n[raiz_n(j)] = raiz_n(membros[0][2])
    raiz_origem = {i: raiz(id_reb[ORIGEM[pop[i]["nome"]]]) for i in nos if ORIGEM.get(pop[i]["nome"]) in id_reb}
    for i in raiz_origem:
        for j in raiz_origem:
            if i < j and raiz_origem[i] == raiz_origem[j]:
                pai_n[raiz_n(j)] = raiz_n(i)
    grupos_n = defaultdict(list)
    for i in nos:
        grupos_n[raiz_n(i)].append((chave_de_familia(*pasta(i), pop[i]["nome"])[1], pop[i]["nome"], i))
    FAMILIAS = {" · ".join(n for _, n, _ in sorted(ms)): sorted(ms) for ms in grupos_n.values() if len(ms) >= 2}
    V = ["# Vizinhos mais perto — a TRIAGEM do EXAME (gerado por `exame_matriz.py`)", "",
         "⚠️ A matriz é TRIAGEM: diz onde olhar. O veredito de um pincel é a PRANCHA lado a lado + a REVISÃO CEGA "
         "(05_REVISAO_CEGA.md).", "",
         f"**Lei** (04_BANCADA §4; medições em `oraculos.md`): «claramente longe» = d ≥ **{limiar:.3f}** = o p10 da "
         "distância ao vizinho mais perto dentro do Rebelle (228 presets), fora da própria série e dos duplicados ao byte. "
         "Uma SÉRIE graduada (dureza/densidade) entra como FAMÍLIA: fora dela, cada membro tem de estar ≥ limiar de tudo; "
         f"dentro, cada passo ≥ **{passo_visivel:.3f}** (o HB → 2B do Graphite Pencil do Rebelle, o menor que a prancha "
         "MOSTRA) e o eixo tem de se VER na prancha. A coluna «origem» é a FIDELIDADE: o preset do Rebelle de que o "
         "nosso nasce tem de ser o 1.º entre os 228.", "",
         "**As famílias** (pela regra do 04 §4: o nome sem dureza, número e ponta, na mesma pasta; e unidas pela "
         "família da ORIGEM no Rebelle): " + " ; ".join(f"[{f}]" for f in FAMILIAS) + ".", "",
         "| pincel | 1.º vizinho NOSSO fora da família (d) | por grupo (forma · tinta · textura · dinâmica · mistura) | 2.º · 3.º nosso "
         "| triagem | Rebelle mais perto (d) | origem no Rebelle: posição entre 228 (d) | ArtRage mais perto (d) |",
         "|---|---|---|---|---|---|---|---|"]
    perto = passo_curto = 0
    fam_de = {n: f for f, ms in FAMILIAS.items() for _, n, _ in ms}
    por_nome = {pop[i]["nome"]: i for i in nos}
    for i in nos:
        n = pop[i]["nome"]
        ordem = sorted((j for j in fica if j != i), key=lambda j: D[i, j])
        top[n] = [(pop[j]["nome"], round(float(D[i, j]), 3)) for j in ordem[:3]]
        fam = fam_de.get(n)
        fora = [j for j in ordem if fam is None or fam_de.get(pop[j]["nome"]) != fam]
        j = fora[0]
        ver = "claramente longe" if D[i, j] >= limiar else "PERTO — olhar a prancha"
        if fam:
            viz = vizinhos_na_familia(FAMILIAS[fam], i)
            curto = [pop[v]["nome"] for v in viz if D[i, v] < passo_visivel]
            ver += " · família: passo " + ", ".join(f"{pop[v]['nome']} {D[i, v]:.2f}" for v in viz)
            if curto:
                ver += " — PASSO INVISÍVEL"
                passo_curto += n not in saiu
        if D[i, j] < limiar and n not in saiu:
            perto += 1
        r = sorted(reb, key=lambda k: D[i, k])
        o = ORIGEM.get(n)
        if o and o in id_reb:
            b = id_reb[o]
            # um duplicado AO BYTE da origem (o «2H Color» do «2H») empata com ela
            pos = 1 + sum(1 for k in reb if D[i, k] < D[i, b] and D[b, k] >= DUPLICADO)
            rank_origem[n] = pos
            orig = f"{'✓ 1.º' if pos == 1 else f'{pos}.º'} ({D[i, id_reb[o]]:.3f})"
        else:
            orig = "—"
        art = min(idx["artrage"], key=lambda k: D[i, k]) if idx["artrage"] else None
        g = " · ".join(f"{v:.2f}" for v in G[i, j])
        V.append(f"| {n}{' (saiu)' if n in saiu else ''} | {pop[j]['nome']} ({D[i, j]:.3f}) | {g} | "
                 f"{', '.join(f'{a} ({b})' for a, b in top[n][1:])} | {ver} | {pop[r[0]]['nome']} ({D[i, r[0]]:.3f}) | "
                 f"{orig} | {pop[art]['nome'] + f' ({D[i, art]:.3f})' if art is not None else '—'} |")
    um = sum(1 for v in rank_origem.values() if v == 1)
    V[4:4] = [f"**{perto} dos {len(fica)} de fábrica** têm o 1.º vizinho nosso (fora da própria família) abaixo do limiar; "
              f"{passo_curto} têm um passo de família menor que o visível ({passo_visivel:.3f}). A origem no Rebelle é o "
              f"mais perto entre os 228 em **{um} de {len(rank_origem)}** (mediana da posição "
              f"{int(np.median(list(rank_origem.values()))) if rank_origem else 0}).", ""]
    open(os.path.join(x.saida, "vizinhos.md"), "w").write("\n".join(V) + "\n")
    json.dump({"pinceis": nomes, "calibracao": {"limiar": limiar, "p10_vizinho_rebelle": p10, "piso_ruido": piso,
                                                "passo_visivel": passo_visivel},
               "familias": {f: [n for _, n, _ in ms] for f, ms in FAMILIAS.items()},
               "top3": top, "rank_origem": rank_origem,
               "dirs": {pop[i]["nome"]: pop[i]["caminho"] for i in nos},
               "grupos": {pop[i]["nome"]: {pop[j]["nome"]: G[i, j].round(3).tolist() for j in nos if j != i}
                          for i in nos}},
              open(os.path.join(x.saida, "matriz.json"), "w"), indent=1, default=float)
    print("\n".join(V[:6]))
    print(f"limiar {limiar:.3f} (p10 {p10:.3f}, piso {piso:.3f}); ruído de distância {q(d_ruido)}")


if __name__ == "__main__":
    main()
