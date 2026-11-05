"""O ALFA dos kernels edges-find, edges-find-horizontal e edges-find-vertical da Convolution Matrix
(Aseprite Trial 1.3.18.6, caixa preta, so corrido). Reusa trial_efeitos_r3 (leitura RGBA pelo clipboard,
estado dos botoes por pixel, sentinela). Grava <dir>/edges_alpha.json a cada caso.

Entrada NOSSA, alfa6.ase 15x15 RGBA: blocos 3x3, alfa = ALPHAS[(x//3 + y//3 * 2) % 6]; cor que varia,
R = 40+12x, G = 200-10y, B = 90. Cada alfa aparece no miolo de um bloco (vizinhos iguais) e na borda.

uso: PH2D_TRIAL_DISPLAY=:91 python3 -I trial_edges_alfa.py <dir> run
     python3 -I trial_edges_alfa.py <dir> fixtura     (acrescenta `edges_alpha` a oracle_trial_round3.json)
Lancador: Xvfb proprio 1600x1000x24 -nolisten tcp, HOME novo com .config/aseprite/aseprite.ini
([GfxMode] Frame = 0 0 1600 1000, screen_scale = 2). Mate por PID so o que lancou.
"""
import json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_efeitos_r3 as r3   # noqa: E402

m = r3.m
D = m.D
OUT = os.path.join(D, 'edges_alpha.json')
ALPHAS = (255, 0, 1, 64, 128, 200)
KERNELS = {17: 'edges-find', 18: 'edges-find-horizontal', 19: 'edges-find-vertical'}
INPUT_LAW = 'alfa = (255,0,1,64,128,200)[(x//3 + 2*(y//3)) % 6]; R = 40+12x, G = 200-10y, B = 90'


