"""P10c M10: Export Sprite Sheet do Aseprite Trial 1.3.18.6 (Source: Sprite Grid, By Grid) sobre .ase NOSSOS; caixa preta, le-se a tela.
uso: python3 trial_tiles_p10c_folha.py gerar <dir>
     python3 trial_tiles_p10c_folha.py lote <dir> [filtro]    todos os casos (Xvfb :101.., HOME limpo); grava <dir>/m10.json e <dir>/<caso>/fim.png
     python3 trial_tiles_p10c_folha.py sessao <dir> <ase> <saida> "<passos>"   uma sessao livre (Xvfb :87)
passos, separados por ';': key K | click X Y | drag X0 Y0 X1 Y1 | wait S | type T | shot NOME
A folha le-se no EDITOR a 100 % (2 px de tela por pixel, screen_scale 2): o rectangulo preto da borda, com o dialogo arrastado para o canto.
A barra de estado NAO serve: fica em «40 24» com o dialogo aberto (medido).
Formato: spec publica do .ase, como trial_tiles.py. O Trial nao grava. Nunca o ecra do dono."""
import json, os, queue, struct, sys, threading, time, zlib
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
from PIL import Image
import trial_tiles_m9 as m

W, H, TW, TH = 40, 24, 16, 8       # 2,5 colunas x 3 linhas: a ultima coluna e cortada pelo canvas
COR = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 0), (255, 0, 255), (0, 255, 255), (255, 128, 0), (128, 0, 255), (255, 255, 255)]
NUM = {(x, y): y * 3 + x + 1 for y in range(3) for x in range(3)}

def _s(t):
    b = t.encode(); return struct.pack('<H', len(b)) + b
def _c(kind, data):
    return struct.pack('<IH', len(data) + 6, kind) + data

def escreve(path, chunks, g):
    fr = b''.join(chunks)
    body = struct.pack('<IHHH2xI', 16 + len(fr), 0xF1FA, len(chunks), 100, len(chunks)) + fr
    hdr = struct.pack('<IHHHHHIHII', 128 + len(body), 0xA5E0, 1, W, H, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, g[0], g[1], g[2], g[3])
    hdr += b'\0' * (128 - len(hdr))
    open(path, 'wb').write(hdr + body)

def tilemap(path, tiles, grid, g, cel=(0, 0), cols=3, rows=3):
    """tiles = lista de cor (cheio) ou {(x,y): cor}; grid = {(cx,cy): id}; g = grelha do SPRITE (x,y,w,h). Devolve a imagem verdadeira."""
    raw = bytes(TW * TH * 4)
    for t in tiles:
        for y in range(TH):
            for x in range(TW):
                c = t if isinstance(t, tuple) else t.get((x, y))
                raw += bytes((*c, 255)) if c else bytes(4)
    z = zlib.compress(raw)
    ts = _c(0x2023, struct.pack('<IIIHHh14x', 0, 2 | 4, len(tiles) + 1, TW, TH, 1) + _s('ts') + struct.pack('<I', len(z)) + z)
    ly = _c(0x2004, struct.pack('<HHHHHHB3x', 3, 2, 0, 0, 0, 0, 255) + _s('tm') + struct.pack('<I', 0))
    data = b''.join(struct.pack('<I', grid.get((x, y), 0)) for y in range(rows) for x in range(cols))
    ce = _c(0x2005, struct.pack('<HhhBHh5x', 0, cel[0], cel[1], 255, 3, 0)
            + struct.pack('<HHHIIII10x', cols, rows, 32, 0x1fffffff, 0x80000000, 0x40000000, 0x20000000) + zlib.compress(data))
    escreve(path, [_c(0x2007, struct.pack('<HHI8x', 1, 0, 0)), ts, ly, ce], g)
    im = np.zeros((H, W, 4), int)
    for (cx, cy), i in grid.items():
        t = tiles[i - 1]
        for y in range(TH):
            for x in range(TW):
                c = t if isinstance(t, tuple) else t.get((x, y))
                px, py = cel[0] + cx * TW + x, cel[1] + cy * TH + y
                if c and 0 <= px < W and 0 <= py < H: im[py, px] = (*c, 255)
    return im

