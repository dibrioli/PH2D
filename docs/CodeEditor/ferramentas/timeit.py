import subprocess,sys,time,resource
best=None;out=""
for i in range(5):
    t=time.perf_counter(); p=subprocess.run(sys.argv[1:],capture_output=True,text=True); dt=time.perf_counter()-t
    best=dt if best is None else min(best,dt); out=p.stdout+p.stderr
print(f"min {best*1000:.0f} ms over 5; maxrss(children,max) {resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss//1024} MB; rc={p.returncode}")
print("\n".join(out.splitlines()[:12]))
