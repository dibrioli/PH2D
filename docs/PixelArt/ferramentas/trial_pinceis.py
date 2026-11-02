"""Pincel de padrao (Edit > New Brush, Ctrl+B) no Aseprite Trial 1.3.18.6, caixa preta (02/11, P7 R-B1..R-B5).

uso: PH2D_TRIAL_DISPLAY=:91 python3 -I trial_pinceis.py <dir> [caso ...]  -> <dir>/pinceis.json
Casos: rb1_* (modos source/destination/paint, ancora 4x2 e 4x3), rb2_* (cor de frente), rb3_* (Shift+H/V),
rb4_* (sem seleccao, elipse, buraco transparente, aparar), rb5_slot (Save Brush Here + Back). Cada caso abre um
sprite NOSSO (64x48, padrao 3x2 RGB/YCP gerado aqui), Ctrl+B, modo, carimbos; grava a grelha de letras lida.

Coordenadas (1600x1000, screen_scale 2, zoom 400 % = 8 px por celula; descobertas por fotografia 02/11):
  dropdown do padrao (542,71) -> source (400,103) · destination (400,121) · Paint brush (400,139)
  botao do tipo de pincel (234,71) -> circle (240,104) · square (272,104) · line (304,104) ·
    slot «Alt+1» (250,145) e o seu dropdown (372,145) · «Save Brush Here» (292,170), dropdown (372,170)
    = Parameters: Brush Type/Size/Angle · Color Foreground/Background/Image Color · Ink Type/Opacity ·
    Extras Shade/Pixel-Perfect;  «Back» (187,71) volta ao pincel normal.
  Quirks: o Open ignora o Return (clica-se OK em (1319,918)) e o dialogo demora: espera-se o titulo;
  Esc com o pincel de imagem activo DESCARTA o pincel; o aviso de fecho tem a barra em (640,437).

Rodada 2 (02/11, passos interactivos sobre estas funcoes; casos rb2b_*/rb5b_*/rb6_* na fixtura):
  HOME novo abre o painel Preview, que estraga o bbox: fecha-se em (1537,723). Cada processo novo tem de
  chamar drv.rebbox() (X0/Y0 sao globais do processo). Com o lapis normal o botao do tipo e (178,71), o
  tamanho (230,71), o angulo (293,71, so com quadrado/linha); no popup o ICONE do espaco (192,145+24n)
  selecciona-o — o rotulo «Alt+n» abre os Keyboard Shortcuts (Cancel em (1327,843)). Com pincel de padrao:
  tinta (580,71) -> Alpha Compositing (663,125) · Lock Alpha (648,169); Opacity (713,71) em %.

Lancador (Xvfb proprio, HOME isolado; aseprite.ini com [GfxMode] Frame = 0 0 1600 1000, screen_scale 2):
  Xvfb :91 -screen 0 1600x1000x24 -nolisten tcp &
  env -u WAYLAND_DISPLAY DISPLAY=:91 SDL_VIDEODRIVER=x11 HOME=<scratch>/home \
      ~/.local/opt/aseprite-trial/Aseprite-trial.AppImage &
"""
import json, os, struct, sys, time, zlib
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_elipse_grelha as drv   # noqa: E402

drv.CW, drv.CH, drv.CELL = 64, 48, 8
W_, H_ = 64, 48
PAT = [[(255, 0, 0), (0, 160, 0), (0, 0, 255)],
       [(255, 220, 0), (0, 200, 220), (160, 0, 200)]]
PX, PY = 8, 8          # celula de cima-esquerda do padrao 3x2
WHITE = (255, 255, 255)


def _string(t):
    b = t.encode(); return struct.pack('<H', len(b)) + b


