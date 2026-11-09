import sys, os
sys.path.insert(0, os.environ['M'])
import numpy as np
from carrega import ph2d
from mede import mede
from lei import janela
S = sys.argv[1]
print('var    | passe | cob p1 (miolo) | rampa p .35 · .5 · .65 · .8')
for v in sys.argv[2:]:
    for n in range(1, 6):
        x = '' if n == 1 else f'__x{n}'
        _, k1 = ph2d(S + '/cena', f'__{v}{x}')
        _, kr = ph2d(S + '/cena', f'__{v}__rampa{x}')
        r = [float(janela(kr, p).mean()) for p in (0.35, 0.5, 0.65, 0.8)]
        print(f'{v:6} | {n} | {mede(k1)["cob_miolo"]:.3f} | ' + ' · '.join(f'{c:.2f}' for c in r))
