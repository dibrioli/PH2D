#!/usr/bin/env python3
"""Oraculo do Select > Load from MSK file / Save to MSK file: o LibreSprite 1.3 (GPL-2, o ancestral
aberto do Aseprite; so CORRIDO, nunca lido) guarda seleccoes NOSSAS em `.msk` e le `.msk` que NOS
fabricamos; a saida vira a fixtura `crates/ph2d-pixel-io/tests/fixtures/oracle_msk.json`.

Dois sentidos, a mesma sessao por caso:
  * ESCRITA — o LibreSprite faz a seleccao com as ferramentas DELE (API JS `selection.select(x,y,w,h)`
    / `selectAll()`, ou Select > Color Range sobre os pixeis brancos de uma imagem nossa) e grava-a.
  * LEITURA — Select > Load from MSK file sobre um ficheiro que montamos (formato inferido das
    escritas, + variantes: bits de enchimento a 0, x/y negativos, maior que a tela, o legado de
    8000 bytes 320x200, cabecalhos estragados) e grava-a de volta.
  Em ambos, depois de gravar: Edit > Clear (Del) sobre a imagem opaca e File > Save As PNG — os pixeis
  que mudaram SAO a seleccao que o app tem (prova independente do ficheiro).

A API JS do LibreSprite nao le a mascara (so `bounds`), por isso tudo vai pelo MENU num Xvfb proprio
(nunca o ecra do dono: `WAYLAND_DISPLAY` fora e `SDL_VIDEODRIVER=x11`), HOME isolado, coordenadas do
menu medidas em fotos a 800x600 (janela movida para 0,0).

    python3 -I docs/PixelArt/ferramentas/oraculo_msk.py <pasta de trabalho> \
        > crates/ph2d-pixel-io/tests/fixtures/oracle_msk.json

Precisa de: libresprite, Xvfb, xdotool, ImageMagick (`import`), Pillow. Display `PH2D_ORACLE_DISPLAY`
(omissao `:79`, para nao colidir com o `oraculo_libresprite.py` em `:77`).
"""
import datetime
import json
import math
import os
import struct
import subprocess
import sys
import time

from PIL import Image

WORK = os.path.abspath(sys.argv[1])
DISPLAY = os.environ.get('PH2D_ORACLE_DISPLAY', ':79')
MENU_SELECT = (307, 14)
ITEM_COLOR_RANGE = (345, 168)
ITEM_LOAD = (370, 232)
ITEM_SAVE = (360, 260)
COLOR_RANGE_OK = (348, 378)
DIALOG_OK = (560, 536)
# Um dialogo de ficheiro pinta (400,300) a (58,58,58); a tela vazia e (37,37,38); o alerta, (83,83,83).
DIALOG_PROBE = (400, 300)
DIALOG_GRAY = (58, 58, 58)
RED = (255, 0, 0, 255)


def xenv():
    return {**os.environ, 'DISPLAY': DISPLAY}


def xdo(*args):
    subprocess.run(['xdotool', *map(str, args)], env=xenv(), check=True)


def click(pt, pause=0.6):
    xdo('mousemove', pt[0], pt[1], 'click', '1')
    time.sleep(pause)


def shot(path):
    subprocess.run(['import', '-window', 'root', path], env=xenv(), check=True)
    return Image.open(path).convert('RGB')


# O caminho DIGITADO vai por um atalho curto: um caminho de 120 letras perdia uma letra e o app
# respondia «Cannot open» (medido 27/10 no caso 400x260).
SHORT = f'/tmp/msk{os.getpid()}'


def type_path(path):
    xdo('key', 'ctrl+a')
    # Digitado depressa perde letras (medido no oraculo_libresprite.py).
    xdo('type', '--delay', '90', path.replace(WORK, SHORT))
    time.sleep(0.4)


