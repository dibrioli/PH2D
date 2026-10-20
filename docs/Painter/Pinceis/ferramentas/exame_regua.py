#!/usr/bin/env python3
"""A RÉGUA do EXAME (00_MISSAO §3.2), generalizada aos gestos da BATERIA: a mesma régua para o oráculo e para nós.

Cada traço da cena é ENDIREITADO: a polilinha EXACTA do `bateria.json` reamostrada a 1 px de arco, e em cada ponto a
cobertura (c = máx |px − papel| / 255, a de `NewFeaturesWetPaint/ferramentas/mede_traco.py`) amostrada ao longo da
NORMAL (bilinear, ±H px). Sobre a faixa endireitada (comprimento × largura), por traço:

  largura      — mediana da largura a ¼ do máximo de cada amostra (a «que se vê», `sonda_gestos::largura_que_se_ve`) e a ½
  cobertura    — média no miolo (|o| ≤ 0,35 · largura) · opacidade = p90 no miolo
  perfil       — o perfil TRANSVERSAL médio (centrado em cada amostra, ÷ o nível do traço), 33 pontos em ±1,5 meia-largura
  rugosidade   — o desvio-padrão (px) das duas bordas (a ½ do nível) em torno da média móvel de 25 px
  textura      — resíduo da média local 5×5 no miolo ÷ a média (%), como o `mede_traco`; `var_local` e o PERÍODO dominante
                 ao longo do traço (fração da potência: carimbo visível)
  cv           — variação da cobertura do miolo ao longo do traço (o «colar de contas»)
  cor          — RGB médio do miolo
  pontas       — largura nos 5 % de cada ponta ÷ a mediana · `alem` = quanto a tinta passa do ponto final (÷ meia-largura)
  pressao      — nos traços com pressão que varia: largura e cobertura por decil de pressão ÷ as da pressão ≥ 0,9
  sobre        — num traço que cruza outro (a cena `sobre`): por cruzamento, o VERMELHO (desmistura das absorvâncias
                 azul × vermelho, 0 = só azul) EM CIMA da faixa e nos 70 px DEPOIS dela (o arrasto), e a cor em cima

As amostras a menos de ½(tamanho dos dois) + 6 px de OUTRO traço da cena saem das métricas «limpas».

uso: exame_regua.py mede <png> <cena> [--bateria b.json]                — as métricas (JSON)
     exame_regua.py compara <oraculo-dir> <nosso-dir> [--bateria b.json] [--json saida.json]
         os PNG/PPM `<cena>_<A|B>` dos dois lados (ver `acha_foto`): a tabela legível, e o JSON com tudo
"""
import argparse
import glob
import json
import math
import os
import sys

import numpy as np
from PIL import Image

AQUI = os.path.dirname(os.path.abspath(__file__))
BATERIA = os.path.join(AQUI, "bateria.json")

# O RUÍDO de cada métrica (a diferença que não conta), MEDIDO no Rebelle (04_BANCADA §3.1, `ruido_regua.py`): o mesmo
# gesto pintado duas vezes pelos 228 presets, o p75. Um número = constante; um par (a, b) = máx(a, b·|o valor do
# oráculo|) (a métrica cujo ruído cresce com o nível: textura 0,2 a 1 % · 9 a 38 %). O `compara` usa o maior entre
# este e o do PRÓPRIO oráculo (o mesmo recto a 40 em três sítios: recto_lento · recto_40 · papel).
RUIDO = {"largura": 1.0, "largura50": 1.0,  # px; 3 ruídos = 3 px ⊃ a promessa ± 2 px (05 §4.1)
         "cobertura": 0.0168, "opacidade": 0.0196, "rugosidade": (0.0412, 0.229), "textura": (0.196, 0.21),
         "var_local": (0.000116, 0.392), "cv": (0.00122, 0.29), "periodo_fracao": (0.034, 0.565), "cor": 5.32,
         "pontas_ini": 0.0357, "pontas_fim": 0.0513, "alem_ini": (0.0579, 0.153), "alem_fim": 0.146,
         "perfil": 0.0346, "pressao_largura": 0.0512, "pressao_cobertura": 0.0982,
         "vermelho_em_cima": (0.0168, 0.149), "vermelho_depois": (7.58e-05, 0.239), "cor_em_cima": 7.81}
