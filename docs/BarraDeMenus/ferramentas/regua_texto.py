#!/usr/bin/env python3
import warnings; warnings.filterwarnings("ignore")
"""Régua de nitidez de um texto (onda 8): numa caixa de uma foto, o fundo = a cor mais comum; por pixel, a
cobertura = contraste dele / contraste do pixel de pico. Saída: pico (WCAG), fração de pixels de tinta que
chegam a ≥ 90 % da cobertura (traço que cai cheio num pixel) e a tinta média.
Uso: regua_texto.py foto.png x y w h [rótulo]"""
import sys
from collections import Counter
from PIL import Image
def L(c):
    f=lambda v:(v/255)/12.92 if v/255<=0.03928 else ((v/255+0.055)/1.055)**2.4
    return 0.2126*f(c[0])+0.7152*f(c[1])+0.0722*f(c[2])
cr=lambda a,b:(max(L(a),L(b))+.05)/(min(L(a),L(b))+.05)
p,x,y,w,h=sys.argv[1],*map(int,sys.argv[2:6]); nome=sys.argv[6] if len(sys.argv)>6 else p
im=Image.open(p).convert('RGB').crop((x,y,x+w,y+h)); px=list(im.getdata())
bg=Counter(px).most_common(1)[0][0]; c=[cr(q,bg) for q in px]; pico=max(c)
cob=[(v-1)/(pico-1) for v in c if v>1.05]
cheio=sum(1 for v in cob if v>=0.9)/max(1,len(cob))
print(f"{nome:28} pico {pico:5.2f}:1  tinta {len(cob):4d} px  cheios {cheio*100:4.1f}%  cobertura média {sum(cob)/max(1,len(cob)):.2f}")
