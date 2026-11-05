#!/usr/bin/env python3
"""Régua igual nas duas apps: traço simples y=120, colunas 150–430 e 570–850 (fora do traço vertical).
s = fundo − G. fundo = mediana das linhas 10–30 (papel limpo). Largura a 50 % do miolo. Borda = pico nos 25 % de cada lado ÷ miolo.
faixa = px contíguos acima de miolo×1,03 · vale = mínimo interior junto da orla ÷ miolo · subida = 10→90 % em px · degrau = maior salto/px do perfil médio ÷ miolo
degrau_loc = p95 do salto vertical por píxel na borda (coluna a coluna), ÷ miolo (dureza local: pixelado) · mancha = CV a 9 px no miolo · riscas = CV do perfil no miolo · contorno = desvio/passo mediano do contorno a 50 %."""
import sys, json
import numpy as np
from PIL import Image
def caixa(a,n):
    p=np.pad(a,n//2,mode="edge"); c=np.cumsum(np.cumsum(np.pad(p,((1,0),(1,0))),0),1)
    return (c[n:,n:]-c[:-n,n:]-c[n:,:-n]+c[:-n,:-n])/(n*n)
def mede(arq):
    a=np.asarray(Image.open(arq).convert("RGB"),float)[:,:,1]
    x=np.r_[150:431,570:851]
    fundo=np.median(a[10:31][:,x])
    j=a[10:226][:,x]; s=np.clip(fundo-j,None,None)
    perfil=s.mean(1); top=np.percentile(perfil,98)
    d=np.where(perfil>0.5*top)[0]; i0,i1=d[0],d[-1]
    q=max(2,(i1-i0+1)//4); c0,c1=i0+q,i1-q
    miolo=perfil[c0:c1+1].mean()
    d=np.where(perfil>0.5*miolo)[0]; i0,i1=d[0],d[-1]; larg=i1-i0+1; q=max(2,larg//4); c0,c1=i0+q,i1-q
    miolo=perfil[c0:c1+1].mean()
    lim=miolo*1.03
    def lado(p,e):  # e = índice do 50 % andando para dentro
        pk=p[e:e+q+1]; ip=e+int(pk.argmax()); f=int((pk>lim).sum())
        v=p[ip:e+q+1].min()/miolo
        # subida
        k=e
        while k>0 and p[k]>0.1*miolo:k-=1
        m=e
        while m<len(p)-1 and p[m]<0.9*miolo:m+=1
        return pk.max()/miolo,f,v,m-k
    A=lado(perfil,i0); B=lado(perfil[::-1],len(perfil)-1-i1)
    grad=np.abs(np.diff(s,axis=0))
    # degrau local na borda (±6 px do contorno médio)
    zona=np.r_[max(i0-6,0):i0+7,i1-6:min(i1+7,grad.shape[0])]
    dl=np.percentile(grad[zona],95)/miolo
    deg=np.abs(np.diff(perfil))[max(0,i0-8):i1+8].max()/miolo
    f=s[c0:c1+1]; b=caixa(f,9)
    sb=caixa(s,3); cont=[]
    for c in range(sb.shape[1]):
        col=sb[:,c]; k=np.argmax(col[max(i0-20,0):]>0.5*miolo)+max(i0-20,0)
        if k>0: cont.append(k-1+(0.5*miolo-col[k-1])/max(col[k]-col[k-1],1e-6))
    cont=np.array(cont)
    return dict(larg=int(larg),miolo=round(miolo,1),borda=round(max(A[0],B[0]),2),faixa=(A[1]+B[1])/2,vale=round(min(A[2],B[2]),2),subida=(A[3]+B[3])/2,
      degrau=round(deg,2),deg_loc=round(dl,2),mancha=round(b.std()/b.mean(),3),riscas=round(perfil[c0:c1+1].std()/miolo,3),cont_std=round(cont.std(),2),cont_passo=round(float(np.median(np.abs(np.diff(cont)))),2)),perfil,i0,i1,miolo
if __name__=="__main__":
    h=None
    for arq in sys.argv[1:]:
        r=mede(arq)[0]
        if not h:
            h=list(r);print("imagem".ljust(30),*[c.rjust(9) for c in h])
        print(arq.split("/")[-1][:30].ljust(30),*[str(r[c]).rjust(9) for c in h])