ESCALARES = ["largura", "largura50", "cobertura", "opacidade", "rugosidade", "textura", "var_local", "cv",
             "periodo_fracao", "pontas_ini", "pontas_fim", "alem_ini", "alem_fim"]


def ruido_de(k, oraculo, proprio=None):
    """O ruído da métrica `k` ao nível do ORÁCULO (o nosso não alarga a própria tolerância)."""
    r = RUIDO.get(k)
    if r is None:
        return None
    if isinstance(r, tuple):
        r = max(r[0], r[1] * abs(oraculo)) if isinstance(oraculo, (int, float)) and math.isfinite(oraculo) else r[0]
    return max(r, (proprio or {}).get(k, 0.0))


def le(png):
    return np.asarray(Image.open(png).convert("RGB"), dtype=np.float32)


def papel_de(a, borda=6):
    f = np.concatenate([a[:borda].reshape(-1, 3), a[-borda:].reshape(-1, 3),
                        a[:, :borda].reshape(-1, 3), a[:, -borda:].reshape(-1, 3)])
    return np.median(f, axis=0)


def tracos(bat, cena):
    """Os traços da cena pela ordem, com a cor e o pincel em vigor."""
    c = next(c for c in bat["cenas"] if c["nome"] == cena)
    out, cor, quem = [], None, "preset"
    for op in c["ops"]:
        if op["op"] == "cor":
            cor = op["rgb"]
        elif op["op"] == "pincel":
            quem = op["quem"]
        elif op["op"] == "traco":
            out.append({"nome": op["nome"], "size": float(op["size_px"]), "cor": cor, "quem": quem,
                        "pts": np.asarray(op["pts"], dtype=np.float64)})
    return out


def densifica(pts, passo=1.0):
    """A polilinha a cada `passo` px de arco: (xy, pressão, s, tangente unitária)."""
    xy, p = pts[:, :2], pts[:, 2]
    seg = np.hypot(*np.diff(xy, axis=0).T)
    keep = np.concatenate([[True], seg > 1e-9])
    xy, p = xy[keep], p[keep]
    s0 = np.concatenate([[0.0], np.cumsum(np.hypot(*np.diff(xy, axis=0).T))])
    s = np.arange(0.0, s0[-1] + 1e-9, passo)
    q = np.stack([np.interp(s, s0, xy[:, 0]), np.interp(s, s0, xy[:, 1])], 1)
    pr = np.interp(s, s0, p)
    k = 3
    pad = np.concatenate([np.repeat(q[:1], k, 0), q, np.repeat(q[-1:], k, 0)])
    t = pad[2 * k:] - pad[:-2 * k]
    t /= np.maximum(np.hypot(*t.T), 1e-9)[:, None]
    return q, pr, s, t


def bilinear(img, x, y):
    h, w = img.shape[:2]
    x = np.clip(x, 0, w - 1.001)
    y = np.clip(y, 0, h - 1.001)
    x0, y0 = np.floor(x).astype(int), np.floor(y).astype(int)
    fx, fy = x - x0, y - y0
    if img.ndim == 3:
        fx, fy = fx[..., None], fy[..., None]
    return (img[y0, x0] * (1 - fx) * (1 - fy) + img[y0, x0 + 1] * fx * (1 - fy)
            + img[y0 + 1, x0] * (1 - fx) * fy + img[y0 + 1, x0 + 1] * fx * fy)


