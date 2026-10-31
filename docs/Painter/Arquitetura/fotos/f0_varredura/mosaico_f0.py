"""Mosaico da varredura F0: um PNG por pincel × papel × lei; linhas = Pigment × Water, colunas = N × pressão.
Legenda (render PLANO): M<manchas_azul_plano %> F<ferrugem_plano %> K<km_p99_plano>. Uso: mosaico_f0.py <tsv> <pasta celulas> <saída>."""
import csv, os, sys
from collections import defaultdict
from PIL import Image, ImageDraw

tsv, cel, out = sys.argv[1:4]
os.makedirs(out, exist_ok=True)
rows = list(csv.DictReader(open(tsv), delimiter='\t'))
g = lambda r, k, alt: float(r.get(k) or r.get(alt) or 'nan')
por = defaultdict(list)
for r in rows:
    por[(r['pincel'], r.get('papel') or 'liso', r.get('lei') or 'velha')].append(r)
TW, TH, LBL, TOP = 150, 65, 120, 18
for (p, pa, lei), rs in por.items():
    linhas = sorted({(float(r['pigment']), float(r['water'])) for r in rs})
    cols = sorted({(int(r['n']), float(r['pressao'])) for r in rs})
    img = Image.new('RGB', (LBL + TW * len(cols), TOP * 2 + (TH + 14) * len(linhas)), (255, 255, 255))
    d = ImageDraw.Draw(img)
    d.text((4, 2), f"{p} · paper {pa} · law {lei}", fill=(0, 0, 0))
    for j, (n, pr) in enumerate(cols):
        d.text((LBL + j * TW + 4, TOP), f"N={n} pressao={pr}", fill=(0, 0, 0))
    idx = {(float(r['pigment']), float(r['water']), int(r['n']), float(r['pressao'])): r for r in rs}
    for i, (pg, w) in enumerate(linhas):
        y = TOP * 2 + i * (TH + 14)
        d.text((4, y + TH // 2), f"Pigment {pg:g}\nWater {w:g}", fill=(0, 0, 0))
        for j, (n, pr) in enumerate(cols):
            r = idx.get((pg, w, n, pr))
            if not r:
                continue
            t = Image.open(os.path.join(cel, r['png'])).resize((TW - 4, TH))
            img.paste(t, (LBL + j * TW, y))
            d.text((LBL + j * TW, y + TH),
                   f"M{g(r, 'manchas_azul_plano', 'manchas_azul_pct'):.0f} F{g(r, 'ferrugem_plano', 'ferrugem_pct'):.1f}"
                   f" K{g(r, 'km_p99_plano', 'km_p99'):.0f}", fill=(80, 0, 0))
    slug = p.rsplit('/', 1)[-1].replace(' ', '_').lower()
    nome = f"{slug}_{pa}_{lei}.png"
    img.save(os.path.join(out, nome))
    print(nome)
