"""Retoque A+B (J10): arrasto do óleo pela aguada no traço (A) e mistura molhada pela média das consistências (B).
Entrada: as sondas diag_aguada_por_cima_do_oleo (p_*, b_*). Saída: pranchas hoje | variantes."""
import sys
import numpy as np
from PIL import Image, ImageDraw, ImageFont

D = sys.argv[1]


def gaussian_filter1d(a, sigma, axis):
    if sigma <= 0:
        return a
    r = int(3 * sigma + 0.5)
    k = np.exp(-0.5 * (np.arange(-r, r + 1) / sigma) ** 2)
    k /= k.sum()
    return np.apply_along_axis(lambda v: np.convolve(np.pad(v, r, mode="edge"), k, "valid"), axis, a)


def gaussian_filter(a, sigma):
    sig = sigma if isinstance(sigma, tuple) else (sigma,) * a.ndim
    for ax, s in enumerate(sig):
        a = gaussian_filter1d(a, s, ax)
    return a


def le(n):
    return np.asarray(Image.open(f"{D}/{n}.ppm").convert("RGB")).astype(np.float64)


def lin(c):
    c = c / 255.0
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def srgb(l):
    l = np.clip(l, 0, 1)
    return 255 * np.where(l <= 0.0031308, l * 12.92, 1.055 * l ** (1 / 2.4) - 0.055)


def km(a, b, t):
    """Kubelka–Munk de uma camada: t = fracção de b."""
    ra, rb = np.clip(lin(a), 1e-4, 1), np.clip(lin(b), 1e-4, 1)
    ks = lambda r: (1 - r) ** 2 / (2 * r)
    k = ks(ra) * (1 - t) + ks(rb) * t
    return srgb(1 + k - np.sqrt(k * k + 2 * k))


def descer(campo, L):
    """Arrasto para BAIXO (o sentido do traço): média exponencial que corre de cima para baixo."""
    out = campo.copy()
    a = 1.0 / L
    for y in range(1, campo.shape[0]):
        out[y] = out[y - 1] * (1 - a) + campo[y] * a
    return out


