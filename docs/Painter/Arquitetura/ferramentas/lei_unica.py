#!/usr/bin/env python3
"""Modelo de REFERÊNCIA da lei única de camada (F2 da refatoração do Wet Paint, 31/10).

Regerar (da raiz da worktree):
    python3 docs/Painter/Arquitetura/ferramentas/lei_unica.py
Saída: docs/Painter/Arquitetura/fotos/f2_lei_unica/{a_grelha_amarelo_sobre_azul,b_cobertura_vs_espessura,c_escala_de_meio}.png
e o relatório numérico no stdout (os asserts são o portão do modelo).

A lei (doc F2_grandezas_fisicas.md §3):
  estado por célula = R_fechado (fundo congelado, por banda) + mistura ABERTA (filme molhado ⊕ assentado aberto),
  cada pigmento i como volume por área v_i [µm]; o veículo (água/óleo) tem K = S = 0.
  𝒦 = Σ v_i K_i ;  𝒮 = s(n_v) Σ v_i S_i            (Duncan 1940: K e S somam pelo volume)
  R = KM_camada(𝒦, 𝒮, R_fundo)                       (Kubelka 1948, solução hiperbólica)
  cobertura C = 1 − (R|fundo=1 − R|fundo=0)          (derivada, nunca knob)
  meio m ∈ [0,1]: 0 = água, 1 = óleo — muda n_v, viscosidade, abertura, veículo que fica (relevo); NUNCA a lei.
Bandas: 3 (R,G,B lineares). Unidades: K, S em 1/µm; v em µm de pigmento puro.
"""
from __future__ import annotations

import os

import numpy as np
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

AQUI = os.path.dirname(os.path.abspath(__file__))
SAIDA = os.path.normpath(os.path.join(AQUI, "..", "fotos", "f2_lei_unica"))

# ---------------------------------------------------------------- pigmentos (R∞ = masstone linear, S em 1/µm)
def pigmento(r_inf, s):
    r = np.asarray(r_inf, float)
    s = np.asarray(s, float)
    k = s * (1 - r) ** 2 / (2 * r)  # K/S = (1−R∞)²/2R∞
    return {"K": k, "S": s, "Rinf": r}


AZUL = pigmento([0.030, 0.060, 0.300], [0.4, 0.4, 0.4])  # ultramar-ish, semi-transparente
AMARELO = pigmento([0.800, 0.650, 0.035], [1.2, 1.2, 1.2])  # hansa-ish, semi-opaco
PAPEL = np.array([0.85, 0.85, 0.82])

# ---------------------------------------------------------------- meio
N_PIG = 1.75  # índice de refração típico de pigmento orgânico


def n_veiculo(m, molhado):
    """Índice do que rodeia a partícula. Água 1,33 · linhaça 1,48; a aquarela SECA tem goma+ar (~1,25 efectivo)."""
    agua = 1.33 if molhado else 1.25
    return (1 - m) * agua + m * 1.48


def s_fator(m, molhado):
    """Espalhamento ∝ contraste de índice (Fresnel/Mie ao 1.º grau), normalizado na linhaça."""
    c = ((N_PIG - n_veiculo(m, molhado)) / (N_PIG + n_veiculo(m, molhado))) ** 2
    c0 = ((N_PIG - 1.48) / (N_PIG + 1.48)) ** 2
    return c / c0


def viscosidade(m, phi):
    """log-linear água (1e-3 Pa·s) → óleo de linhaça (5e-2 Pa·s) × Krieger–Dougherty no pigmento."""
    eta_v = 10 ** ((1 - m) * -3.0 + m * np.log10(5e-2))
    phi_max = 0.64
    return eta_v * (1 - np.minimum(phi, 0.6) / phi_max) ** (-2.5 * phi_max)


