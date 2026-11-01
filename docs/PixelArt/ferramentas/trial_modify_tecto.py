"""Select > Modify (Border/Expand/Contract) no Aseprite Trial 1.3.18.6: o TECTO do campo e o que ele faz
com texto que nao e um inteiro (decimal, %, expressao). Caixa preta, so corrido (01/11).

uso: PH2D_TRIAL_DISPLAY=:79 python3 -I trial_modify_tecto.py <dir>   -> <dir>/modify.json {caso: leitura}

Reusa trial_elipse_grelha.py (abre/drag/Fill/fotografia) a 200 % (4 px de tela por celula) sobre um
canvas vermelho 320 x 206 (Contract: Ctrl+A). Cada caso: marquee rectangular de 1 celula em (160, 95), Select > Modify >
<tipo>, campo <- texto, pincel Square, OK; fotografa o dialogo antes do OK (<caso>_dlg.png) e le a
mascara (Fill magenta): "x" = [x0, x1] seleccionados na linha 103 e "y" = [y0, y1] na coluna 160.
Expand n quadrado de 1 celula = [160-n, 160+n] (preso ao canvas: 0..319 x 0..205); Contract n de Ctrl+A = [n, 319-n] x [n, 205-n].
Coordenadas do dialogo (as de trial_selecao.py c_morph): campo (853,451), Square (792,518), OK (734,568).
"""
import json, os, sys, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_elipse_grelha as drv   # noqa: E402

drv.CW, drv.CH, drv.CELL = 320, 206, 4   # 206: Contract 101..103 ainda deixa linhas
_abre = drv.abre
drv.abre = lambda nome, zoom='2': _abre(nome, zoom)
SEL = (318, 10)
MODIFY = {'border': (558, 156), 'expand': (562, 178), 'contract': (566, 200)}
C = (160, 103)
DLG_TITULO = None   # lido do 1.o dialogo (cor da barra de titulo em (700, 417))

CASOS = {   # nome -> (tipo, texto)
    'expand_100': ('expand', '100'), 'expand_101': ('expand', '101'), 'expand_150': ('expand', '150'),
    'expand_999': ('expand', '999'), 'expand_2.4': ('expand', '2.4'), 'expand_2.5': ('expand', '2.5'),
    'expand_2.6': ('expand', '2.6'), 'expand_50pct': ('expand', '50%'), 'expand_3pct': ('expand', '3%'),
    'expand_expr': ('expand', '3*4'), 'expand_0': ('expand', '0'), 'expand_neg': ('expand', '-5'),
    'border_150': ('border', '150'),
    'expand_99': ('expand', '99'), 'expand_70': ('expand', '70'),
    **{f'expand_{n}': ('expand', str(n)) for n in (50, 60, 63, 64, 65, 66, 80, 90, 102, 103, 104)},
    'contract_150': ('contract', '150'), 'contract_100': ('contract', '100'), 'border_100': ('border', '100'), 'borderA_150': ('border', '150'), 'borderA_100': ('border', '100'), 'expand_5pct_circ': ('expand', '5%'),
}


def um(nome, tipo, texto):
    drv.abre('red_big.ase')
    drv.key('Escape'); drv.key('m')
    drv.xdo('mousemove', 178, 72, 'click', 1); time.sleep(0.3)        # modo Replace
    if tipo == 'contract' or nome.startswith('borderA'):
        drv.key('ctrl+a')                         # Contract / Border sobre o canvas inteiro
    else:
        drv.drag(drv.cc(*C), drv.cc(*C))
    for k, p in enumerate([SEL, (337, 156), MODIFY[tipo]]):
        drv.xdo('mousemove', *p); time.sleep(0.5)
        if k in (0, 2):
            drv.xdo('click', 1); time.sleep(0.6)
    drv._settle((853, 451)); drv.xdo('click', 1); time.sleep(0.2); drv.key('ctrl+a')
    drv.xdo('type', texto); time.sleep(0.3)
    drv._settle((792, 518)); drv.xdo('click', 1); time.sleep(0.3)
    global DLG_TITULO
    d = drv.shot(nome + '_dlg.png', park=False)
    DLG_TITULO = tuple(d[417, 700])
    drv.key('Return', t=0.8)                      # OK = Enter (o clique no OK falhou 10/12 em 01/11)
    t0 = time.time()
    while True:   # o Trial CALCULA com o dialogo ainda na tela: Expand 80 leva segundos (medido 01/11)
        a = drv.shot(nome + '_sel.png')
        fechou = tuple(a[417, 700]) != DLG_TITULO   # a barra de titulo do dialogo (670..930, 417)
        if fechou or time.time() - t0 > 90:
            break
        time.sleep(2)
    espera = round(time.time() - t0, 1)
    if not fechou:
        drv.key('Escape', t=0.4); drv.fecha()
        return {'texto': texto, 'tipo': tipo, 'fechou': False}
    drv.key('f', t=0.8); drv.key('Escape', t=0.4)
    a = drv.shot(nome + '.png')
    m = [[p == drv.MAG for p in row] for row in drv.cells(a)]
    drv.fecha()
    row = [i for i in range(drv.CW) if m[C[1]][i]]
    col = [j for j in range(drv.CH) if m[j][C[0]]]
    tot = sum(map(sum, m))
    return {'texto': texto, 'tipo': tipo, 'fechou': True, 'segundos': espera, 'x': [row[0], row[-1]] if row else None,
            'y': [col[0], col[-1]] if col else None, 'celulas': tot}


def main():
    out_p = os.path.join(drv.D, 'modify.json')
    out = json.load(open(out_p)) if os.path.exists(out_p) else {}
    drv.write_red(os.path.join(drv.D, 'red_big.ase'), drv.CW, drv.CH)
    drv.abre('red_big.ase'); drv.key('m'); drv.ensure_fg(); drv.fecha()
    nomes = sys.argv[2:] or list(CASOS)
    for n in nomes:
        out[n] = um(n, *CASOS[n]); print(n, out[n], flush=True)
        json.dump(out, open(out_p, 'w'), indent=1)


if __name__ == '__main__':
    main()
