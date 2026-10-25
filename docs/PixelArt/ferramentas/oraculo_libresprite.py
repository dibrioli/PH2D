#!/usr/bin/env python3
"""Oraculo da conversao de modo de cor (onda P2): o LibreSprite 1.3 (GPL-2, o ancestral aberto do
Aseprite; so CORRIDO, nunca lido) converte entradas NOSSAS e a saida vira fixtura.

A API JS dele nao converte (`colorMode` sem setter, `app.launch('ChangePixelFormat')` no-op), por
isso a conversao vai pelo MENU, num Xvfb proprio (nunca o ecra do dono: `WAYLAND_DISPLAY` fora e
`SDL_VIDEODRIVER=x11`), com HOME isolado (a arrumacao da janela fixa as coordenadas do menu).

    python3 -I docs/PixelArt/ferramentas/oraculo_libresprite.py <pasta de trabalho> \
        > crates/ph2d-pixel-core/tests/fixtures/oracle_color_mode.json

Precisa de: libresprite, Xvfb, xdotool, Pillow.
"""
import json
import os
import subprocess
import sys
import time

from PIL import Image

WORK = os.path.abspath(sys.argv[1])
DISPLAY = ':77'
# Sprite (menu) -> Color Mode -> item; medidos na foto do menu a 800x600 (docs/PixelArt/ferramentas).
MENU_SPRITE = (120, 14)
MENU_COLOR_MODE = (157, 76)
SAVE_OK = (560, 536)
ITEM = {'gray': (340, 104), 'indexed_none': (390, 132), 'indexed_ordered': (390, 160)}

DB32 = [
    0x000000, 0x222034, 0x45283C, 0x663931, 0x8F563B, 0xDF7126, 0xD9A066, 0xEEC39A,
    0xFBF236, 0x99E550, 0x6ABE30, 0x37946E, 0x4B692F, 0x524B24, 0x323C39, 0x3F3F74,
    0x306082, 0x5B6EE1, 0x639BFF, 0x5FCDE4, 0xCBDBFC, 0xFFFFFF, 0x9BADB7, 0x847E87,
    0x696A6A, 0x595652, 0x76428A, 0xAC3232, 0xD95763, 0xD77BBA, 0x8F974A, 0x8A6F30,
]
PALETTES = {
    'db32': [((c >> 16) & 255, (c >> 8) & 255, c & 255) for c in DB32],
    'gb4': [(0x0F, 0x38, 0x0F), (0x30, 0x62, 0x30), (0x8B, 0xAC, 0x0F), (0x9B, 0xBC, 0x0F)],
    'bw2': [(0, 0, 0), (255, 255, 255)],
}