def dist_a(q, outro):
    """A distância de cada ponto de `q` à polilinha densificada `outro` (força bruta, em blocos)."""
    d = np.full(len(q), np.inf)
    for i in range(0, len(q), 512):
        b = q[i:i + 512]
        d[i:i + 512] = np.sqrt(((b[:, None, :] - outro[None, ::2, :]) ** 2).sum(2)).min(1)
    return d


def meia_janela(q, size):
    """±H da faixa: o tamanho, mas no máximo metade do PASSO da própria espiral (os braços vizinhos)."""
    H = max(6.0, float(size))
    sub = q[::4]
    s = np.arange(len(sub)) * 4.0
    if len(sub) > 8:
        d = np.sqrt(((sub[:, None] - sub[None]) ** 2).sum(2))
        longe = np.abs(s[:, None] - s[None]) > 3 * size + 20
        if longe.any():
            H = min(H, 0.5 * float(d[longe].min()))
    return max(6.0, H)


def faixa(c, rgb, q, t, H, ext):
    """A faixa endireitada: `ext` px antes e depois das pontas (as tampas), ±H na normal."""
    if ext > 0:
        e = np.arange(ext, 0, -1.0)
        q = np.concatenate([q[:1] - e[:, None] * t[:1], q, q[-1:] + e[::-1, None] * t[-1:]])
        t = np.concatenate([np.repeat(t[:1], len(e), 0), t, np.repeat(t[-1:], len(e), 0)])
    n = np.stack([-t[:, 1], t[:, 0]], 1)
    o = np.arange(-H, H + 1.0)
    X = q[:, 0, None] + n[:, 0, None] * o[None]
    Y = q[:, 1, None] + n[:, 1, None] * o[None]
    return bilinear(c, X, Y), bilinear(rgb, X, Y), o


def box(a, r):
    p = np.pad(a, r, mode="edge")
    s = np.zeros((p.shape[0] + 1, p.shape[1] + 1))
    s[1:, 1:] = p.cumsum(0).cumsum(1)
    k = 2 * r + 1
    n, m = a.shape
    return (s[k:k + n, k:k + m] - s[:n, k:k + m] - s[k:k + n, :m] + s[:n, :m]) / (k * k)


def extensao(row, lim):
    ys = np.where(row >= lim)[0]
    return (ys[0], ys[-1]) if len(ys) else (None, None)


def desmistura(rgb, papel, azul, verm):
    """A fração de VERMELHO de cada pixel: d = papel − rgb ≈ α·d_azul + β·d_vermelho (mínimos quadrados, ≥ 0)."""
    da, dv = papel - np.asarray(azul, float), papel - np.asarray(verm, float)
    M = np.stack([da, dv], 1)  # 3×2
    d = (papel - rgb).reshape(-1, 3)
    ab, *_ = np.linalg.lstsq(M, d.T, rcond=None)
    ab = np.clip(ab, 0, None)
    tot = ab.sum(0)
    return np.where(tot > 0.15, ab[1] / np.maximum(tot, 1e-9), np.nan).reshape(rgb.shape[:-1])


