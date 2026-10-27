"""Régua da K3b (W3, cerda e marca): `python3 cerda_regua.py <item> <tamanhos_A.(png|ppm)> [escala]`.

Itens: fan · flat · round · d3 · splats. A cena `tamanhos` da bateria: traços rectos y = 70/190/440, x 60..940.
`escala` = a da W2 (a largura nossa ÷ a do Rebelle; a régua mede tudo ÷ D, então só serve para recortar a faixa).
"""
import sys
from collections import deque
import numpy as np
from PIL import Image

PAPEL = np.array([250, 250, 250], float)


def carrega(p):
    a = np.asarray(Image.open(p).convert("RGB"), float)
    return a


def cobertura(a):
    # a fracção de tinta: a distância ao papel ÷ a do azul cheio (23,101,179) ao papel
    tinta = np.array([23, 101, 179], float)
    d = np.linalg.norm(a - PAPEL, axis=2) / np.linalg.norm(tinta - PAPEL)
    return np.clip(d, 0, 1.5)


def faixa(a, yc, meia):
    return a[yc - meia:yc + meia, :]


def componentes(m, min_area=1):
    h, w = m.shape
    lab = np.zeros((h, w), int)
    out = []
    n = 0
    for y in range(h):
        for x in range(w):
            if m[y, x] and not lab[y, x]:
                n += 1
                q = deque([(y, x)])
                lab[y, x] = n
                pts = []
                while q:
                    cy, cx = q.popleft()
                    pts.append((cy, cx))
                    for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        yy, xx = cy + dy, cx + dx
                        if 0 <= yy < h and 0 <= xx < w and m[yy, xx] and not lab[yy, xx]:
                            lab[yy, xx] = n
                            q.append((yy, xx))
                if len(pts) >= min_area:
                    out.append(np.array(pts, float))
    return out


def alongamento(p):
    c = np.cov((p - p.mean(0)).T) if len(p) > 2 else np.eye(2)
    ev = np.sort(np.linalg.eigvalsh(c))
    return float(np.sqrt(max(ev[1], 1e-9) / max(ev[0], 1e-3)))


def solidez(p):
    """Área ÷ área do invólucro convexo (K5): a lasca ANGULOSA e convexa ≈ 0,8–0,95; a esponja e o rendilhado, baixo."""
    if len(p) < 6:
        return 1.0
    pts = sorted(set(map(tuple, np.concatenate([p + d for d in ((0, 0), (0, 1), (1, 0), (1, 1))]))))

    def cruz(o, a, b):
        return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])

    lo, hi = [], []
    for q in pts:
        while len(lo) >= 2 and cruz(lo[-2], lo[-1], q) <= 0:
            lo.pop()
        lo.append(q)
    for q in reversed(pts):
        while len(hi) >= 2 and cruz(hi[-2], hi[-1], q) <= 0:
            hi.pop()
        hi.append(q)
    h = np.array(lo[:-1] + hi[:-1], float)
    area = 0.5 * abs(np.dot(h[:, 0], np.roll(h[:, 1], 1)) - np.dot(h[:, 1], np.roll(h[:, 0], 1)))
    return float(len(p) / max(area, 1.0))


def anisotropia(m):
    m = m.astype(float)
    ex = np.abs(np.diff(m, axis=1)).mean()
    ey = np.abs(np.diff(m, axis=0)).mean()
    return ex / max(ey, 1e-9)


def extensao(c, limiar=0.3):
    cols = np.where((c > limiar).mean(0) > 0.05)[0]
    rows = np.where((c > limiar).mean(1) > 0.05)[0]
    return cols.min(), cols.max(), rows.min(), rows.max()


def main():
    item, p = sys.argv[1], sys.argv[2]
    a = carrega(p)
    c = cobertura(a)
    r = {}
    for nome, yc, meia in (("40", 190, 50), ("200", 440, 150)):
        f = faixa(c, yc, meia)
        if item == "splats":
            esc = float(sys.argv[3]) if len(sys.argv) > 3 else 1.0
            D = int(int(nome) * esc)
            x0, x1, y0, y1 = 60, 940, meia - D // 2, meia + D // 2
        else:
            x0, x1, y0, y1 = extensao(f)
            D = y1 - y0 + 1
        nucleo = f[y0 + D // 6:y1 - D // 6 + 1, x0 + D // 2:x1 - D // 2]
        m = nucleo > 0.5
        if item == "fan":
            r[nome] = dict(D=int(D), aniso=round(anisotropia(m), 2))
        elif item == "flat":
            w = max(D // 4, 6)
            pontas = [f[y0:y1 + 1, x0:x0 + w] > 0.5, f[y0:y1 + 1, x1 - w + 1:x1 + 1] > 0.5]
            r[nome] = dict(D=int(D), aniso_pontas=round(np.mean([anisotropia(q) for q in pontas]), 2),
                           aniso_meio=round(anisotropia(m), 2))
        elif item == "round":
            lum = a[yc - meia:yc + meia, :].mean(2)[y0 + D // 6:y1 - D // 6 + 1, x0 + D // 2:x1 - D // 2]
            med = np.median(lum)
            r[nome] = dict(D=int(D), p5_sobre_med=round(float(np.percentile(lum, 5) / med), 3),
                           desvio=round(float(lum.std() / med), 3),
                           buracos=round(float((nucleo < 0.5).mean()), 3))
        elif item == "d3":
            buracos = componentes(~m, 6)
            if buracos:
                ar = np.array([len(b) for b in buracos], float)
                al = np.array([alongamento(b) for b in buracos])
                so = np.array([solidez(b) for b in buracos])
                r[nome] = dict(D=int(D), frac=round(float((~m).mean()), 3),
                               tam_D=round(float(np.sqrt((ar * ar).sum() / ar.sum())) / D, 3),
                               along=round(float((al * ar).sum() / ar.sum()), 2),
                               solidez=round(float((so * ar).sum() / ar.sum()), 2), n=len(buracos))
            else:
                r[nome] = dict(D=int(D), frac=0.0)
        elif item == "splats":
            reg = f[y0:y1 + 1, x0 + D // 2:x1 - D // 2]
            pts = componentes(reg > 0.25, 1)
            ar = np.array([len(b) for b in pts], float) if pts else np.zeros(1)
            grandes = ar >= max(12, (0.03 * D) ** 2)
            tons = [float(np.mean([reg[int(y), int(x)] for y, x in b])) for b in pts] if pts else [0]
            tons = np.array(tons)
            r[nome] = dict(D=int(D), tom=round(float(reg.mean()), 3), n=len(pts),
                           massa_grandes=round(float(ar[grandes].sum() / max(ar.sum(), 1)), 3),
                           tom_grandes=round(float(tons[grandes].mean()) if grandes.any() else 0, 3),
                           tom_pequenos=round(float(tons[~grandes].mean()) if (~grandes).any() else 0, 3))
    print(item, p.rsplit("/", 2)[-2] + "/" + p.rsplit("/", 1)[-1], r)


main()
