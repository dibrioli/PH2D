import sys, json, os, time; FD=sys.argv[1]; sys.path.insert(0, FD); step=sys.argv[3]; extra=sys.argv[4:]; sys.argv=[sys.argv[0], sys.argv[2]]
import trial_efeitos_menus as m
import trial_efeitos_ferramentas as tf
drv=m.drv; xdo, key, shot = m.xdo, m.key, m.shot
P='r3.json'; out=json.load(open(P)) if os.path.exists(P) else {}
def save(): json.dump(out, open(P,'w'))
if step == 'inv':
    run = extra[0]
    drv.fecha(); m.setup(16, 1, 16, False); drv.abre('idx.ase', '4')
    a = shot('i_in.png'); base = m.grid(a); pin = m.palette_panel(a, 16)
    for idx_on in (False, True):
        m.menu(m.EDIT, m.INVERT)
        if idx_on: m.menu((803, 567), t=0.3)
        shot(f'i_dlg_{int(idx_on)}_{run}.png', park=False)
        xdo('mousemove', 803, 420, 'click', 1); time.sleep(0.8)
        a = shot(f'i_after_{int(idx_on)}_{run}.png')
        out[f'inv_{int(idx_on)}_{run}'] = {'cells': m.grid(a)[0], 'panel': m.palette_panel(a, 16), 'cells_in': base[0], 'panel_in': pin}
        xdo('mousemove', 856, 700); key('ctrl+z', t=0.6)
        if idx_on:  # desliga o Index lembrado
            m.menu(m.EDIT, m.INVERT); m.menu((803, 567), t=0.3); shot(f'i_dlg_off_{run}.png', park=False); key('Escape', t=0.4)
        save(); print(idx_on, out[f'inv_{int(idx_on)}_{run}']['cells'][:4], out[f'inv_{int(idx_on)}_{run}']['panel'][:4], flush=True)
    print('in', base[0][:4], pin[:4])
EDGE = [(0, 6), (0, 7), (1, 6), (7, 0), (8, 0), (15, 12)]
def outline_run(doc, preset, place, toggles, tiled, tag, pre=None):
    tf.opendoc(doc, 16, 16)
    key('shift+o', t=1.0)
    xdo('mousemove', *tf.PRESET[preset], 'click', 1); time.sleep(0.4)
    xdo('mousemove', *tf.PLACE[place], 'click', 1); time.sleep(0.4)
    for t in toggles:
        xdo('mousemove', *t, 'click', 1); time.sleep(0.3)
    if tiled: xdo('mousemove', 910, 611, 'click', 1); time.sleep(0.4)
    if pre: pre()
    a = shot(tag + '_dlg.png', park=False)
    mat = [[('on' if a[y, x].sum() < 200 else 'off') for x in (617, 649, 681)] for y in (558, 590, 622)]
    xdo('mousemove', 950, 395, 'click', 1); time.sleep(0.8)
    g = tf.grid(shot(tag + '.png'))
    if tiled:   # desliga o Tiled lembrado
        key('shift+o', t=1.0); xdo('mousemove', 910, 611, 'click', 1); time.sleep(0.3); shot(tag + '_off.png', park=False); key('Escape', t=0.5)
    drv.fecha()
    return mat, g
if step == 'outline':
    run = extra[0]
    tf.write_outline_inputs()
    px = [(0, 0, 0, 0)] * 256
    for i, j in EDGE: px[j * 16 + i] = (0, 0, 255, 255)
    tf.write_rgba(os.path.join(drv.D, 'shape_e.ase'), 16, 16, px)
    for nome, args in (('top', ('shape_t.ase', 'circle', 'outside', [(617, 590), (681, 590), (649, 622)], False)),
                       ('top_inside', ('shape_t.ase', 'circle', 'inside', [(617, 590), (681, 590), (649, 622)], False)),
                       ('tiled', ('shape_e.ase', 'circle', 'outside', [], True)),
                       ('untiled', ('shape_e.ase', 'circle', 'outside', [], False))):
        mat, g = outline_run(*args, f'o3_{nome}_{run}')
        out[f'outline_{nome}_{run}'] = {'matrix_read': mat, 'grid': g}; save()
        print(nome, mat, flush=True)
        for row in g: print(' '.join({'000000': '#', '0000ff': 'B', 'ffffff': '.'}.get(c, '_') for c in row))