def mede_traco(c, rgb, papel, tr, todos, bat_cores, amostras=False):
    """`amostras`: devolve também, em `_amostras`, as séries por amostra da resposta à pressão (o `ruido_regua.py`
    parte a rampa que afunila em subida × descida)."""
    q, pr, s, t = densifica(tr["pts"])
    size = tr["size"]
    H = meia_janela(q, size)
    ext = int(round(0.75 * size + 4))
    S, R, o = faixa(c, rgb, q, t, H, ext)
    core = slice(ext, ext + len(q))
    Sc, Rc = S[core], R[core]
    # amostras LIMPAS: longe de qualquer outro traço da cena
    limpa = np.ones(len(q), bool)
    perto = {}
    for u in todos:
        if u is tr:
            continue
        qu = densifica(u["pts"], 2.0)[0]
        d = dist_a(q, qu)
        perto[u["nome"]] = d
        limpa &= d > 0.5 * (size + u["size"]) + 6
    m = {"nome": tr["nome"], "size": size, "H": H, "n": int(len(q)), "limpas": int(limpa.sum())}
    if limpa.sum() < 10:
        limpa = np.ones(len(q), bool)
        m["sem_amostras_limpas"] = True
    mx = Sc.max(1)
    nivel = float(np.percentile(mx[limpa], 90)) if limpa.any() else 0.0
    m["nivel"] = nivel
    if nivel < 0.06:
        m["vazio"] = True
        return m
    larg25, larg50, cen, lo, hi = (np.full(len(q), np.nan) for _ in range(5))
    for i in range(len(q)):
        if mx[i] < 0.08:
            larg25[i] = larg50[i] = 0.0
            continue
        a, b = extensao(Sc[i], 0.25 * mx[i])
        larg25[i] = b - a + 1
        a, b = extensao(Sc[i], 0.5 * mx[i])
        larg50[i] = b - a + 1
        w = Sc[i]
        cen[i] = float((w * o).sum() / max(w.sum(), 1e-9))
        a, b = extensao(Sc[i], 0.5 * nivel)
        if a is not None:
            lo[i], hi[i] = o[a], o[b]
    L = len(q)
    miolo = limpa.copy()
    miolo[: int(0.1 * L)] = False
    miolo[int(0.9 * L):] = False
    if miolo.sum() < 10:
        miolo = limpa
    larg = float(np.nanmedian(larg25[limpa]))
    m["largura"] = larg
    m["largura50"] = float(np.nanmedian(larg50[limpa]))
    hw = max(larg / 2, 1.0)
    banda = np.abs(o[None, :] - np.nan_to_num(cen)[:, None]) <= 0.35 * larg
    pix = banda & miolo[:, None]
    if pix.sum() > 20:
        m["cobertura"] = float(Sc[pix].mean())
        m["opacidade"] = float(np.percentile(Sc[pix], 90))
        res = Sc - box(Sc, 2)
        m["textura"] = float(100 * res[pix].std() / max(Sc[pix].mean(), 1e-6))
        var = box(Sc ** 2, 2) - box(Sc, 2) ** 2
        m["var_local"] = float(var[pix].mean() / max(Sc[pix].mean() ** 2, 1e-6))
        serie = np.array([Sc[i][banda[i]].mean() if banda[i].any() else np.nan for i in range(L)])
        sm = serie[miolo]
        sm = sm[np.isfinite(sm)]
        m["cv"] = float(sm.std() / max(sm.mean(), 1e-6)) if len(sm) > 10 else float("nan")
        if len(sm) > 64:
            det = sm - box(sm[None, :], 15)[0]
            sp = np.abs(np.fft.rfft(det * np.hanning(len(det)))) ** 2
            f = np.fft.rfftfreq(len(det))
            sp[f < 1 / 200] = 0
            k = int(sp.argmax())
            m["periodo"] = float(1 / f[k]) if f[k] > 0 else float("inf")
            m["periodo_fracao"] = float(sp[k] / max(sp.sum(), 1e-12))
        tinta = pix & (Sc > 0.5 * nivel)
        m["cor"] = (Rc[tinta].mean(0) if tinta.sum() > 10 else Rc[pix].mean(0)).round(1).tolist()
    # rugosidade das duas bordas
    rug = []
    for e in (lo, hi):
        ok = limpa & np.isfinite(e)
        if ok.sum() > 40:
            v = e[ok]
            rug.append(float((v - box(v[None, :], 12)[0]).std()))
    m["rugosidade"] = float(np.mean(rug)) if rug else float("nan")
    # perfil transversal médio, centrado, ÷ nível, a 33 pontos em ±1,5 hw
    grade = np.linspace(-1.5 * hw, 1.5 * hw, 33)
    perf = [np.interp(grade, o - cen[i], Sc[i], left=0, right=0) for i in np.where(limpa & np.isfinite(cen))[0]]
    m["perfil"] = (np.mean(perf, 0) / nivel).round(4).tolist() if perf else []
    # pontas e o que passa das pontas
    n5 = max(3, int(0.05 * L))
    med = max(larg, 1.0)
    m["pontas_ini"] = float(np.nanmedian(larg25[:n5]) / med)
    m["pontas_fim"] = float(np.nanmedian(larg25[-n5:]) / med)
    for lado, bloco in (("alem_ini", S[:ext][::-1]), ("alem_fim", S[ext + L:])):
        tem = np.where(bloco.max(1) >= 0.25 * nivel)[0]
        m[lado] = float((tem[-1] + 1) / hw) if len(tem) else 0.0
    # a resposta à pressão
    if np.ptp(pr) > 0.3:
        topo = limpa & (pr >= 0.9)
        wl = np.nanmedian(larg25[topo]) if topo.any() else np.nan
        serie_cob = np.array([Sc[i][banda[i]].mean() if banda[i].any() else np.nan for i in range(L)])
        cl = np.nanmean(serie_cob[topo]) if topo.any() else np.nan
        pl, pc = [], []
        for k in range(10):
            sel = limpa & (pr >= k / 10) & (pr < (k + 1) / 10 + (k == 9))
            pl.append(float(np.nanmedian(larg25[sel]) / wl) if sel.sum() > 3 and wl > 0 else float("nan"))
            pc.append(float(np.nanmean(serie_cob[sel]) / cl) if sel.sum() > 3 and cl > 0 else float("nan"))
        m["pressao_largura"] = [round(v, 4) for v in pl]
        m["pressao_cobertura"] = [round(v, 4) for v in pc]
        if amostras:
            m["_amostras"] = {"pr": pr, "larg25": larg25, "cob": serie_cob, "limpa": limpa, "wl": wl, "cl": cl}
    # sobre outra tinta: os traços-base que este cruza
    azul, verm = bat_cores
    cruz = {}
    for u in todos:
        if u is tr or u["nome"] not in perto or u["cor"] == tr["cor"]:
            continue
        d = perto[u["nome"]]
        if d.min() > 0.5 * u["size"]:
            continue
        sx = s[int(d.argmin())]
        em = np.abs(s - sx) < 0.5 * u["size"] - 4
        depois = (s > sx + 0.5 * u["size"] + 4) & (s < sx + 0.5 * u["size"] + 74)
        r = desmistura(Rc, papel, azul, verm)
        bandaq = banda & (Sc > 0.3 * nivel)

        def media(sel, x=r):
            v = x[sel[:, None] & bandaq]
            v = v[np.isfinite(v)] if v.ndim == 1 else v
            return float(np.nanmean(v)) if len(v) > 10 else float("nan")

        cruz[u["nome"]] = {
            "vermelho_em_cima": media(em), "vermelho_depois": media(depois),
            "cor_em_cima": (Rc[em[:, None] & bandaq].mean(0).round(1).tolist()
                            if (em[:, None] & bandaq).sum() > 10 else None)}
    if cruz:
        m["sobre"] = cruz
    return m