def raster(path, pix, g):
    raw = b''.join(bytes((*pix(x, y), 255)) if pix(x, y) else bytes(4) for y in range(H) for x in range(W))
    ly = _c(0x2004, struct.pack('<HHHHHHB3x', 3, 0, 0, 0, 0, 0, 255) + _s('ras'))
    ce = _c(0x2005, struct.pack('<HhhBHh5x', 0, 0, 0, 255, 2, 0) + struct.pack('<HH', W, H) + zlib.compress(raw))
    escreve(path, [_c(0x2007, struct.pack('<HHI8x', 1, 0, 0)), ly, ce], g)
    im = np.zeros((H, W, 4), int)
    for y in range(H):
        for x in range(W):
            if pix(x, y): im[y, x] = (*pix(x, y), 255)
    return im

G106, G117, G168, G168F, G106F = (0, 0, 10, 6), (0, 0, 11, 7), (0, 0, 16, 8), (3, 2, 16, 8), (3, 2, 10, 6)
PIX = [{(5, 3): COR[0]}]              # pixel solitario: no sprite em (16+5, 8+3) = (21, 11)
RAMPA = lambda x, y: (x * 6, y * 10, 128)
ARQ = {  # nome: (tipo, args, grelha do sprite, fase da grelha do tile ou None)
    'num': ('t', (COR, NUM, (0, 0)), G106), 'num_g117': ('t', (COR, NUM, (0, 0)), G117), 'num_g168f': ('t', (COR, NUM, (0, 0)), G168F),
    'fase': ('t', (COR, NUM, (3, 2)), G106), 'so11': ('t', ([COR[0]], {(1, 1): 1}, (0, 0)), G106), 'so11_g168': ('t', ([COR[0]], {(1, 1): 1}, (0, 0)), G168),
    'pix': ('t', (PIX, {(1, 1): 1}, (0, 0)), G106), 'pix_g168': ('t', (PIX, {(1, 1): 1}, (0, 0)), G168), 'pix_g106f': ('t', (PIX, {(1, 1): 1}, (0, 0)), G106F),
    'ras': ('r', RAMPA, G106), 'ras_g168': ('r', RAMPA, G168), 'ras_g168f': ('r', RAMPA, G168F),
    'raspix': ('r', lambda x, y: COR[0] if (x, y) == (21, 11) else None, G106),
}
def gerar(D):
    os.makedirs(D, exist_ok=True)
    out = {}
    for k, (tp, a, g) in ARQ.items():
        p = f'{D}/{k}.ase'
        out[k] = tilemap(p, a[0], a[1], g, cel=a[2]) if tp == 't' else raster(p, a, g)
    return out

