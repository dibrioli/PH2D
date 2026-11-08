"""Grão visível = desvio do passa-alto (píxel − média 7×7) da luminância no núcleo do traço A (y 74..86), por região."""
import sys, os
import numpy as np
W,H=600,300
d=sys.argv[1]
def hp(n,v):
    p=os.path.join(d,f"{n}__{v}.rgba")
    a=np.frombuffer(open(p,'rb').read(),np.uint8).reshape(H,W,4).astype(float)
    al=a[...,3]/255; L=(0.3*a[...,0]+0.59*a[...,1]+0.11*a[...,2])*al
    from numpy.lib.stride_tricks import sliding_window_view as sw
    k=7; pad=np.pad(L,3,mode='edge'); m=sw(pad,(k,k)).mean(axis=(-1,-2))
    h=(L-m)[74:86]
    pa=np.pad(al,3,mode='edge'); ma=sw(pa,(k,k)).mean(axis=(-1,-2)); ha=(al-ma)[214:226]
    return h[:,230:370].std(), h[:,40:170].std(), ha[:,30:170].std()
nomes=sorted({f.split("__")[0] for f in os.listdir(d) if f.endswith('.rgba')})
print("pincel\tvar\thp_branco\thp_vazioA\thpalfa_vazioB")
for n in nomes:
    for v in ["liso","liso_off","grao","grao_off","papel","papel_off"]:
        r=hp(n,v); print(f"{n}\t{v}\t{r[0]:.2f}\t{r[1]:.2f}\t{r[2]:.4f}")
