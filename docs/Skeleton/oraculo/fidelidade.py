# A SEGUNDA REGUA: a arte ainda SEGUE os ossos?
# Uma cura que torne a pele quase rigida tem dobra zero por nao deformar nada.
import json, math
d = json.load(open('/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/288048cc-7a6f-40b2-8ea6-37cc673393d2/scratchpad/pesos_blender.json'))
V, F, ARCO = d["verts"], d["faces"], d["arco"]
L = ARCO/2
def dist2_seg(p,a,b):
    vx,vy=b[0]-a[0],b[1]-a[1]; wx,wy=p[0]-a[0],p[1]-a[1]
    n2=vx*vx+vy*vy; t=0.0 if n2<=0 else max(0.0,min(1.0,(wx*vx+wy*vy)/n2))
    dx,dy=wx-t*vx,wy-t*vy; return dx*dx+dy*dy
def pesos(p,r):
    w=[0.0,0.0]; s=0.0; perto=0; pd=1e30
    for i,(a,b) in enumerate([([0,0],[L,0]),([L,0],[2*L,0])]):
        d2=dist2_seg(p,a,b)
        if d2<pd: perto,pd=i,d2
        x2=d2/(r*r); v=(1-x2)**2 if x2<1 else 0.0; w[i]=v; s+=v
    return [v/s for v in w] if s>0 else [1.0 if i==perto else 0.0 for i in range(2)]
def poses(t):
    ms,ox,oy,fi=[],0.0,0.0,0.0
    for i in range(2):
        if i>0: fi+=t
        c,s=math.cos(fi),math.sin(fi)
        ms.append((c,s,-i*L,ox,oy)); ox+=L*c; oy+=L*s
    return ms
def aplica(p,ms,w):
    x=y=0.0
    for (c,s,dx,ox,oy),wi in zip(ms,w):
        if wi==0.0: continue
        px,py=p[0]+dx,p[1]
        x+=wi*(px*c-py*s+ox); y+=wi*(px*s+py*c+oy)
    return (x,y)
def mede(r, total):
    ms=poses(total); P=[aplica(v,ms,pesos(v,r)) for v in V]
    # dobra
    inv=tot=0.0
    for f in F:
        for tri in ((f[0],f[1],f[2]),(f[0],f[2],f[3])):
            a,b,c=(P[t] for t in tri); ra,rb,rc=(V[t] for t in tri)
            s=(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
            s0=(rb[0]-ra[0])*(rc[1]-ra[1])-(rb[1]-ra[1])*(rc[0]-ra[0])
            tot+=abs(s0)
            if s*s0<=0: inv+=abs(s0)
    # FIDELIDADE: a arte sobre o 2.º osso roda o quanto o 2.º osso rodou?
    # amostra os vertices que projectam na metade final do 2.º osso
    angs=[]
    for k,v in enumerate(V):
        if v[0] < L*1.5: continue
        # o vector do ponto a' ponta do osso, no repouso e depois
        r0=(v[0]-2*L, v[1])
        pt=aplica((2*L,0.0),ms,pesos((2*L,0.0),r))
        r1=(P[k][0]-pt[0], P[k][1]-pt[1])
        if r0[0]**2+r0[1]**2 < 1e-9: continue
        a0=math.atan2(r0[1],r0[0]); a1=math.atan2(r1[1],r1[0])
        da=(a1-a0+math.pi)%(2*math.pi)-math.pi
        angs.append(da)
    angs.sort()
    med=angs[len(angs)//2] if angs else 0.0
    return 100*inv/tot, math.degrees(med)
print(f"{'alcance':>28} | {'dobra 150°':>12} | {'a arte segue o osso?':>22}")
print("-"*72)
MEIA=2.4
for m,nome in [(L, 'osso (hoje)'), (MEIA*0.8,'0,80x arte'), (MEIA*1.0,'1,00x arte'),
               (MEIA*1.25,'1,25x arte'), (MEIA*1.67,'1,67x arte'), (MEIA*2.08,'2,08x arte')]:
    inv,ang = mede(m, math.radians(150))
    print(f"{nome:>20} r={m:5.2f} | {inv:9.3f}%  | rodou {ang:6.1f}° de 150°")
