# Cena de zigue-zagues (08/11, fork de comparação): blocos secos + 5 zigue-zagues coloridos.
import sys, os, math
from PIL import Image, ImageDraw
W,H=420,280
AZUL,AMAR,VERM,VERDE=(0,70,190),(240,200,0),(200,20,20),(78,160,60)
BLOCOS=[(15,135,AZUL),(150,270,AMAR),(285,405,VERM)]
def zz_v(x0,x1,y0,y1,n):  # dentes horizontais, avança em y
    pts=[]
    for k in range(n+1):
        y=y0+(y1-y0)*k/n; pts.append((x0 if k%2==0 else x1, y))
    return pts
def zz_h(x0,x1,y0,y1,n):  # dentes verticais, avança em x
    pts=[]
    for k in range(n+1):
        x=x0+(x1-x0)*k/n; pts.append((x, y0 if k%2==0 else y1))
    return pts
def lerp(poly,step=3.0):
    out=[poly[0]]
    for (ax,ay),(bx,by) in zip(poly,poly[1:]):
        n=max(1,math.ceil(math.hypot(bx-ax,by-ay)/step))
        for i in range(1,n+1): out.append((ax+(bx-ax)*i/n, ay+(by-ay)*i/n))
    return out
TRACOS=[ # (nome, cor, polilinha)
 ('y',AMAR, zz_v(95,185,30,120,7)),     # amarelo sobre azul|amarelo
 ('b',AZUL, zz_v(230,320,30,120,7)),    # azul sobre amarelo|vermelho
 ('r',VERM, zz_v(165,255,135,210,6)),   # vermelho sobre amarelo
 ('g',VERDE,zz_h(20,140,195,250,8)),    # verde sobre azul|branco
 ('x',VERM, zz_h(70,350,45,105,8)),     # 2.ª camada: cruza as três
]
def celula():
    im=Image.new('RGB',(W,H),'white'); d=ImageDraw.Draw(im)
    for x0,x1,c in BLOCOS: d.rectangle([x0,20,x1-1,219],fill=c)
    return im
if __name__=='__main__':
    d=sys.argv[1]; os.makedirs(d,exist_ok=True)
    c=celula(); open(os.path.join(d,'base_celula.rgba'),'wb').write(c.convert('RGBA').tobytes())
    t=Image.new('RGB',(870,690),'white')
    for (x,y) in [(10,10),(440,10),(10,300),(440,300)]: t.paste(c,(x,y))
    dd=ImageDraw.Draw(t)
    for i,col in enumerate([AMAR,AZUL,VERM,VERDE]): dd.rectangle([20+60*i,610,50+60*i,640],fill=col)
    t.save(os.path.join(d,'base_csp.png'))
    for nome,cor,p in TRACOS:
        open(os.path.join(d,f'zz_{nome}.txt'),'w').write(' '.join(f'{x:.2f},{y:.2f}' for x,y in lerp(p))+'\n')
