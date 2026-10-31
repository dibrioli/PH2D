#!/usr/bin/env python3
"""F2b — as leis candidatas de MISTURA de cor (só o espaço em que duas tintas se misturam; a lei de CAMADA/cobertura
é outra coisa — `lei_unica.py`). Regenerável: `python3 -I docs/Painter/Arquitetura/ferramentas/misturas.py`.

Leis (todas implementáveis por nós sem código de licença restrita):
  sRGB          lerp nos valores codificados (o que o Photoshop/Procreate fazem por omissão num doc 8-bit sRGB)
  RGB linear    lerp na luz linear
  Oklab         lerp no Oklab (Ottosson 2020, domínio público/MIT)
  K-M 3b        Kubelka–Munk de uma constante por canal RGB linear: K/S = (1−R)²/2R, lerp de K/S, volta analítica
  WGM 10b       média geométrica pesada de reflectâncias de 10 bandas (libmypaint 1.6, ISC — `espectral.rs`)
  K-M 10b       K-M de uma constante sobre as MESMAS 10 bandas, peso = t (princípio do spectral.js, MIT)
  K-M 10b L     idem com a concentração do spectral.js c = f²·L (luminância da tinta)
Mixbox (CC BY-NC 4.0) NÃO entra: não é utilizável num produto comercial sem licença paga.

Saída: figuras e `metricas.tsv` em `docs/Painter/Arquitetura/fotos/f2b_misturas/`, tabela no stdout.
"""

from __future__ import annotations

import os
import time

import numpy as np
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

AQUI = os.path.dirname(os.path.abspath(__file__))
SAIDA = os.path.normpath(os.path.join(AQUI, "..", "fotos", "f2b_misturas"))
EPS = 1e-4


# ------------------------------------------------------------------ espaços
def s2l(c):
    c = np.asarray(c, float)
    return np.where(c <= 0.04045, c / 12.92, ((np.maximum(c, 0) + 0.055) / 1.055) ** 2.4)


def l2s(c):
    c = np.asarray(c, float)
    a = np.abs(c)
    return np.sign(c) * np.where(a <= 0.0031308, 12.92 * a, 1.055 * a ** (1 / 2.4) - 0.055)


M1 = np.array([[0.4122214708, 0.5363325363, 0.0514459929],
               [0.2119034982, 0.6806995451, 0.1073969566],
               [0.0883024619, 0.2817188376, 0.6299787005]])
M2 = np.array([[0.2104542553, 0.7936177850, -0.0040720468],
               [1.9779984951, -2.4285922050, 0.4505937099],
               [0.0259040371, 0.7827717662, -0.8086757660]])
M1i, M2i = np.linalg.inv(M1), np.linalg.inv(M2)


def lin2ok(c):
    lms = np.asarray(c, float) @ M1.T
    return np.cbrt(lms) @ M2.T


def ok2lin(o):
    return (np.asarray(o, float) @ M2i.T) ** 3 @ M1i.T


def ok_de(a_srgb, b_srgb):
    """ΔE no Oklab ×100 entre duas cores sRGB (clipadas)."""
    a = lin2ok(s2l(np.clip(a_srgb, 0, 1)))
    b = lin2ok(s2l(np.clip(b_srgb, 0, 1)))
    return 100 * np.linalg.norm(a - b, axis=-1)


def lum(c_srgb):
    return s2l(np.clip(c_srgb, 0, 1)) @ np.array([0.2126, 0.7152, 0.0722])


# ------------------------------------------------------------------ libmypaint 10 bandas (ISC — ver espectral.rs)
SR = np.array([0.009281363, 0.009732627, 0.011254253, 0.015105579, 0.024797924, 0.08362259, 0.977865, 1.0, 0.999961, 1.0])
SG = np.array([0.0028541274, 0.00391759, 0.012132152, 0.7482592, 1.0, 0.8656959, 0.03747747, 0.02281679, 0.02174742, 0.02138494])
SB = np.array([0.53705215, 0.5466464, 0.5755018, 0.25877883, 0.041709924, 0.012662639, 0.007485593, 0.006766901, 0.006699765, 0.00667622])
T = np.array([
    [0.02659562, 0.04977943, 0.02244985, -0.2184537, -0.2568949, 0.4458817, 0.7723659, 0.19449877, 0.014038158, 0.007687264],
    [-0.03260167, -0.06102104, -0.05249, 0.2066591, 0.5724963, 0.31783725, -0.021216624, -0.01938767, -0.001521339, -0.0008351816],
    [0.33947547, 0.6354014, 0.7715208, 0.11322264, -0.055251114, -0.04822258, -0.012966666, -0.0015238145, -0.00009471895, -0.000051604595],
])
WGM_EPS = 0.001


