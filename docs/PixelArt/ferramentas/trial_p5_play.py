"""Condutor da onda P5 (tocar: subtags, velocidade, opcoes, saltos, loop section, preview) no
Aseprite Trial 1.3.18.6 — caixa preta, so CORRIDO.

uso:
  python3 -I trial_p5_play.py gera <dir>                 escreve <dir>/*.ase (entradas NOSSAS, spec publica)
  python3 -I trial_p5_play.py abre <dir> <f.ase>         fecha tudo sem gravar e abre <f.ase>
  python3 -I trial_p5_play.py g <dir> <foto> GESTO...    gestos e foto <dir>/<foto>.png (gestos de trial_timeline.py)
  python3 -I trial_p5_play.py toca <dir> <nome> <s> [GESTO...]
                                                          gestos, video <s> s do centro do canvas, Enter a 1 s,
                                                          Enter no fim; grava <dir>/<nome>.json (sequencia)
  python3 -I trial_p5_play.py caso <dir> <nome>          corre um caso de CASOS (abre o ficheiro, gestos, toca)
  python3 -I trial_p5_play.py todos <dir> [2]            corre CASOS1 (ou CASOS2) -> <dir>/play.json (play2.json)

Arranque (o mesmo dos irmaos; NUNCA o ecra do dono):
  Xvfb :78 -screen 0 1600x1000x24 &
  env -u WAYLAND_DISPLAY DISPLAY=:78 SDL_VIDEODRIVER=x11 HOME=<scratch>/home \\
      ~/.local/opt/aseprite-trial/Aseprite-trial.AppImage &
  xdotool windowsize <w> 1600 1000 windowmove <w> 0 0   (sem gestor de janelas)
  HOME limpo: escala de UI de omissao, timeline em baixo.

Entradas: 32x32 RGBA, 8 quadros de 100 ms, quadro k = cor solida COR[k] (le-se o indice pela cor);
tags (base 0 no ficheiro; o Trial mostra base 1), na ORDEM da lista do ficheiro:
  s1.ase  out 1..6 Forward  ;  in 2..4 PingPong repeat 2
  s2.ase  out 1..6 Forward repeat 2 ; in 2..4 Reverse
  s3.ase  a 1..4 Forward ; b 3..6 Forward  (sobrepostas, nenhuma contem a outra)
  s4.ase  in 2..4 PingPong repeat 2 ; out 1..6 Forward  (s1 com a ordem da lista trocada)
  s0.ase  sem tags (velocidade, Play Once, Rewind on Stop, Set Loop Section, Preview)
  s5.ase  pp 2..4 PingPong repeat 0 isolada (Play Once com raiz PingPong)
Continuacao (ronda3b da fixtura): gestos corridos a mao com estas funcoes; Jump to pelo menu Frame
(251,10) > Jump to (300,252) > First/Last Frame In Tag (560,348/370); Set Loop Section = F2 sobre um
intervalo arrastado nos cabecalhos (s0: y=826); Play Once pelo menu Frame > Playback (568,400).
Leitura: ffmpeg x11grab 50 q/s de 8x8 px no centro do canvas -> sequencia (indice, ms) por cor.
"""
import json, os, struct, subprocess, sys, time, zlib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_timeline as T   # noqa: E402  (xdo, shot, gestos, acha)

ENV = T.ENV
W = H = 32
N = 8
COR = [[230, 25, 75], [60, 180, 75], [255, 225, 25], [0, 130, 200], [245, 130, 48], [145, 30, 180],
       [70, 240, 240], [240, 50, 230]]
ENTRADAS = {
    's0': [],
    's1': [(1, 6, 0, 0, 'out'), (2, 4, 2, 2, 'in')],
    's2': [(1, 6, 0, 2, 'out'), (2, 4, 1, 0, 'in')],
    's3': [(1, 4, 0, 0, 'a'), (3, 6, 0, 0, 'b')],
    's4': [(2, 4, 2, 2, 'in'), (1, 6, 0, 0, 'out')],
    's5': [(2, 4, 2, 0, 'pp')],            # PingPong infinito isolado (Play Once com raiz PingPong)
}


def string(t):
    b = t.encode(); return struct.pack('<H', len(b)) + b
