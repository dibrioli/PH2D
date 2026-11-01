"""Condutor da medicao das PALETAS no Aseprite Trial 1.3.18.6 — caixa preta, so CORRIDO.

uso:
  python3 -I trial_paleta.py gera <dir>              escreve <dir>/idx.ase (INDEXADO) e <dir>/rgb.ase (RGBA)
  python3 -I trial_paleta.py abre <dir> <idx|rgb>    Ctrl+O do ficheiro (fecha o que estiver aberto sem gravar)
  python3 -I trial_paleta.py g <dir> <foto> GESTO... executa os gestos e fotografa <dir>/<foto>.png
  python3 -I trial_paleta.py px <foto> x,y ...       le o RGB de pontos de uma foto
  python3 -I trial_paleta.py caso <dir> <nome>       corre um caso de CASOS (reabre a entrada, gestos, fotos)
  python3 -I trial_paleta.py todos <dir>             todos os CASOS -> <dir>/fotos/*.png

GESTO: k:<tecla xdotool>  t:<texto>  c:x,y (clique)  rc:x,y (botao direito)  cc:x,y (ctrl+clique)
       sc:x,y (shift+clique)  d:x0,y0,x1,y1 (arrasto)  dk:x0,y0,x1,y1 (arrasto com Ctrl desde o inicio)
       w:<s> (espera)  s:<nome> (fotografa a meio)

Arranque (NUNCA o ecra do dono; ecra :80 desta medicao):
  Xvfb :80 -screen 0 1600x1000x24 &
  env -u WAYLAND_DISPLAY DISPLAY=:80 SDL_VIDEODRIVER=x11 HOME=<scratch>/trial80/home \\
      ~/.local/opt/aseprite-trial/Aseprite-trial.AppImage &
  (sem gestor de janelas: xdotool windowsize <w> 1600 1000 windowmove <w> 0 0)
  HOME novo = escala de UI 2 (a de omissao do Trial a 1600x1000), barra de cores a esquerda.

Entradas NOSSAS (spec publica do .ase):
  idx.ase: 8x8 INDEXADO, camada Background, pixel (x,y) = indice x (cada COLUNA e um indice),
           paleta de 8 cores PAL abaixo, indice transparente 0.
  rgb.ase: a mesma imagem em RGBA (pixel (x,y) = PAL[x]) com a mesma paleta de 8.
  Leitura: o Trial nao grava; tudo por fotografia (cor das colunas no canvas, swatches da barra,
  campo hex/RGB do seletor de cor). A fixtura oracle_trial_palette.json guarda o que se leu.
"""
import os, struct, subprocess, sys, time, zlib

ENV = dict(os.environ, DISPLAY=':80'); ENV.pop('WAYLAND_DISPLAY', None)
W = H = 8
PAL = [[0, 0, 0], [255, 0, 0], [0, 255, 0], [0, 0, 255],
       [255, 255, 0], [0, 255, 255], [255, 0, 255], [255, 255, 255]]


def string(t):
    b = t.encode(); return struct.pack('<H', len(b)) + b
def chunk(kind, data):
    return struct.pack('<IH', len(data) + 6, kind) + data
def layer(name, flags):
    return chunk(0x2004, struct.pack('<HHHHHHB3x', flags, 0, 0, 0, 0, 0, 255) + string(name))
def cel(raw):
    return chunk(0x2005, struct.pack('<HhhBHh5x', 0, 0, 0, 255, 2, 0) + struct.pack('<HH', W, H) + zlib.compress(raw))
def palette():
    body = struct.pack('<III8x', len(PAL), 0, len(PAL) - 1)
    for r, g, b in PAL:
        body += struct.pack('<HBBBB', 0, r, g, b, 255)
    return chunk(0x2019, body)


