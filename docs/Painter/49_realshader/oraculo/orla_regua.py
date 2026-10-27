"""Régua da ORLA (K6): a tinta a <= 3 células do papel, luminância sRGB nossa × o Cycles da MESMA malha.

    python3 orla_regua.py <entrada> <saida_cycles> <luz> [<entrada> <saida> <luz> ...]

<entrada> = pasta da sonda `medida_oleo_real_oraculo` (cobertura.pgm, oleo.ppm); <saida> = a do `cycles.py`.
"""
import sys, numpy as np, OpenImageIO as oiio
def pnm(n):
    b=open(n,'rb').read(); p=b.split(maxsplit=4); w,h=int(p[1]),int(p[2])
    d=np.frombuffer(p[4],np.uint8); return d[:w*h*(len(d)//(w*h))].reshape(h,w,-1).squeeze().astype(float)
def srgb(x):
    x=np.clip(x,0,1); return 255*np.where(x<=0.0031308,12.92*x,1.055*x**(1/2.4)-0.055)
lum=lambda im: im@np.array([.2126,.7152,.0722])
for ent,sai,luz in zip(sys.argv[1::3],sys.argv[2::3],sys.argv[3::3]):
    cob=pnm(ent+'/cobertura.pgm')/255
    # orla = tinta a <=3 células do papel
    t=cob>0.02; papel=~t; perto=np.zeros_like(t)
    for dy in range(-3,4):
        for dx in range(-3,4):
            perto|=np.roll(np.roll(papel,dy,0),dx,1)
    orla=t&perto
    nos=lum(pnm(ent+'/oleo.ppm')); cy=lum(srgb(oiio.ImageBuf(f'{sai}/{luz}_cor.exr').get_pixels(oiio.FLOAT)[...,:3]))
    f=lambda L: f"<40 {100*(L[orla]<40).mean():5.2f}% <60 {100*(L[orla]<60).mean():5.2f}% p10 {np.percentile(L[orla],10):5.1f}"
    d=np.abs(nos-cy)[orla].mean()
    print(f"{ent.split('/')[-1]:<5} {luz:<13} orla {orla.sum()} · nós {f(nos)} · Cycles {f(cy)} · |Δ| {d:5.1f} · corr {np.corrcoef(nos[orla],cy[orla])[0,1]:.3f}")
