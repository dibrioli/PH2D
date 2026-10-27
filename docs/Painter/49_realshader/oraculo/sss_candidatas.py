"""Candidatas M1 (W3): a lei da Subsurface sobre a irradiância do PRÓPRIO Cycles (studio_sol10_cinza), contra o
Cycles com Subsurface (base sss001_r4). python3 sss_candidatas.py <entrada> <saida_cycles>. Modos: cons, fill, fillm,
fillz, perda, perda2 (README §M-SSS)."""
import sys, numpy as np, OpenImageIO as oiio
def gaussian_filter(a,s,mode=None):
    if s<=0: return a
    R=int(3*s+1); x=np.arange(-R,R+1); k=np.exp(-x*x/(2*s*s)); k/=k.sum()
    p=np.pad(a,R,mode='edge')
    t=sum(k[i]*p[:,i:i+a.shape[1]+0] for i in range(2*R+1))[R:-R] if False else None
    p1=np.pad(a,((0,0),(R,R)),mode='edge'); a1=sum(k[i]*p1[:,i:i+a.shape[1]] for i in range(2*R+1))
    p2=np.pad(a1,((R,R),(0,0)),mode='edge'); return sum(k[i]*p2[i:i+a.shape[0],:] for i in range(2*R+1))
E, C = sys.argv[1], sys.argv[2]
def pnm(p):
    b=open(p,'rb').read(); s=b.split(maxsplit=4); w,h=int(s[1]),int(s[2])
    return np.frombuffer(s[4],np.uint8).reshape(h,w,-1).squeeze().astype(float)
def lin(s):
    s=s/255; return np.where(s<=0.04045,s/12.92,((s+0.055)/1.055)**2.4)
ex=lambda n: oiio.ImageBuf(f"{C}/{n}.exr").get_pixels(oiio.FLOAT)[...,:3].astype(float)
L=np.array([.2126,.7152,.0722])
cob=pnm(f"{E}/cobertura.pgm")/255; gh,gw=cob.shape
h=np.fromfile(f"{E}/h_fina.f32","<f4").reshape(gh,gw).astype(float)
alb=lin(pnm(f"{E}/albedo.ppm"))
n=ex("mat_sss001_r4_cor")
Et=ex("studio_sol10_cinza").mean(-1)/0.5
tinta=cob>0.9
sat=lambda c:(c.max(-1)-c.min(-1))/np.maximum(c.max(-1),1e-6)
cyc={m:ex(f"mat_{m}_cor") for m in ["sss1_r1","sss1_r4","sss1_r16"]}
masks={"fina":tinta&(h<2),"h2-8":tinta&(h>=2)&(h<8),"sombra":tinta&(Et<0.5),"lit":tinta&(Et>0.9),"tinta":tinta,"borda":(cob>0.1)&(cob<0.9)}
RAD=np.array([1,.2,.1])
def law(w,r,k,modo,thin):
    out=n.copy()
    for c in range(3):
        rc=r*RAD[c]
        b=gaussian_filter(Et,k*rc,mode='nearest')
        d=b-Et
        if modo=="fill": d=np.maximum(d,0)
        if modo=="fillm": d=np.maximum(gaussian_filter(Et*cob,k*rc)/np.maximum(gaussian_filter(cob,k*rc),1e-6)-Et,0)
        if modo=="fillz": d=np.maximum(gaussian_filter(Et*cob,k*rc)-Et,0)
        if modo=="perda": d=np.maximum(d,0)-Et*(1-gaussian_filter(cob,k*rc))
        if modo=="perda2": d=np.maximum(gaussian_filter(Et*cob,k*rc)-Et,0)-Et*(1-gaussian_filter(cob,k*rc))
        D=w*d
        if thin is not None:
            g,p=thin
            T=np.exp(-np.maximum(h,0)/(p*rc))
            D=D+w*g*T
        out[...,c]=n[...,c]+alb[...,c]*D
    return out
def score(m,pred):
    cy=cyc[m]; s=[]
    for c in range(3):
        a=(pred-n)[...,c][tinta]; b=(cy-n)[...,c][tinta]
        s.append(np.corrcoef(a,b)[0,1])
    rms=np.sqrt(np.mean(((pred-cy)[tinta])**2))
    rms0=np.sqrt(np.mean(((n-cy)[tinta])**2))
    parts=" ".join(f"{k}:{(pred-n)[...,0][v].mean():+.3f}/{(cy-n)[...,0][v].mean():+.3f}" for k,v in masks.items())
    return f"corrRGB {s[0]:+.2f} {s[1]:+.2f} {s[2]:+.2f} rms {rms:.4f} (sem {rms0:.4f}) | ΔR nós/cy {parts}"
cands=[(k,m,None) for k in (1.0,1.5) for m in ("fill","fillm","fillz")]
for m in cyc:
    w=1.0; r=float(m.split("_r")[1])
    print("==",m)
    for k,modo,thin in cands:
        print(f"  k{k} {modo:4} {score(m,law(w,r,k,modo,thin))}")
