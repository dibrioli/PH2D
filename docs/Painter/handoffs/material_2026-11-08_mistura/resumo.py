import sys,numpy as np
from PIL import Image,ImageDraw
from png import load
pares=[l.split('\t') for l in open('pares.tsv').read().strip().split('\n')]
cw,ch=300,200
img=Image.new('RGB',(2*(2*cw+10)+30,len(pares)//2*(ch+24)+(ch+24)),(255,255,255)); dr=ImageDraw.Draw(img)
for i,(n,a,d) in enumerate(pares):
    X=(i%2)*(2*cw+40); Y=(i//2)*(ch+24)
    dr.text((X,Y+2),n+'  (antes | depois)',fill=(0,0,0))
    for j,f in enumerate((a,d)):
        im=load('cena/out_ph2d/'+f+'.rgba')[40:600,1600:2400]
        img.paste(Image.fromarray(im).resize((cw,ch)),(X+j*(cw+5),Y+18))
img.save(sys.argv[1])
