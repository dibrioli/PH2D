import sys, json, subprocess, numpy as np
sys.path.insert(0, '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor/docs/Painter/Pinceis/ferramentas')
import mypaint_bateria as mb
S = sys.argv[1]
import os
bat = json.load(open(S + '/bat/' + os.environ.get('BAT', 'bateria_pastel.json')))
SUF = os.environ.get('SUF', '')
exe = mb.binario()
w, h = bat['tela']
for p in sys.argv[2:]:
    raw = f"{S}/mp_raw/{p.replace('/', '__')}{SUF}.raw"
    lin = [f"tela {w} {h}", "limpa"] + mb.comandos_da_cena(bat['cenas'][0], f"/usr/share/mypaint-data/2.0/brushes/{p}.myb", None, 0.010, w, h, raw)
    r = subprocess.run([exe], input="\n".join(lin) + "\n", text=True, capture_output=True); assert r.returncode == 0, r.stderr
    a = np.fromfile(raw, dtype=np.uint16).reshape(h, w, 4).astype(np.float64) / mb.UM
    np.save(raw[:-4] + '.npy', a.astype(np.float32))  # pré-multiplicado 0..1
    print(p, a[..., 3].max(), (a[..., 3] > 0.01).sum())
