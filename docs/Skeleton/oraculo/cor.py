# CENTROS DE ROTACAO OPTIMIZADOS (Le & Hodgins 2016), em 2D — o candidato que
# ataca a MISTURA em vez dos pesos, logo nao mexe na localidade (a fidelidade).
#   q_v = R(theta_v) * (v - c_v) + LBS(c_v)
# com c_v = integral de p ponderada pela SEMELHANCA dos vectores de peso.
import json, math
d = json.load(open('/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/288048cc-7a6f-40b2-8ea6-37cc673393d2/scratchpad/pesos_blender.json'))
V, F, ARCO, N = d["verts"], d["faces"], d["arco"], d["n"]
L = ARCO/2
SIGMA = 0.1
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
    ms,ox,oy,fi,angs=[],0.0,0.0,0.0,[]
    for i in range(2):
        if i>0: fi+=t
        c,s=math.cos(fi),math.sin(fi)
        ms.append((c,s,-i*L,ox,oy)); angs.append(fi); ox+=L*c; oy+=L*s
    return ms,angs
def lbs(p,ms,w):
    x=y=0.0
    for (c,s,dx,ox,oy),wi in zip(ms,w):
        if wi==0.0: continue
        px,py=p[0]+dx,p[1]
        x+=wi*(px*c-py*s+ox); y+=wi*(px*s+py*c+oy)
    return (x,y)
def semelhanca(wp, wv):
    # Le & Hodgins eq. 1, para todos os pares j<k
    s=0.0
    for j in range(2):
        for k in range(j+1,2):
            t=(wp[j]*wv[k]-wp[k]*wv[j])
            s += wp[j]*wp[k]*wv[j]*wv[k]*math.exp(-(t*t)/(SIGMA*SIGMA))
    return s
def centros(W, passo=3):
    """Precomputo: o centro de rotacao de cada vertice. O dominio da integral e' subamostrado."""
    dom=[(V[k], W[k]) for k in range(0,len(V),passo)]
    out=[]
    for wv in W:
        sx=sy=ss=0.0
        for (p,wp) in dom:
            s=semelhanca(wp,wv)
            if s>0: sx+=s*p[0]; sy+=s*p[1]; ss+=s
        out.append((sx/ss, sy/ss) if ss>1e-12 else None)
    return out
def mede(r, total, usar_cor):
    ms,angs = poses(total)
    W=[pesos(v,r) for v in V]
    C=centros(W) if usar_cor else None
    P=[]
    for k,v in enumerate(V):
        if usar_cor and C[k] is not None:
            th = W[k][0]*angs[0] + W[k][1]*angs[1]     # a rotacao misturada (em 2D, o angulo)
            c  = C[k]
            cx, cy = lbs(c, ms, W[k])                   # o centro, levado pela mistura linear
            dx, dy = v[0]-c[0], v[1]-c[1]
            co,si = math.cos(th), math.sin(th)
            P.append((cx + dx*co - dy*si, cy + dx*si + dy*co))
        else:
            P.append(lbs(v,ms,W[k]))
    inv=tot=0.0
    for f in F:
        for tri in ((f[0],f[1],f[2]),(f[0],f[2],f[3])):
            a,b,c=(P[t] for t in tri); ra,rb,rc=(V[t] for t in tri)
            s=(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
            s0=(rb[0]-ra[0])*(rc[1]-ra[1])-(rb[1]-ra[1])*(rc[0]-ra[0])
            tot+=abs(s0)
            if s*s0<=0: inv+=abs(s0)
    angs2=[]
    ponta=(2*L,0.0)
    pt = P[min(range(len(V)), key=lambda k:(V[k][0]-ponta[0])**2+(V[k][1]-ponta[1])**2)]
    for k,v in enumerate(V):
        if v[0] < L*1.5: continue
        r0=(v[0]-2*L, v[1]); r1=(P[k][0]-pt[0], P[k][1]-pt[1])
        if r0[0]**2+r0[1]**2 < 1e-9: continue
        a0=math.atan2(r0[1],r0[0]); a1=math.atan2(r1[1],r1[0])
        angs2.append((a1-a0+math.pi)%(2*math.pi)-math.pi)
    angs2.sort()
    return 100*inv/tot, math.degrees(angs2[len(angs2)//2]) if angs2 else 0.0
print(f"{'lei':>34} | {'dobra 150°':>11} | {'segue o osso':>13}")
print("-"*66)
for r,nome,cor in [(L,'HOJE: mistura linear, raio=osso',False),
                   (2.4*2.08,'alcance largo (a cura recusada)',False),
                   (L,'CENTROS DE ROTACAO, raio=osso',True)]:
    inv,ang = mede(r, math.radians(150), cor)
    print(f"{nome:>34} | {inv:9.3f}%  | {ang:8.1f}° de 150")
