#!/usr/bin/env python3
"""Verificador das fixtures do pincel de tecido: relê repouso+deformado, reconta os movidos e o
máximo, e compara com o cabeçalho. Exit 0 = coerente. (Harness — não carrega algoritmo nenhum.)"""
import gzip, sys, os, math, glob
def read(path):
    hdr={}; path_pts=[]; verts=[]
    with gzip.open(path,'rt') as f:
        for line in f:
            if line.startswith('#'): continue
            t=line.split()
            if t[0]=='c': path_pts.append(tuple(map(float,t[1:4])))
            elif t[0] in ('v','d'): verts.append(tuple(map(float,t[1:4])))
            elif t[0] not in ('caminho','vertices'): hdr[t[0]]=t[1]
    return hdr, path_pts, verts
def main(d):
    ok=True
    rest={s: read(os.path.join(d,f'{s}.repouso.txt.gz'))[2] for s in ('plano','esfera') if os.path.exists(os.path.join(d,f'{s}.repouso.txt.gz'))}
    for f in sorted(glob.glob(os.path.join(d,'*.deformado.txt.gz'))):
        hdr, pts, dv = read(f); rv = rest[hdr['superficie']]
        assert len(dv)==len(rv), f
        n=[math.dist(a,b) for a,b in zip(rv,dv)]
        moved=sum(1 for x in n if x>1e-5); mx=max(n)
        good = moved==int(hdr['movidos']) and abs(mx-float(hdr['max_deslocamento']))<2e-6 and len(pts)==int(hdr['passos'])
        ok &= good
        print(('OK ' if good else 'BAD'), os.path.basename(f), moved, f'{mx:.6f}')
    sys.exit(0 if ok else 1)
if __name__=='__main__': main(sys.argv[1] if len(sys.argv)>1 else os.path.dirname(os.path.abspath(__file__)))