def chunk(kind, data):
    return struct.pack('<IH', len(data) + 6, kind) + data
def tags(lst):
    body = struct.pack('<H8x', len(lst))
    for a, b, d, r, name in lst:
        body += struct.pack('<HHBH6x3Bx', a, b, d, r, 0, 0, 0) + string(name)
    return chunk(0x2018, body)


def gera(d):
    os.makedirs(d, exist_ok=True)
    for nome, tg in ENTRADAS.items():
        body = b''
        for f in range(N):
            ch = [chunk(0x2007, struct.pack('<HHI8x', 1, 0, 0)),
                  chunk(0x2004, struct.pack('<HHHHHHB3x', 3, 0, 0, 0, 0, 0, 255) + string('A'))] if f == 0 else []
            if f == 0 and tg:
                ch.append(tags(tg))
            raw = bytes(COR[f] + [255]) * (W * H)
            ch.append(chunk(0x2005, struct.pack('<HhhBHh5x', 0, 0, 0, 255, 2, 0) + struct.pack('<HH', W, H) + zlib.compress(raw)))
            fr = b''.join(ch)
            body += struct.pack('<IHHH2xI', 16 + len(fr), 0xF1FA, len(ch), 100, len(ch)) + fr
        hdr = struct.pack('<IHHHHHIHII', 128 + len(body), 0xA5E0, N, W, H, 32, 1 | 2, 100, 0, 0)
        hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
        hdr += b'\0' * (128 - len(hdr))
        open(os.path.join(d, nome + '.ase'), 'wb').write(hdr + body)


def abre(d, f):
    for _ in range(6):
        T.xdo('key', 'Escape'); T.xdo('key', 'ctrl+w'); time.sleep(0.6)
        p = T.acha('tl_dont_save')
        if p:
            T.xdo('mousemove', *p, 'click', 1); time.sleep(0.5)
    T.xdo('key', 'ctrl+o'); time.sleep(1.2)
    T.xdo('type', '--delay', 12, os.path.join(d, f)); time.sleep(0.3)
    T.xdo('key', 'Return'); time.sleep(1.8)


def centro():
    """Centro do maior bloco de uma das cores COR no ecra (o canvas)."""
    import numpy as np
    from PIL import Image
    p = '/tmp/_p5_c.png'
    subprocess.run(['import', '-window', 'root', p], env=ENV, check=True)
    a = np.asarray(Image.open(p).convert('RGB')).astype(int)[60:700, 150:1140]  # fora da janela Preview
    best = None
    for c in COR:
        ys, xs = np.nonzero(np.all(a == c, axis=2))
        if len(xs) > 400 and (best is None or len(xs) > best[0]):
            best = (len(xs), int(np.median(xs)) + 150, int(np.median(ys)) + 60)
    return best[1], best[2]


def idx(p):
    d = [sum((a - b) ** 2 for a, b in zip(p, c)) for c in COR]
    k = min(range(N), key=d.__getitem__)
    return k if d[k] < 300 else -1


def toca(d, nome, segs, gs=(), tecla='Return', parar=True):
    T.gestos(d, list(gs))
    cx, cy = centro()
    T.xdo('mousemove', 1590, 995)
    mkv = os.path.join(d, nome + '.mkv')
    rec = subprocess.Popen(['ffmpeg', '-loglevel', 'error', '-y', '-f', 'x11grab', '-framerate', '50',
                            '-video_size', '8x8', '-i', f':78+{cx - 4},{cy - 4}', '-t', str(segs), '-c:v', 'ffv1', mkv], env=ENV)
    time.sleep(1.0)
    T.xdo('key', tecla)
    rec.wait()
    if parar:
        T.xdo('key', tecla); time.sleep(0.5)
    raw = subprocess.run(['ffmpeg', '-loglevel', 'error', '-i', mkv, '-vf', 'crop=1:1:4:4', '-f', 'rawvideo',
                          '-pix_fmt', 'rgb24', '-'], capture_output=True, check=True).stdout
    runs = []
    for i in range(0, len(raw), 3):
        k = idx(list(raw[i:i + 3]))
        if runs and runs[-1][0] == k:
            runs[-1][1] += 20
        else:
            runs.append([k, 20])
    time.sleep(0.4)
    T.shot(d, nome + '_parado')
    from PIL import Image
    parado = idx(list(Image.open(os.path.join(d, nome + '_parado.png')).convert('RGB').getpixel((cx, cy))))
    out = {'runs_idx_ms': runs, 'seq': [r[0] for r in runs], 'parado': parado}
    json.dump(out, open(os.path.join(d, nome + '.json'), 'w'))
    return out