def espectro(lin):
    c = np.clip(lin, 0, 1) * (1 - WGM_EPS) + WGM_EPS
    return c[..., 0:1] * SR + c[..., 1:2] * SG + c[..., 2:3] * SB


def de_espectro(r):
    return (r @ T.T - WGM_EPS) / (1 - WGM_EPS)


def ks(r):
    r = np.clip(r, EPS, 1.0)
    return (1 - r) ** 2 / (2 * r)


def r_de_ks(x):
    return 1 + x - np.sqrt(x * x + 2 * x)


# ------------------------------------------------------------------ as leis: (a, b sRGB [0,1], t∈[0,1]) → sRGB sem clip
def m_srgb(a, b, t):
    return (1 - t) * a + t * b


def m_lin(a, b, t):
    return l2s((1 - t) * s2l(a) + t * s2l(b))


def m_oklab(a, b, t):
    return l2s(ok2lin((1 - t) * lin2ok(s2l(a)) + t * lin2ok(s2l(b))))


def m_km3(a, b, t):
    return l2s(r_de_ks((1 - t) * ks(s2l(a)) + t * ks(s2l(b))))


def m_wgm10(a, b, t):
    la, lb = np.log(espectro(s2l(a))), np.log(espectro(s2l(b)))
    return l2s(de_espectro(np.exp((1 - t) * la + t * lb)))


def m_km10(a, b, t):
    ka, kb = ks(espectro(s2l(a))), ks(espectro(s2l(b)))
    return l2s(de_espectro(r_de_ks((1 - t) * ka + t * kb)))


def m_km10L(a, b, t):
    la, lb = lum(a)[..., None] + 1e-6, lum(b)[..., None] + 1e-6
    ca, cb = (1 - t) ** 2 * la, t ** 2 * lb
    w = cb / (ca + cb)
    ka, kb = ks(espectro(s2l(a))), ks(espectro(s2l(b)))
    return l2s(de_espectro(r_de_ks((1 - w) * ka + w * kb)))


LEIS = {
    "sRGB": m_srgb,
    "RGB linear": m_lin,
    "Oklab": m_oklab,
    "K-M 3b": m_km3,
    "WGM 10b": m_wgm10,
    "K-M 10b": m_km10,
    "K-M 10b L": m_km10L,
}


def u8(*v):
    return np.array(v, float) / 255


PARES = {
    "azul+amarelo": (u8(0, 33, 133), u8(252, 211, 0)),
    "cobalto+hansa": (u8(0, 71, 171), u8(252, 211, 0)),
    "vermelho+azul": (u8(220, 30, 40), u8(0, 33, 133)),
    "branco+vermelho": (u8(255, 255, 255), u8(220, 30, 40)),
    "preto+amarelo": (u8(20, 20, 20), u8(252, 211, 0)),
    "laranja+azul (compl.)": (u8(255, 128, 0), u8(0, 100, 255)),
    "magenta+verde (compl.)": (u8(200, 0, 160), u8(0, 170, 60)),
    "ftalo+branco": (u8(13, 27, 68), u8(255, 255, 255)),
    "quinacr.+hansa": (u8(128, 2, 46), u8(252, 211, 0)),
    "ftalo+hansa": (u8(13, 27, 68), u8(252, 211, 0)),
    "dono_6 azul pálido+amarelo": (u8(120, 160, 220), u8(240, 200, 40)),
    "preto+branco": (u8(0, 0, 0), u8(255, 255, 255)),
    "quase iguais": (u8(100, 120, 140), u8(101, 121, 141)),
}
CROMATICOS = [k for k in PARES if k not in ("preto+branco", "quase iguais", "branco+vermelho", "ftalo+branco", "preto+amarelo")]


def croma(c_srgb):
    o = lin2ok(s2l(np.clip(c_srgb, 0, 1)))
    return np.hypot(o[..., 1], o[..., 2]), np.degrees(np.arctan2(o[..., 2], o[..., 1])) % 360, o[..., 0]


def q8(c):
    return np.round(np.clip(c, 0, 1) * 255) / 255


