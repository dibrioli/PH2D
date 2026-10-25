import sys, itertools; sys.path.insert(0,'/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/c142c8eb-76c4-4200-a92d-b3bc73e244a0/scratchpad/oraculo/tools')
from trial_fit_dither import *
def fsv(img,P,mode,firstdir=1,ytie=0):
    """serpentina, valor somado ao erro e cortado a [0,255], erro = cortado - paleta, pesos 7/3/5/1."""
    H,W,_=img.shape; E=np.zeros((H+1,W+2,3)); out=np.zeros((H,W),int)
    for y in range(H):
        rev=(y%2==1) if firstdir==1 else (y%2==0)
        xs=range(W-1,-1,-1) if rev else range(W); s=-1 if rev else 1
        for x in xs:
            acc=E[y,x+1]
            if mode=='float': v=img[y,x]+acc
            elif mode=='acc_div_late': v=img[y,x]+np.trunc(acc/16)
            elif mode=='acc_div_late_floor': v=img[y,x]+np.floor(acc/16)
            elif mode=='acc_div_late_round': v=img[y,x]+np.round(acc/16)
            elif mode=='each_trunc': v=img[y,x]+acc
            elif mode=='each_floor': v=img[y,x]+acc
            vc=np.clip(v,0,255)
            i=int(nn(P,vc[None])[0]); out[y,x]=i; e=vc-P[i]
            for (dx,dy),w in zip(((s,0),(-s,1),(0,1),(s,1)),(7,3,5,1)):
                if mode=='float': d=e*w/16
                elif mode.startswith('acc_div_late'): d=e*w
                elif mode=='each_trunc': d=np.trunc(e*w/16)
                else: d=np.floor(e*w/16)
                E[y+dy,x+dx+1]+=d
    return out.ravel()
MODES=('float','acc_div_late','acc_div_late_floor','acc_div_late_round','each_trunc','each_floor')
if __name__=='__main__':
    for r in records(sys.argv[1]):
        if r['dither']!=7 or r['best_fit']!=1: continue
        img=IMGS[r['image']].astype(float); P=np.array(PALS[r['palette']],float); got=np.array(r['indices'])
        res=sorted((int((fsv(img,P,m,fd)!=got).sum()),m,fd) for m in MODES for fd in (1,0))
        print(r['image'],r['palette'],len(got),res[:4])