def mede(png, cena, bat):
    a = le(png)
    papel = papel_de(a)
    c = np.abs(a - papel).max(2) / 255.0
    ts = tracos(bat, cena)
    azul = next((t["cor"] for t in ts if t["quem"] == "preset"), [23, 101, 179])
    verm = next((t["cor"] for t in ts if t["cor"] != azul), [191, 0, 18])
    return {"png": png, "cena": cena, "papel": papel.round(1).tolist(),
            "tracos": {t["nome"]: mede_traco(c, a, papel, t, ts, (azul, verm)) for t in ts}}


def acha_foto(d, cena, foto):
    """A foto `<cena>_<foto>` numa pasta: `.png` ou `.ppm`, com prefixo (`<preset>_<cena>_<foto>`), ou só `<cena>`
    (um oráculo sem água — MyPaint, Krita — tem uma foto por cena: vale para A e B). `d` que não é pasta é um
    PREFIXO: `fixtures/rebelle_bateria/PENCIL/graphite_pencil_8b` ⇒ `…_8b_<cena>_<foto>.png`."""
    if d and not os.path.isdir(d):
        d, pre = os.path.dirname(d), glob.escape(os.path.basename(d)) + "_"
        padroes = (f"{pre}{cena}_{foto}", f"{pre}{cena}")
    else:
        padroes = (f"{cena}_{foto}", f"*_{cena}_{foto}", f"{cena}", f"*_{cena}")
    for padrao in padroes:
        for ext in (".png", ".ppm"):
            achados = sorted(glob.glob(os.path.join(d, padrao + ext)))
            if achados:
                return achados[0]
    return None


