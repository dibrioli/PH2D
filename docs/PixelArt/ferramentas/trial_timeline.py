"""Condutor da onda P6h (timeline) no Aseprite Trial 1.3.18.6 — caixa preta, so CORRIDO.

uso:
  python3 -I trial_timeline.py gera <dir>            escreve <dir>/tl.ase (entrada NOSSA, spec publica do .ase)
  python3 -I trial_timeline.py abre <dir>            Ctrl+O tl.ase (fecha o que estiver aberto sem gravar)
  python3 -I trial_timeline.py g <dir> <foto> GESTO... executa os gestos e fotografa <dir>/<foto>.png
  python3 -I trial_timeline.py caso <dir> <nome>     corre um caso de CASOS (reabre tl.ase, gestos, fotos)
  python3 -I trial_timeline.py todos <dir>           corre todos os CASOS -> <dir>/fotos/*.png

GESTO: k:<tecla xdotool>  t:<texto>  c:x,y (clique)  sc:x,y (shift+clique)  cc:x,y (ctrl+clique)
       d:x0,y0,x1,y1 (arrasto, 3 movimentos finais dentro do destino — quirk do Trial, ver trial_selecao.py)
       w:<s> (espera)  s:<nome> (fotografa a meio)

Arranque (o mesmo dos irmaos; NUNCA o ecra do dono):
  Xvfb :78 -screen 0 1600x1000x24 &
  env -u WAYLAND_DISPLAY DISPLAY=:78 SDL_VIDEODRIVER=x11 HOME=<scratch>/home \
      ~/.local/opt/aseprite-trial/Aseprite-trial.AppImage &
  (sem gestor de janelas: xdotool windowsize <w> 1600 1000 windowmove <w> 0 0)

Entrada tl.ase: 16x16 RGBA, 6 quadros com duracoes 100,200,300,400,500,600 ms, duas camadas:
  'A' (baixo, flags 3 = visivel|editavel, NAO continua) com um cel de cor solida distinta por quadro;
  'B' (cima, flags 3|16 = continua) com cel so nos quadros 0 e 2; tag 'walk' de 1 a 3 (base 0 no
  ficheiro; o Trial mostra 2..4). O Trial nao grava (versao Trial): a leitura e por fotografia da
  tela e dos dialogos; a fixtura oracle_trial_timeline.json guarda o que cada foto mostrou.
"""
import os, struct, subprocess, sys, time, zlib

ENV = dict(os.environ, DISPLAY=':78'); ENV.pop('WAYLAND_DISPLAY', None)
W = H = 16
DUR = [100, 200, 300, 400, 500, 600]
COR = [[230, 25, 75], [60, 180, 75], [255, 225, 25], [0, 130, 200], [245, 130, 48], [145, 30, 180]]


def string(t):
    b = t.encode(); return struct.pack('<H', len(b)) + b
def chunk(kind, data):
    return struct.pack('<IH', len(data) + 6, kind) + data
def layer(name, flags):
    return chunk(0x2004, struct.pack('<HHHHHHB3x', flags, 0, 0, 0, 0, 0, 255) + string(name))
def cel(index, px):
    raw = bytes(v for p in px for v in p[:4])
    return chunk(0x2005, struct.pack('<HhhBHh5x', index, 0, 0, 255, 2, 0) + struct.pack('<HH', W, H) + zlib.compress(raw))
def profile():
    return chunk(0x2007, struct.pack('<HHI8x', 1, 0, 0))
def tags(lst):
    body = struct.pack('<H8x', len(lst))
    for a, b, d, r, name in lst:
        body += struct.pack('<HHBH6x3Bx', a, b, d, r, 0, 0, 0) + string(name)
    return chunk(0x2018, body)


