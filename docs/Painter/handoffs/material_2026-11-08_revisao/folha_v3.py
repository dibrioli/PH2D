# Folha v3 (08/11): por pincel, CSP à esquerda e PH2D à direita, uma linha por fundo (vazio · branco · blocos),
# e a medida por fundo: cobertura, largura da reta, cor média do pintado (sobre o papel branco), e o veredito.
# Uso: python3 -I folha_v3.py <cena> <dir_csp_out...> -- <saida>   (o CSP: <PH2D_nome>.png 2400x880 (ou base_<fundo> 800x880 por fundo), sobre o papel branco)
import sys, pathlib, json
import numpy as np
from PIL import Image, ImageDraw

W, H, COL = 2400, 880, 800
FUNDOS = ['vazio', 'branco', 'blocos']


def sobre_branco(a):
    f = a.astype(np.float32) / 255
    if f.shape[2] == 4:
        al = f[..., 3:4]; f = f[..., :3] * al + (1 - al)
    return f * 255


def mede(img, base):
    """img, base: HxWx3 float sobre branco. Por fundo: cobertura %, largura da reta (px), cor média do pintado."""
    pint = np.abs(img - base).max(-1) > 12
    out = {}
    for k, nome in enumerate(FUNDOS):
        x0 = k * COL; m = pint[:, x0:x0 + COL]; im = img[:, x0:x0 + COL]
        cov = m.mean() * 100
        larg = float(np.median(m[:220, 200:600].sum(0)))
        cor = im[m].mean(0).round().astype(int).tolist() if m.any() else None
        cores = None
        if nome == 'blocos':  # por bloco: a cor média do pintado
            cores = []
            for b in range(3):
                s = slice(b * COL // 3 + 20, (b + 1) * COL // 3 - 20)
                mb = m[:, s]; cores.append(im[:, s][mb].mean(0).round().astype(int).tolist() if mb.any() else None)
        out[nome] = dict(cov=round(cov, 2), larg=larg, cor=cor, cores=cores)
    return out


def dist(c1, c2):
    if c1 is None or c2 is None:
        return 0.0 if c1 is c2 else 999.0
    return float(np.linalg.norm(np.array(c1) - np.array(c2)))


def veredito(c, p):
    if c['cov'] < 0.3 and p['cov'] < 0.3:
        return 'igual', 'nenhum pinta'
    if (c['cov'] < 0.3) != (p['cov'] < 0.3):
        return 'diferente', ('PH2D não pinta' if p['cov'] < 0.3 else 'CSP não pinta')
    rc = p['cov'] / c['cov']; rl = (p['larg'] + 1) / (c['larg'] + 1)
    dc = dist(c['cor'], p['cor'])
    if c['cores']:
        dc = max([dc] + [dist(a, b) for a, b in zip(c['cores'], p['cores']) if a and b])
    r = max(abs(np.log(rc)), abs(np.log(rl)))
    txt = f'cob ×{rc:.2f} · larg ×{rl:.2f} · cor Δ{dc:.0f}'
    if r <= np.log(1.2) and dc <= 15:
        return 'igual', txt
    if r <= np.log(1.6) and dc <= 40:
        return 'perto', txt
    return 'diferente', txt


if __name__ == '__main__':
    a = sys.argv[1:]; i = a.index('--'); cena = pathlib.Path(a[0]); csps = [pathlib.Path(p) for p in a[1:i]]
    saida = pathlib.Path(a[i + 1]); saida.mkdir(parents=True, exist_ok=True)
    base = sobre_branco(np.fromfile(cena / 'base.rgba', np.uint8).reshape(H, W, 4))
    ph = cena / 'out_ph2d'
    tabela = []
    for l in (cena / 'pinceis.tsv').read_text().splitlines():
        csp_n, tam, nome, _ = l.split('\t'); s = nome.replace(' ', '_')
        fc = next((d / f'{s}.png' for d in csps if (d / f'{s}.png').exists()), None)
        partes = [next((d / f'{s}_{n}.png' for d in csps if (d / f'{s}_{n}.png').exists()), None)
                  for n in ('a_vazio', 'b_branco', 'c_blocos')]
        if fc is not None:
            ci = sobre_branco(np.asarray(Image.open(fc).convert('RGBA')))
        elif all(partes):  # o CSP de avaliação não exporta: uma foto a 100 % por fundo, sobre o papel branco
            ci = np.concatenate([sobre_branco(np.asarray(Image.open(q).convert('RGBA'))) for q in partes], 1)
        else:
            print('sem CSP:', nome); continue
        if ci.shape[:2] != (H, W):
            print('tamanho errado', nome, ci.shape); continue
        pi = sobre_branco(np.fromfile(ph / f'{s}.rgba', np.uint8).reshape(H, W, 4))
        mc, mp = mede(ci, base), mede(pi, base)
        linha = dict(pincel=nome, csp=csp_n, tam=int(tam))
        # folha: 3 linhas (fundos) × 2 colunas (CSP, PH2D), metade do tamanho
        t = 400; fol = Image.new('RGB', (2 * t + 30, 3 * (H // 2 + 40) + 40), (40, 40, 40)); d = ImageDraw.Draw(fol)
        d.text((10, 8), f'{nome}  (CSP «{csp_n}», {tam} px)   esquerda: Clip Studio · direita: PH2D', fill=(255, 255, 255))
        for k, f in enumerate(FUNDOS):
            v, txt = veredito(mc[f], mp[f]); linha[f] = dict(veredito=v, medida=txt, csp=mc[f], ph2d=mp[f])
            y = 40 + k * (H // 2 + 40)
            for j, im in enumerate((ci, pi)):
                c = Image.fromarray(im[:, k * COL:(k + 1) * COL].clip(0, 255).astype(np.uint8)).resize((t, H // 2))
                fol.paste(c, (10 + j * (t + 10), y + 20))
            d.text((10, y + 4), f'{f}: {v} — {txt}', fill=(255, 230, 120) if v != 'igual' else (160, 255, 160))
        fol.save(saida / f'folha_{s}.png'); tabela.append(linha)
    (saida / 'medidas_v3.json').write_text(json.dumps(tabela, ensure_ascii=False, indent=1))
    for t_ in tabela:
        print(f"{t_['pincel']:18s} " + ' | '.join(f"{f}: {t_[f]['veredito']:9s} {t_[f]['medida']}" for f in FUNDOS))
