import sys
from PIL import Image, ImageDraw
import numpy as np
S,M=sys.argv[1],sys.argv[2]
csp=Image.open(f'{S}/csp_canvas.png').convert('RGB')
CELL={'round_mixing':(10,10),'gouache':(440,10),'oil_paint':(10,300),'paint_and_apply':(440,300)}
W,H=420,280
def ph(n):
    a=np.frombuffer(open(f'{S}/out/{n}.rgba','rb').read(),np.uint8).reshape(H,W,4).astype(float)
    al=a[...,3:4]/255; return (a[...,:3]*al+255*(1-al)).round().astype(np.uint8)
def mede(img):
    a=np.asarray(img).astype(int)
    base=np.asarray(Image.open(f'{S}/base_csp.png').convert('RGB').crop((10,10,10+W,10+H))).astype(int)
    pint=(np.abs(a-base).max(-1)>12)
    # interior: pintado e com os 4 vizinhos pintados (a 3 px)
    inter=pint.copy()
    for dy,dx in [(3,0),(-3,0),(0,3),(0,-3)]: inter&=np.roll(np.roll(pint,dy,0),dx,1)
    dx=np.abs(np.diff(a,axis=1)).max(-1); dy=np.abs(np.diff(a,axis=0)).max(-1)
    gx=dx[inter[:,1:]&inter[:,:-1]]; gy=dy[inter[1:,:]&inter[:-1,:]]
    g=np.concatenate([gx,gy])
    p99=np.percentile(g,99); p999=np.percentile(g,99.9); frac=(g>25).mean()*100
    def med(x0,x1,y0,y1): r=a[y0:y1,x0:x1].reshape(-1,3); return tuple(int(v) for v in np.median(r,0))
    pats={'y/azul(40-60,60-90)':med(40,60,60,90),'y/amar(160-175,60-90)':med(160,175,60,90),'b/amar(235-255,60-90)':med(235,255,60,90),'b/verm(295-315,60-90)':med(295,315,60,90),'r/amar(190-230,150-190)':med(190,230,150,190),'g/branco(40-120,232-246)':med(40,120,232,246)}
    return p99,p999,frac,pats
rows=[]
out=open(f'{S}/medidas.txt','w')
for k,(cx,cy) in CELL.items():
    c=csp.crop((cx,cy,cx+W,cy+H)); p0=Image.fromarray(ph(f'wb_{k}_p0')); p1=Image.fromarray(ph(f'wb_{k}_p1'))
    row=Image.new('RGB',(W*3+20,H+18),(40,40,40)); d=ImageDraw.Draw(row)
    for i,(im,t) in enumerate([(c,'CSP '+k),(p0,'PH2D Wet blend RGB'),(p1,'PH2D Wet blend Pigment 1')]):
        row.paste(im,(i*(W+10),18)); d.text((i*(W+10)+4,3),t,fill=(230,230,230))
    rows.append(row)
    for t,im in [('CSP',c),('RGB',p0),('PIG',p1)]:
        m=mede(im); out.write(f'{k:16s} {t:4s} p99={m[0]:.0f} p99.9={m[1]:.0f} >25={m[2]:.2f}% {m[3]}\n')
sheet=Image.new('RGB',(rows[0].width,sum(r.height for r in rows)),(40,40,40))
y=0
for r in rows: sheet.paste(r,(0,y)); y+=r.height
sheet.save(f'{M}/wetblend_vs_csp_ziguezague.png')
for i,r in enumerate(rows): r.save(f'{S}/linha{i}.png')
# zoom 3x de uma fronteira (amarelo/azul topo) para o pixelado
z=[]
for k,(cx,cy) in CELL.items():
    c=csp.crop((cx+80,cy+20,cx+200,cy+120)).resize((360,300),Image.NEAREST)
    p=Image.fromarray(ph(f'wb_{k}_p0')).crop((80,20,200,120)).resize((360,300),Image.NEAREST)
    r=Image.new('RGB',(730,300),(40,40,40)); r.paste(c,(0,0)); r.paste(p,(370,0)); z.append(r)
zs=Image.new('RGB',(730,1200)); [zs.paste(r,(0,300*i)) for i,r in enumerate(z)]
zs.save(f'{M}/wetblend_vs_csp_zoom3x.png')