# ------------------------------------------------------------------ métricas
def metricas(nome, f, rng):
    t = np.linspace(0, 1, 51)[:, None]
    m = {}
    gam, gam_max, dip_max, nonmono, lama_min = 0, 0.0, 0.0, 0, 9.0
    ends_err = 0.0
    for par, (a, b) in PARES.items():
        g = f(a[None], b[None], t)
        fora = np.maximum(np.maximum(-g, g - 1), 0).max(axis=1)
        gam += int((fora > 1e-3).sum())
        gam_max = max(gam_max, float(fora.max()))
        ends_err = max(ends_err, float(ok_de(g[0], a)), float(ok_de(g[-1], b)))
        y = lin2ok(s2l(np.clip(g, 0, 1)))[:, 0]
        ya, yb = y[0], y[-1]
        dip_max = max(dip_max, 100 * float(min(ya, yb) - y.min()))
        dy = np.diff(y)
        if (dy > 1e-4).any() and (dy < -1e-4).any():
            nonmono += 1
        if par in CROMATICOS:
            c, _, _ = croma(g)
            lama_min = min(lama_min, float(c[25] / (0.5 * (c[0] + c[-1]))))
    m["fora_de_gama_amostras"] = gam
    m["fora_de_gama_max"] = gam_max
    m["extremos_dE"] = ends_err
    m["L_abaixo_do_mais_escuro_x100"] = dip_max
    m["pares_L_nao_monotona"] = nonmono
    m["lama_min_C_meio_rel"] = lama_min
    # azul+amarelo: verde?
    a, b = PARES["azul+amarelo"]
    mid = f(a, b, 0.5)
    c, h, L = croma(mid)
    m["azul+amarelo_meio_srgb"] = tuple(int(v) for v in np.round(np.clip(mid, 0, 1) * 255))
    m["azul+amarelo_hue"] = float(h)
    m["azul+amarelo_C"] = float(c)
    # ruído ±1/255 nas entradas → ganho (ΔE saída / ΔE entrada), máx sobre pares
    ganho = 0.0
    dmax = 0.0
    for par, (a, b) in PARES.items():
        base = f(a, b, 0.5)
        da = rng.integers(-1, 2, (300, 3)) / 255
        db = rng.integers(-1, 2, (300, 3)) / 255
        pa, pb = np.clip(a + da, 0, 1), np.clip(b + db, 0, 1)
        out = f(pa, pb, 0.5)
        d_out = ok_de(out, np.broadcast_to(base, out.shape))
        d_in = np.maximum(ok_de(pa, np.broadcast_to(a, pa.shape)), ok_de(pb, np.broadcast_to(b, pb.shape)))
        ok = d_in > 1e-9
        ganho = max(ganho, float((d_out[ok] / d_in[ok]).max()))
        dmax = max(dmax, float(d_out.max()))
    m["ruido_ganho_max"] = ganho
    m["ruido_dE_max"] = dmax
    # identidade: mix(c,c,0.5)=c ?
    cs = rng.random((4000, 3))
    cs = np.vstack([cs, np.repeat(np.linspace(0, 1, 64)[:, None], 3, 1)])
    m["identidade_dE_max"] = float(ok_de(f(cs, cs, 0.5), cs).max())
    # N passadas: (i) deriva da identidade em 8 bits, 20×
    c = q8(cs[:500])
    c0 = c.copy()
    for _ in range(20):
        c = q8(f(c, c, 0.5))
    m["deriva_20x_dE_max"] = float(ok_de(c, c0).max())
    m["deriva_20x_L_medio_x100"] = float(100 * (lin2ok(s2l(c))[:, 0] - lin2ok(s2l(c0))[:, 0]).mean())
    # (ii) vidrado: c ← mix(c, b, 0.1) 20× em 8 bits; aproxima-se de b sem passar abaixo do mais escuro?
    pior_escuro, nao_converge = 0.0, 0.0
    curvas = {}
    for par, (a, b) in PARES.items():
        c = q8(a)
        Ls = [lin2ok(s2l(c))[0]]
        for _ in range(20):
            c = q8(f(c, q8(b), 0.1))
            Ls.append(lin2ok(s2l(c))[0])
        Ls = np.array(Ls)
        pior_escuro = max(pior_escuro, 100 * float(min(Ls[0], lin2ok(s2l(q8(b)))[0]) - Ls.min()))
        nao_converge = max(nao_converge, float(ok_de(c, q8(b))))
        curvas[par] = Ls
    m["vidrado20_escurece_alem_x100"] = pior_escuro
    m["vidrado20_dist_final_dE"] = nao_converge
    # custo: µs por mistura (numpy vectorizado, 2e5 misturas)
    A, B = rng.random((200000, 3)), rng.random((200000, 3))
    t0 = time.perf_counter()
    f(A, B, 0.37)
    m["custo_ns_por_mistura_numpy"] = 1e9 * (time.perf_counter() - t0) / 200000
    return m, curvas