# ---------------------------------------------------------------- Kubelka 1948
def km_camada(kt, st, rg):
    """Reflectância de uma camada de espessura óptica (kt = Σv K, st = Σv S) sobre fundo rg. Vectorizado por banda."""
    kt = np.asarray(kt, float)
    st = np.asarray(st, float)
    rg = np.asarray(rg, float)
    out = np.empty(np.broadcast(kt, st, rg).shape)
    kt, st, rg = np.broadcast_arrays(kt, st, rg)
    puro = st < 1e-9  # sem espalhamento: Beer–Lambert de ida e volta
    out[puro] = rg[puro] * np.exp(-2 * kt[puro])
    q = ~puro
    a = 1 + kt[q] / st[q]
    b = np.sqrt(a * a - 1)
    bs = np.minimum(b * st[q], 50.0)
    coth = 1 / np.tanh(np.maximum(bs, 1e-12))
    out[q] = (1 - rg[q] * (a - b * coth)) / (a - rg[q] + b * coth)
    return out


def r_infinito(ks):
    """A camada infinita (masstone): R∞ = 1 + K/S − √((K/S)² + 2K/S)."""
    return 1 + ks - np.sqrt(ks * ks + 2 * ks)


def cobertura(kt, st):
    return 1 - (km_camada(kt, st, 1.0) - km_camada(kt, st, 0.0))


def totais(mistura, m, molhado):
    """mistura = lista de (pigmento, v[µm]); devolve (𝒦, 𝒮) por banda."""
    kt = sum(v * p["K"] for p, v in mistura)
    st = s_fator(m, molhado) * sum(v * p["S"] for p, v in mistura)
    return kt, st


def lin2srgb(x):
    x = np.clip(x, 0, 1)
    return np.where(x <= 0.0031308, 12.92 * x, 1.055 * x ** (1 / 2.4) - 0.055)


def luminancia(r):
    return 0.2126 * r[..., 0] + 0.7152 * r[..., 1] + 0.0722 * r[..., 2]


# ---------------------------------------------------------------- (a) grelha pigmento × água
def figura_a():
    m = 0.85  # óleo grosso (Thick Impasto)
    v_azul = 30.0  # µm de pigmento azul já assentado (hiding)
    abertura = 0.5  # metade do azul ainda aberto (re-mistura)
    carga = 40.0  # µm de tinta (pigmento+veículo) que o pincel larga por passada
    pig = np.linspace(0.05, 0.6, 8)  # fracção volúmica de pigmento na tinta do tubo (φ)
    agua = np.linspace(0.0, 0.9, 8)  # fracção de diluente acrescentado à tinta
    passadas = [1, 3, 6]
    r_fech = km_camada(*totais([(AZUL, (1 - abertura) * v_azul)], m, False), PAPEL)
    s = s_fator(m, False)
    r_azul_puro = r_infinito(AZUL["K"] / (s * AZUL["S"]))  # masstone no MESMO veículo e estado
    r_amar_puro = r_infinito(AMARELO["K"] / (s * AMARELO["S"]))
    fig, eixos = plt.subplots(1, len(passadas), figsize=(4.2 * len(passadas), 4.6))
    pior_escuro = np.inf
    pior_ferrugem = -np.inf
    for ax, n in zip(eixos, passadas):
        img = np.zeros((len(agua), len(pig), 3))
        for i, w in enumerate(agua):
            for j, phi in enumerate(pig):
                phi_ef = phi * (1 - w)  # diluir só baixa a concentração; o pigmento por passada cai junto
                v_am = n * carga * phi_ef
                mist = [(AZUL, abertura * v_azul), (AMARELO, v_am)]
                r = km_camada(*totais(mist, m, False), r_fech)
                img[i, j] = r
                # «mais escuro que o azul puro» = abaixo do masstone azul nas 3 bandas
                pior_escuro = min(pior_escuro, float(np.max(r - r_azul_puro)))
                # ferrugem: R acima de G mais do que em qualquer ingrediente
                pior_ferrugem = max(pior_ferrugem, float((r[0] - r[1]) - max(r_amar_puro[0] - r_amar_puro[1], 0)))
        ax.imshow(lin2srgb(img), origin="lower", extent=[pig[0], pig[-1], agua[0], agua[-1]], aspect="auto",
                  interpolation="nearest")
        ax.set_title(f"{n} passada(s) de amarelo sobre azul assentado")
        ax.set_xlabel("pigmento φ (fracção volúmica no tubo)")
        ax.set_ylabel("água / diluente acrescentado (fracção)")
    fig.suptitle("(a) Lei única, óleo m=0,85: amarelo sobre azul (30 µm, metade aberto) — "
                 f"min_banda(R − R_azul_puro) ≥ {pior_escuro:+.3f} (>0 ⇒ nunca mais escuro que o azul)")
    fig.tight_layout()
    fig.savefig(os.path.join(SAIDA, "a_grelha_amarelo_sobre_azul.png"), dpi=110)
    plt.close(fig)
    assert pior_escuro > 0, "uma célula ficou mais escura que o azul puro nas 3 bandas"
    assert pior_ferrugem <= 1e-9, f"apareceu R−G acima do amarelo puro: {pior_ferrugem}"
    return pior_escuro, pior_ferrugem


