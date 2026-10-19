import re,sys
L=open(sys.argv[1]).read().splitlines()
print(L[0])
d={}
for l in L[1:]:
    m=re.match(r'\[custo\] (.+)@(\w+) r ([\d.]+): .*quadro ([\d.]+).*· (\d+) dabs',l)
    d.setdefault((m[1],m[3]),[]).append(f"{m[2]} {float(m[4]):6.2f} ({int(m[5]):4})")
for (p,r),v in d.items(): print(f"{p:<14} r {r:<5} "+" · ".join(v))
