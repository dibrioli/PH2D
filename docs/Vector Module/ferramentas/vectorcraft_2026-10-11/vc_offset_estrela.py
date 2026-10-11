import os, math, subprocess, json, sys
from concurrent.futures import ThreadPoolExecutor
import numpy as np
from PIL import Image
VC="/home/enio/Apps/vectorcraft/squashfs-root/usr/bin/vectorcraft-cli"
D=os.environ.get("VC_OUT","target/vectorcraft")+"/star"; os.makedirs(D,exist_ok=True)
W=1200; C=600
# a estrela do gate do PH2D (cook Star 5 pontas, razão 0.45, caixa 2.8) escalada ×100
R=140; r=R*0.45
pts=[]
for i in range(10):
    a=-math.pi/2+i*math.pi/5; rr=R if i%2==0 else r
    pts.append((C+rr*math.cos(a),C+rr*math.sin(a)))
d="M"+" L".join(f"{x:.4f},{y:.4f}" for x,y in pts)+" Z"
HDR=f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{W}" viewBox="0 0 {W} {W}">'
src=f"{D}/star.svg"; open(src,"w").write(HDR+f'<path d="{d}" fill="#000"/></svg>')
def mask(p):
    return np.asarray(Image.open(p).convert("RGBA"))[:,:,3]>127
def ref(dist,join):
    p=f"{D}/ref_{join}_{dist}.svg"
    open(p,"w").write(HDR+f'<path d="{d}" fill="#000" stroke="#000" stroke-width="{2*dist}" stroke-linejoin="{join}" stroke-miterlimit="4"/></svg>')
    subprocess.run(["resvg",p,p+".png"],check=True); return mask(p+".png")
def job(a):
    dist,join=a; out=f"{D}/o_{join}_{dist}"
    p=subprocess.run([VC,"run","--in",src,"--cmd","select.all","--cmd","object.path.offsetPath","--params",json.dumps({"offset":dist,"joins":join}),"--export",out+".svg"],capture_output=True,text=True,timeout=300)
    res={"d":dist,"join":join,"rc":p.returncode}
    for l in p.stdout.splitlines():
        if l.startswith("{"):
            s=json.loads(l)
            if "error" in s: res["err"]=str(s["error"])[:120]
    if p.returncode==0 and os.path.exists(out+".svg"):
        txt=open(out+".svg").read(); res["paths"]=txt.count("<path")
        if dist>0:
            # só o offset (sem o original): renderiza e compara com fill∪stroke
            subprocess.run(["resvg",out+".svg",out+".png"])
            m=mask(out+".png"); rf=ref(dist,join)
            res["diff_pct"]=round(100*(m^rf).sum()/max(rf.sum(),1),3)
    return res
dists=[round(4*i,1) for i in range(1,101)]  # 4..400 = 0.04..4.0 ×100
jobs=[(s*x,j) for x in dists for j in ("miter","round","bevel") for s in (1,-1)]
with ThreadPoolExecutor(8) as ex: rows=list(ex.map(job,jobs))
json.dump(rows,open(f"{D}/../star_results.json","w"),indent=0)
for j in ("miter","round","bevel"):
  for s,name in ((1,"fora"),(-1,"dentro")):
    rs=[x for x in rows if x["join"]==j and (x["d"]>0)==(s>0)]
    ok=[x for x in rs if x["rc"]==0]; errs={}
    for x in rs:
        if x["rc"]!=0: errs[x.get("err","rc%d"%x["rc"])[:60]]=errs.get(x.get("err","rc%d"%x["rc"])[:60],0)+1
    dif=[x["diff_pct"] for x in ok if "diff_pct" in x]
    print(f"{j:6} {name:6} ok {len(ok)}/{len(rs)}  diff max {max(dif) if dif else '-'}%  med {sorted(dif)[len(dif)//2] if dif else '-'}%  falhas {errs}")