class Session:
    def __init__(self, png, js):
        env = {k: v for k, v in os.environ.items() if k != 'WAYLAND_DISPLAY'}
        env.update(DISPLAY=DISPLAY, SDL_VIDEODRIVER='x11', HOME=os.path.join(WORK, 'home'))
        self.p = subprocess.Popen(['libresprite', png, '--script', js], env=env,
                                  stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        w = []
        for _ in range(60):
            time.sleep(0.25)
            w = subprocess.run(['xdotool', 'search', '--name', 'LibreSprite'], env=xenv(),
                               capture_output=True, text=True).stdout.split()
            if w:
                break
        time.sleep(1.5)
        xdo('windowmove', w[0], 0, 0)
        time.sleep(0.8)

    def kill(self):
        # SIGTERM nao o fecha (medido): fica vivo e a proxima busca de janela apanha-o.
        self.p.kill()
        self.p.wait()

    def color_range(self):
        click(MENU_SELECT)
        click(ITEM_COLOR_RANGE, 1.0)
        click(COLOR_RANGE_OK, 0.8)

    def load_msk(self, path, tag):
        """`'loaded'`; `'error'` (a consola «Error loading .msk file»); `'hang'` (o app deixa de
        responder com o OK premido — medido com um bloco de tamanho 0 depois da mascara)."""
        click(MENU_SELECT)
        click(ITEM_LOAD, 1.0)
        if screen_state(tag + '_open') != 'file_dialog':
            raise RuntimeError('o dialogo Load .msk nao abriu')
        type_path(path)
        try:
            r = 'loaded' if submit(None, tag + '_load') == 'done' else 'error'
        except Hang:
            return 'hang'
        return r

    def save_msk(self, path, tag):
        """`'saved'`; `'no_dialog'` (o item nao abriu o dialogo); `'error'` (o app respondeu
        «Error saving .msk file» — medido sem seleccao)."""
        if os.path.exists(path):
            os.remove(path)
        click(MENU_SELECT)
        click(ITEM_SAVE, 1.0)
        if screen_state(tag + '_open') != 'file_dialog':
            xdo('key', 'Escape')
            time.sleep(0.4)
            return 'no_dialog'
        type_path(path)
        return 'saved' if submit(path, tag + '_save') == 'done' else 'error'

    def clear_and_save_png(self, path):
        if os.path.exists(path):
            os.remove(path)
        xdo('key', 'Delete')
        time.sleep(0.5)
        for k in range(3):
            xdo('key', 'ctrl+shift+s')
            time.sleep(1.2)
            type_path(path)
            if submit(path, f'{os.path.basename(path)}_{k}') == 'done':
                return
        raise RuntimeError(f'o LibreSprite nao gravou {path}')


def screen_state(tag):
    """O que esta a frente, por pixeis medidos nas fotos (800x600, janela em 0,0)."""
    im = shot(os.path.join(WORK, f'{tag}.png'))
    if im.getpixel((100, 136)) == (33, 33, 33) and im.getpixel((433, 446)) == (70, 70, 70):
        return 'console'
    if im.getpixel((390, 245)) == (83, 83, 83) and im.getpixel((390, 325)) == (83, 83, 83):
        return 'alert'
    if im.getpixel((300, 46)) == (33, 33, 33) and im.getpixel(DIALOG_PROBE) == DIALOG_GRAY:
        return 'file_dialog'
    return 'none'


def submit(path, tag):
    """Clica o OK do dialogo de ficheiro ate ele fechar. Um clique sem o rato SAIR e voltar ao botao
    nao chega ao OK (medido: o 2.o clique no mesmo ponto fica preso), por isso cada tentativa entra
    de fora e num ponto diferente. `'done'`, ou `'console'`/`'alert'` (o erro do app, ja fechado)."""
    for k in range(5):
        xdo('mousemove', 300, 300)
        time.sleep(0.2)
        click((DIALOG_OK[0] - 4 * k, DIALOG_OK[1] - 2 * k), 0.3)
        for _ in range(8):
            time.sleep(0.25)
            if path is None or (os.path.exists(path) and os.path.getsize(path) > 0):
                break
        time.sleep(0.6)
        st = screen_state(f'{tag}_{k}')
        if st in ('console', 'alert'):
            xdo('key', 'Return')
            time.sleep(0.6)
            if screen_state(f'{tag}_{k}_fechou') != 'none':
                raise RuntimeError(f'{tag}: o erro do app nao fechou')
            return st
        if st == 'none':
            if path is not None and not os.path.exists(path):
                raise RuntimeError(f'{tag}: o dialogo fechou sem gravar {path}')
            return 'done'
    raise Hang(tag)


class Hang(RuntimeError):
    """O dialogo nao fecha: o app parou de responder."""


# ---------------------------------------------------------------- entradas NOSSAS

def solid(w, h, rgba):
    return Image.new('RGBA', (w, h), rgba)


def painted(w, h, cells):
    """Preto opaco, brancos nas celulas `cells` (o Color Range apanha o branco)."""
    im = solid(w, h, (0, 0, 0, 255))
    for (x, y) in cells:
        im.putpixel((x, y), (255, 255, 255, 255))
    return im


def ellipse_cells(cx, cy, rx, ry):
    return [(x, y) for y in range(int(cy - ry), int(cy + ry) + 1) for x in range(int(cx - rx), int(cx + rx) + 1)
            if ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2 <= 1.0]


def holes_cells():
    c = [(x, y) for y in range(3, 12) for x in range(5, 16) if not (8 <= x <= 10 and 6 <= y <= 8)]
    return c + [(30, 20)]


def checker_cells(x0, y0, w, h):
    return [(x, y) for y in range(y0, y0 + h) for x in range(x0, x0 + w) if (x + y) % 2 == 0]


def ring_cells(cx, cy, r_out, r_in):
    return [c for c in ellipse_cells(cx, cy, r_out, r_out)
            if math.hypot(c[0] - cx, c[1] - cy) > r_in]


def craft(x, y, w, h, bit, pad=0, magic=0x9500, size=None, chunk_type=2, count=1):
    """Um `.msk` no formato INFERIDO das escritas (ver oraculo_msk.md), com alavancas para estragar."""
    stride = (w + 7) // 8
    rows = bytearray()
    for yy in range(h):
        row = bytearray(stride)
        for i in range(stride * 8):
            on = bit(i, yy) if i < w else pad
            if on:
                row[i // 8] |= 0x80 >> (i % 8)
        rows += row
    chunk = struct.pack('<IH', 6 + len(rows), chunk_type) + rows
    head = bytearray(64)
    total = 64 + len(chunk) if size is None else size
    struct.pack_into('<IHHHhh', head, 0, total, magic, w, h, x, y)
    struct.pack_into('<H', head, 18, count)
    return bytes(head) + chunk


def sized(b, delta):
    """O campo de tamanho total = comprimento real + `delta`."""
    b = bytearray(b)
    struct.pack_into('<I', b, 0, len(b) + delta)
    return bytes(b)


def reserved_garbage(b):
    b = bytearray(b)
    for i in list(range(14, 18)) + list(range(20, 64)):
        b[i] = 0xA5
    return bytes(b)


def chunk_size(b, delta):
    b = bytearray(b)
    struct.pack_into('<I', b, 64, struct.unpack_from('<I', b, 64)[0] + delta)
    return bytes(b)


def two_chunks(t1, t2):
    """Dois blocos (contagem 2 no byte 18): o 1.o com tipo `t1` e bits `x<y`, o 2.o `t2` e `x>y`."""
    a = craft(2, 3, 7, 4, lambda x, y: x < y, chunk_type=t1, count=2)
    b = craft(2, 3, 7, 4, lambda x, y: x > y, chunk_type=t2)[64:]
    return sized(a + b, 0)


def legacy_8000(bit):
    out = bytearray(8000)
    for y in range(200):
        for x in range(320):
            if bit(x, y):
                out[y * 40 + x // 8] |= 0x80 >> (x % 8)
    return bytes(out)


def pattern(x, y):
    """Diagonal, buracos e uma barra: nada que um retangulo explique."""
    return (x + 2 * y) % 5 == 0 or (y == 3 and x != 6) or (x == 0)


WRITE_CASES = [
    # (nome, (w,h), seleccao JS ou None, celulas brancas para o Color Range ou None)
    ('w_rect_1x1_origin', (40, 30), 'select(0,0,1,1)', None),
    ('w_rect_1x1_offset', (40, 30), 'select(3,2,1,1)', None),
    ('w_rect_9x2', (40, 30), 'select(0,0,9,2)', None),
    ('w_rect_8x1', (40, 30), 'select(0,0,8,1)', None),
    ('w_rect_17x3_offset', (40, 30), 'select(5,7,17,3)', None),
    ('w_rect_13x5_odd', (40, 30), 'select(11,9,13,5)', None),
    ('w_rect_partly_outside', (40, 30), 'select(-3,-2,8,6)', None),
    ('w_full_canvas', (40, 30), 'selectAll()', None),
    ('w_full_canvas_400x260', (400, 260), 'selectAll()', None),
    ('w_rect_beyond_320x200', (400, 260), 'select(17,9,350,230)', None),
    ('w_range_holes', (40, 30), None, holes_cells()),
    ('w_range_ellipse', (40, 30), None, ellipse_cells(18.0, 11.0, 11.0, 7.0)),
    ('w_range_checker_odd', (40, 30), None, checker_cells(2, 3, 13, 5)),
    ('w_range_ring_400x260', (400, 260), None, ring_cells(201, 117, 110, 60)),
    ('w_range_corners', (40, 30), None, [(0, 0), (39, 0), (0, 29), (39, 29)]),
    ('w_no_selection', (40, 30), None, None),
    ('w_range_no_match', (40, 30), None, []),
]

READ_CASES = [
    # (nome, (w,h) da tela, bytes do .msk)
    ('r_pattern_offset_pad0', (40, 30), craft(6, 5, 13, 9, pattern, pad=0)),
    ('r_pattern_offset_pad1', (40, 30), craft(6, 5, 13, 9, pattern, pad=1)),
    ('r_one_column', (40, 30), craft(21, 4, 1, 11, lambda x, y: y % 3 != 1)),
    ('r_wide_33', (40, 30), craft(2, 20, 33, 3, lambda x, y: (x * 7 + y) % 4 != 0)),
    ('r_negative_origin', (40, 30), craft(-3, -2, 10, 6, lambda x, y: (x + y) % 3 != 0)),
    ('r_past_right_bottom', (40, 30), craft(35, 25, 10, 8, lambda x, y: True)),
    ('r_bigger_than_canvas', (40, 30), craft(0, 0, 50, 40, lambda x, y: (x // 5 + y // 5) % 2 == 0)),
    ('r_all_zero_bits', (40, 30), craft(4, 4, 6, 3, lambda x, y: False)),
    ('r_legacy_8000', (320, 200), legacy_8000(lambda x, y: (x // 16 + y // 10) % 2 == 0 or x == 319)),
    ('r_legacy_8000_small_canvas', (40, 30), legacy_8000(lambda x, y: (x + y) % 7 == 0)),
    ('r_bad_magic', (40, 30), craft(1, 1, 5, 5, lambda x, y: True, magic=0x1234)),
    ('r_bad_total_size', (40, 30), craft(1, 1, 5, 5, lambda x, y: True, size=999)),
    ('r_chunk_type_3', (40, 30), craft(1, 1, 5, 5, lambda x, y: True, chunk_type=3)),
    ('r_count_0', (40, 30), craft(1, 1, 5, 5, lambda x, y: True, count=0)),
    ('r_truncated', (40, 30), craft(1, 1, 16, 5, lambda x, y: True)[:-3]),
    ('r_empty_file', (40, 30), b''),
    # 2.a ronda (a 1.a mostrou que o cabecalho e verificado e que o legado le 6 bytes adiante):
    ('r_total_size_smaller', (40, 30), sized(craft(2, 3, 7, 4, lambda x, y: x != y), -3)),
    ('r_trailing_bytes', (40, 30), craft(2, 3, 7, 4, lambda x, y: x != y) + bytes(5)),
    ('r_trailing_bytes_sized', (40, 30), sized(craft(2, 3, 7, 4, lambda x, y: x != y) + bytes(5), 0)),
    ('r_reserved_garbage', (40, 30), reserved_garbage(craft(2, 3, 7, 4, lambda x, y: x != y))),
    ('r_count_2_type3_then_mask', (40, 30), two_chunks(3, 2)),
    ('r_count_2_mask_then_type3', (40, 30), two_chunks(2, 3)),
    ('r_count_2_two_masks', (40, 30), two_chunks(2, 2)),
    ('r_chunk_size_bigger', (40, 30), chunk_size(craft(2, 3, 7, 4, lambda x, y: x != y), +4)),
    ('r_chunk_size_smaller', (40, 30), chunk_size(craft(2, 3, 7, 4, lambda x, y: x != y), -2)),
    ('r_legacy_8001', (320, 200), legacy_8000(lambda x, y: x < 160) + b'\x00'),
    ('r_legacy_7999', (320, 200), legacy_8000(lambda x, y: x < 160)[:-1]),
    ('r_legacy_8000_bytes_index', (320, 200), bytes(range(256)) * 31 + bytes(range(64))),
]


def rows_from_png(path, ref):
    """'1' onde o Clear mudou o pixel (a seleccao dentro da tela)."""
    im = Image.open(path).convert('RGBA')
    w, h = im.size
    return [''.join('1' if im.getpixel((x, y)) != ref.getpixel((x, y)) else '0' for x in range(w))
            for y in range(h)]


def twice(run):
    """Um erro do app conta so se se repetir numa sessao NOVA: uma letra perdida na digitacao da o
    mesmo erro (medido), e repetir na MESMA sessao desarruma os menus (medido: o Load abriu o Save)."""
    r = run('')
    if r['saved'] == 'error' or r.get('load') == 'error':
        r = run('_b')
    return r


def run_write(name, size, js_sel, cells):
    return twice(lambda sfx: _run_write(name, sfx, size, js_sel, cells))


def run_read(name, size, data):
    return twice(lambda sfx: _run_read(name, sfx, size, data))


def _run_write(name, sfx, size, js_sel, cells):
    w, h = size
    img = painted(w, h, cells) if cells is not None else solid(w, h, RED)
    png = os.path.join(WORK, f'{name}_tela.png')  # nome sem digito final: senao o app propoe abrir uma SEQUENCIA (medido)
    img.save(png)
    js = os.path.join(WORK, f'{name}.js')
    with open(js, 'w') as f:
        f.write(f'app.activeSprite.selection.{js_sel};\n' if js_sel else '\n')
    s = Session(png, js)
    try:
        if cells is not None:
            s.color_range()
        msk = os.path.join(WORK, f'{name}.msk')
        saved = s.save_msk(msk, name + sfx)
        out_png = os.path.join(WORK, f'{name}_clear.png')
        s.clear_and_save_png(out_png)
    finally:
        s.kill()
    return {
        'name': name, 'direction': 'write', 'canvas': [w, h],
        'how': f'selection.{js_sel}' if js_sel else ('Select > Color Range #ffffff tol 0' if cells is not None else 'sem seleccao'),
        'white_cells': len(cells) if cells is not None else None,
        'saved': saved,
        'oracle_msk_hex': open(msk, 'rb').read().hex() if saved == 'saved' else None,
        'clear_rows': rows_from_png(out_png, img),
    }


def _run_read(name, sfx, size, data):
    w, h = size
    img = solid(w, h, RED)
    png = os.path.join(WORK, f'{name}_tela.png')  # nome sem digito final: senao o app propoe abrir uma SEQUENCIA (medido)
    img.save(png)
    js = os.path.join(WORK, f'{name}.js')
    with open(js, 'w') as f:
        f.write('\n')
    src = os.path.join(WORK, f'{name}_in.msk')
    with open(src, 'wb') as f:
        f.write(data)
    s = Session(png, js)
    try:
        load = s.load_msk(src, name + sfx)
        msk = os.path.join(WORK, f'{name}_resaved.msk')
        out_png = os.path.join(WORK, f'{name}_clear.png')
        saved = None
        if load != 'hang':
            saved = s.save_msk(msk, name + sfx)
            s.clear_and_save_png(out_png)
    finally:
        s.kill()
    return {
        'name': name, 'direction': 'read', 'canvas': [w, h],
        'input_msk_hex': data.hex(),
        'load': load,
        'saved': saved,
        'oracle_msk_hex': open(msk, 'rb').read().hex() if saved == 'saved' else None,
        'clear_rows': rows_from_png(out_png, img) if saved else None,
    }


def main():
    os.makedirs(os.path.join(WORK, 'home'), exist_ok=True)
    os.symlink(WORK, SHORT)
    xvfb = subprocess.Popen(['Xvfb', DISPLAY, '-screen', '0', '1280x800x24'],
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(1.0)
    only = set(sys.argv[2:])
    try:
        cases = []
        for (name, size, js_sel, cells) in WRITE_CASES:
            if not only or name in only:
                cases.append(run_write(name, size, js_sel, cells))
                print(f'{name} ok', file=sys.stderr)
        for (name, size, data) in READ_CASES:
            if not only or name in only:
                cases.append(run_read(name, size, data))
                print(f'{name} ok', file=sys.stderr)
        lv = subprocess.run(['pacman', '-Q', 'libresprite'], capture_output=True, text=True).stdout.strip()
        json.dump({
            'source': f'{lv} (GPL-2, corrido como oraculo; nunca lido; o rodape do app diz «1.2-dev (local build)»)',
            'method': 'Select > Save to MSK file / Load from MSK file pelo menu (xdotool, Xvfb proprio); '
                      'clear_rows = Edit > Clear sobre a imagem opaca + Save As PNG (pixeis mudados = seleccao)',
            'date': datetime.date.today().isoformat(),
            'command': 'python3 -I docs/PixelArt/ferramentas/oraculo_msk.py <pasta> > crates/ph2d-pixel-io/tests/fixtures/oracle_msk.json',
            'cases': cases,
        }, sys.stdout, indent=1)
        sys.stdout.write('\n')
    finally:
        os.remove(SHORT)
        xvfb.kill()
        xvfb.wait()


if __name__ == '__main__':
    main()