def cenas_e_fotos(bat):
    for c in bat["cenas"]:
        for op in c["ops"]:
            if op["op"] == "foto":
                yield c["nome"], op["nome"]


def dist_metrica(k, a, b):
    """A diferença de uma métrica entre dois lados (escalar, vector médio, ou cor euclidiana)."""
    if a is None or b is None:
        return float("nan")
    if k in ("cor", "cor_em_cima"):
        return float(np.linalg.norm(np.asarray(a) - np.asarray(b)))
    if isinstance(a, list):
        x, y = np.asarray(a, float), np.asarray(b, float)
        if x.shape != y.shape:
            return float("nan")
        ok = np.isfinite(x) & np.isfinite(y)
        return float(np.abs(x[ok] - y[ok]).mean()) if ok.any() else float("nan")
    return float(b - a)


RECTOS_40 = [("gestos", "recto_lento"), ("tamanhos", "recto_40"), ("sobre", "papel")]


def ruido_do_oraculo(med):
    """O ruído PRÓPRIO do oráculo: o maior desvio entre os três rectos a 40 da bateria (o mesmo gesto, três sítios)."""
    r = {}
    for k in ESCALARES + ["perfil", "cor"]:
        vs = []
        for foto in ("A", "B"):
            xs = [med.get((c, foto), {}).get("tracos", {}).get(t, {}).get(k) for c, t in RECTOS_40]
            xs = [x for x in xs if x is not None and not (isinstance(x, float) and math.isnan(x))]
            for i in range(len(xs)):
                for j in range(i + 1, len(xs)):
                    vs.append(abs(dist_metrica(k, xs[i], xs[j])))
        if vs:
            r[k] = float(np.nanmax(vs))
    return r


def compara(oraculo, nosso, bat):
    med_o, med_n = {}, {}
    for cena, foto in cenas_e_fotos(bat):
        po, pn = acha_foto(oraculo, cena, foto), acha_foto(nosso, cena, foto)
        if po and pn:
            med_o[(cena, foto)] = mede(po, cena, bat)
            med_n[(cena, foto)] = mede(pn, cena, bat)
    proprio = ruido_do_oraculo(med_o)

    def item(k, o, n):
        d = dist_metrica(k, o, n)
        r = ruido_de(k, o, proprio)
        return {"oraculo": o, "nosso": n, "dif": d, "ruido": r,
                "ruidos": abs(d) / r if r and math.isfinite(d) else None}

    linhas = []
    for (cena, foto), mo in med_o.items():
        mn = med_n[(cena, foto)]
        for nome, to in mo["tracos"].items():
            tn = mn["tracos"][nome]
            chaves = [k for k in ESCALARES + ["perfil", "cor", "pressao_largura", "pressao_cobertura"]
                      if k in to or k in tn]
            met = {k: item(k, to.get(k), tn.get(k)) for k in chaves}
            for base, so in to.get("sobre", {}).items():
                sn = tn.get("sobre", {}).get(base, {})
                for k in ("vermelho_em_cima", "vermelho_depois", "cor_em_cima"):
                    met[f"{k}:{base}"] = item(k, so.get(k), sn.get(k))
            linhas.append({"cena": cena, "foto": foto, "traco": nome, "metricas": met,
                           "vazio": {"oraculo": bool(to.get("vazio")), "nosso": bool(tn.get("vazio"))}})
    return {"oraculo": oraculo, "nosso": nosso, "ruido": {k: list(v) if isinstance(v, tuple) else v
                                                          for k, v in RUIDO.items()},
            "ruido_do_oraculo": proprio, "fotos": [f"{c}_{f}" for c, f in med_o], "linhas": linhas}