def F(k):
    """Activo = quadro k (base 0): Home e k vezes Right (Right anda 1, nao respeita tags)."""
    return ['k:Escape', 'k:Home'] + ['k:Right'] * k


# Menu do botao direito no Play da timeline (x=240; y=820 com 2 filas de tags, 794 sem tags):
# Speed 0.25x/0.5x/1x/1.5x/2x/3x, Play Once, Play All Frames (Ignore Tags), Play Subtags & Repetitions,
# Rewind on Stop. y do item relativo ao clique: +y abaixo.
_OPT = {'0.25': -52, '0.5': -30, '1': -8, '1.5': 14, '2': 36, '3': 58, 'once': 88, 'all': 110, 'sub': 132, 'rewind': 162}
def O(op):
    """So com um ficheiro de 2 filas de tags aberto (s1..s4): em s0 o menu abre noutro sitio e o
    clique cai UM item acima (medido na corrida 1: 's0_v*' e 's0_rewind_f2' tocaram o item de cima)."""
    return ['rc:240,820', 'w:0.5', 'c:330,%d' % (820 + _OPT[op]), 'w:0.5']


# Casos: nome -> (ficheiro, gestos antes de tocar, segundos de video). A ORDEM importa: as opcoes
# do menu do Play sao preferencias globais (persistem entre documentos) e cada caso que liga uma desliga-a.
CASOS1 = {
    # 1. Subtags (Play Subtags & Repetitions ligado = omissao)
    's1_on_f0': ('s1.ase', F(0), 6), 's1_on_f3': ('s1.ase', F(3), 6), 's1_on_f1': ('s1.ase', F(1), 6),
    's2_on_f0': ('s2.ase', F(0), 8), 's2_on_f3': ('s2.ase', F(3), 8), 's2_on_f1': ('s2.ase', F(1), 8),
    's3_on_f0': ('s3.ase', F(0), 6), 's3_on_f2': ('s3.ase', F(2), 6), 's3_on_f3': ('s3.ase', F(3), 6),
    's3_on_f5': ('s3.ase', F(5), 6),
    's4_on_f0': ('s4.ase', F(0), 6), 's4_on_f3': ('s4.ase', F(3), 6),
    # 1. Subtags desligado (liga-se de novo no ultimo)
    's1_off_f0': ('s1.ase', O('sub') + F(0), 6), 's1_off_f3': ('s1.ase', F(3), 6), 's1_off_f1': ('s1.ase', F(1), 6),
    's2_off_f0': ('s2.ase', F(0), 6), 's2_off_f3': ('s2.ase', F(3), 6),
    's3_off_f0': ('s3.ase', F(0), 6), 's3_off_f3': ('s3.ase', F(3), 6),
    's4_off_f3': ('s4.ase', F(3), 6),
    's1_off_once_f0': ('s1.ase', O('once') + F(0), 5), 's1_off_once_f3': ('s1.ase', F(3) + O('once'), 5),
    's1_on_back': ('s1.ase', O('sub') + F(0), 3),
    # 4. Play Once / Play All / Rewind on Stop
    's1_once_f0': ('s1.ase', O('once') + F(0), 5), 's1_once_f3': ('s1.ase', F(3), 5), 's1_once_f1': ('s1.ase', F(1), 5),
    's2_once_f0': ('s2.ase', F(0), 6), 's0_once_f2': ('s0.ase', F(2), 4),
    's1_once_back': ('s1.ase', O('once') + F(0), 1),
    's1_all_f3': ('s1.ase', O('all') + F(3), 5), 's1_all_once_f3': ('s1.ase', O('once') + F(3), 4),
    's1_all_back': ('s1.ase', O('once') + O('all') + F(0), 1),
    's0_norewind_f2': ('s0.ase', F(2), 2.5),
    's0_rewind_f2': ('s0.ase', O('rewind') + F(2), 2.5),
    's1_rewind_f3': ('s1.ase', F(3), 2.5),
    's1_rewind_back': ('s1.ase', O('rewind') + F(0), 1),
    # 3. Velocidade (s0, 100 ms por quadro)
    's0_v0.5': ('s0.ase', O('0.5') + F(0), 6), 's0_v2': ('s0.ase', O('2') + F(0), 4),
    's0_v3': ('s0.ase', O('3') + F(0), 3), 's0_v0.25': ('s0.ase', O('0.25') + F(0), 8),
    's0_v1.5': ('s0.ase', O('1.5') + F(0), 4),
    's0_v1.5_reaberto': ('s0.ase', F(0), 4),           # a velocidade sobrevive a fechar e reabrir?
    's1_v1.5_outro_doc': ('s1.ase', F(0), 4),          # ... e passa para outro documento?
    's0_v1': ('s0.ase', O('1') + F(0), 3),
}

