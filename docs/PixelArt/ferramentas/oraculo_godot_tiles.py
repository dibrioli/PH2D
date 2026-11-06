"""P10 M2/M3: corre o Godot --headless sobre mapas NOSSOS e grava as fixturas com cabecalho.
uso (pesado: pelo guarda): PH2D_GPU=1 bash scripts/ph2d-run.sh python3 -I docs/PixelArt/ferramentas/oraculo_godot_tiles.py <saida_dir>
Godot (MIT, pacman) so corrido; nada dele se le nem se copia."""
import json, os, subprocess, sys, datetime
OUT = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else 'crates/ph2d-pixel-tiles/tests/fixtures'
AQUI = os.path.dirname(os.path.abspath(__file__))
env = dict(os.environ); env.pop('WAYLAND_DISPLAY', None)
r = subprocess.run(['timeout', '170', 'godot', '--headless', '--script', os.path.join(AQUI, 'oraculo_godot_tiles.gd')],
                   capture_output=True, text=True, env=env)
ver = subprocess.run(['godot', '--version'], capture_output=True, text=True).stdout.strip()
got = {'M0': [], 'M2A': [], 'M2B': [], 'M2C': [], 'M3': []}
for l in r.stdout.splitlines():
    k, _, v = l.partition(' ')
    if k in got:
        got[k].append(json.loads(v))
if not got['M2A']:
    print(r.stdout[-2000:], r.stderr[-2000:]); sys.exit(1)
hd = {'oraculo': 'Godot ' + ver + ' (MIT), --headless --script', 'comando': 'PH2D_GPU=1 bash scripts/ph2d-run.sh python3 -I docs/PixelArt/ferramentas/oraculo_godot_tiles.py',
      'data': datetime.date.today().isoformat(), 'mascara': '[lados TRBL, cantos TR,BR,BL,TL]; regra: canto so conta com as duas arestas dele'}
os.makedirs(OUT, exist_ok=True)
for nome, ks in [('oracle_godot_autotile.json', ['M0', 'M2A', 'M2B', 'M2C']), ('oracle_godot_grelhas.json', ['M3'])]:
    json.dump({'_cabecalho': hd, **{k: got[k] for k in ks}}, open(os.path.join(OUT, nome), 'w'), separators=(',', ':'))
a = got['M2A'][0]; b = got['M2B'][0]
print('M2A 256 vizinhancas: diferem', a['diferem'])
print('M2B celulas', b['celulas'], 'diferem', b['diferem'], 'mascaras distintas', b['mascaras_distintas_exercitadas'])
for c in got['M2C']:
    print('M2C', {k: v for k, v in c.items() if k not in ('amostra_pedida_x_escolhida', 'outras_amostra')})
tot = ok = 0
for m in got['M3']:
    tot += 1; ok += m['volta_ok'] == m['celulas']
print('M3 configuracoes', tot, 'com volta exata em todas as celulas', ok)