# ---------------------------------------------------------------- (b) cobertura × espessura
def figura_b():
    h = np.geomspace(0.05, 300, 400)  # espessura de tinta molhada em µm (pigmento + veículo)
    fig, ax = plt.subplots(figsize=(7, 4.5))
    estilos = ["-", "--", ":"]
    for phi, ls in zip([0.05, 0.2, 0.45], estilos):
        for p, nome, cor in [(AZUL, "azul", "#1f4fb4"), (AMARELO, "amarelo", "#c9a400")]:
            v = h * phi
            c = np.array([luminancia(cobertura(*totais([(p, vi)], 1.0, False))) for vi in v])
            assert np.all(np.diff(c) >= -1e-12), "cobertura não monótona na espessura"
            ax.plot(h, c, ls, color=cor, label=f"{nome}, φ={phi:.2f}")
    ax.set_xlabel("espessura de tinta h [µm] (pigmento + veículo, escala log)")
    ax.set_ylabel("cobertura C = 1 − (R|branco − R|preto)  (luminância)")
    ax.set_xscale("log")
    ax.set_title("(b) Cobertura derivada de φ × h (óleo seco): monótona, satura")
    ax.set_ylim(0, 1.02)
    ax.grid(alpha=0.3)
    ax.legend(fontsize=8, ncol=2)
    fig.tight_layout()
    fig.savefig(os.path.join(SAIDA, "b_cobertura_vs_espessura.png"), dpi=110)
    plt.close(fig)


# ---------------------------------------------------------------- (c) escala de meio
def traco_1d(m, v_total, nx=160):
    """Corte transversal de um traço: depósito em chapéu, espalha por difusão ∝ 1/η e leva pigmento à orla
    (Curtis 1997: evaporação na orla puxa fluxo para fora) com força ∝ 1/η. Conserva o pigmento (assert)."""
    x = np.linspace(-1.6, 1.6, nx)
    dep = (np.abs(x) < 0.8).astype(float)
    dep *= v_total / dep.sum()
    eta = viscosidade(m, 0.25)
    mob = np.clip(np.log10(5e-2 / eta) / np.log10(5e-2 / 1e-3), 0, 1)  # 1 = água, 0 = óleo
    sig = 0.02 + 0.18 * mob
    ker = np.exp(-0.5 * (x / sig) ** 2)
    ker /= ker.sum()
    v = np.convolve(dep, ker, mode="same")
    molhado = v > 0.15 * v.max()  # a frente húmida: onde a água ainda cobre o papel
    borda = molhado & ~np.roll(molhado, 4) | molhado & ~np.roll(molhado, -4)
    leva = 0.35 * mob * v[molhado & ~borda]
    v2 = v.copy()
    v2[molhado & ~borda] -= leva
    v2[borda] += leva.sum() / max(borda.sum(), 1)
    assert abs(v2.sum() - v.sum()) < 1e-9 * max(v.sum(), 1), "a orla criou/perdeu pigmento"
    return x, v2


