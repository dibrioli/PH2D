"""Candidato C para Ordered/Old: par (c1 = mais próxima, c2), fracção s ao longo de c1->c2,
contagem minoritária m = #{M : (M+off)/N < s}; a cor MAIS CLARA ocupa os M mais baixos (Bayer transposta)."""
import sys, itertools; sys.path.insert(0,'/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/c142c8eb-76c4-4200-a92d-b3bc73e244a0/scratchpad/oraculo/tools')
from trial_fit_dither import *
def L(c): return 0.2126*c[0]+0.7152*c[1]+0.0722*c[2]
def ordC(img,P,n,c2rule,smode,off,light,tr=True,place='light'):
    M=bayer(n); N=n*n; H,W,_=img.shape; out=np.zeros((H,W),int); cache={}
    for y in range(H):
        for x in range(W):
            c=img[y,x]; key=tuple(c)
            if key not in cache:
                i=int(nn(P,c[None])[0]); e=c-P[i]
                if c2rule=='second':
                    Q=P.copy(); Q[i]=1e9; j=int(nn(Q,c[None])[0])
                else:
                    k=float(c2rule); j=int(nn(P,np.clip(c+k*e,0,255)[None])[0])
                if j==i: cache[key]=(i,i,0.0)
                else:
                    v=P[j]-P[i]
                    if smode=='proj': s=(e@v)/(v@v)
                    elif smode=='ratio': s=np.sqrt(e@e)/np.sqrt(v@v)
                    elif smode=='proj256': s=(e@v)/(v@v)*255/256
                    s=max(0.0,min(1.0,s))
                    cache[key]=(i,j,s)
            i,j,s=cache[key]
            m=(M[x%n,y%n] if tr else M[y%n,x%n])
            cnt=sum(1 for t in range(N) if (t+off)/N < s)   # minoritárias
            if i==j or cnt==0: out[y,x]=i; continue
            if place=='light':
                li = (light(P[j]) > light(P[i]))   # c2 é a mais clara?
                if li: out[y,x]= j if m < cnt else i
                else: out[y,x]= i if m < N-cnt else j
            else:
                out[y,x]= j if m < cnt else i
    return out.ravel()
LIGHT={'luma':L,'sum':lambda c:c.sum()}
if __name__=='__main__':
    want=[int(a) for a in sys.argv[2:]]
    for r in records(sys.argv[1]):
        if r['best_fit']!=1 or r['dither'] not in want: continue
        img=IMGS[r['image']].astype(float); P=np.array(PALS[r['palette']],float); got=np.array(r['indices']); n=NS[r['dither']]
        res=[]
        for c2r,sm,off,lt,pl in itertools.product(('1','2','second'),('proj','ratio'),(1.0,1.5,0.5),('luma',),('light','c2')):
            res.append((int((ordC(img,P,n,c2r,sm,off,LIGHT[lt],True,pl)!=got).sum()),c2r,sm,off,pl))
        res.sort(); print(r['image'],r['palette'],r['dither'],len(got),res[:4],flush=True)