def alpha_at(x, y):
    return ALPHAS[(x // 3 + y // 3 * 2) % 6]


def gen():
    m.write_ase(os.path.join(D, 'alfa6.ase'), [[(40 + 12 * x, 200 - 10 * y, 90, alpha_at(x, y)) for x in range(15)]
                                               for y in range(15)])


def fresh():
    sent = r3.sentinel()
    m.drv.fecha(); m.setup(15, 15, 16, False); m.drv.abre('alfa6.ase', '4')
    return sent


def run():
    m.gen3(); gen(); m.drv.fecha = r3.fecha_robusto
    out = json.load(open(OUT)) if os.path.exists(OUT) else {}
    cases = out.setdefault('cases_rgba', {}); st = out.setdefault('button_state', {})
    save = lambda: (json.dump(out, open(OUT + '.tmp', 'w')), os.rename(OUT + '.tmp', OUT))
    if 'input' not in out:
        out['sentinel'] = fresh(); out['input'] = r3.clip_rgba('e_in'); save()
    for k, name in KERNELS.items():
        for mode in ('A_on', 'A_off'):
            key = f'{name}:{mode}'
            if key in cases:
                continue
            fresh()
            m.open_fx(m.CONV_ITEM); m.time.sleep(0.4); m.xdo('mousemove', 800, 700); m.time.sleep(0.2)
            at_open = r3.ch_state()
            m.xdo('mousemove', 700, 424, 'click', 1); m.time.sleep(0.2); m.key('Home', t=0.15)
            for _ in range(k):
                m.key('Down', t=0.06)
            m.xdo('mousemove', 800, 700); m.time.sleep(0.4)
            kern = r3.ch_state()
            if kern['A'] != (mode == 'A_on'):
                m.menu((969, 527), t=0.4); m.xdo('mousemove', 800, 700); m.time.sleep(0.3)
            at_ok = r3.ch_state()
            assert at_ok['A'] == (mode == 'A_on'), (key, at_ok)
            m.subprocess_shot(f'e_{k}_{mode}')
            m.menu((933, 404), t=0.8)
            cases[key] = r3.clip_rgba(f'e_{k}_{mode}')
            st[key] = {'at_open': at_open, 'after_kernel': kern, 'at_ok': at_ok}
            if cases[key] == out['sentinel']:
                st[key]['reads_as_sentinel'] = True
            save()
            print(key, st[key], [c[3] for c in cases[key][7]], cases[key][7][1], flush=True)


def run_opaco():
    """Controlo: entradas OPACAS. impl/imp0l (impulso +8/-8 sobre 128) com A solto dao o RGB da matriz, sem a
    tinta sumir; impg40 (40x40) com o A que o kernel traz: grelha do ECRA e clipboard, para ver o que era o
    «128 constante» das rodadas 1-3 (o xadrez sob alfa 0?)."""
    m.gen3(); gen(); m.drv.fecha = r3.fecha_robusto
    out = json.load(open(OUT)); op = out.setdefault('opaque', {})
    save = lambda: (json.dump(out, open(OUT + '.tmp', 'w')), os.rename(OUT + '.tmp', OUT))
    plan = [(f, 15, k, 'A_off') for f in ('impl', 'imp0l') for k in KERNELS] + [('impg40', 40, k, 'A_on') for k in KERNELS]
    for f, n, k, mode in plan:
        key = f'{f}:{KERNELS[k]}:{mode}'
        if key in op:
            continue
        sent = r3.sentinel()
        m.drv.fecha(); m.setup(n, n, 16, False); m.drv.abre(f + '.ase', '4')
        before = m.grid(m.shot('o_before.png'))
        m.open_fx(m.CONV_ITEM); m.time.sleep(0.4)
        m.xdo('mousemove', 700, 424, 'click', 1); m.time.sleep(0.2); m.key('Home', t=0.15)
        for _ in range(k):
            m.key('Down', t=0.06)
        m.xdo('mousemove', 800, 700); m.time.sleep(0.4)
        if r3.ch_state()['A'] != (mode == 'A_on'):
            m.menu((969, 527), t=0.4); m.xdo('mousemove', 800, 700); m.time.sleep(0.3)
        at_ok = r3.ch_state()
        assert at_ok['A'] == (mode == 'A_on'), (key, at_ok)
        m.subprocess_shot(f'o_{f}_{k}_{mode}')
        m.menu((933, 404), t=0.8)
        screen = m.grid(m.shot(f'o_{f}_{k}_{mode}.png'))
        clip = r3.clip_rgba(f'o_{f}_{k}_{mode}')
        op[key] = {'at_ok': at_ok, 'screen_rgb': screen, 'clipboard_is_sentinel': clip == sent}
        if clip != sent:
            op[key]['clipboard_rgba'] = clip
        save()
        print(key, 'sentinel' if clip == sent else clip[7][5:10], sorted({tuple(c) for r in screen for c in r})[:6], flush=True)


OBSERVED = (
    'MEDIDO: os tres NAO sao constantes. edges-find = Laplaciano (-1 em volta, 8 no centro), -vertical = Sobel '
    '(1 0 -1 / 2 0 -2 / 1 0 -1), -horizontal = Sobel (1 2 1 / 0 0 0 / -1 -2 -1), resposta ao impulso, div 1, bias 0; '
    'canais RGBA (A premido, A5). Em imagem OPACA o alfa sai 0 (soma dos pesos 0, 255*0), logo a imagem some; o '
    '"128 constante" de round2_gray e A2 era o XADREZ da tela lido no ecra (impg40: ver screen_rgb, dois tons fora '
    'do 1.o quadrado de 16; clipboard_is_sentinel = toda transparente). Lei de cor e alfa como A4b; a unica lei nova: '
    'quando div - soma dos pesos dos vizinhos de alfa 0 da 0, a celula FICA como estava (RGB e alfa, mesmo com A '
    'premido; 8 celulas em -horizontal e 10 em -vertical de alfa6, nos dois estados do A). Bytes sob alfa 0 nao '
    'contam (A4b). Com A solto o alfa fica o da entrada.')


def fixtura():
    """acrescenta `edges_alpha` a fixtura, em texto: o resto do ficheiro fica byte a byte"""
    fix = 'crates/ph2d-pixel-core/tests/fixtures/oracle_trial_round3.json'
    r = json.load(open(OUT))
    op = {k: {a: b for a, b in c.items() if a != 'screen_rgb' or k.startswith('impg40')} for k, c in r['opaque'].items()}
    entry = {
        'source': 'Aseprite Trial 1.3.18.6 (caixa preta, so corrido; nunca lido codigo nem dados dele)',
        'method': 'Xvfb proprio 1600x1000x24, HOME limpo. alfa6.ase NOSSO 15x15 RGBA: ' + INPUT_LAW + '. Edit > FX > '
                  'Convolution Matrix, clique na lista, Home, Down k (17, 18, 19); estado dos botoes R G B A lido por '
                  'pixel antes do OK; A_off = A clicado depois do kernel. Um documento acabado de abrir por caso, '
                  'precedido da sentinela stepg; leitura RGBA pelo clipboard (como A4b). Controlos opacos: impl, imp0l '
                  '(15x15, A solto, clipboard) e impg40 (40x40, A premido, ecra e clipboard), entradas de gen3.',
        'date': '2026-11-05',
        'command': 'PH2D_TRIAL_DISPLAY=:91 python3 -I docs/PixelArt/ferramentas/trial_edges_alfa.py <dir> run ; ... '
                   '<dir> opaco ; ... <dir> fixtura (cwd = raiz do repo)',
        'observed': OBSERVED, 'input_law': INPUT_LAW, 'button_state': r['button_state'], 'input_rgba': r['input'],
        'cases_rgba': r['cases_rgba'], 'opaque': op,
    }
    s = open(fix).read()
    assert s.endswith('\n}\n') and '"edges_alpha"' not in s
    body = json.dumps(entry, ensure_ascii=False, separators=(',', ':'))
    json.loads(s[:-3] + ',\n "edges_alpha": ' + body + '\n}\n')
    open(fix + '.tmp', 'w').write(s[:-3] + ',\n "edges_alpha": ' + body + '\n}\n'); os.rename(fix + '.tmp', fix)


if __name__ == '__main__':
    {'run': run, 'opaco': run_opaco, 'gen': gen, 'fixtura': fixtura}[sys.argv[2]]()
