"""Conta, em cada bloco 8x8 plano da sonda, quantos pixels saem de cada índice e onde (máscara 8x8)."""
import sys; sys.path.insert(0,'/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/c142c8eb-76c4-4200-a92d-b3bc73e244a0/scratchpad/oraculo/tools')
from trial_fit_dither import *
for r in records(sys.argv[1]):
    if r['image']!='probe': continue
    if len(sys.argv)>2 and r['dither'] not in [int(a) for a in sys.argv[2:]]: continue
    g=np.array(r['indices']).reshape(40,72); img=IMGS['probe']; P=np.array(PALS[r['palette']],float)
    print('==',r['palette'],'d',r['dither'])
    for k in range(45):
        by,bx=divmod(k,9); blk=g[by*8:by*8+8,bx*8:bx*8+8]; c=img[by*8,bx*8]
        u,cn=np.unique(blk,return_counts=True)
        print(k,c.tolist(),'near',int(nn(P,c[None].astype(float))[0]),dict(zip(u.tolist(),cn.tolist())))