# --- o dialogo, arrastado 400,280 para o canto (coordenadas medidas nas fotos)
ABRE = 'key ctrl+e; wait 2; drag 700 392 300 112; wait 1;'
T_LAY, T_SPR, T_BOR = 'click 292 143;', 'click 349 143;', 'click 412 143;'
SG = T_SPR + 'click 523 183; wait .5; click 250 234; wait 1;'         # Source: Sprite Grid
FAIXA = T_LAY + 'click 665 183; wait .5; click 380 216; wait 1;'      # Sheet Type: Horizontal Strip
IE = T_LAY + 'click 427 260; wait 1;'                                 # Ignore Empty
TS, TC, BG = 'click 397 180; wait 1;', 'click 397 212; wait 1;', 'click 397 243; wait 1;'  # Trim Sprite, Trim Cels, By Grid (na aba Borders)
CASOS = {
    'a_num_sg': ('num', SG + FAIXA), 'a_num_g117_sg': ('num_g117', SG + FAIXA), 'a_num_g168f_sg': ('num_g168f', SG + FAIXA),
    'a_fase_sg': ('fase', SG + FAIXA), 'a_ras_sg': ('ras', SG + FAIXA), 'a_ras_g168_sg': ('ras_g168', SG + FAIXA), 'a_ras_g168f_sg': ('ras_g168f', SG + FAIXA),
    'a_so11_sg': ('so11', SG + FAIXA), 'a_so11_sg_ie': ('so11', SG + FAIXA + IE),
    'a_so11t_sg': ('so11_g168', SG + FAIXA), 'a_so11t_sg_ie': ('so11_g168', SG + FAIXA + IE),
    'a_pix_sg_ie': ('pix', SG + FAIXA + IE), 'a_pixt_sg_ie': ('pix_g168', SG + FAIXA + IE),
    'c_pix_base': ('pix', ''), 'c_pix_ts': ('pix', T_BOR + TS), 'c_pix_tc': ('pix', T_BOR + TC),
    'c_pix_ts_bg': ('pix', T_BOR + TS + BG), 'c_pix_tc_bg': ('pix', T_BOR + TC + BG), 'c_pix_tstc_bg': ('pix', T_BOR + TS + TC + BG),
    'c_pix_tc_bg_tiraTC': ('pix', T_BOR + TC + BG + TC), 'c_pixt_ts_bg': ('pix_g168', T_BOR + TS + BG), 'c_pixt_tc_bg': ('pix_g168', T_BOR + TC + BG),
    'c_pixf_ts_bg': ('pix_g106f', T_BOR + TS + BG), 'c_raspix_ts_bg': ('raspix', T_BOR + TS + BG), 'c_raspix_tc_bg': ('raspix', T_BOR + TC + BG),
    'c_pix_sg_tc': ('pix', SG + FAIXA + T_BOR + TC), 'c_pix_sg_tc_bg': ('pix', SG + FAIXA + T_BOR + TC + BG), 'c_pix_sg_ts_bg': ('pix', SG + FAIXA + T_BOR + TS + BG),
    'c_pixt_sg_tc_bg': ('pix_g168', SG + FAIXA + T_BOR + TC + BG),
    'e_so11t_ie_depois_sg': ('so11_g168', IE + SG), 'e_so11t_sg_ie_sem_faixa': ('so11_g168', SG + IE), 'e_so11t_so_ie': ('so11_g168', IE),
    'e_ras168_sg_ie': ('ras_g168', SG + IE), 'e_pix_sg_ts': ('pix', SG + T_BOR + TS), 'e_pix_sg_ts_bg': ('pix', SG + T_BOR + TS + BG), 'e_so11t_sg_ts': ('so11_g168', SG + T_BOR + TS),
}

# View > Grid > Grid Settings: o dialogo centrado (coordenadas medidas); F = campo, valor
DLG = 'click 379 10; wait .5; click 393 164; wait 1; click 710 163; wait 1;'
def campos(x, y, w, h):
    t = ''
    for (cx, cy), v in zip(((745, 462), (915, 462), (745, 500), (915, 500)), (x, y, w, h)):
        t += f'click {cx} {cy}; key ctrl+a; type {v};'
    return DLG + 'shot dlg;' + t + 'wait .5; shot dlg2; click 775 554; wait 1;'
PRE = {'g_num_ok': (DLG + 'shot dlg; click 775 554; wait 1;', (16, 8, 16, 8)), 'g_ras_ok': (DLG + 'shot dlg; click 775 554; wait 1;', None),
       'g_fase_ok': (DLG + 'shot dlg; click 775 554; wait 1;', (19, 10, 16, 8)), 'g_num_8x4': (campos(0, 0, 8, 4), (0, 0, 8, 4)), 'g_ras_8x4': (campos(0, 0, 8, 4), (0, 0, 8, 4)),
       'g_pix_8x4_ts_bg': (campos(0, 0, 8, 4), (0, 0, 8, 4)), 'g_pix_8x4_tc_bg': (campos(0, 0, 8, 4), (0, 0, 8, 4))}
CASOS.update({'g_num_ok': ('num', SG + FAIXA), 'g_ras_ok': ('ras', SG + FAIXA), 'g_fase_ok': ('fase', SG + FAIXA), 'g_num_8x4': ('num', SG + FAIXA), 'g_ras_8x4': ('ras', SG + FAIXA),
              'g_pix_8x4_ts_bg': ('pix', T_BOR + TS + BG), 'g_pix_8x4_tc_bg': ('pix', T_BOR + TC + BG)})