# Corrida 1 (01/11): os casos s0_* com O(...) tocaram o item de CIMA (ver O); a fixtura regista o
# item realmente clicado. Corrida 2 parte do estado lido no menu (fotos menu_antes_2).
CASOS2 = {
    # estado lido: sub OFF, rewind ON, Play Preview Animation a tocar (o clique errado da corrida 1)
    'fix_s1_f0': ('s1.ase', ['rc:240,820', 'w:0.5', 'c:330,738', 'w:0.5'] + O('sub') + F(0), 3),
    's0_rewind_f2': ('s0.ase', F(2), 2.5), 's1_rewind_f3': ('s1.ase', F(3), 2.5), 's1_rewind_f1': ('s1.ase', F(1), 2.5),
    's1_norewind_f3': ('s1.ase', O('rewind') + F(3), 2.5),
    's1_v0.5': ('s1.ase', O('0.5') + F(0), 5), 's0_v0.5_outro_doc': ('s0.ase', F(0), 5),
    's1_v2': ('s1.ase', O('2') + F(0), 3), 's1_v3': ('s1.ase', O('3') + F(0), 3),
    's1_v0.25': ('s1.ase', O('0.25') + F(0), 6), 's1_v1.5': ('s1.ase', O('1.5') + F(0), 3),
    's1_v1': ('s1.ase', O('1') + F(0), 3),
    's1_once_f2': ('s1.ase', O('once') + F(2), 3), 's1_once_f5': ('s1.ase', F(5), 3), 's1_once_f6': ('s1.ase', F(6), 3),
    's1_once_f3b': ('s1.ase', F(3), 3), 's3_once_f2': ('s3.ase', F(2), 3), 's3_once_f5': ('s3.ase', F(5), 3),
    's2_once_f1': ('s2.ase', F(1), 3), 's2_once_f3': ('s2.ase', F(3), 3), 's0_once_f7': ('s0.ase', F(7), 3),
    's1_once_off': ('s1.ase', O('once') + F(0), 1),
}
CASOS = {**CASOS1, **CASOS2}

if __name__ == '__main__':
    cmd, d = sys.argv[1], os.path.abspath(sys.argv[2])
    if cmd == 'gera':
        gera(d)
    elif cmd == 'abre':
        abre(d, sys.argv[3])
    elif cmd == 'g':
        T.gestos(d, sys.argv[4:]); T.shot(d, sys.argv[3])
    elif cmd == 'toca':
        r = toca(d, sys.argv[3], float(sys.argv[4]), sys.argv[5:])
        print(r['runs_idx_ms'])
    elif cmd in ('caso', 'todos'):
        res = {}
        for nome in ([sys.argv[3]] if cmd == 'caso' else list(CASOS2 if sys.argv[3:] == ['2'] else CASOS1)):
            f, pre, segs = CASOS[nome]
            abre(d, f)
            res[nome] = toca(d, nome, segs, pre)
            print(nome, res[nome]['runs_idx_ms'], flush=True)
        json.dump(res, open(os.path.join(d, ('play%s.json' % ''.join(sys.argv[3:])) if cmd == 'todos' else nome + '_caso.json'), 'w'), indent=1)
