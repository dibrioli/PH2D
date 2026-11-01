import numpy as np, sys
from PIL import Image
M1=np.array([[0.4122214708,0.5363325363,0.0514459929],[0.2119034982,0.6806995451,0.1073969566],[0.0883024619,0.2817188376,0.6299787005]])
M2=np.array([[0.2104542553,0.7936177850,-0.0040720468],[1.9779984951,-2.4285922050,0.4505937099],[0.0259040371,0.7827717662,-0.8086757660]])
M2I=np.linalg.inv(M2); M1I=np.linalg.inv(M1)
def lin(s): s=np.asarray(s,float)/255; return np.where(s<=0.04045,s/12.92,((s+0.055)/1.055)**2.4)
def srgb(l): l=np.clip(l,0,1); return 255*np.where(l<=0.0031308,12.92*l,1.055*l**(1/2.4)-0.055)
def ok(s): return (M2@np.cbrt(M1@lin(s).T)).T
def okinv(o): return srgb((M1I@((M2I@np.asarray(o).T)**3)).T)
A,B=ok([40,90,220]),ok([245,205,25]); CR=lin([236,226,206])
seg=okinv(np.array([A+(B-A)*t for t in np.linspace(0,1,201)]))
def dist(px):  # opaque pixels
    d=np.linalg.norm(px[:,None,:]-seg[None,:,:],axis=2); return d.min(1), d.argmin(1)/200
# diag points: dry colour from sums
pts={100:([2.431,.518,-.023,-.210],[1.218,.518,-.023,-.210],[28,76,216]),
112:([4.283,.520,-.020,-.200],[1.503,.518,-.021,-.203],[33,78,211]),
124:([2.045,.674,-.018,-.034],[1.003,.557,-.022,-.165],[92,126,175]),
128:([2.582,.759,-.015,.060],[.937,.596,-.020,-.121],[141,153,141]),
132:([3.481,.801,-.014,.107],[.925,.638,-.019,-.075],[176,169,114]),
140:([5.680,.834,-.012,.140],[.973,.710,-.017,.003],[210,183,83]),
154:([6.549,.847,-.013,.157],[.960,.764,-.015,.065],[226,189,61]),
170:([2.212,.850,-.012,.160],[.572,.763,-.015,.063],[223,188,66])}
for y,(f,a,wet) in pts.items():
    v=f[0]+a[0]; lab=(f[0]*np.array(f[1:])+a[0]*np.array(a[1:]))/v
    dry=okinv(lab[None])[0]
    dw,_=dist(np.array([wet],float)); dd,_=dist(dry[None])
    print(f"y{y}: seca(das somas) {np.round(dry).astype(int)} d_seg={dd[0]:.1f} · molhada(px sem óleo) {wet} d_seg={dw[0]:.1f}")
for n in sys.argv[1:]:
    im=np.asarray(Image.open(n).convert('RGBA')).astype(float); a=im[...,3]/255
    m=a>=0.9; px=im[...,:3][m]
    d,t=dist(px[::7])
    print(n.split('/')[-1], 'n',len(d),'p50 %.1f p90 %.1f p99 %.1f max %.1f'%tuple(np.percentile(d,[50,90,99,100])))
    for lo,hi in ((0,.1),(.1,.4),(.4,.6),(.6,.9),(.9,1.01)):
        s=(t>=lo)&(t<hi); 
        if s.any(): print(f'  t∈[{lo},{hi}) n={s.sum()} p50 {np.median(d[s]):.1f} p99 {np.percentile(d[s],99):.1f}')