def passos(s, txt):
    for p in (q.strip() for q in txt.split(';')):
        if not p: continue
        a = p.split(' ', 1); op = a[0]; r = a[1] if len(a) > 1 else ''
        if op == 'key': s.key(*r.split())
        elif op == 'click':
            x, y = r.split(); s.x('mousemove', x, y); time.sleep(.15); s.x('click', 1); time.sleep(.5)
        elif op == 'drag':
            v = list(map(int, r.split())); s.drag(v[:2], v[2:], step=40)
        elif op == 'wait': time.sleep(float(r))
        elif op == 'type': s.x('type', r); time.sleep(.3)
        elif op == 'shot': s.shot(r + '.png', park=False)
        else: raise SystemExit('passo? ' + p)

def sessao(D, caso, saida, txt):
    w = os.path.join(D, saida)
    s = m.S(w, os.environ.get('M10_DISP', '87'), os.path.join(D, caso + '.ase'))
    try: passos(s, txt)
    finally:
        s.ap.terminate(); time.sleep(.3); s.xv.terminate()

X0, X1, Y0, Y1 = 170, 1545, 355, 705   # o editor, abaixo do dialogo e acima da janela Preview
def le(a):
    """Devolve a folha lida (h, w, 4), ou None. Sprite = dentro do rectangulo preto, 2 px de tela por pixel."""
    reg = a[Y0:Y1, X0:X1]
    pre = (reg == 0).all(axis=2)
    if not pre.any(): return None
    ys, xs = np.nonzero(pre)
    x0, x1, y0, y1 = xs.min(), xs.max(), ys.min(), ys.max()
    # a borda tem 2 px de tela de espessura em cada lado (medido: 160x6 dao 324x16 de preto a preto)
    ins = reg[y0 + 2:y1 - 1, x0 + 2:x1 - 1]
    h, w = ins.shape[0] // 2, ins.shape[1] // 2
    out = np.zeros((h, w, 4), int)
    for y in range(h):
        for x in range(w):
            c = ins[y * 2, x * 2]
            cinza = c[0] == c[1] == c[2] and c[0] != 255
            out[y, x] = (0, 0, 0, 0) if cinza else (*c, 255)
    return out, (int(ins.shape[1]), int(ins.shape[0]))

def pedacos(im, g):
    """As candidatas: pedacos da grelha g, por linha, cortados pelo canvas."""
    ox, oy, gw, gh = g
    xs = list(range(ox % gw - gw if ox % gw else 0, W, gw)); ys = list(range(oy % gh - gh if oy % gh else 0, H, gh))
    return [im[max(y, 0):min(y + gh, H), max(x, 0):min(x + gw, W)] for y in ys for x in xs]

def faixa(pcs):
    if not pcs: return np.zeros((0, 0, 4), int)
    h, w = max(p.shape[0] for p in pcs), sum(p.shape[1] for p in pcs)
    o = np.zeros((h, w, 4), int); x = 0
    for p in pcs:
        o[:p.shape[0], x:x + p.shape[1]] = p; x += p.shape[1]
    return o

def candidatas(im, sg, cel):
    H_ = {}
    tg = (cel[0], cel[1], TW, TH)
    for nome, g in (('sprite', sg), ('tile', tg)):
        ps = pedacos(im, g)
        H_[nome + '_cortada'] = faixa(ps); H_[nome + '_cortada_sem_vazias'] = faixa([p for p in ps if p[..., 3].any()])
        # o pedaco de tamanho INTEIRO, com a parte fora do canvas transparente
        ox, oy, gw, gh = g
        cheia = []
        for y in range(oy % gh - gh if oy % gh else 0, H, gh):
            for x in range(ox % gw - gw if ox % gw else 0, W, gw):
                q = np.zeros((gh, gw, 4), int)
                for j in range(gh):
                    for i in range(gw):
                        if 0 <= x + i < W and 0 <= y + j < H: q[j, i] = im[y + j, x + i]
                cheia.append(q)
        H_[nome + '_inteira'] = faixa(cheia); H_[nome + '_inteira_sem_vazias'] = faixa([p for p in cheia if p[..., 3].any()])
    # so as celulas que CABEM a direita e em baixo; a esquerda e em cima a celula que comeca fora do canvas fica, transparente la fora
    ox, oy, gw, gh = sg
    cab = []
    for y in range(oy % gh - gh if oy % gh else 0, H - gh + 1, gh):
        for x in range(ox % gw - gw if ox % gw else 0, W - gw + 1, gw):
            q = np.zeros((gh, gw, 4), int)
            for j in range(gh):
                for i in range(gw):
                    if 0 <= x + i < W and 0 <= y + j < H: q[j, i] = im[y + j, x + i]
            cab.append(q)
    H_['sprite_so_as_que_cabem'] = faixa(cab); H_['sprite_so_as_que_cabem_sem_vazias'] = faixa([p for p in cab if p[..., 3].any()])
    return H_

