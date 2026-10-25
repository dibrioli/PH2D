"""Tabela de concordância (% de pixels iguais) por dither, candidato a candidato, sobre todos os casos Default(Octree)+RGB."""
import sys; sys.path.insert(0,'/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/c142c8eb-76c4-4200-a92d-b3bc73e244a0/scratchpad/oraculo/tools')
from trial_fit_dither import *
from trial_fs import fsv
from trial_ordD import ordD
CANDS={
 0:{'nearest':lambda im,P,n: nn(P,im).ravel()},
 1:None,2:None,3:None,4:None,5:None,6:None,
 7:{'fs serp int-trunc':lambda im,P,n: fsv(im,P,'each_trunc',1),'fs serp float':lambda im,P,n: fsv(im,P,'float',1),
    'fs ltr float':lambda im,P,n: fs(im,P,False,False)}}
ORD={'A S=64 tr':lambda im,P,n: ordA(im,P,n,64,tr=True),'A S=255/(k-1) tr':lambda im,P,n: ordA(im,P,n,255/(len(P)-1),tr=True),
     'D luma709 c2=near(c+e) off1.5':lambda im,P,n: ordD(im,P,n,'1','709'),
     'old ratio c2=near(c+e)':lambda im,P,n: old(im,P,n,1.0,'ratio'),'old ratio tr':lambda im,P,n: old(im,P,n,1.0,'ratio',True),
     'knoll m=0.5':lambda im,P,n: knoll(im,P,n,0.5,False)}
for d in range(1,7): CANDS[d]=ORD
tot={}
for path in sys.argv[1:]:
    for r in records(path):
        if r['best_fit']!=1 or r['mapping']!=0: continue
        d=r['dither']; img=IMGS[r['image']].astype(float); P=np.array(PALS[r['palette']],float); got=np.array(r['indices'])
        n=NS.get(d,0)
        for k,f in CANDS[d].items():
            ok=int((f(img,P,n)==got).sum()); a=tot.setdefault((d,k),[0,0]); a[0]+=ok; a[1]+=len(got)
            if d in (0,7): print(r['image'],r['palette'],d,k,len(got)-ok,'wrong',flush=True)
DN=['none','ordered2','ordered4','ordered8','old2','old4','old8','floyd']
for (d,k),(ok,n) in sorted(tot.items()):
    print(f'{DN[d]:9s} {k:32s} {100*ok/n:6.2f}%  ({n-ok} wrong / {n})')
