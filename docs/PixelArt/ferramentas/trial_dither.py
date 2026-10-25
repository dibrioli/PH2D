"""Driver: RGB->Indexed no Aseprite Trial 1.3.18.6 (Xvfb :78, janela 1600x1000), lido por fotografia.
uso: python3 -I trial_dither.py <workdir> <plano> <saida.jsonl>
plano: linhas 'img pal dither map fit' (dither 0..7 = No,Ord2,Ord4,Ord8,Old2,Old4,Old8,FS;
map 0..2 = Default(Octree),Table RGB5+A3,Octree; fit 0..4 = Default(Euclid),RGB,LinRGB,CIEXYZ,CIELAB).
Os widgets acham-se por casamento exacto de modelo (o diálogo muda de forma: Advanced, Amount)."""
import hashlib, json, os, subprocess, sys, time
import numpy as np
from PIL import Image
WD, PLANO, OUT = sys.argv[1:4]
TOOLS = os.path.dirname(os.path.abspath(__file__))
TPL = os.path.join(TOOLS, 'trial_tpl') if os.path.isdir(os.path.join(TOOLS, 'trial_tpl')) else os.path.join(WD, 'tpl')
ENV = dict(os.environ, DISPLAY=':78'); ENV.pop('WAYLAND_DISPLAY', None)
SIZES = {'cubo': (36, 6), 'grade': (16, 16), 'rampas': (32, 8), 'probe': (72, 40)}

def xdo(*a): subprocess.run(['xdotool', *map(str, a)], env=ENV, check=True)
def click(x, y, d=0.45): xdo('mousemove', x, y, 'click', 1); time.sleep(d)
def shot(name='cur.png'):
    xdo('mousemove', 1400, 250); time.sleep(0.25)
    p = os.path.join(WD, name); subprocess.run(['import', '-window', 'root', p], env=ENV, check=True); return p
def find(p, *names):
    r = subprocess.run([sys.executable, '-I', f'{TOOLS}/trial_ui.py', TPL, p, *names], capture_output=True, text=True, check=True)
    out = {}
    for line in r.stdout.split('\n'):
        if line:
            n, x, y = line.split(); out.setdefault(n, []).append((int(x), int(y)))
    return out
def read(p, img, ref=None):
    w, h = SIZES[img]
    a = [sys.executable, '-I', (f'{TOOLS}/le_tela.py' if os.path.exists(f'{TOOLS}/le_tela.py') else f'{TOOLS}/trial_le_tela.py'), p, str(w), str(h)] + ([ref] if ref else [])
    return json.loads(subprocess.run(a, capture_output=True, text=True, check=True).stdout)
def dlabel(p, ax, ay):
    im = Image.open(p).convert('RGB').crop((ax - 560, ay - 17, ax - 220, ay - 1))
    return hashlib.sha1(im.tobytes()).hexdigest()[:10]

def fhash(p, ax, ay, w):
    im = Image.open(p).convert('RGB').crop((ax - w, ay - 8, ax - 14, ay + 8))
    return hashlib.sha1(im.tobytes()).hexdigest()[:10]

def convert(d, m, f, tag):
    click(185, 74); click(222, 124); click(462, 198, 1.1)
    click(732, 406)
    p = shot(); ar = find(p, 'arrow')['arrow']
    ax, ay = ar[0]
    click(ax, ay, 0.9)
    name = 'd%d' % d
    def item():
        hs = find(shot(), name).get(name, [])
        hs = [h for h in hs if abs(h[1] - ay) > 20]
        return hs[0] if hs else None
    hit = item()
    for py in (ay - 150, ay + 250):
        if hit: break
        xdo('mousemove', 900, py); [xdo('click', 5 if d == 7 else 4) for _ in range(5)]; time.sleep(0.5)
        hit = item()
    if hit:
        click(*hit, 1.0)
    else:   # já é o seleccionado (realçado): fecha a lista
        xdo('key', 'Escape'); time.sleep(0.6)
    # relayout: Advanced ON depois do dither (o Amount do FS não cabe no diálogo antigo)
    for _ in range(6):
        p = shot(); h = find(p, 'cb_off', 'arrow', 'ok', 'ok2')
        if len(h['arrow']) == 3 and ('ok' in h or 'ok2' in h):
            break
        merge_y = min(y for _, y in h['cb_off'])
        click(673, merge_y + 32, 1.2)
    else:
        raise AssertionError('advanced layout')
    p = shot(); ar = find(p, 'arrow')['arrow']; assert len(ar) == 3, ('arrows', ar)
    lab = dlabel(p, *ar[0])
    for k, want in ((1, m), (2, f)):
        ax2, ay2 = ar[k]
        click(ax2, ay2, 0.6); click(1000, ay2 + 32 + 18 * want, 0.6)
    p = shot(); ar = find(p, 'arrow')['arrow']; assert len(ar) == 3, ('arrows2', ar)
    labs = (lab, fhash(p, ar[1][0], ar[1][1], 300), fhash(p, ar[2][0], ar[2][1], 300))
    hh = find(p, 'ok', 'ok2'); ok = hh.get('ok') or hh.get('ok2'); assert ok, 'no OK'
    os.replace(p, os.path.join(WD, tag + '.dlg.png'))
    click(*ok[0], 1.6)
    xdo('mousemove', 850, 300); time.sleep(0.3)
    return labs

def main():
    cur_img = cur_pal = None
    with open(PLANO) as fp, open(OUT, 'a') as out:
        for line in fp:
            if not line.strip(): continue
            img, pal, d, m, f = line.split(); d, m, f = int(d), int(m), int(f)
            ref = os.path.join(WD, f'rgb_{img}.png')
            if img != cur_img:
                xdo('key', 'ctrl+o'); time.sleep(1.5)
                xdo('type', '--delay', 40, ref); time.sleep(0.4); xdo('key', 'Return'); time.sleep(2.5)
                if img == 'probe': xdo('mousemove', 856, 522); xdo('key', '3'); time.sleep(1)
                else: xdo('mousemove', 850, 300); xdo('key', '4'); time.sleep(1)
                c = read(shot(f'ctl_{img}.png'), img, ref)
                print('control', img, c['control_mismatch'], file=sys.stderr)
                cur_img, cur_pal = img, None
            if pal != cur_pal:
                click(177, 134, 0.8); click(234, 442, 1.5)
                xdo('type', '--delay', 40, os.path.join(WD, f'pal_{pal}.gpl')); time.sleep(0.4)
                click(1261, 860, 1.5); cur_pal = pal
            tag = f'r_{img}_{pal}_d{d}_m{m}_f{f}'
            lab = convert(d, m, f, tag)
            r = read(shot(tag + '.png'), img)
            xdo('key', 'ctrl+z'); time.sleep(1.2)
            u = read(shot('undo.png'), img, ref)
            rec = dict(image=img, palette=pal, dither=d, mapping=m, best_fit=f, label=lab,
                       cells=r['cells'], undo_mismatch=u['control_mismatch'])
            out.write(json.dumps(rec) + '\n'); out.flush()
            print(img, pal, d, m, f, lab, 'undo', u['control_mismatch'], file=sys.stderr)
main()