def figura_c():
    ms = np.linspace(0, 1, 9)
    v_total = 160 * 0.6  # mesma massa de pigmento por traço em todo m
    linhas = 40
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(11, 5.4), gridspec_kw={"height_ratios": [2.2, 1]})
    tira = []
    relevo = []
    for m in ms:
        x, v = traco_1d(m, v_total)
        r = np.stack([km_camada(*totais([(AZUL, vi)], m, False), PAPEL) for vi in v])
        tira.append(np.repeat(lin2srgb(r)[None], linhas, 0))
        phi = 0.25
        h_seco = v * (1 + m * (1 - phi) / phi)  # o veículo de óleo FICA (filme), a água evapora
        relevo.append(h_seco)
        tira.append(np.ones((linhas // 6, len(x), 3)))
    ax1.imshow(np.concatenate(tira, 0), aspect="auto")
    ax1.set_yticks([(linhas + linhas // 6) * k + linhas / 2 for k in range(len(ms))])
    ax1.set_yticklabels([f"m={m:.2f}" for m in ms])
    ax1.set_xticks([])
    ax1.set_title("(c) Escala de meio água (m=0) → óleo (m=1): mesmo pigmento azul, mesmo volume; secos")
    ax1.set_ylabel("meio m")
    for m, h in zip(ms, relevo):
        ax2.plot(np.linspace(-1.6, 1.6, len(h)), h, color=plt.cm.viridis(m), label=f"m={m:.2f}")
    ax2.set_xlabel("posição transversal ao traço (raios do pincel)")
    ax2.set_ylabel("relevo seco h [µm]")
    ax2.legend(fontsize=7, ncol=9, loc="upper center")
    fig.tight_layout()
    fig.savefig(os.path.join(SAIDA, "c_escala_de_meio.png"), dpi=110)
    plt.close(fig)


# ---------------------------------------------------------------- propriedades (o portão do modelo)
def provas():
    rng = np.random.default_rng(7)
    for _ in range(4000):
        a, b = rng.uniform(0, 80, 2)
        m = rng.uniform()
        rg = rng.uniform(0.01, 0.95, 3)
        r = km_camada(*totais([(AZUL, a), (AMARELO, b)], m, False), rg)
        kt, st = totais([(AZUL, a), (AMARELO, b)], m, False)
        rinf = r_infinito(kt / st)
        lo = np.minimum(rg, rinf) - 1e-9
        hi = np.maximum(rg, rinf) + 1e-9
        assert np.all((r >= lo) & (r <= hi)), "camada fora do segmento [fundo, R∞]"
        # diluir (mesmo pigmento, menos volume) aproxima do fundo — nunca passa do R∞
        r_meio = km_camada(*totais([(AZUL, a / 2), (AMARELO, b / 2)], m, False), rg)
        assert np.all(np.abs(r_meio - rg) <= np.abs(r - rg) + 1e-9), "diluir afastou do fundo"
    print("provas: 4000 casos — R ∈ [fundo, R∞_mistura] e diluir é monótono: OK")


if __name__ == "__main__":
    os.makedirs(SAIDA, exist_ok=True)
    provas()
    e, f = figura_a()
    print(f"(a) min sobre a grelha de max_banda(R − R_azul_puro) = {e:+.4f} (>0 ⇒ nenhuma célula mais escura que o azul)")
    print(f"(a) max (R−G) − (R−G)_amarelo_puro = {f:+.4f} (≤0 ⇒ sem ferrugem)")
    figura_b()
    figura_c()
    print("figuras em", SAIDA)
