#!/bin/bash
# 🔬 PROVA DE MUTAÇÃO — o HORIZONTE da normal do relevo (docs/3D/29 §8).
# Sem placa: os gates `wgsl_gate` e `relevo_normal` da `ph2d-mesh-render`.
# A mutação que só a placa vê (o `K` do WGSL diferente do da CPU) corre à mão
# com `o_horizonte_le_o_mesmo_na_placa_e_na_cpu` — medida em 01/10, sangra.
# `MUTA_SO_ANCORAS=1` é o pré-voo: confere as âncoras e não corre teste nenhum.
# Quatro controlos: âncora única (aborta) · rc 2 = NÃO COMPILA (nunca «sangra»)
# · o CONTROLO C0 tem de sobreviver · a árvore é restaurada com `touch`.
cd "$(dirname "$0")/../../.." || exit 9
exec python3 - "$@" <<'PY'
import subprocess, shutil, os, sys
R='crates/ph2d-mesh-render/src/'
M=[
 ('H1', R+'fonte.rs', 'return tinta_inclina(in.n_view, gv, corpo);', 'let nn = normalize(in.n_view); let nb = nn - clamp(corpo,0.0,1.0)*(gv - nn*dot(nn,gv)); return normalize(nb);'),
 ('H2', R+'shaders/tinta.wgsl', 'return tinta_horizonte(n, nb / l);', 'return nb / l;'),
 ('H3', R+'relevo_normal.rs', 'if t <= 0.0 || z >= t {', 'if true {'),
 ('H4', R+'relevo_normal.rs', '    if c <= 0.0 {\n        return n;\n    }\n    let d', '    let d'),
 ('H5', R+'relevo_normal.rs', '((z - t) / (t - zmin)).exp()', '(2.0 * (z - t) / (t - zmin)).exp()'),
 ('H6', R+'relevo_normal.rs', 'let zmin = HORIZONTE_K * t;', 'let zmin = -HORIZONTE_K * t;'),
 ('C0', R+'relevo_normal.rs', 'pub const HORIZONTE_K: f32 = 0.1;', 'pub const HORIZONTE_K: f32 = 0.1;'),
]
only=os.environ.get('MUTA_SO_ANCORAS')
res=[]
for nome,f,a,b in M:
    s=open(f).read(); n=s.count(a)
    if n!=1: print(f'{nome}: ANCORA casa {n}x — ABORTA'); sys.exit(2)
    if only: continue
    bak=s; open(f,'w').write(s.replace(a,b))
    try:
        p=subprocess.run(['bash','scripts/cargo-test-narrow.sh','ph2d-mesh-render','--','wgsl_gate','relevo_normal'],capture_output=True,text=True)
        out=p.stdout+p.stderr
        rc=p.returncode
        v='SANGRA' if rc==1 else ('NAO COMPILA' if rc==2 else 'SOBREVIVE')
        tail=[l for l in out.splitlines() if 'passaram' in l or 'falh' in l.lower()][-2:]
        print(nome, v, rc, tail); res.append((nome,v))
    finally:
        open(f,'w').write(bak); os.utime(f,None)
print(f'PRE-VOO: {len(M)} de {len(M)} ancoras casam exactamente uma vez (ZERO testes corridos)' if only else res)
if not only and [n for n,v in res if (v!='SANGRA') != n.startswith('C')]: sys.exit(1)
PY
