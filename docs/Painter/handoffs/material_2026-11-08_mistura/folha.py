# folha.py <nome> <antes.rgba> <depois.rgba> <saida.png> <rotulo_antes> <rotulo_depois>
import sys,numpy as np
from PIL import Image,ImageDraw
sys.path.insert(0,__import__('os').path.dirname(__file__))
from png import load
n,a,d,out,la,ld=sys.argv[1:7]
A=load(a);D=load(d)
cw,ch=400,440; head=40
img=Image.new('RGB',(2*cw+30,head+3*ch+20),(255,255,255)); dr=ImageDraw.Draw(img)
dr.text((10,5),n,fill=(0,0,0)); dr.text((10,22),'antes: '+la,fill=(0,0,0)); dr.text((cw+30,22),'depois: '+ld,fill=(0,0,0))
for i in range(3):
    for j,X in enumerate((A,D)):
        c=Image.fromarray(X[:,i*800:(i+1)*800]).resize((cw,ch))
        img.paste(c,(j*(cw+30),head+i*ch))
img.save(out)