def variante(pre, arrasto, media, sigma):
    antes, fim = le(f"{pre}_antes"), le(f"{pre}_fim")
    papel = antes[2, 2]
    # O óleo: o azul que domina (máscara suave).
    s = np.clip((antes[..., 2] - np.maximum(antes[..., 0], antes[..., 1]) - 20) / 90, 0, 1)
    s = gaussian_filter(s, 0.7)
    # A banda da aguada: o perfil das colunas onde o fim ficou amarelo FORA do óleo.
    amarelo = np.clip(((fim[..., 0] + fim[..., 1]) / 2 - fim[..., 2] - ((papel[0] + papel[1]) / 2 - papel[2])) / 60, 0, 1)
    perfil = np.maximum(amarelo[5:45].mean(0), amarelo[125:150].mean(0))
    perfil = np.clip(perfil / max(perfil.max(), 1e-6), 0, 1)
    ys = np.arange(fim.shape[0])
    vertical = np.clip((ys - 5) / 6, 0, 1) * np.clip((156 - ys) / 6, 0, 1)
    W = perfil[None, :] * vertical[:, None]
    # A borda da aguada também é molhada: a mistura escorre uns píxeis para os lados.
    W = np.clip(gaussian_filter1d(W, 3.0, 1) * 1.15, 0, 1)
    # A cor da aguada (sem óleo) por coluna, e a imagem «só aguada» por baixo.
    fundo = fim.copy()
    cor_ag = np.zeros((fim.shape[1], 3))
    for x in range(fim.shape[1]):
        col = fim[125:150, x]
        cor_ag[x] = col.mean(0)
    so_ag = np.broadcast_to(cor_ag[None], fim.shape)
    fundo = np.where(s[..., None] > 0.05, so_ag * W[..., None] + papel * (1 - W[..., None]), fim)
    if arrasto == 0 and media == 0:
        return fim
    # A — o arrasto: o óleo (cor pré-multiplicada e cobertura) desce com o traço onde a aguada passou.
    sd = s.copy()
    cor = antes * s[..., None]
    if arrasto > 0:
        sd_a, cor_a = descer(s, arrasto), np.stack([descer(cor[..., c], arrasto) for c in range(3)], -1)
        k = np.clip(W * 0.85, 0, 1)
        sd = s * (1 - k) + sd_a * k
        cor = cor * (1 - k[..., None]) + cor_a * k[..., None]
    # B — a borda com transição: a cobertura do óleo esbate-se dentro da banda (sigma pela média das consistências).
    if sigma > 0:
        sb = gaussian_filter1d(gaussian_filter1d(sd, sigma, axis=0), sigma * 0.5, axis=1)
        cb = np.stack([gaussian_filter1d(gaussian_filter1d(cor[..., c], sigma, axis=0), sigma * 0.5, axis=1) for c in range(3)], -1)
        sd = sd * (1 - W) + sb * W
        cor = cor * (1 - W[..., None]) + cb * W[..., None]
    cor_oleo = cor / np.maximum(sd[..., None], 1e-4)
    cor_oleo = np.where(sd[..., None] > 1e-3, cor_oleo, so_ag)
    # A mistura: dentro da banda, o óleo leva a aguada pela média das consistências.
    t = np.clip(W * media, 0, 1)[..., None]
    # A tinta da aguada é o PIGMENTO amarelo (a sonda pinta [220,200,40]); a média das consistências diz quanto entra.
    misto = km(cor_oleo, np.array([220.0, 200.0, 40.0]), t * 0.6)
    # O relevo/luz do óleo: o detalhe alto do antes, onde ainda há óleo.
    det = (antes - gaussian_filter(antes, (2, 2, 0))) * s[..., None] * (1 - 0.5 * W[..., None])
    out = fundo * (1 - sd[..., None]) + misto * sd[..., None] + det
    out = np.where(W[..., None] > 0.01, out, fim)
    return np.clip(out, 0, 255)



AZUL = np.array([40.0, 90.0, 220.0])
AMARELO = np.array([220.0, 200.0, 40.0])


def remolha(pre, d, sigma, mistura):
    """O óleo RE-MOLHA: a fracção `d` do azul sob a aguada dissolve-se na água dela e espalha-se (sigma) dentro da
    água; o que fica do óleo é mais fino (cobertura e relevo × 1−d) e mistura-se com o amarelo."""
    antes, fim = le(f"{pre}_antes"), le(f"{pre}_fim")
    if d == 0:
        return fim
    papel = antes[2, 2]
    s = np.clip((antes[..., 2] - np.maximum(antes[..., 0], antes[..., 1]) - 20) / 90, 0, 1)
    s = gaussian_filter(s, 0.7)
    amarelo = np.clip(((fim[..., 0] + fim[..., 1]) / 2 - fim[..., 2] - ((papel[0] + papel[1]) / 2 - papel[2])) / 60, 0, 1)
    perfil = np.maximum(amarelo[5:45].mean(0), amarelo[125:150].mean(0))
    perfil = np.clip(perfil / max(perfil.max(), 1e-6), 0, 1)
    ys = np.arange(fim.shape[0])
    vertical = np.clip((ys - 5) / 6, 0, 1) * np.clip((156 - ys) / 6, 0, 1)
    W = perfil[None, :] * vertical[:, None]
    W = np.clip(gaussian_filter1d(W, 3.0, 1) * 1.15, 0, 1)
    cor_ag = fim[125:150].mean(0)
    agua = np.broadcast_to(cor_ag[None], fim.shape) * W[..., None] + papel * (1 - W[..., None])
    # O azul que se dissolve e viaja na água (mais para baixo: o sentido do traço).
    D = s * W * d
    Dsp = gaussian_filter1d(gaussian_filter1d(D, sigma, 0), sigma * 0.45, 1)
    Dsp = np.roll(Dsp, int(sigma * 0.4), axis=0) * 0.6 + Dsp * 0.4
    Dsp = Dsp * W
    # A água tingida: a aguada (amarelo sobre o papel) leva o azul dissolvido (Kubelka–Munk, pigmento).
    tinta = km(agua, AZUL, np.clip(Dsp * 1.6, 0, 0.85)[..., None])
    # O óleo que fica: mais fino, misturado com o amarelo, relevo achatado.
    so = s * (1 - D)
    oleo = km(antes, AMARELO, np.clip(W * mistura, 0, 1)[..., None])
    det = (antes - gaussian_filter(antes, (2, 2, 0))) * so[..., None] * (1 - d * W[..., None])
    base = antes - (antes - gaussian_filter(antes, (2, 2, 0)))
    oleo = km(base, AMARELO, np.clip(W * mistura, 0, 1)[..., None])
    # Na borda dissolvida a cobertura cai aos poucos (a borda deixa de ser dura).
    so_b = gaussian_filter1d(so, 1.0 + 3.0 * d, 0) * W + so * (1 - W)
    out = tinta * (1 - so_b[..., None]) + oleo * so_b[..., None] + det
    out = np.where(W[..., None] > 0.01, out, fim)
    return np.clip(out, 0, 255)