A, B = (0x20, 0x40, 0x60), (0xc0, 0x80, 0x10)
def mix(t): return tuple(round(a + (b - a) * t) for a, b in zip(A, B))
IPAL = [(255, 255, 255), A, B, mix(1 / 3), mix(2 / 3), (0, 0, 0), (255, 0, 0), (0, 0, 255)]
def gcase(doc, dither, tag):
    tf.opendoc(doc, 32, 8)
    key('shift+g', t=0.4)
    tf.set_dither(dither)
    xdo('mousemove', 510, 72, 'click', 1); time.sleep(0.3)
    drv.drag(tf.px_of((0, 4)), tf.px_of((31, 4))); time.sleep(0.3)
    a = shot(tag + '.png'); g = tf.grid(a)
    pan = m.palette_panel(a, 8)
    drv.fecha()
    return g, pan
if step == 'grad':
    run = extra[0]
    import trial_dinamica as din
    din.write_white(os.path.join(drv.D, 'gw32x8.ase'), 32, 8)
    m.write_ase(os.path.join(drv.D, 'gi32x8.ase'), [[0] * 32 for _ in range(8)], IPAL)
    for doc, dither, nome in (('gw32x8.ase', 'solid', 'rgb_solid'), ('gi32x8.ase', 'solid', 'idx_solid'), ('gi32x8.ase', 'bayer4', 'idx_bayer4')):
        g, pan = gcase(doc, dither, f'g3_{nome}_{run}')
        out[f'grad_{nome}_{run}'] = {'grid': g, 'palette_panel': pan}; save()
        print(nome, g[4], pan, flush=True)
if step == 'cprobe':
    m.open_adj('curve'); time.sleep(0.5)
    p = m.curve_xy(128, 128)
    xdo('mousemove', *p, 'click', 1); time.sleep(0.5); shot('cp1.png', park=False)
    xdo('mousemove', *p, 'click', '--repeat', 2, '--delay', 80, 1); time.sleep(0.8); shot('cp2.png', park=False)
PX, PY, POK = (763, 470), (763, 508), (720, 545)
def setpt(src, dst, tag):
    a = m.curve_xy(*src)
    xdo('mousemove', *a, 'click', 1); time.sleep(0.4)
    xdo('mousemove', *a, 'click', 3); time.sleep(0.7)
    shot(tag + '_pp.png', park=False)
    for xy, v in ((PX, dst[0]), (PY, dst[1])):
        xdo('mousemove', *xy, 'click', '--repeat', 2, '--delay', 60, 1); time.sleep(0.2); key('ctrl+a', t=0.1); xdo('type', str(v)); time.sleep(0.2)
    xdo('mousemove', *POK, 'click', 1); time.sleep(0.5)
if step == 'curve':
    run = extra[0]
    m.setup(256, 8, 4, False); drv.X0, drv.Y0 = int(extra[1]), int(extra[2])
    ref = m.ramp_rows()
    for nome, pts in (('p64_128', [((64, 64), (64, 128))]), ('p128_64', [((128, 128), (128, 64))]),
                      ('p64_32__192_224', [((64, 64), (64, 32)), ((192, 192), (192, 224))])):
        m.open_adj('curve'); time.sleep(0.4)
        for i, (s, d) in enumerate(pts): setpt(s, d, f'cv3_{nome}_{i}_{run}')
        shot(f'cv3_{nome}_{run}_dlg.png', park=False)
        g = m.ok_read((933, 420), f'cv3_{nome}_{run}')
        gb = m.grid(shot('cvbase.png')); nb = sum(tuple(gb[j][i]) != ref[j][i] for j in range(8) for i in range(256))
        out[f'curve_{nome}_{run}'] = {'points': pts, 'lut_gray': [g[3][i][0] for i in range(256)], 'rgb_same': all(g[0][i][0] == g[1][i][1] == g[2][i][2] == g[3][i][0] for i in range(256)), 'undo_bad': nb}
        save(); print(nome, out[f'curve_{nome}_{run}']['lut_gray'][::16], 'undo_bad', nb, flush=True)