def images():
    """As entradas: grelha de canais, rampas de matiz x valor, alfa parcial, cubo 6x6x6."""
    out = {}
    g = Image.new('RGBA', (16, 16))
    for y in range(16):
        for x in range(16):
            g.putpixel((x, y), (x * 16, y * 16, (x * y) % 256, 255))
    out['grade'] = g
    hues = [(255, 0, 0), (255, 128, 0), (255, 255, 0), (0, 255, 0), (0, 255, 255), (0, 0, 255), (255, 0, 255), (128, 128, 128)]
    r = Image.new('RGBA', (32, 8))
    for y, (hr, hg, hb) in enumerate(hues):
        for x in range(32):
            t = x / 31
            r.putpixel((x, y), (round(hr * t), round(hg * t), round(hb * t), 255))
    out['rampas'] = r
    a = Image.new('RGBA', (16, 4))
    alphas = [0, 1, 64, 127, 128, 129, 192, 255]
    for y in range(4):
        for x in range(16):
            a.putpixel((x, y), (x * 17, 255 - x * 17, y * 80, alphas[(x + y) % 8]))
    out['alfa'] = a
    c = Image.new('RGBA', (36, 6))
    for i in range(216):
        c.putpixel((i % 36, i // 36), ((i // 36) * 51, ((i // 6) % 6) * 51, (i % 6) * 51, 255))
    out['cubo'] = c
    return out


def write_gpl(path, name, cols):
    with open(path, 'w') as f:
        f.write(f'GIMP Palette\nName: {name}\nColumns: 8\n#\n')
        for (r, g, b) in cols:
            f.write(f'{r:3d} {g:3d} {b:3d}\tc\n')


def xdo(*args):
    subprocess.run(['xdotool', *args], env={**os.environ, 'DISPLAY': DISPLAY}, check=True)


def run_case(img_path, pal_path, mode, out_path):
    js = os.path.join(WORK, 'setpal.js')
    with open(js, 'w') as f:
        f.write(f"app.activeSprite.loadPalette({json.dumps(pal_path)});\n" if pal_path else '\n')
    env = {k: v for k, v in os.environ.items() if k != 'WAYLAND_DISPLAY'}
    env.update(DISPLAY=DISPLAY, SDL_VIDEODRIVER='x11', HOME=os.path.join(WORK, 'home'))
    if os.path.exists(out_path):
        os.remove(out_path)
    p = subprocess.Popen(['libresprite', img_path, '--script', js], env=env,
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        for _ in range(60):
            time.sleep(0.25)
            w = subprocess.run(['xdotool', 'search', '--name', 'LibreSprite'], env={**os.environ, 'DISPLAY': DISPLAY},
                               capture_output=True, text=True).stdout.split()
            if w:
                break
        time.sleep(1.5)
        # Num HOME novo a janela nasce ao centro; as coordenadas do menu contam a partir de (0,0).
        xdo('windowmove', w[0], '0', '0')
        time.sleep(1.0)
        xdo('mousemove', *map(str, MENU_SPRITE), 'click', '1'); time.sleep(0.6)
        xdo('mousemove', *map(str, MENU_COLOR_MODE), 'click', '1'); time.sleep(0.6)
        xdo('mousemove', *map(str, ITEM[mode]), 'click', '1'); time.sleep(1.0)
        xdo('key', 'ctrl+shift+s'); time.sleep(1.2)
        # O diálogo abre na pasta da entrada (= WORK); digitado depressa perde letras (medido).
        # O Enter às vezes fica no autocompletar do campo: clica-se o OK do diálogo.
        xdo('key', 'ctrl+a'); xdo('type', '--delay', '80', os.path.basename(out_path)); time.sleep(0.5)
        xdo('mousemove', *map(str, SAVE_OK), 'click', '1')
        for _ in range(40):
            time.sleep(0.25)
            if os.path.exists(out_path) and os.path.getsize(out_path) > 0:
                time.sleep(0.5)
                break
        if not os.path.exists(out_path):
            subprocess.run(['import', '-window', 'root', out_path + '.falhou.png'],
                           env={**os.environ, 'DISPLAY': DISPLAY})
    finally:
        p.kill()
        p.wait()
    if not os.path.exists(out_path):
        raise RuntimeError(f'o LibreSprite nao salvou {out_path}')
    return out_path


def read_result(path):
    im = Image.open(path)
    res = {'png_mode': im.mode, 'size': list(im.size)}
    if im.mode == 'P':
        pal = im.getpalette()
        n = len(pal) // 3
        res['palette'] = [pal[i * 3:i * 3 + 3] for i in range(n)]
        res['transparency'] = im.info.get('transparency') if not isinstance(im.info.get('transparency'), bytes) else list(im.info['transparency'])
        res['indices'] = list(im.get_flattened_data())
    else:
        res['pixels'] = [list(px) if isinstance(px, tuple) else [px] for px in im.get_flattened_data()]
    return res


def main():
    os.makedirs(os.path.join(WORK, 'home'), exist_ok=True)
    xvfb = subprocess.Popen(['Xvfb', DISPLAY, '-screen', '0', '1280x800x24'],
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(1.0)
    try:
        imgs = images()
        inputs, cases = {}, []
        for name, im in imgs.items():
            path = os.path.join(WORK, f'in_{name}.png')
            im.save(path)
            inputs[name] = {'size': list(im.size), 'rgba': [list(px) for px in im.get_flattened_data()]}
        pal_paths = {}
        for name, cols in PALETTES.items():
            pal_paths[name] = os.path.join(WORK, f'pal_{name}.gpl')
            write_gpl(pal_paths[name], name, cols)
        for name in imgs:
            src = os.path.join(WORK, f'in_{name}.png')
            out = run_case(src, None, 'gray', os.path.join(WORK, f'out_{name}_gray.png'))
            cases.append({'image': name, 'mode': 'gray', 'palette': None, 'result': read_result(out)})
            for pname in PALETTES:
                for mode in ('indexed_none', 'indexed_ordered'):
                    out = run_case(src, pal_paths[pname], mode, os.path.join(WORK, f'out_{name}_{pname}_{mode}.png'))
                    cases.append({'image': name, 'mode': mode, 'palette': pname, 'result': read_result(out)})
                    print(f'{name} {pname} {mode} ok', file=sys.stderr)
        lv = subprocess.run(['pacman', '-Q', 'libresprite'], capture_output=True, text=True).stdout.strip()
        json.dump({
            'source': f'{lv} (GPL-2, corrido como oraculo; nunca lido) — docs/PixelArt/ferramentas/oraculo_libresprite.py',
            'menu': 'Sprite > Color Mode > Grayscale | Indexed (No Dithering) | Indexed (Ordered Dither); a paleta e carregada por sprite.loadPalette(.gpl)',
            'palettes': {k: [list(c) for c in v] for k, v in PALETTES.items()},
            'inputs': inputs, 'cases': cases,
        }, sys.stdout)
        sys.stdout.write('\n')
    finally:
        xvfb.kill()
        xvfb.wait()


if __name__ == '__main__':
    main()