def figura_gradientes():
    pares = list(PARES)
    leis = list(LEIS)
    fig, axs = plt.subplots(len(pares), 1, figsize=(14, 1.15 * len(pares)))
    t = np.linspace(0, 1, 64)[:, None]
    for i, par in enumerate(pares):
        a, b = PARES[par]
        rows = []
        for lei in leis:
            g = np.clip(LEIS[lei](a[None], b[None], t), 0, 1)
            rows.append(np.repeat(g[None], 6, 0))
            rows.append(np.ones((1, 64, 3)))
        img = np.concatenate(rows, 0)
        axs[i].imshow(img, aspect="auto", interpolation="nearest")
        axs[i].set_yticks([3 + 7 * k for k in range(len(leis))])
        axs[i].set_yticklabels(leis, fontsize=6)
        axs[i].set_xticks([])
        axs[i].set_title(par, fontsize=8, loc="left")
    fig.suptitle("F2b — gradiente de mistura 0..1 (esq.=1.ª cor) por lei; linhas = leis", fontsize=10)
    fig.tight_layout()
    fig.savefig(os.path.join(SAIDA, "gradientes.png"), dpi=110)
    plt.close(fig)


def figura_meio(res_meio):
    """Swatch do meio (t=0,5) azul+amarelo, ftalo+hansa, dono_6 por lei."""
    pares = ["azul+amarelo", "cobalto+hansa", "ftalo+hansa", "dono_6 azul pálido+amarelo", "vermelho+azul", "laranja+azul (compl.)"]
    leis = list(LEIS)
    img = np.zeros((len(pares), len(leis) + 2, 3))
    for i, par in enumerate(pares):
        a, b = PARES[par]
        img[i, 0] = a
        img[i, -1] = b
        for j, lei in enumerate(leis):
            img[i, j + 1] = np.clip(LEIS[lei](a, b, 0.5), 0, 1)
    fig, ax = plt.subplots(figsize=(11, 4.5))
    ax.imshow(img, interpolation="nearest")
    ax.set_xticks(range(len(leis) + 2))
    ax.set_xticklabels(["cor A"] + leis + ["cor B"], fontsize=7, rotation=20)
    ax.set_yticks(range(len(pares)))
    ax.set_yticklabels(pares, fontsize=7)
    ax.set_title("F2b — a mistura 50/50 por lei", fontsize=10)
    fig.tight_layout()
    fig.savefig(os.path.join(SAIDA, "meio_50_50.png"), dpi=120)
    plt.close(fig)


def figura_vidrado(curvas_por_lei):
    pares = ["azul+amarelo", "preto+amarelo", "ftalo+branco", "dono_6 azul pálido+amarelo"]
    fig, axs = plt.subplots(1, len(pares), figsize=(15, 3.6))
    for k, par in enumerate(pares):
        for lei, curvas in curvas_por_lei.items():
            axs[k].plot(curvas[par], label=lei, lw=1.2)
        axs[k].set_title(par, fontsize=8)
        axs[k].set_xlabel("passada (10 % de B cada, 8 bits)", fontsize=7)
        axs[k].set_ylabel("L Oklab", fontsize=7)
    axs[0].legend(fontsize=6)
    fig.suptitle("F2b — 20 passadas de vidrado: a luminância vai de A a B sem passar abaixo do mais escuro?", fontsize=9)
    fig.tight_layout()
    fig.savefig(os.path.join(SAIDA, "vidrado_20x.png"), dpi=110)
    plt.close(fig)


def main():
    os.makedirs(SAIDA, exist_ok=True)
    rng = np.random.default_rng(31102026)
    figura_gradientes()
    figura_meio(None)
    linhas, curvas_por_lei = [], {}
    chaves = None
    for nome, f in LEIS.items():
        m, curvas = metricas(nome, f, rng)
        curvas_por_lei[nome] = curvas
        chaves = list(m)
        linhas.append((nome, m))
    figura_vidrado(curvas_por_lei)
    with open(os.path.join(SAIDA, "metricas.tsv"), "w", encoding="utf-8") as fh:
        fh.write("# gerado por docs/Painter/Arquitetura/ferramentas/misturas.py (F2b, 31/10); semente 31102026\n")
        fh.write("lei\t" + "\t".join(chaves) + "\n")
        for nome, m in linhas:
            fh.write(nome + "\t" + "\t".join(fmt(m[k]) for k in chaves) + "\n")
    for k in chaves:
        print(f"{k:34s}" + "".join(f"{fmt(m[k]):>16s}" for _, m in linhas))
    print(f"{'':34s}" + "".join(f"{n:>16s}" for n, _ in linhas))


def fmt(v):
    if isinstance(v, float):
        return f"{v:.3g}"
    return str(v)


if __name__ == "__main__":
    main()