def gera(d):
    os.makedirs(d, exist_ok=True)
    body = b''
    for f, ms in enumerate(DUR):
        ch = [profile(), layer('A', 3), layer('B', 3 | 16), tags([(1, 3, 0, 0, 'walk')])] if f == 0 else []
        ch.append(cel(0, [COR[f] + [255]] * (W * H)))
        if f in (0, 2):
            ch.append(cel(1, [[255, 255, 255, 255] if (x + y) % 4 == 0 else [0, 0, 0, 0]
                              for y in range(H) for x in range(W)]))
        fr = b''.join(ch)
        body += struct.pack('<IHHH2xI', 16 + len(fr), 0xF1FA, len(ch), ms, len(ch)) + fr
    hdr = struct.pack('<IHHHHHIHII', 128 + len(body), 0xA5E0, len(DUR), W, H, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    open(os.path.join(d, 'tl.ase'), 'wb').write(hdr + body)


def xdo(*a): subprocess.run(['xdotool', *map(str, a)], env=ENV, check=True)
def shot(d, nome):
    xdo('mousemove', 1590, 995); time.sleep(0.35)
    subprocess.run(['import', '-window', 'root', os.path.join(d, nome + '.png')], env=ENV, check=True)
def mv_dentro(x, y):
    for dx in (-2, 1, 0):
        xdo('mousemove', x + dx, y); time.sleep(0.05)


def hx(k):
    """x do cabecalho do quadro k (base 0) na timeline: 372, 395, 419, 443, 467, 491, ..."""
    return 372 + round(23.8 * k)


def montagem(d, saida, fotos, caixa):
    """Empilha o recorte `caixa` de cada foto (para ler muitas de uma vez)."""
    from PIL import Image
    ims = [Image.open(os.path.join(d, f + '.png')).crop(caixa) for f in fotos]
    m = Image.new('RGB', (ims[0].width, sum(i.height for i in ims)))
    y = 0
    for i in ims:
        m.paste(i, (0, y)); y += i.height
    m.save(os.path.join(d, saida + '.png'))


def gestos(d, gs):
    for g in gs:
        op, _, a = g.partition(':')
        n = [int(v) for v in a.split(',')] if op in ('c', 'sc', 'cc', 'd', 'dk', 'rc', 'dc') else None
        if op == 'k':
            xdo('key', a); time.sleep(0.4)
        elif op == 't':
            xdo('type', '--delay', 20, a); time.sleep(0.3)
        elif op == 'w':
            time.sleep(float(a))
        elif op == 's':
            shot(d, a)
        elif op == 'c':
            mv_dentro(*n); xdo('click', 1); time.sleep(0.4)
        elif op == 'dc':
            mv_dentro(*n); xdo('click', '--repeat', 2, '--delay', 80, 1); time.sleep(0.6)
        elif op == 'rc':
            mv_dentro(*n); xdo('click', 3); time.sleep(0.4)
        elif op in ('sc', 'cc'):
            m = 'shift' if op == 'sc' else 'ctrl'
            mv_dentro(*n); xdo('keydown', m); xdo('click', 1); xdo('keyup', m); time.sleep(0.4)
        elif op == 'd':
            mv_dentro(n[0], n[1]); xdo('mousedown', 1); time.sleep(0.1)
            for i in range(1, 7):
                xdo('mousemove', n[0] + (n[2] - n[0]) * i // 6, n[1] + (n[3] - n[1]) * i // 6); time.sleep(0.05)
            mv_dentro(n[2], n[3]); xdo('mouseup', 1); time.sleep(0.4)
        elif op == 'dk':  # o arrasto do `d`, com Ctrl premido só ANTES de largar (copiar ao largar)
            mv_dentro(n[0], n[1]); xdo('mousedown', 1); time.sleep(0.1)
            for i in range(1, 7):
                xdo('mousemove', n[0] + (n[2] - n[0]) * i // 6, n[1] + (n[3] - n[1]) * i // 6); time.sleep(0.05)
            mv_dentro(n[2], n[3]); xdo('keydown', 'ctrl'); time.sleep(0.2)
            xdo('mousemove', n[2] + 1, n[3]); time.sleep(0.1); xdo('mousemove', n[2], n[3]); time.sleep(0.1)
            xdo('mouseup', 1); xdo('keyup', 'ctrl'); time.sleep(0.4)
        elif op == 'D':   # D:<prefixo>,<n>: clica o cabecalho de cada quadro e fotografa (a barra de estado diz «ms/total»)
            pre, nq = a.split(',')
            for k in range(int(nq)):
                mv_dentro(hx(k), 826); xdo('click', 1); time.sleep(0.3)
                shot(d, f'{pre}_q{k + 1}')
        else:
            raise SystemExit(f'gesto desconhecido {g}')


def acha(nome):
    """Centro do modelo trial_tpl/<nome>.png na tela (casamento exacto), ou None."""
    import numpy as np
    from PIL import Image
    p = '/tmp/_tl_acha.png'
    subprocess.run(['import', '-window', 'root', p], env=ENV, check=True)
    S = np.asarray(Image.open(p).convert('RGB')).astype(np.int32)
    T = np.asarray(Image.open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'trial_tpl', nome + '.png')).convert('RGB')).astype(np.int32)
    th, tw = T.shape[:2]
    for y, x in np.argwhere(np.all(S == T[0, 0], axis=2)):
        if y + th <= S.shape[0] and x + tw <= S.shape[1] and np.array_equal(S[y:y + th, x:x + tw], T):
            return x + tw // 2, y + th // 2
    return None


def abre(d):
    """Fecha TODOS os documentos (o Trial abre o mesmo ficheiro duas vezes se ja estiver aberto, e o
    dialogo «Don't Save» muda de sitio) e reabre tl.ase limpo."""
    for _ in range(8):
        xdo('key', 'Escape'); xdo('key', 'Escape'); xdo('key', 'ctrl+w'); time.sleep(0.7)
        p = acha('tl_dont_save')
        if p:
            xdo('mousemove', *p, 'click', 1); time.sleep(0.6)
    xdo('key', 'ctrl+o'); time.sleep(1.2)
    xdo('type', '--delay', 12, os.path.join(d, 'tl.ase')); time.sleep(0.3)
    xdo('key', 'Return'); time.sleep(1.8)


# Casos: nome -> gestos (as coordenadas sao as da janela 1600x1000 com a timeline em baixo; ver
# oraculo_timeline.md, «Mapa da tela»). Preenchido a partir das fotos da 1.a corrida.
_CFR = ['c:251,10', 'w:0.4', 'c:329,282', 'w:0.6']          # menu Frame ▸ Constant Frame Rate
_TAG = ['dc:410,797', 'w:0.6']                              # duplo clique na etiqueta da tag
_N = ['c:372,874']                                          # normaliza: cel A, quadro 1
_R = ['d:395,826,443,826']                                  # intervalo 2..4 arrastado pelos cabecalhos
def _ft(a, b): return ['c:882,438', 'k:ctrl+a', f't:{a}', 'c:882,476', 'k:ctrl+a', f't:{b}', 'c:805,618', 'w:0.5']
CASOS = {
    # 1. Constant Frame Rate
    'cfr_sem_intervalo': ['c:443,874', *_CFR, 's:dlg', 'k:ctrl+a', 't:250', 'k:Return', 'w:0.5', 'D:dur,6', 'k:ctrl+z', 'D:undo,6'],
    'cfr_com_intervalo': ['c:419,826', 'sc:467,826', *_CFR, 's:dlg', 'k:Escape'],
    'cfr_prefill': ['c:372,826', 'k:p', 'k:ctrl+a', 't:700', 'k:Return', 'c:467,826', 'k:p', 'k:ctrl+a', 't:50', 'k:Return',
                    *_CFR, 's:dlg', 'k:Escape', 'd:443,826,491,826', 'k:p', 'w:0.4', 's:fp46', 'k:Escape'],
    # 2. Frame Properties com intervalo
    'fp_picks': ['c:395,826', 'sc:443,826', 'k:p', 'w:0.5', 's:dlg', 'k:ctrl+a', 't:333', 'k:Return', 'w:0.5', 'D:dur,6', 'k:ctrl+z', 'D:undo,6'],
    # 3. Tag
    'tag_dlg': [*_N, *_TAG, 's:dlg', 'k:Escape', 'rc:410,797', 'w:0.4', 's:menu', 'k:Escape'],
    'tag_5_3': [*_N, *_TAG, *_ft(5, 3), 's:tl'],
    'tag_2_99': [*_N, *_TAG, *_ft(2, 99), 's:tl', *_TAG, 's:dlg', 'k:Escape'],
    'tag_0_4': [*_N, *_TAG, *_ft(0, 4), 's:tl'],
    'tag_arrasta': [*_N, 'd:452,809,491,809', 'w:0.4', 's:dir', 'd:390,809,372,809', 'w:0.4', 's:esq', 'd:420,797,467,797', 'w:0.4', 's:rotulo'],
    # 4. Continuous
    'cont_newframe': ['c:372,874', 'k:alt+n', 's:tl'],
    'cont_toggle': ['c:220,874', 's:tl', 'c:372,874', 'k:alt+n', 's:nf'],
    'cont_props': [*_N, 'dc:300,874', 'w:0.6', 's:dlg', 'k:Escape', 'k:shift+n', 'w:0.5', 's:nova', 'c:220,874', 's:tog', 'k:ctrl+z', 'w:0.4', 's:undo'],
    # 5. Seleccao e comandos
    'sel_shift_ctrl': [*_N, 'c:372,826', 'sc:443,826', 's:shift', 'cc:491,826', 's:ctrl', 'cc:395,826', 's:ctrl_dentro', 'k:Escape', 's:esc'],
    'sel_arrastos': [*_N, 'd:395,826,467,826', 's:hdr', 'd:395,850,443,874', 's:cels', 'c:280,850', 's:camada', 'c:900,920', 's:fora'],
    'sel_shift_depois_arrasto': [*_N, 'd:395,826,419,826', 'sc:467,826', 's:tl', 'k:p', 'w:0.5', 's:dlg', 'k:Escape', 'c:372,874', 'sc:419,874', 's:cel'],
    'tag_undo': [*_N, *_TAG, 'c:882,476', 'k:ctrl+a', 't:6', 'c:805,618', 'w:0.5', 's:tl', 'k:ctrl+z', 'w:0.4', 's:undo'],
    'del_frame': [*_N, *_R, 'k:alt+c', 'w:0.4', 's:tl', 'D:dur,3', 'k:ctrl+z', 'w:0.4', 'D:undo,6'],
    'del_picks': [*_N, 'c:395,826', 'sc:443,826', 'k:alt+c', 'w:0.4', 's:tl', 'k:Escape', 'D:dur,4'],
    'reverse': [*_N, *_R, 'k:alt+i', 'w:0.4', 's:tl', 'k:Escape', 'D:dur,6'],
    'mover': [*_N, 'd:395,826,419,826', 'd:408,887,480,887', 'w:0.4', 's:tl', 'k:Escape', 'D:dur,6'],
    'clear_cels': [*_N, 'd:395,874,443,874', 'k:Delete', 'w:0.4', 's:tl'],
    'link_cels': [*_N, 'd:372,874,419,874', 'rc:395,874', 'w:0.5', 's:menu', 'k:l', 'w:0.5', 's:tl', 'k:ctrl+z', 'w:0.4', 's:undo'],
    # 6. O que a P6h deixou por medir (corrida de 01/11)
    'mover_ctrl': [*_N, 'd:395,826,419,826', 'dk:408,887,480,887', 'w:0.4', 's:tl', 'k:Escape', 'D:dur,8', 'k:ctrl+z', 'w:0.4', 's:undo'],
    'mover_cels': [*_N, 'd:395,874,419,874', 's:sel', 'd:408,887,480,887', 'w:0.4', 's:tl', 'k:Escape', 'D:dur,6'],
    'mover_cels_ctrl': [*_N, 'd:395,874,419,874', 'dk:408,887,480,887', 'w:0.4', 's:tl', 'k:Escape', 'D:dur,6', 'k:ctrl+z', 'w:0.4', 's:undo'],
    'mover_cels_b':[*_N, 'd:372,850,372,850', 's:sel', 'd:378,862,432,862', 'w:0.4', 's:tl', 'k:Escape', 'D:dur,6'],
    'sel_shift_cel_depois_arrasto': [*_N, 'd:395,874,419,874', 'sc:467,874', 's:cel', 'k:p', 'w:0.5', 's:dlg', 'k:Escape', 'k:Escape',
                                     'd:395,874,419,874', 'sc:467,850', 's:outra'],
    'esc_sel_e_intervalo': [*_N, 'k:ctrl+a', 'w:0.3', *_R, 's:antes', 'k:Escape', 'w:0.3', 's:esc1', 'k:Escape', 'w:0.3', 's:esc2'],
    'esc_intervalo_e_sel': [*_N, *_R, 'k:ctrl+a', 'w:0.3', 's:antes', 'k:Escape', 'w:0.3', 's:esc1', 'k:Escape', 'w:0.3', 's:esc2'],
    'reverse_picks': [*_N, 'c:395,826', 'sc:443,826', 'sc:491,826', 'c:251,10', 'w:0.4', 's:menu', 'k:Escape', 'k:alt+i', 'w:0.4', 's:tl',
                      'k:Escape', 'D:dur,6'],
    'reverse_picks_span': [*_N, 'd:395,826,419,826', 'sc:467,826', 'k:alt+i', 'w:0.4', 's:tl', 'k:Escape', 'D:dur,6'],
    'clear_cels_undo': [*_N, 'd:395,874,443,874', 'k:Delete', 'w:0.4', 's:tl', 'k:ctrl+z', 'w:0.4', 's:undo1', 'k:ctrl+z', 'w:0.4', 's:undo2'],
    'clear_cels_duas': [*_N, 'd:372,850,419,874', 'k:Delete', 'w:0.4', 's:tl', 'k:ctrl+z', 'w:0.4', 's:undo1'],
    # 7. Paste as New Sprite num sprite RGB (sobrevivente #8: a paleta do nascido)
    'paste_new_sprite_rgb': [*_N, 'k:ctrl+a', 'w:0.3', 'k:ctrl+c', 'w:0.3', 'c:65,10', 'w:0.4', 'c:120,200', 'w:0.5', 'c:510,200',
                             'w:1.2', 's:novo', 'c:65,10', 'w:0.4', 's:menu', 'k:Escape', 'k:Escape', 'c:120,10', 'w:0.4', 's:sprite', 'k:Escape'],
}

if __name__ == '__main__':
    cmd, d = sys.argv[1], os.path.abspath(sys.argv[2])
    if cmd == 'gera':
        gera(d)
    elif cmd == 'abre':
        abre(d)
    elif cmd == 'g':
        gestos(d, sys.argv[4:]); shot(d, sys.argv[3])
    elif cmd == 'm':   # m <dir> <saida> x0,y0,x1,y1 foto...
        montagem(d, sys.argv[3], sys.argv[5:], tuple(int(v) for v in sys.argv[4].split(',')))
    elif cmd in ('caso', 'todos'):
        fd = os.path.join(d, 'fotos'); os.makedirs(fd, exist_ok=True)
        for nome in ([sys.argv[3]] if cmd == 'caso' else list(CASOS)):
            abre(d)
            gestos(fd, [g[:2] + f'{nome}_' + g[2:] if g[:2] in ('s:', 'D:') else g for g in CASOS[nome]])
            shot(fd, nome + '_fim')
            print(nome, flush=True)
