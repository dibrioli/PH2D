#!/usr/bin/env python3
"""Rodada 3 da Convolution Matrix (offline): recupera os kernels que as rodadas 1–2 não fixaram, sobre
`oracle_trial_round3.json` — 17-largos em 40×40 (A1), os saturados em baixo contraste (A2), e confere o
Tiled = volta à imagem (A3). Mesma lei: clamp(trunc0(Σ w·v / div) + bias), borda replicada.

Uso: python3 -I fit_convolucao_r3.py <saida.json>   (cwd = raiz do repo)
     python3 -I fit_convolucao.py --rust <fit_r12.json> <fit_r3.json> crates/ph2d-pixel-core/src/filter/kernels.rs
"""
import json, itertools, sys
d=json.load(open('crates/ph2d-pixel-core/tests/fixtures/oracle_trial_round3.json'))
def t0(a,b):
    q=abs(a)//abs(b); return q if (a>=0)==(b>0) else -q
def conv(src,w,div,bias,wrap=False):
    H=len(src);W=len(src[0])
    cl=(lambda v,n: v%n) if wrap else (lambda v,n: min(max(v,0),n-1))
    return [[max(0,min(255,t0(sum(k*src[cl(y-dy,H)][cl(x-dx,W)] for (dx,dy),k in w.items()),div)+bias)) for x in range(W)] for y in range(H)]
def pairs(sec,k,suffix=''):
    c=d[sec]['cases']; out=[]
    for key in c:
        f,_,kk=key.partition(':')
        if kk==f'{k}{suffix}': out.append((c[f+':input'],c[key],f))
    return out
def miss(ps,w,div,bias,wrap=False):
    n=0
    for i,o,f in ps:
        r=conv(i,w,div,bias,wrap); n+=sum(r[y][x]!=o[y][x] for y in range(len(o)) for x in range(len(o[0])))
    return n
def blur(r): return {(x,y):1+(r-abs(x))+(r-abs(y)) for x in range(-r,r+1) for y in range(-r,r+1)}
res={}
w=blur(8); print('blur-17x17', miss(pairs('A1_conv_17wide',8),w,sum(w.values()),0)); res['blur-17x17']=(w,sum(w.values()),0)
for name,k,w in [('drunk-17x17_x',34,{(0,0):1,(8,8):1,(-8,8):1,(8,-8):1,(-8,-8):1}),('drunk-17x17_+',35,{(0,0):1,(0,8):1,(0,-8):1,(8,0):1,(-8,0):1})]:
    print(name, miss(pairs('A1_conv_17wide',k),w,5,0)); res[name]=(w,5,0)
# ring _o: read support from imp40 (255 on black): cells != 0
for name,k in [('drunk-17x17_o',36),('blur-17x3-left',10),('blur-3x17-top',11)]:
    ps=pairs('A1_conv_17wide',k); imp=[p for p in ps if p[2]=='imp40'][0][1]; ig=[p for p in ps if p[2]=='impg40'][0][1]
    supp={(x-20,y-20):imp[y][x] for y in range(40) for x in range(40) if imp[y][x]}
    gsup={(x-20,y-20):ig[y][x]-128 for y in range(40) for x in range(40) if ig[y][x]!=128}
    print(name,'imp40 resp',sorted(supp.items())[:60]); print('  impg40',sorted(gsup.items())[:60])

w={(x,y):1 for (x,y) in [(int(a),int(b)) for a,b in []]}
ps=pairs('A1_conv_17wide',36); imp=[p for p in ps if p[2]=='imp40'][0][1]
ring={(x-20,y-20):1 for y in range(40) for x in range(40) if imp[y][x]}
print('ring n',len(ring), 'miss div40', miss(ps,ring,40,0)); res['drunk-17x17_o']=(ring,40,0)

def fit_iv(ps, C=20, R=16, smax=3000):
    get=lambda f: [p for p in ps if p[2]==f][0][1]
    imp,ig,i0=get('imp40'),get('impg40'),get('imp040')
    cells=[(dx,dy) for dy in range(-R,R+1) for dx in range(-R,R+1) if 0<=C+dx<40 and 0<=C+dy<40]
    for s in range(1,smax):
        lo={};hi={};ok=True
        for (dx,dy) in cells:
            y,x=C+dy,C+dx; l,h=0,s
            for m,r,ce in [(255,imp[y][x],0),(127,ig[y][x]-128,0),(128,128-i0[y][x],1)]:
                if ce: l=max(l,((r-1)*s)//m+1); h=min(h,(r*s)//m)
                else: l=max(l,-((-r*s)//m)); h=min(h,((r+1)*s-1)//m)
            if l>h: ok=False;break
            lo[(dx,dy)]=l;hi[(dx,dy)]=h
        if not ok or not (sum(lo.values())<=s<=sum(hi.values())): continue
        free=[c for c in lo if lo[c]<hi[c]]
        need=s-sum(lo.values())
        if len(free)<=18:
            for pick in itertools.combinations(free,need):
                w={c:v for c,v in lo.items() if v}
                for c in pick: w[c]=w.get(c,0)+1
                if miss(ps,w,s,0)==0: return w,s
        else:
            print('  s',s,'free',len(free),'need',need)
    return None
for name,k in [('blur-17x3-left',10),('blur-3x17-top',11)]:
    r=fit_iv(pairs('A1_conv_17wide',k))
    print(name, r and (r[1], sorted(r[0].items())))
    if r: res[name]=(r[0],r[1],0)
json.dump({n:{'w':{f'{a},{b}':v for (a,b),v in w.items()},'div':dv,'bias':bs} for n,(w,dv,bs) in res.items()}, open(sys.argv[1],'w'))

print('--- A2')
for name,k in [('misc-texturize',21),('misc-emboss',22),('misc-rock',24)]:
    ps=pairs('A2_conv_low_contrast',k); imp=[p for p in ps if p[2]=='impl'][0][1]; i0=[p for p in ps if p[2]=='imp0l'][0][1]
    rp={(x-7,y-7):imp[y][x]-128 for y in range(6,9) for x in range(6,9)}
    rn={(x-7,y-7):128-i0[y][x] for y in range(6,9) for x in range(6,9)}
    print(name,'impl',rp,'imp0l',rn)
    best=None
    for bias in (0,128,64,-128,255):
      for div in (1,2,3,4,8):
        w={c:round(v*div/8) for c,v in rp.items() if round(v*div/8)}
        m=miss(ps,w,div,bias)
        if best is None or m<best[0]: best=(m,w,div,bias)
    print('  best',best)
    if best[0]==0: res[name]=(best[1],best[2],best[3])
ps=pairs('A2_conv_low_contrast',17)
print('edges zero kernel', [miss(pairs('A2_conv_low_contrast',k),{(0,0):0},1,128) for k in (17,18,19)])
for n in ('edges-find','edges-find-horizontal','edges-find-vertical'): res[n]=({(0,0):0},1,128)
json.dump({n:{'w':{f'{a},{b}':v for (a,b),v in w.items()},'div':dv,'bias':bs} for n,(w,dv,bs) in res.items()}, open(sys.argv[1],'w'))
print('A3 tiled', miss(pairs('A3_conv_tiled',3,':tiled'),{(x,y):[1,2,1][x+1]*[1,2,1][y+1] for x in (-1,0,1) for y in (-1,0,1)},16,0,True),
 miss(pairs('A3_conv_tiled',14,':tiled'),{**{(x,y):-1 for x in (-1,0,1) for y in (-1,0,1)},(0,0):16},8,0,True))