def escreve(path, depth):
    if depth == 8:
        raw = bytes(x for y in range(H) for x in range(W))
    else:
        raw = bytes(v for y in range(H) for x in range(W) for v in PAL[x] + [255])
    ch = [palette(), layer('Background', 1 | 2 | 8), cel(raw)]
    fr = b''.join(ch)
    body = struct.pack('<IHHH2xI', 16 + len(fr), 0xF1FA, len(ch), 100, len(ch)) + fr
    hdr = struct.pack('<IHHHHHIHII', 128 + len(body), 0xA5E0, 1, W, H, depth, 1, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, len(PAL), 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    open(path, 'wb').write(hdr + body)


def gera(d):
    os.makedirs(d, exist_ok=True)
    escreve(os.path.join(d, 'idx.ase'), 8)
    escreve(os.path.join(d, 'rgb.ase'), 32)


def xdo(*a): subprocess.run(['xdotool', *map(str, a)], env=ENV, check=True)
def shot(d, nome, mexe=True):
    if mexe:
        xdo('mousemove', 1590, 995); time.sleep(0.35)
    subprocess.run(['import', '-window', 'root', os.path.join(d, nome + '.png')], env=ENV, check=True)
def mv_dentro(x, y):
    for dx in (-2, 1, 0):
        xdo('mousemove', x + dx, y); time.sleep(0.05)


def gestos(d, gs):
    for g in gs:
        op, _, a = g.partition(':')
        n = [int(v) for v in a.split(',')] if op in ('c', 'sc', 'cc', 'd', 'dk', 'rc') else None
        if op == 'k':
            xdo('key', a); time.sleep(0.4)
        elif op == 't':
            xdo('type', '--delay', 20, a); time.sleep(0.3)
        elif op == 'w':
            time.sleep(float(a))
        elif op == 's':
            shot(d, a)
        elif op == 'S':   # foto SEM tirar o rato (hover)
            shot(d, a, False)
        elif op == 'c':
            mv_dentro(*n); xdo('click', 1); time.sleep(0.4)
        elif op == 'rc':
            mv_dentro(*n); xdo('click', 3); time.sleep(0.4)
        elif op in ('sc', 'cc'):
            m = 'shift' if op == 'sc' else 'ctrl'
            mv_dentro(*n); xdo('keydown', m); xdo('click', 1); xdo('keyup', m); time.sleep(0.4)
        elif op in ('d', 'dk'):
            mv_dentro(n[0], n[1])
            if op == 'dk':
                xdo('keydown', 'ctrl')
            xdo('mousedown', 1); time.sleep(0.1)
            for i in range(1, 9):
                xdo('mousemove', n[0] + (n[2] - n[0]) * i // 8, n[1] + (n[3] - n[1]) * i // 8); time.sleep(0.05)
            mv_dentro(n[2], n[3]); xdo('mouseup', 1)
            if op == 'dk':
                xdo('keyup', 'ctrl')
            time.sleep(0.4)
        else:
            raise SystemExit(f'gesto desconhecido {g}')


def px(foto, pts):
    from PIL import Image
    im = Image.open(foto).convert('RGB')
    return {p: im.getpixel(tuple(int(v) for v in p.split(','))) for p in pts}


def mata():
    """Mata POR PID so os processos do Trial com DISPLAY=:80 (o AppRun e o binario); nunca pkill."""
    alvo = []
    for p in os.listdir('/proc'):
        if not p.isdigit():
            continue
        try:
            cmd = open(f'/proc/{p}/cmdline', 'rb').read()
            env = open(f'/proc/{p}/environ', 'rb').read().split(b'\0')
        except OSError:
            continue
        if b'Aseprite-trial' in cmd and b'DISPLAY=:80' in env:
            alvo.append(int(p))
    for p in alvo:
        try:
            os.kill(p, 15)
        except ProcessLookupError:
            pass
    if alvo:
        time.sleep(1.5)


def abre(d, qual):
    """REINICIA o Trial (mata por PID em <d>/ase.pid — fechar documentos sujos por teclado deixou
    estado alheio na 1.a corrida), abre <qual>.ase e faz zoom 3200 % (colunas em x = 660 + 64*k)."""
    pf = os.path.join(d, 'ase.pid')
    mata()
    env = dict(ENV, SDL_VIDEODRIVER='x11', HOME=os.path.join(d, 'home'))
    p = subprocess.Popen([os.path.expanduser('~/.local/opt/aseprite-trial/Aseprite-trial.AppImage')], env=env,
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    open(pf, 'w').write(str(p.pid))
    for _ in range(40):
        time.sleep(0.5)
        r = subprocess.run(['xdotool', 'search', '--name', 'Aseprite'], env=ENV, capture_output=True, text=True)
        if r.stdout.strip():
            break
    time.sleep(2)
    w = r.stdout.split()[0]
    xdo('windowsize', w, 1600, 1000); xdo('windowmove', w, 0, 0); time.sleep(1)
    xdo('key', 'ctrl+o'); time.sleep(1.2)
    xdo('type', '--delay', 12, os.path.join(d, qual + '.ase')); time.sleep(0.3)
    xdo('key', 'Return'); time.sleep(1.8)
    xdo('mousemove', 900, 300); time.sleep(0.2); xdo('key', '6'); time.sleep(0.6)


# Casos: nome -> (entrada, gestos). Coordenadas da janela 1600x1000 (ver oraculo_paleta.md, «Mapa»).
_SEL_1_5 = ['c:46,110', 'd:46,110,22,134']            # seleccao 1..5 por ARRASTO (Shift+clique nao estende aqui)
_SORT = ['c:56,70', 'w:0.5']                               # botao «↓»: Reverse/Gradient/Sort
CASOS = {
    # 1. arrastar-reordenar (agarrar a BORDA da seleccao: arrastar de dentro estende a seleccao)
    'drag_idx': ('idx', ['c:46,110', 'd:35,110,118,110', 's:drag', 'c:77,712', 'w:0.6', 's:remap',
                         'k:ctrl+z', 'w:0.5', 's:undo1', 'k:ctrl+z', 'w:0.5', 's:undo2']),
    'drag_rgb': ('rgb', ['c:46,110', 'd:35,110,118,110', 's:drag', 'k:ctrl+z', 'w:0.5', 's:undo1']),
    'drag_ctrl_idx': ('idx', ['c:46,110', 'dk:35,110,118,110', 's:drag', 'k:ctrl+z', 'w:0.5', 's:undo1']),
    # 4. Gradient / Gradient by Hue (entradas 1..5 = vermelho -> ciano; 0..4 = preto -> amarelo)
    'gradient': ('rgb', [*_SEL_1_5, *_SORT, 'c:96,126', 'w:0.6', 's:rgb15', 'k:ctrl+z', 'w:0.4', 's:undo',
                         'd:22,110,118,110', *_SORT, 'c:96,126', 'w:0.6', 's:rgb04']),
    'gradient_hue': ('rgb', [*_SEL_1_5, *_SORT, 'c:128,148', 'w:0.6', 's:hue15']),
    # 5. Sort / Reverse (sprite INDEXADO: os pixels seguem o indice?)
    'sort_reverse': ('idx', ['c:22,110', 'd:22,110,70,134', *_SORT, 'c:112,177', 'w:0.6', 's:hue', 'k:ctrl+z', 'w:0.4',
                             *_SORT, 'c:124,103', 'w:0.6', 's:rev', 'k:ctrl+z', 'w:0.4', *_SORT, 'c:140,221', 'w:0.6', 's:bri',
                             'k:ctrl+z', 'w:0.4', 'd:46,110,118,110', *_SORT, 'c:124,103', 'w:0.6', 's:rev14']),
    # menus e presets (so os nomes)
    'menus': ('idx', ['c:112,70', 'w:0.6', 's:menu', 'k:Escape', *_SORT, 's:sort', 'k:Escape', 'c:84,70', 'w:1.5', 'S:presets']),
    # 2/3. seletores: a caixa do Tint/Shade/Tone em x 10..145, y 744..859 (fisicos); rodas pelo menu ≡
    'seletores': ('rgb', ['c:46,110', 's:tst', 'c:112,70', 'w:0.5', 'c:183,281', 'w:0.6', 's:spec', 'c:112,70', 'w:0.5',
                          'c:188,303', 'w:0.6', 's:rgbw', 'c:112,70', 'w:0.5', 'c:188,325', 'w:0.6', 's:ryb',
                          'c:112,70', 'w:0.5', 'c:222,347', 'w:0.6', 's:nm']),
}

if __name__ == '__main__':
    cmd = sys.argv[1]
    if cmd == 'px':
        for k, v in px(sys.argv[2], sys.argv[3:]).items():
            print(k, '%d,%d,%d' % v, '#%02x%02x%02x' % v)
        raise SystemExit
    if cmd == 'mata':
        mata(); raise SystemExit
    d = os.path.abspath(sys.argv[2])
    if cmd == 'gera':
        gera(d)
    elif cmd == 'abre':
        abre(d, sys.argv[3])
    elif cmd == 'g':
        gestos(d, sys.argv[4:]); shot(d, sys.argv[3])
    elif cmd in ('caso', 'todos'):
        fd = os.path.join(d, 'fotos'); os.makedirs(fd, exist_ok=True)
        for nome in ([sys.argv[3]] if cmd == 'caso' else list(CASOS)):
            ent, gs = CASOS[nome]
            abre(d, ent)
            gestos(fd, [g[:2] + f'{nome}_' + g[2:] if g[:2] in ('s:', 'S:') else g for g in gs])
            shot(fd, nome + '_fim')
            print(nome, flush=True)
