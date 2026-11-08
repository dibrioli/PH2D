import sys,numpy as np,json,os
G=np.array([0,160,60],np.float32)/255
base=np.fromfile(os.path.join(os.path.dirname(__file__),'cena/base.rgba'),np.uint8).reshape(880,2400,4).astype(np.float32)/255
def mede(p):
    o=np.fromfile(p,np.uint8).reshape(880,2400,4).astype(np.float32)/255
    def comp(img,y,xs):  # cor composta sobre a base (a base já é opaca nos blocos/branco)
        return img[y,xs,:3]
    y=110
    # 1. blocos: parte do pincel r (0 = base, 1 = pincel)
    xs=np.arange(1720,2281); oc=o[y,xs,:3]; bc=base[y,xs,:3]
    dob=np.linalg.norm(oc-bc,axis=1); dog=np.linalg.norm(oc-G,axis=1)
    r=dob/(dob+dog+1e-6)
    M=float(((r>0.2)&(r<0.8)).mean())
    # contraste entre azul e vermelho sob o traço (0 = chapou uma cor, 1 = intocado)
    cb=o[y,1760:1840,:3].mean(0); cr=o[y,2200:2280,:3].mean(0); cy=o[y,2040:2120,:3].mean(0)
    C=float(np.linalg.norm(cb-cr)/np.linalg.norm(base[y,1800,:3]-base[y,2240,:3]))
    # largura da transição azul→amarelo (fronteira 1866): progresso 10%→90%
    xt=np.arange(1846,2120); pt=np.linalg.norm(o[y,xt,:3]-cb,axis=1)/(np.linalg.norm(cy-cb)+1e-6)
    try: W=int(xt[np.argmax(pt>0.9)]-xt[np.argmax(pt>0.1)])
    except: W=-1
    step=float(np.abs(np.diff(oc,axis=0)).max()*255)
    # 2. branco: parte do pincel
    ow=o[y,950:1450,:3]; rw=float((np.linalg.norm(ow-1,axis=1)/np.linalg.norm(G-1)).mean())
    # 3. vazio: alfa e cor
    av=float(o[y,150:650,3].mean()); dv=float(np.linalg.norm(o[y,150:650,:3]-G,axis=1).mean()*255) if av>0.05 else -1
    # 4. orla: rampa de alfa por lado, na coluna x=400 (vazio)
    col=o[60:160,400,3]; rampa=float(((col>0.1)&(col<0.9)).sum()/2)
    # serrilhado: nº de níveis distintos de alfa na orla (1-2 = degrau)
    return dict(M=round(M,2),C=round(C,2),W=W,step=round(step,1),branco=round(rw,2),vazio_a=round(av,2),vazio_dcor=round(dv,1),orla_px=rampa)
if __name__=='__main__':
    for p in sys.argv[1:]: print(os.path.basename(p), json.dumps(mede(p)))
