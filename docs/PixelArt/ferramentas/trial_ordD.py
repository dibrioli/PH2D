"""Candidato D (Ordered): c1 = mais próxima (RGB); c2 por regra; s = fracção de LUMA de c1 para c2;
minoritárias m = #{M : M + off < N*s}; a cor mais clara (luma) ocupa os M mais baixos da Bayer transposta (M[x%n][y%n])."""
import sys, itertools; sys.path.insert(0,'/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/c142c8eb-76c4-4200-a92d-b3bc73e244a0/scratchpad/oraculo/tools')
from trial_fit_dither import *
LW={'709':np.array([0.2126,0.7152,0.0722]),'601':np.array([0.299,0.587,0.114])}
def ordD(img,P,n,c2rule,lw,off=1.5,gate=None):
    M=bayer(n); N=n*n; H,W,_=img.shape; out=np.zeros((H,W),int); w=LW[lw]; Lp=P@w
    for y in range(H):
        for x in range(W):
            c=img[y,x]; i=int(nn(P,c[None])[0]); e=c-P[i]; Lc=c@w
            if c2rule=='second':
                Q=P.copy(); Q[i]=1e9; j=int(nn(Q,c[None])[0])
            elif c2rule=='lumaside':   # mais próxima entre as do outro lado em luma
                side=np.sign(Lc-Lp[i]); cand=[k for k in range(len(P)) if k!=i and np.sign(Lp[k]-Lp[i])==side and side!=0]
                j=min(cand,key=lambda k:((P[k]-c)**2).sum()) if cand else i
            else:
                j=int(nn(P,np.clip(c+float(c2rule)*e,0,255)[None])[0])
            if j==i or Lp[j]==Lp[i]: out[y,x]=i; continue
            s=max(0.0,min(1.0,(Lc-Lp[i])/(Lp[j]-Lp[i])))
            m=sum(1 for t in range(N) if t+off < N*s)
            mm=M[x%n,y%n]
            if Lp[j]>Lp[i]: out[y,x]= j if mm<m else i
            else: out[y,x]= i if mm<N-m else j
    return out.ravel()
if __name__=='__main__':
    want=[int(a) for a in sys.argv[2:]]
    for r in records(sys.argv[1]):
        if r['best_fit']!=1 or r['dither'] not in want: continue
        img=IMGS[r['image']].astype(float); P=np.array(PALS[r['palette']],float); got=np.array(r['indices']); n=NS[r['dither']]
        res=sorted((int((ordD(img,P,n,c2,lw,off)!=got).sum()),c2,lw,off) for c2 in ('1','2','3','second','lumaside') for lw in ('709','601') for off in (1.5,))
        print(r['image'],r['palette'],r['dither'],len(got),res[:3],flush=True)
