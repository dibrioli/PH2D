from sonda_rotacao_fast import *
import itertools
def epx2(img,W,H,eq):
    o=[[0]*(2*W) for _ in range(2*H)]
    g=lambda x,y,p: img[y][x] if 0<=x<W and 0<=y<H else p
    for y in range(H):
        for x in range(W):
            p=img[y][x]; a=g(x,y-1,p); b=g(x+1,y,p); c=g(x-1,y,p); d=g(x,y+1,p)
            o[2*y][2*x]=a if c==a and c!=d and a!=b else p
            o[2*y][2*x+1]=b if a==b and a!=c and b!=d else p
            o[2*y+1][2*x]=c if d==c and d!=b and c!=a else p
            o[2*y+1][2*x+1]=d if b==d and b!=a and d!=c else p
    return o,2*W,2*H
def run_rs(c,pad,K,samp):
    bx,by,bw,bh=D['inputs'][c['block']]['box']
    f=c['fields']; t=around(compose(-f.get('angle',0),1,1,-f.get('skew',0)),(bx,by,bw,bh))
    W,H=bw+2*pad,bh+2*pad
    img=[[ (y-pad)*bw+(x-pad)+1 if pad<=x<bw+pad and pad<=y<bh+pad else 0 for x in range(W)] for y in range(H)]
    k=1
    while k<K: img,W,H=epx2(img,W,H,None); k*=2
    a,b,cc,d,e,ff=t; det=a*d-b*cc
    want=c['ids']; n=0
    for y in range(24):
        for x in range(32):
            ox,oy=x+0.5,y+0.5
            dx,dy=ox-e,oy-ff
            u=(d*dx-cc*dy)/det+pad; v=(-b*dx+a*dy)/det+pad
            if samp=='round':
                iu,iv=math.floor(u*K+0.5)-0, math.floor(v*K+0.5)
            else: iu,iv=math.floor(u*K),math.floor(v*K)
            g=img[iv][iu] if 0<=iu<W and 0<=iv<H else 0
            n+= g!=want[y][x]
    return n
CS=[(k,c) for k,c in D['cases'].items() if c['algorithm']=='rotsprite']
for pad,K,samp in itertools.product([0,1],[2,4,8],['floor','round']):
    r=[run_rs(c,pad,K,samp) for k,c in CS]
    print(pad,K,samp,sum(r),r,flush=True)