VAR2 = [
    ("hoje", 0, 0, 0),
    ("re-molha pouco (20 %)", 0.2, 7, 0.25),
    ("re-molha médio (35 %)", 0.35, 10, 0.35),
    ("re-molha muito (55 %)", 0.55, 14, 0.45),
]
for pre in ["b", "p"]:
    imgs = []
    for nome, d, sig, mis in VAR2:
        im = Image.fromarray(remolha(pre, d, sig, mis).astype(np.uint8)).resize((480, 320), Image.NEAREST)
        cab = Image.new("RGB", (480, 30), (30, 30, 30))
        ImageDraw.Draw(cab).text((8, 6), nome, fill=(240, 240, 240), font=ImageFont.truetype('/usr/share/fonts/TTF/DejaVuSans.ttf', 16))
        col = Image.new("RGB", (480, 350))
        col.paste(cab, (0, 0))
        col.paste(im, (0, 30))
        imgs.append(col)
    tira = Image.new("RGB", (480 * 2 + 10, 350 * 2 + 10), (60, 60, 60))
    for k, im in enumerate(imgs):
        tira.paste(im, ((k % 2) * 490, (k // 2) * 360))
    tira.save(f"{D}/J10_remolha_{pre}.png")
raise SystemExit("ok")

VAR = [
    ("hoje", 0, 0, 0),
    ("só A: arrasta no traço", 10, 0.25, 0),
    ("A+B: média 0,5 (proposta)", 10, 0.5, 3),
    ("A+B forte: 0,75", 16, 0.75, 5),
]
for pre in ["b", "p"]:
    imgs = []
    for nome, arr, med, sig in VAR:
        im = Image.fromarray(variante(pre, arr, med, sig).astype(np.uint8)).resize((480, 320), Image.NEAREST)
        cab = Image.new("RGB", (480, 30), (30, 30, 30))
        ImageDraw.Draw(cab).text((8, 6), nome, fill=(240, 240, 240), font=ImageFont.truetype('/usr/share/fonts/TTF/DejaVuSans.ttf', 16))
        col = Image.new("RGB", (480, 350))
        col.paste(cab, (0, 0))
        col.paste(im, (0, 30))
        imgs.append(col)
    tira = Image.new("RGB", (480 * 2 + 10, 350 * 2 + 10), (60, 60, 60))
    for k, im in enumerate(imgs):
        tira.paste(im, ((k % 2) * 490, (k // 2) * 360))
    tira.save(f"{D}/J10_simula_AB_{pre}.png")
print("ok")
