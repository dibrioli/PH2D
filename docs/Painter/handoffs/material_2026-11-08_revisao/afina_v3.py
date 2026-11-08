# Afina (08/11): cada variante `<slug>__<suf>.rgba` (sonda com PH2D_V3_VARIANTES) contra o CSP do pincel, pela régua
# da folha_v3. Uso: python3 -I afina_v3.py <cena> <dir_csp...> [-- <filtro de slug>]
import sys, pathlib
import numpy as np
from PIL import Image
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from folha_v3 import W, H, FUNDOS, sobre_branco, mede, veredito  # noqa: E402

a = sys.argv[1:]
filtro = a[a.index('--') + 1] if '--' in a else ''
a = a[:a.index('--')] if '--' in a else a
cena = pathlib.Path(a[0]); csps = [pathlib.Path(p) for p in a[1:]]
base = sobre_branco(np.fromfile(cena / 'base.rgba', np.uint8).reshape(H, W, 4))
NOTA = {'igual': 2, 'perto': 1, 'diferente': 0}


def csp(slug):
    partes = [next((d / f'{slug}_{n}.png' for d in csps if (d / f'{slug}_{n}.png').exists()), None)
              for n in ('a_vazio', 'b_branco', 'c_blocos')]
    return np.concatenate([sobre_branco(np.asarray(Image.open(q).convert('RGBA'))) for q in partes], 1)


for f in sorted((cena / 'out_ph2d').glob('*.rgba')):
    slug, _, suf = f.stem.partition('__')
    if filtro and filtro not in slug:
        continue
    mc = mede(csp(slug), base)
    mp = mede(sobre_branco(np.fromfile(f, np.uint8).reshape(H, W, 4)), base)
    vs = [veredito(mc[k], mp[k]) for k in FUNDOS]
    nota = sum(NOTA[v] for v, _ in vs)
    print(f"{slug:18s} {suf or '(hoje)':7s} {nota}  " + ' | '.join(f'{v[:5]} {t}' for v, t in vs))
