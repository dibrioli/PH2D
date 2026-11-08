import sys, numpy as np
from PIL import Image

class nd:
    @staticmethod
    def gaussian_filter(g,s):
        k=np.array([1,4,6,4,1],float);k/=k.sum()
        g=np.apply_along_axis(lambda r:np.convolve(r,k,'same'),0,g)
        return np.apply_along_axis(lambda r:np.convolve(r,k,'same'),1,g)
    @staticmethod
    def label(m):
        H,W=m.shape; lab=np.zeros((H,W),int); n=0
        for y0,x0 in zip(*np.nonzero(m)):
            if lab[y0,x0]: continue
            n+=1; st=[(y0,x0)]; lab[y0,x0]=n
            while st:
                y,x=st.pop()
                for dy,dx in ((1,0),(-1,0),(0,1),(0,-1)):
                    yy,xx=y+dy,x+dx
                    if 0<=yy<H and 0<=xx<W and m[yy,xx] and not lab[yy,xx]:
                        lab[yy,xx]=n; st.append((yy,xx))
        return lab,n
    @staticmethod
    def sum(m,lab,r):
        return np.bincount(lab.ravel(),minlength=max(r)+1)[1:]
import colorsys
def carrega(f):
    return np.array(Image.open(f).convert('RGB')).astype(float)
def mede(nome, im):
    d = (255-im).max(-1)
    m = d>12
    out={}
    # orientação dominante (tensor de estrutura) na reta (y 60..170, x 200..600)
    for reg,(ys,xs) in {'reta':(slice(50,175),slice(200,600))}.items():
        g = im.mean(-1)[ys,xs]
        gy,gx = np.gradient(nd.gaussian_filter(g,1.0))
        jxx=(gx*gx).sum(); jyy=(gy*gy).sum(); jxy=(gx*gy).sum()
        ang=0.5*np.degrees(np.arctan2(2*jxy,jxx-jyy))  # direção do gradiente
        coh=np.hypot(jxx-jyy,2*jxy)/(jxx+jyy)
        out['orient_traco_deg']=round((ang+90)%180,1); out['coerencia']=round(coh,2)
    # componentes isolados (pontos soltos) em todo o quadro
    lab,n=nd.label(m)
    areas=nd.sum(m,lab,range(1,n+1))
    stats=[]
    for i,a in enumerate(areas,1):
        if 30<=a<=1200:
            ys,xs=np.nonzero(lab==i)
            c=np.cov(np.vstack([xs,ys]))
            ev,evec=np.linalg.eigh(c)
            L=4*np.sqrt(ev[1]); W=4*np.sqrt(max(ev[0],1e-6))
            ang=np.degrees(np.arctan2(evec[1,1],evec[0,1]))%180
            pix=im[ys,xs]
            stats.append((a,L,W,ang,pix))
    st=np.array([s[:4] for s in stats])
    out['n_pontos_soltos']=len(stats)
    if len(stats):
        out['comp_mediana']=round(np.median(st[:,1]),1); out['larg_mediana']=round(np.median(st[:,2]),1)
        out['aspecto']=round(np.median(st[:,2]/st[:,1]),2)
        out['comp_p10_p90']=[round(x,1) for x in np.percentile(st[:,1],[10,90])]
        out['ang_mediana']=round(np.median(st[:,3]),1)
        # opacidade do ponto: quão longe do branco está o interior (mediana d / dmax global)
        dint=[np.median((255-s[4]).max(-1)) for s in stats]
        out['ponto_d_interior_p10_p50_p90']=[round(x) for x in np.percentile(dint,[10,50,90])]
        # uniformidade interna (desvio de cor dentro de um ponto)
        out['desvio_dentro_ponto']=round(np.median([s[4].std(0).mean() for s in stats]),1)
        # variação de cor ENTRE pontos (cor mediana de cada ponto)
        cs=np.array([np.median(s[4],0) for s in stats])/255
        hsv=np.array([colorsys.rgb_to_hsv(*c) for c in cs])
        out['entre_pontos_h_std_deg']=round(np.degrees(np.std(np.unwrap(hsv[:,0]*2*np.pi))) ,1)
        out['entre_pontos_s_std']=round(hsv[:,1].std(),3); out['entre_pontos_v_std']=round(hsv[:,2].std(),3)
    # no miolo (pintado): distribuição de d (opacidade aparente) e de cor
    pix=im[m]; dd=(255-pix).max(-1)
    out['miolo_d_p10_p50_p90']=[round(x) for x in np.percentile(dd,[10,50,90])]
    hs=np.array([colorsys.rgb_to_hsv(*c) for c in pix[::7]/255])
    out['miolo_h_p10_p90_deg']=[round(x*360) for x in np.percentile(hs[:,0],[10,90])]
    out['miolo_cor_media']=pix.mean(0).round().astype(int).tolist()
    # aglomeração: fração de buracos dentro do envelope da reta
    band=m[60:165,200:600]; out['cobertura_reta']=round(band.mean(),3)
    print(nome, out)
for f in sys.argv[1:]:
    mede(f, carrega(f))
def orient_local(f):
    im=carrega(f); g=im.mean(-1)[50:175,200:600]
    gy,gx=np.gradient(nd.gaussian_filter(g,1.0)); mag=np.hypot(gx,gy)
    a=(np.degrees(np.arctan2(gy,gx))+90)%180; w=mag>8
    a2=np.radians(a[w]*2); R=np.hypot(np.cos(a2).mean(),np.sin(a2).mean())
    return round(np.degrees(np.sqrt(-2*np.log(R))/2),1)