def lote(D, filt, par):
    verd = gerar(D)
    res = {}
    Q = queue.Queue()
    for k, v in CASOS.items():
        if filt in k: Q.put((k, v))
    def corre(k, arq, txt, disp):
        w = os.path.join(D, k)
        s = m.S(w, disp, os.path.join(D, arq + '.ase'))
        try:
            passos(s, PRE.get(k, ('', None))[0] + ABRE + txt)
            # o titulo diz «(Preview N%)» enquanto a folha nao esta pronta: a foto antes disso le a folha VELHA (medido)
            for i in range(60):
                time.sleep(.5); s.x('mousemove', 800, 800 + i % 2); a = s.shot('fim.png', park=False)  # o rato a mexer: eventos para a tarefa da UI
                if (a[104:120, 295:400] == a[104, 295]).all(): break
            else: raise RuntimeError('preview nunca acabou')
            time.sleep(1); a = s.shot('fim.png')
        finally:
            s.ap.terminate(); time.sleep(.3); s.xv.terminate()
        r = le(a)
        if r is None: res[k] = {'erro': 'sem rectangulo preto'}; return
        out, tela = r
        ys, xs = np.nonzero(out[..., 3])
        Image.fromarray(out.astype('uint8'), 'RGBA').save(os.path.join(w, 'folha.png'))
        tipo, args, g = ARQ[arq]
        g = PRE.get(k, ('', None))[1] or g
        cel = args[2] if tipo == 't' else (0, 0)
        cas = [n for n, e in candidatas(verd[arq], g, cel).items() if e.shape == out.shape and (e == out).all()]
        res[k] = {'arq': arq, 'grelha_sprite': g, 'tela': tela, 'folha': [out.shape[1], out.shape[0]], 'opacos': int(len(xs)),
                  'bbox_opacos': [int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max())] if len(xs) else None, 'casa_com': cas}
    def op(disp):
        while True:
            try: k, (arq, txt) = Q.get_nowait()
            except queue.Empty: return
            try: corre(k, arq, txt, disp)
            except Exception as e: res[k] = {'erro': repr(e)}
    ts = [threading.Thread(target=op, args=(101 + i,)) for i in range(par)]
    [t.start() for t in ts]; [t.join() for t in ts]
    json.dump(res, open(os.path.join(D, f'm10{filt}.json'), 'w'), indent=0)
    for k in sorted(res): print(k, res[k])

def rele(D):
    verd = gerar(D)
    for k, (arq, _) in sorted(CASOS.items()):
        f = os.path.join(D, k, 'fim.png')
        if not os.path.exists(f): print(k, 'sem foto'); continue
        a = np.asarray(Image.open(f).convert('RGB')).astype(int)
        r = le(a)
        if r is None: print(k, 'sem rectangulo'); continue
        out, tela = r
        ys, xs = np.nonzero(out[..., 3])
        tipo, args, g = ARQ[arq]
        g = PRE.get(k, ('', None))[1] or g
        cel = args[2] if tipo == 't' else (0, 0)
        cas = [n for n, e in candidatas(verd[arq], g, cel).items() if e.shape == out.shape and (e == out).all()]
        print(k, 'grelha', g, 'folha', out.shape[1], 'x', out.shape[0], 'opacos', len(xs), 'bbox', (xs.min(), ys.min(), xs.max(), ys.max()) if len(xs) else None, 'casa', cas)

if __name__ == '__main__':
    cmd, D = sys.argv[1], os.path.abspath(sys.argv[2])
    if cmd == 'gerar': gerar(D)
    elif cmd == 'lote': lote(D, sys.argv[3] if len(sys.argv) > 3 else '', int(os.environ.get('M10_PAR', '4')))
    elif cmd == 'rele': rele(D)
    elif cmd == 'sessao': sessao(D, sys.argv[3], sys.argv[4], sys.argv[5])
