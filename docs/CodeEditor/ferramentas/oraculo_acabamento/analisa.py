"""Régua do (A) «Ctrl + rato parado»: procura um SUBLINHADO na faixa da linha do alvo, em cada estado.

Uso: python3 -I analisa.py <raw> <saida> <app> <alvo> <texto_da_linha> <y> <meia_faixa> <x0> <largura_col> <rato_x> <estados...>
  <raw>/<app>_<alvo>_ref.png = a MESMA cena com o rato longe (nenhum balão, nenhum realce): a referência.
  <raw>/<app>_<alvo>_<estado>.png = cada estado; cada argumento de estado é «nome=cursor».
Um sublinhado é uma corrida HORIZONTAL de pixels de cor uniforme que (1) não está na referência, (2) passa
por baixo do rato (± 1 coluna: o rato sobre o espaço logo a seguir a «x» ainda sublinha «x», medido), (3) não toca as bordas da faixa nem passa do fim do texto (a borda de um balão passa), (4) tem no máximo
4 px de espessura (um realce de fundo tem a altura da linha), (5) é TINTA: dista > 150 (soma |ΔRGB|)
do fundo — o realce do balão do VS Code (#222324 sobre #121314) é um tom, não um traço. Imprime, por estado: a linha y, o intervalo
x → colunas → o TEXTO coberto, a espessura, a cor do sublinhado e a cor do texto por cima (referência →
estado); grava <saida>/<app>_A_<alvo>_c.png (ref + estados empilhados, 2×, com o cursor do X ao lado)."""
import sys
from collections import Counter

from PIL import Image, ImageDraw

raw, saida, app, alvo, linha, y, meia, x0, cw, mx = sys.argv[1:11]
y, meia, x0, cw, mx = int(y), int(meia), int(x0), float(cw), int(mx)
estados = sys.argv[11:]
Y0, Y1, X0, X1 = y - meia, y + meia, x0 - 8, x0 + 420


def faixa(nome):
    return Image.open(f"{raw}/{app}_{alvo}_{nome}.png").convert("RGB").crop((X0, Y0, X1, Y1 + 1))


def hexa(c):
    return "#%02x%02x%02x" % c


def dif(a, b):
    return sum(abs(a[k] - b[k]) for k in range(3))


ref = faixa("ref")
W, H = ref.size
rp = ref.load()
fundo = Counter(rp[i, j] for i in range(W) for j in range(H)).most_common(1)[0][0]
m = mx - X0


def corridas(fp, j):
    """corridas de pixels novos (≠ ref) e de cor uniforme na linha j da faixa"""
    out, i = [], 0
    while i < W:
        if dif(fp[i, j], rp[i, j]) <= 40:
            i += 1
            continue
        c, a = fp[i, j], i
        while i < W and dif(fp[i, j], rp[i, j]) > 40 and dif(fp[i, j], c) <= 40:
            i += 1
        out.append((a, i - 1, c))
    # uma perna de letra (p, g, y) corta o traço (o VS Code salta a tinta): junta pedaços de tinta a ≤ 1 coluna
    junto = []
    for a, b, c in (r for r in out if dif(r[2], fundo) > 150):  # só TINTA (o realce do balão é um tom)
        if junto and a - junto[-1][1] - 1 <= cw + 1:
            # a cor do traço é a do pedaço mais LONGO (a perna da letra é um pedaço curto)
            cor = junto[-1][2] if junto[-1][1] - junto[-1][0] >= b - a else c
            junto[-1] = (junto[-1][0], b, cor)
        else:
            junto.append((a, b, c))
    return junto


def sublinhado(fp):
    melhor = None
    fim_texto = x0 + cw * (len(linha) + 1) - X0  # um traço não passa do fim do texto da linha
    for j in range(meia + 5, H):  # só abaixo da linha de base (o corpo das letras fica de fora)
        for a, b, c in corridas(fp, j):
            if not (a - cw <= m <= b + cw) or a <= 2 or b >= min(W - 3, fim_texto) or b - a + 1 < 5:
                continue
            if melhor is None or b - a > melhor[2] - melhor[1]:
                melhor = (j, a, b, c)
    if melhor is None:
        return None
    j, a, b, c = melhor

    def igual(jj):
        if not 0 <= jj < H:
            return False
        ok = sum(1 for i in range(a, b + 1) if dif(fp[i, jj], c) <= 40 and dif(fp[i, jj], rp[i, jj]) > 40)
        return ok >= 0.7 * (b - a + 1)
    esp, k = 1, 1
    while igual(j - k):
        esp, k = esp + 1, k + 1
    k = 1
    while igual(j + k):
        esp, k = esp + 1, k + 1
    if esp > 4:
        return None
    return j, a, b, c, esp


def tinta(pix, a, b, j):
    cs = Counter()
    for jj in range(2, j - 1):
        for i in range(a, b + 1):
            if dif(pix[i, jj], fundo) > 120:
                cs[pix[i, jj]] += 1
    return hexa(cs.most_common(1)[0][0]) if cs else "?"


res, nomes, cursores = [], [], {}
for e in estados:
    nm, cu = e.split("=", 1)
    nomes.append(nm)
    cursores[nm] = cu
for nm in nomes:
    fp = faixa(nm).load()
    s = sublinhado(fp)
    if s is None:
        res.append(f"{app} A {alvo} {nm}: cursor={cursores[nm]} · sem sublinhado")
        continue
    j, a, b, c, esp = s
    xa, xb = a + X0, b + X0
    ca, cb = (xa - x0) / cw, (xb + 1 - x0) / cw
    res.append(
        f"{app} A {alvo} {nm}: cursor={cursores[nm]} · SUBLINHA y={j + Y0} x=[{xa},{xb}] col≈[{ca:.1f},{cb:.1f}) "
        f"→ «{linha[max(0, round(ca)):round(cb)]}» · {esp} px · cor {hexa(c)} · texto {tinta(rp, a, b, j)} → "
        f"{tinta(fp, a, b, j)} · fundo {hexa(fundo)}")

ESC, ROT = 2, 230
todos = ["ref"] + nomes
cursores["ref"] = "rato longe"
out = Image.new("RGB", (ROT + W * ESC, len(todos) * (H * ESC + 4)), (40, 40, 40))
d = ImageDraw.Draw(out)
for k, nm in enumerate(todos):
    yy = k * (H * ESC + 4)
    out.paste(faixa(nm).resize((W * ESC, H * ESC), Image.NEAREST), (ROT, yy))
    d.text((6, yy + 6), f"{nm}  [{cursores[nm]}]", fill=(255, 255, 255))
d.line([(ROT + m * ESC, 0), (ROT + m * ESC, 3)], fill=(255, 0, 255), width=2)  # marca do rato (topo)
out.save(f"{saida}/{app}_A_{alvo}_c.png")
print("\n".join(res))