def write_ase(path, w, h, px):
    """px: lista h x w de (r,g,b,a); uma camada, um cel, RGBA"""
    raw = b''.join(bytes(p) for row in px for p in row)
    ck = drv._chunk
    chunks = [ck(0x2007, struct.pack('<HHI8x', 1, 0, 0)),
              ck(0x2004, struct.pack('<HHHHHHB3x', 3, 0, 0, 0, 0, 0, 255) + _string('src')),
              ck(0x2005, struct.pack('<HhhBHh5x', 0, 0, 0, 255, 2, 0) + struct.pack('<HH', w, h)
                 + zlib.compress(raw))]
    fr = b''.join(chunks)
    body = struct.pack('<IHHH2xI', 16 + len(fr), 0xF1FA, len(chunks), 100, len(chunks)) + fr
    hdr = struct.pack('<IHHHHHIHII', 128 + len(body), 0xA5E0, 1, w, h, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    open(path, 'wb').write(hdr + body)


def sprite_padrao(path, fundo=(255, 255, 255, 255), buraco=None, bloco=None, at=(PX, PY)):
    px = [[fundo] * W_ for _ in range(H_)]
    for j in range(2):
        for i in range(3):
            px[at[1] + j][at[0] + i] = PAT[j][i] + (255,)
    if buraco:
        px[at[1] + buraco[1]][at[0] + buraco[0]] = (0, 0, 0, 0)
    if bloco:   # (x0, y0, x1, y1) castanho opaco, para ver se o transparente do padrao carimba
        x0, y0, x1, y1 = bloco
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                px[y][x] = (90, 60, 30, 255)
    write_ase(path, W_, H_, px)


# ── a tela (coordenadas de ecra a 1600x1000, screen_scale 2, zoom 400 % = 8 px por celula) ─────────
MODO_DD = (542, 71)                                   # dropdown do padrao (so com pincel de imagem)
MODO = {'source': (400, 103), 'destination': (400, 121), 'paint': (400, 139)}
TIPO_BTN = (234, 71)                                  # botao do tipo de pincel (abre o popup)
TIPO = {'circle': (240, 104), 'square': (272, 104), 'line': (304, 104)}
SLOT1, SLOT1_DD, SAVE_HERE, SAVE_DD = (250, 145), (372, 145), (292, 170), (372, 170)
NEUTRO = (800, 983)                                   # barra de estado: fecha popups sem Esc
LETRA = {(255, 0, 0): 'R', (0, 160, 0): 'G', (0, 0, 255): 'B', (255, 220, 0): 'Y', (0, 200, 220): 'C',
         (160, 0, 200): 'P', (255, 255, 255): '.', (0, 0, 0): 'K', (90, 60, 30): 'k'}


def letra(p):
    p = tuple(p)
    if p in LETRA:
        return LETRA[p]
    if p[0] == p[1] == p[2]:
        return 't'      # cinzento do xadrez = transparente
    return '?'


def grelha(nome):
    a = drv.shot(nome + '.png')
    return [''.join(letra(p) for p in row) for row in drv.cells(a)]


def clique(i, j):
    c = drv.cc(i, j)
    drv.xdo('mousemove', *c); drv._settle(c); drv.xdo('mousedown', 1); time.sleep(0.08)
    drv.xdo('mouseup', 1); time.sleep(0.2)


def modo(m):
    drv.xdo('mousemove', *MODO_DD); time.sleep(0.2); drv.xdo('click', 1); time.sleep(0.6)
    drv.xdo('mousemove', *MODO[m]); time.sleep(0.2); drv.xdo('click', 1); time.sleep(0.4)


def novo_pincel(sel, forma='rect'):
    """marquee (rect) ou elipse sobre sel=(x0,y0,x1,y1); sel=None = sem seleccao; depois Ctrl+B"""
    if sel is None:
        drv.key('ctrl+d')
    else:
        drv.key('shift+m' if forma == 'elipse' else 'm')
        drv.drag(drv.cc(sel[0], sel[1]), drv.cc(sel[2], sel[3]))
    drv.key('ctrl+b', t=0.8)
    drv.key('b')


def caso(nome, sprite, sel, passos, forma='rect', modo_=None, extra=None, depois=None):
    abre(sprite)
    drv.key('Escape')
    novo_pincel(sel, forma)
    antes = drv.shot(nome + '_ctx.png', park=False)
    if modo_:
        modo(modo_)
    if extra:
        extra(nome)
    for p in passos:
        if p[0] == 'clique':
            clique(*p[1])
        else:
            drv.drag(drv.cc(*p[1]), drv.cc(*p[2]), steps=12)
    g = grelha(nome)
    if depois:
        g = [g, depois(nome)]
    fecha()
    return {'sprite': sprite, 'selecao': sel, 'forma_selecao': forma, 'modo': modo_, 'passos': passos,
            'grelha': g}



def fecha():
    """Ctrl+W; o aviso «Saving changes» tem a barra de titulo em (640, 437): «Don't Save»"""
    drv.xdo('key', 'ctrl+w'); time.sleep(0.9)
    c = drv.shot('z2.png', park=False)
    if tuple(c[437, 640]) == (124, 144, 159):
        drv.xdo('mousemove', 799, 546, 'click', 1); time.sleep(0.6)


def abre(nome):
    """Ctrl+O e ESPERA o dialogo (o titulo «Open» em (110, 66)) antes de escrever: o atraso do dialogo
    mandou o caminho como atalhos para o editor (medido 02/11: abriu o Canvas Size)"""
    drv.xdo('mousemove', 856, 523); drv.xdo('key', 'ctrl+o')
    t0 = time.time()
    while tuple(drv.shot('z0.png', park=False)[66, 110]) != (124, 144, 159):
        assert time.time() - t0 < 10, 'o dialogo Open nao abriu'
        time.sleep(0.3)
    time.sleep(0.4)
    drv.xdo('type', '--delay', 12, os.path.join(drv.D, nome)); time.sleep(0.4)
    drv.xdo('mousemove', 1319, 918, 'click', 1); time.sleep(1.8)   # OK (o Return foi ignorado 02/11)
    assert tuple(drv.shot('z0.png', park=False)[66, 110]) != (124, 144, 159), 'o Open ficou aberto'
    drv.xdo('mousemove', 856, 523); drv.xdo('key', '1'); time.sleep(0.6)
    x0, y0, x1, y1 = drv.bbox(drv.shot('z1.png'))
    drv.xdo('mousemove', (x0 + x1) // 2, (y0 + y1) // 2); drv.xdo('key', '3'); time.sleep(0.8)
    return drv.rebbox('z1b.png')


CLIQUES = [(30, 20), (41, 29), (52, 37)]
TRACOS = [('traco', (10, 42), (22, 42)), ('traco', (58, 6), (58, 16))]


def casos():
    c = {}
    for m in ('source', 'destination', 'paint'):
        c[f'rb1_{m}'] = dict(sprite='pad.ase', sel=(8, 8, 10, 9), modo_=m,
                             passos=[('clique', p) for p in CLIQUES] + TRACOS)
    c['rb1_source_b'] = dict(sprite='pad_b.ase', sel=(13, 9, 15, 10), modo_='source',
                             passos=[('clique', p) for p in CLIQUES] + TRACOS)
    c['rb1_destination_b'] = dict(sprite='pad_b.ase', sel=(13, 9, 15, 10), modo_='destination',
                                  passos=[('clique', p) for p in CLIQUES] + TRACOS)
    # ancora com largura par: seleccao 4x2 = padrao + uma coluna branca a direita
    c['rb1_ancora_4x2'] = dict(sprite='pad.ase', sel=(8, 8, 11, 9), modo_='paint',
                               passos=[('clique', p) for p in CLIQUES])
    c['rb1_ancora_4x3'] = dict(sprite='pad.ase', sel=(8, 7, 11, 9), modo_='paint',
                               passos=[('clique', p) for p in CLIQUES])
    # R-B2: cor de frente depois do Ctrl+B (azul da paleta), carimbo; depois o botao «Back»
    c['rb2_cor_frente'] = dict(sprite='pad.ase', sel=(8, 8, 10, 9), modo_='source',
                               extra=lambda n: (cor_paleta(n)), passos=[('clique', (30, 20))])
    # R-B3: Shift+H / Shift+V com o pincel activo (modo Paint brush), e o menu Edit fotografado
    c['rb3_shift_h'] = dict(sprite='pad.ase', sel=(8, 8, 10, 9), modo_='paint',
                            extra=lambda n: (menu_edit(n), drv.key('shift+h', t=0.6)), passos=[('clique', (30, 20))])
    c['rb3_shift_v'] = dict(sprite='pad.ase', sel=(8, 8, 10, 9), modo_='paint',
                            extra=lambda n: drv.key('shift+v', t=0.6), passos=[('clique', (30, 20))])
    # R-B4: sem seleccao; elipse; buraco transparente no padrao; tudo sobre fundo TRANSPARENTE
    # com um bloco castanho opaco onde se carimba
    c['rb4_sem_selecao'] = dict(sprite='pad_t.ase', sel=None, modo_=None, passos=[('clique', (30, 20))])
    c['rb4_elipse'] = dict(sprite='pad_t.ase', sel=(7, 7, 11, 10), forma='elipse', modo_='paint',
                           passos=[('clique', (30, 20))])
    c['rb4_rect_maior'] = dict(sprite='pad_t.ase', sel=(7, 7, 11, 10), modo_='paint',
                               passos=[('clique', (30, 20))])
    c['rb4_buraco'] = dict(sprite='pad_buraco.ase', sel=(8, 8, 10, 9), modo_='paint',
                           passos=[('clique', (30, 20))])
    c['rb4_buraco_source'] = dict(sprite='pad_buraco.ase', sel=(8, 8, 10, 9), modo_='source',
                                  passos=[('clique', (30, 20))])
    c['rb2_cor_frente_bis'] = dict(c['rb2_cor_frente'])
    c['rb2_cor_frente_paint'] = dict(sprite='pad.ase', sel=(8, 8, 10, 9), modo_='paint',
                                     extra=lambda n: (cor_paleta(n)), passos=[('clique', (30, 20))])
    # R-B4: mascara eliptica sobre fundo BRANCO opaco (o que fica fora da elipse entra no pincel?)
    c['rb4_elipse_branco'] = dict(sprite='pad_wb.ase', sel=(7, 7, 11, 10), forma='elipse', modo_='paint',
                                  passos=[('clique', (30, 20))])
    # R-B4: o transparente a volta e aparado? seleccao 6x2 com o padrao a esquerda, fundo transparente
    c['rb4_apara_6x2'] = dict(sprite='pad_t.ase', sel=(8, 8, 13, 9), modo_='paint', passos=[('clique', (30, 20))])
    # R-B5: Save Brush Here, voltar ao pincel normal (Back), escolher o slot, carimbar
    c['rb5_slot'] = dict(sprite='pad.ase', sel=(8, 8, 10, 9), modo_='paint', extra=salva_e_volta,
                         passos=[('clique', (30, 20))])
    return c


def cor_paleta(n):
    drv.xdo('mousemove', 118, 110); time.sleep(0.2); drv.xdo('click', 1); time.sleep(0.5)
    drv.shot(n + '_fg.png', park=False)


def menu_edit(n):
    drv.xdo('mousemove', 65, 10); time.sleep(0.2); drv.xdo('click', 1); time.sleep(0.7)
    drv.shot(n + '_menu_edit.png', park=False)
    drv.xdo('mousemove', 65, 10); time.sleep(0.2); drv.xdo('click', 1); time.sleep(0.4)


def salva_e_volta(n):
    drv.xdo('mousemove', *TIPO_BTN); time.sleep(0.2); drv.xdo('click', 1); time.sleep(0.7)
    drv.shot(n + '_popup0.png', park=False)
    drv.xdo('mousemove', *SAVE_HERE); time.sleep(0.2); drv.xdo('click', 1); time.sleep(0.7)
    drv.shot(n + '_popup1.png', park=False)
    drv.xdo('mousemove', *NEUTRO); time.sleep(0.2); drv.xdo('click', 1); time.sleep(0.4)
    drv.xdo('mousemove', 187, 71); time.sleep(0.2); drv.xdo('click', 1); time.sleep(0.6)   # «Back»
    drv.shot(n + '_back.png', park=False)
    drv.xdo('mousemove', *TIPO_BTN); time.sleep(0.2); drv.xdo('click', 1); time.sleep(0.7)
    drv.shot(n + '_popup2.png', park=False)


def sprites():
    sprite_padrao(os.path.join(drv.D, 'pad.ase'))
    sprite_padrao(os.path.join(drv.D, 'pad_b.ase'), at=(13, 9))
    T = (0, 0, 0, 0)
    sprite_padrao(os.path.join(drv.D, 'pad_wb.ase'), bloco=(24, 14, 37, 26))
    sprite_padrao(os.path.join(drv.D, 'pad_t.ase'), fundo=T, bloco=(24, 14, 37, 26))
    sprite_padrao(os.path.join(drv.D, 'pad_buraco.ase'), fundo=T, buraco=(1, 0), bloco=(24, 14, 37, 26))


def main():
    out_p = os.path.join(drv.D, 'pinceis.json')
    out = json.load(open(out_p)) if os.path.exists(out_p) else {}
    sprites()
    todos = casos()
    for n in (sys.argv[2:] or list(todos)):
        t = time.time()
        out[n] = caso(n, **todos[n])
        json.dump(out, open(out_p, 'w'), indent=1)
        print(n, f'{time.time() - t:.0f}s', flush=True)


if __name__ == '__main__':
    main()
