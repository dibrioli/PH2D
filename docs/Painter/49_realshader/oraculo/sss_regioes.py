"""Régua M1 (W3): o Δ da Subsurface do Cycles por região (máscaras da ENTRADA: h_fina, cobertura; a sombra pela
irradiância). python3 sss_regioes.py <entrada> <saida_cycles> mat1,mat2 (base = mat_neutro)."""
import sys, numpy as np, OpenImageIO as oiio
E, C = sys.argv[1], sys.argv[2]
def pnm(p):
    b=open(p,'rb').read(); s=b.split(maxsplit=4); w,h=int(s[1]),int(s[2])
    return np.frombuffer(s[4],np.uint8).reshape(h,w,-1).squeeze().astype(float)
def lin(s):
    s=s/255; return np.where(s<=0.04045,s/12.92,((s+0.055)/1.055)**2.4)
ex=lambda n: oiio.ImageBuf(f"{C}/{n}.exr").get_pixels(oiio.FLOAT)[...,:3].astype(float)
L=np.array([.2126,.7152,.0722])
cob=pnm(f"{E}/cobertura.pgm")/255; gh,gw=cob.shape
h=np.fromfile(f"{E}/h_fina.f32","<f4").reshape(gh,gw)
alb=lin(pnm(f"{E}/albedo.ppm"))
n=np.clip(ex("mat_neutro_cor"),0,None)
rel=(n@L)/np.maximum(alb@L,1e-4)
sat=lambda c:(c.max(-1)-c.min(-1))/np.maximum(c.max(-1),1e-6)
tinta=cob>0.9
M={"borda .1-.9":(cob>0.1)&(cob<0.9),"fina h<2":tinta&(h<2),"h 2-8":tinta&(h>=2)&(h<8),"h 8-30":tinta&(h>=8)&(h<30),"grossa h>30":tinta&(h>=30),
   "sombra rel<.5":tinta&(rel<0.5),"lit rel>.9":tinta&(rel>0.9)}
for k,v in M.items(): print(f"{k:14} n={v.sum()}")
for m in sys.argv[3].split(","):
    c=np.clip(ex(f"mat_{m}_cor"),0,None)
    print(m)
    for k,v in M.items():
        d=(c@L-n@L)[v]
        print(f"  {k:14} lum {np.mean((n@L)[v]):.3f}->{np.mean((c@L)[v]):.3f} Δ{d.mean():+.4f} | R Δ{(c-n)[...,0][v].mean():+.4f} G {(c-n)[...,1][v].mean():+.4f} B {(c-n)[...,2][v].mean():+.4f} | sat {np.mean(sat(n)[v]):.3f}->{np.mean(sat(c)[v]):.3f}")
