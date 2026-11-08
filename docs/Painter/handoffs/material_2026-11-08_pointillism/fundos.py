import sys,numpy as np, importlib.util
from PIL import Image
sp=importlib.util.spec_from_file_location('f',sys.argv[1]); f=importlib.util.module_from_spec(sp); sp.loader.exec_module(f)
R=sys.argv[2]
base=f.sobre_branco(np.frombuffer(open('cena/base.rgba','rb').read(),np.uint8).reshape(880,2400,4))
csp=np.concatenate([np.array(Image.open(R+f'csp3b/out/Pointillism_{n}.png').convert('RGB')).astype(float) for n in ['a_vazio','b_branco','c_blocos']],1)
c=f.mede(csp,base); print('CSP',c)
for v in sys.argv[3:]:
    p=f.sobre_branco(np.frombuffer(open(f'cena/out_ph2d/Pointillism__{v}.rgba','rb').read(),np.uint8).reshape(880,2400,4))
    m=f.mede(p,base); print(v,[ (n,)+f.veredito(c[n],m[n]) for n in f.FUNDOS], {n:m[n]['cores'] or m[n]['cor'] for n in f.FUNDOS})
