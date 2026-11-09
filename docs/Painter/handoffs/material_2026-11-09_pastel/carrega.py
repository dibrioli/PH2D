# Carrega cada pincel como COBERTURA k (0..1) do verde 00A03C numa tela 2400x880 (col a vazio · b branco · c blocos).
# k = (255 - R)/255 sobre o branco (o verde tem R = 0): exacto no Krita (erro G/B <= 0,5), <= 2,4 na foto do CSP.
import numpy as np
from PIL import Image

REPO = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor'
W, H, COL = 2400, 880, 800
VERDE = np.array([0, 160, 60], np.float32)


def base_rgb(cena):
    b = np.asarray(Image.open(cena + '/base.png').convert('RGBA')).astype(np.float32)
    a = b[..., 3:4] / 255
    return b[..., :3] * a + 255 * (1 - a)  # a coluna vazia sobre o papel branco


def k_de_rgb_sobre_branco(rgb):
    return np.clip((255 - rgb[..., 0].astype(np.float32)) / 255, 0, 1)


def compoe_camada(k, base):
    """O traço numa camada própria por cima do fundo (vazio · branco · blocos)."""
    return base * (1 - k[..., None]) + VERDE * k[..., None]


def ph2d(cena, suf=''):
    """O NOSSO: a sonda pinta direto na tela (a camada a vazia; b, c opacas). `suf` = variante `__<sufixo>`."""
    a = np.frombuffer(open(f'{cena}/out_ph2d/Pastel{suf}.rgba', 'rb').read(), np.uint8).reshape(H, W, 4) \
        .astype(np.float32)
    al = a[..., 3:4] / 255
    rgb = a[..., :3] * al + 255 * (1 - al)
    k = k_de_rgb_sobre_branco(rgb[:, :COL])
    return rgb, k


def krita(cru_png, base):
    a = np.asarray(Image.open(cru_png).convert('RGB')).astype(np.float32)
    k = k_de_rgb_sobre_branco(a)
    return compoe_camada(k, base), k[:, :COL]


def mypaint(npy, base):
    a = np.load(npy)  # pré-multiplicado 0..1
    al = a[..., 3:4]
    over = (a[..., :3] * 255 + base * (1 - al))  # camada própria sobre o fundo
    k = k_de_rgb_sobre_branco(a[..., :3] * 255 + 255 * (1 - al))
    return over, k[:, :COL]


def csp():
    """CSP 1.13.2 corrido em 08/11 (foto a 100 %); só sobrevive a folha de 08/11, reduzida a 50 % (bicúbica)."""
    f = np.asarray(Image.open(REPO + '/docs/Painter/handoffs/material_2026-11-08_revisao/folha_Pastel.png')
                   .convert('RGB')).astype(np.float32)
    meia = np.concatenate([f[60 + k * 480:500 + k * 480, 10:410] for k in range(3)], 1)  # 440 x 1200
    cheia = np.asarray(Image.fromarray(meia.astype(np.uint8)).resize((W, H), Image.BICUBIC)).astype(np.float32)
    return cheia, meia, k_de_rgb_sobre_branco(meia[:, :400])
