import sys,numpy as np
from PIL import Image
exec(open('mede.py').read().split('def mede')[0])
tr=[[tuple(map(float,p.split(','))) for p in l.split()] for l in open('cena/tracos.txt') if l.strip()]
def erro(f):
    g=nd.gaussian_filter(carrega(f).mean(-1),1.0); gy,gx=np.gradient(g)
    res={}
    for k in range(3):
        pts=np.array(tr[k]); errs=[]
        for i in range(3,len(pts)-3,4):
            t=pts[i+3]-pts[i-3]; tang=np.degrees(np.arctan2(t[1],t[0]))%180
            x,y=pts[i].astype(int); sl=(slice(y-14,y+14),slice(x-14,x+14))
            jxx=(gx[sl]**2).sum(); jyy=(gy[sl]**2).sum(); jxy=(gx[sl]*gy[sl]).sum()
            if jxx+jyy<1e3: continue
            o=(0.5*np.degrees(np.arctan2(2*jxy,jxx-jyy))+90)%180
            d=abs(o-tang)%180; errs.append(min(d,180-d))
        res[['reta','onda','zigue'][k]]=round(float(np.median(errs)),1)
    return res
for f in sys.argv[1:]: print(f,'|orientação − tangente| mediana (graus):',erro(f))
