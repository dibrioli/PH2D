"""Régua K8 — o papel por baixo do planalto do Knife Smooth 2 (kL 1, cena `smooth2`): a luminância (sRGB da
fracção da luz do papel longe, y 440..455) por linha, x 150..500, nossa (`oleo.ppm`) × o Cycles da MESMA tinta.

Uso: python3 -I papel_regua_k8.py <entrada da sonda> <saida cycles h_fina> [<saida cycles h_sombra> …]
"""
import sys, numpy as np
from PIL import Image
import OpenImageIO as oiio
ent=sys.argv[1]; sais=sys.argv[2:]
GW,GH=700,460
X0,X1=150,500
lin=lambda s: np.where(s/255<=0.04045,(s/255)/12.92,((s/255+0.055)/1.055)**2.4)
srgb=lambda l: 255*np.where(l<=0.0031308,12.92*l,1.055*np.clip(l,0,None)**(1/2.4)-0.055)
W=np.array([0.2126,0.7152,0.0722])
def nosso(d):
    a=np.asarray(Image.open(f"{d}/oleo.ppm").convert("RGB")).astype(float)
    return lin(a)@W
def cyc(d,luz="studio_sol10"):
    return oiio.ImageBuf(f"{d}/{luz}_cor.exr").get_pixels(oiio.FLOAT)[...,:3].astype(float)@W
rows=list(range(284,296))+list(range(322,352))
def prof(L):
    ref=np.median(L[440:455,X0:X1])
    return [round(float(srgb(min(1,np.median(L[y,X0:X1])/ref)))) for y in rows]
print("rows ", rows)
print("nosso", prof(nosso(ent)))
for s in sais:
    for luz in ("studio_sol10","sol10"):
        print(s.split('/')[-1], luz, prof(cyc(s,luz)))