def fmt(v):
    if v is None:
        return "—"
    if isinstance(v, list):
        if len(v) == 3:
            return "(" + ",".join(f"{x:.0f}" for x in v) + ")"
        return "[…]"
    if isinstance(v, float) and math.isnan(v):
        return "nan"
    return f"{v:.3g}"


TABELA = ["largura", "cobertura", "opacidade", "rugosidade", "textura", "cv", "periodo_fracao", "cor", "pontas_ini",
          "pontas_fim", "perfil", "pressao_largura", "pressao_cobertura"]


def tabela(r, so_foto=None):
    def rr(v):
        return f"máx({v[0]:.3g}, {v[1]:.3g}·oráculo)" if isinstance(v, list) else f"{v:.3g}"

    out = [f"oráculo {r['oraculo']}\nnosso   {r['nosso']}",
           "ruído (MEDIDO, 04 §3.1; o maior entre este e o do próprio oráculo): "
           + " ".join(f"{k} {rr(v)}" for k, v in r["ruido"].items()),
           "ruído do próprio oráculo: " + " ".join(f"{k} {v:.3g}" for k, v in r["ruido_do_oraculo"].items()), "",
           "✓ = |dif| ≤ 3 ruídos · ✗ = fora · cada célula: oráculo → nosso", ""]
    tot = ok = 0
    for ln in r["linhas"]:
        if so_foto and ln["foto"] != so_foto:
            continue
        cel = []
        for k, v in ln["metricas"].items():
            if k not in TABELA and ":" not in k:
                continue
            rr = v["ruidos"]
            marca = "" if rr is None else ("✓" if rr <= 3 else "✗")
            if rr is not None:
                tot += 1
                ok += rr <= 3
            if isinstance(v["oraculo"], list) and len(v["oraculo"]) != 3:
                cel.append(f"{k} Δ{fmt(v['dif'])}{marca}")
            else:
                cel.append(f"{k} {fmt(v['oraculo'])}→{fmt(v['nosso'])}{marca}")
        vz = " (VAZIO no oráculo)" if ln["vazio"]["oraculo"] else (" (VAZIO no nosso)" if ln["vazio"]["nosso"] else "")
        out.append(f"{ln['cena']}_{ln['foto']} · {ln['traco']}{vz}: " + " · ".join(cel))
    out.append("")
    out.append(f"dentro de 3 ruídos: {ok}/{tot} — ⚠️ régua verde é condição NECESSÁRIA, nunca suficiente (P53).")
    return "\n".join(out)


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("mede")
    a.add_argument("png"); a.add_argument("cena"); a.add_argument("--bateria", default=BATERIA)
    b = sub.add_parser("compara")
    b.add_argument("oraculo"); b.add_argument("nosso"); b.add_argument("--bateria", default=BATERIA)
    b.add_argument("--json"); b.add_argument("--txt")
    x = ap.parse_args()
    bat = json.load(open(x.bateria))
    if x.cmd == "mede":
        print(json.dumps(mede(x.png, x.cena, bat), indent=1))
        return
    r = compara(x.oraculo, x.nosso, bat)
    if not r["linhas"]:
        sys.exit(f"nenhuma foto em comum entre {x.oraculo} e {x.nosso}")
    t = tabela(r)
    print(t)
    if x.json:
        json.dump(r, open(x.json, "w"), indent=1, default=float)
    if x.txt:
        open(x.txt, "w").write(t + "\n")


if __name__ == "__main__":
    main()
