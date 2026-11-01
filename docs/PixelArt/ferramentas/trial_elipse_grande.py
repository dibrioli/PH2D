"""A marquee ELIPTICA do Aseprite Trial em caixas GRANDES (lados > 150 x 96), lida do ecra.

uso: PH2D_TRIAL_DISPLAY=:79 python3 -I trial_elipse_grande.py <dir> [lista.json]
     -> <dir>/grelha.json no formato de trial_elipse_grelha.py ({"tl": {"<w>x<h>": {"at", "mask"}}})

Reusa trial_elipse_grelha.py inteiro e so muda a escala: canvas 320 x 190 a zoom 200 % (tecla 2 = 4 px
de tela por celula), folga 5 celulas. A amostra: todo lado PAR de 152 a 316 (com o outro lado 3) e de 98 a
186 (com o outro lado 3), caixas gerais grandes, e 4 caixas de controlo ja medidas a 400 %.
Resultado 01/11: 141/141 iguais a ellipse_rows -> crates/ph2d-pixel-core/tests/fixtures/oracle_trial_ellipse_big.json.
"""
import os, sys, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_elipse_grelha as drv   # noqa: E402  (le sys.argv[1] = <dir>)

drv.CW, drv.CH, drv.CELL, drv.GAP = 320, 190, 4, 5
# MEDIDO 01/11: a 100 % (2 px de tela por celula) o controlo (24x24, 13x7, 22x3... ja na fixtura de 27/10)
# deu 8/8 ERRADOS -- o premir/soltar perde a celula; a 200 % (4 px) o assentamento original (-2, +1, 0) cabe
# na celula. O tecto do canvas e entao 320 x 190 (a area do editor a 4 px).


def mask_now(name):
    drv.key('f', t=0.8); drv.key('Escape', t=0.4)
    a = drv.shot(name + '.png')
    return [[p == drv.MAG for p in row] for row in drv.cells(a)]


_abre = drv.abre
drv.mask_now = mask_now
drv.abre = lambda nome, zoom='2': _abre(nome, zoom)

BOXES = ([(24, 24), (13, 7), (22, 3), (3, 23)]   # controlo: ja medidos a 400 %
         + [(w, 3) for w in range(152, 317, 2)] + [(3, h) for h in range(98, 187, 2)]
         + [(151, 97), (152, 98), (153, 99), (160, 100), (200, 120), (300, 17), (17, 186), (176, 154),
            (198, 186), (316, 186), (255, 185), (154, 154)])

if __name__ == '__main__':
    import json
    lst = os.path.join(drv.D, 'caixas.json')
    json.dump(json.load(open(sys.argv[2])) if len(sys.argv) > 2 else BOXES, open(lst, 'w'))   # [lista.json] = outra amostra
    sys.argv = [sys.argv[0], drv.D, 'tl', '--caixas', lst]
    drv.main()
