import os, subprocess, sys
import numpy as np
from PIL import Image
D=os.environ.get("VC_OUT","target/vectorcraft")+"/ours"  # rows.tsv = saída da sonda_offset
W=1200; HDR=f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{W}" viewBox="0 0 {W} {W}">'
star=open(f"{D}/star.d").read()
def rast(body,name):
    p=f"{D}/{name}.svg"; open(p,"w").write(HDR+body+"</svg>"); subprocess.run(["resvg",p,p+".png"],check=True)
    return np.asarray(Image.open(p+".png").convert("RGBA"))[:,:,3]>127
rows=[l.rstrip("\n").split("\t") for l in open(f"{D}/rows.tsv")]
res={}
for jn,d,n,us,fr,ds in rows:
    d=float(d); n=int(n); k=(jn,d>0); res.setdefault(k,[])
    rec={"d":d,"n":n,"us":int(us)}
    if d>0 and n>0:
        rule="evenodd" if fr=="EvenOdd" else "nonzero"
        m=rast(f'<path d="{ds}" fill="#000" fill-rule="{rule}"/>',f"o_{jn}_{d:g}")
        rf=rast(f'<path d="{star}" fill="#000" stroke="#000" stroke-width="{2*d}" stroke-linejoin="{jn}" stroke-miterlimit="4"/>',f"r_{jn}_{d:g}")
        rec["diff"]=round(100*(m^rf).sum()/max(rf.sum(),1),3)
    if d<0 and n>0:
        rec["area"]=int(rast(f'<path d="{ds}" fill="#000"/>',f"o_{jn}_{d:g}").sum())
    res[k].append(rec)
for (jn,out),rs in sorted(res.items()):
    ok=[r for r in rs if r["n"]>0]; us=sorted(r["us"] for r in rs)
    if out:
        df=sorted(r["diff"] for r in ok)
        print(f"{jn:6} fora   ok {len(ok)}/100 diff max {df[-1] if df else '-'}% med {df[len(df)//2] if df else '-'}%  por d: {[(r['d'],r.get('diff')) for r in rs if r['d'] in (4,20,40,100,200,400)]}  µs med {us[50]} max {us[-1]}")
    else:
        last=max((-r["d"] for r in ok),default=None); vaz=[-r["d"] for r in rs if r["n"]==0 and -r["d"]<53.76]
        print(f"{jn:6} dentro ok {len(ok)}/100 último d com forma={last} vazios ANTES do raio inscrito(53.76)={vaz} sobras depois={[(-r['d'],r.get('area')) for r in ok if -r['d']>53.76]} µs med {us[50]}")
