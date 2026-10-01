#!/usr/bin/env python3
"""Prova de mutação da FAIXA do traço esticado (doc 121 §9.4).

Uso (a placa por exclusão, e o pré-voo das âncoras primeiro):
    python3 "docs/Motion Nodes/ferramentas/mutacao_a_faixa_do_traco_2026-10-01.py" --so-ancoras
    PH2D_GPU=1 PH2D_GPU_ESPERA=1500 bash scripts/ph2d-run.sh \
        python3 "docs/Motion Nodes/ferramentas/mutacao_a_faixa_do_traco_2026-10-01.py" [ids...]

Controlos: as âncoras casam UMA vez cada (senão um `cargo fmt` faz uma mutação não entrar e ela
lê-se como sobrevivente); a corrida LIMPA tem de estar verde e correr testes; uma mutação que não
compila é dita, nunca contada como sangrar. Restaura SEMPRE o ficheiro (try/finally) e toca-o.

Medido a 01/10 (RTX): 9 de 11 sangram. Sobrevivem, NOMEADAS: S6 (a folga de 0,1 px num vértice
liso) e S8 (a folga de 0,1 px na caixa da peça) — as duas são cercas à escala de 0,1 px, abaixo do
ruído de 0,25 px do aplanamento que as barras de curva toleram.
"""
import os, subprocess, sys, shutil, re
R='/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value'
W=R+'/crates/ph2d-shape-gpu/src/shape.wgsl'; E=R+'/crates/ph2d-shape-gpu/src/eixo.rs'
M=[
 ('S1','faixa desligada',W,'    return vec3<f32>(m, 1.0);','    return vec3<f32>(0.0);'),
 ('S2','quina lida como liso',W,'let quina = (it.ponta & 8u) != 0u;','let quina = false;'),
 ('S3','a junta de recurso ignora o estilo',W,'select(2u, it.junta, quina)','2u'),
 ('S4','sem a cerca do recuo',W,' || recuo > 0.5 * min(l0, l1)',''),
 ('S5','sem o limite da esquadria na quina',W,'if junta != 0u || 2.0 > (1.0 + dt) * limite * limite {','if junta != 0u {'),
 ('S6','sem a cerca da folga num liso',W,'(!quina && dot(m, m) > fora * fora) || ',''),
 ('S7','a caixa da peça esquece a esquadria',W,'it.tipo == 0u && it.junta == 0u && (it.ponta & 12u) != 0u','false'),
 ('S8','a caixa da peça sem a folga',W,'let fp = r * alcance_da_peca(it) + FAIXA_FOLGA;','let fp = r * alcance_da_peca(it);'),
 ('R1','sem bits de quina',E,'flags |= FAIXA_FIM | if q[j] { QUINA_FIM } else { 0 };','flags |= FAIXA_FIM;'),
 ('R2','sem deduplicar em f32',E,'if p.last() == Some(&pf) {','if false {'),
 ('R3','o bloco esquece a esquadria',E,'ITEM_TROCO if alcanca_a_esquadria(it) =>','ITEM_TROCO if false =>'),
]
so=('--so-ancoras' in sys.argv); ids=[a for a in sys.argv[1:] if not a.startswith('--')]
bad=0
for m in M:
    n=open(m[2]).read().count(m[3])
    if n!=1: print('ANCORA MORTA',m[0],n); bad+=1
if bad: sys.exit(3)
print('ancoras: %d de %d casam uma vez'%(len(M),len(M)))
if so: sys.exit(0)
def corre():
    env=dict(os.environ)
    p=subprocess.run(['cargo','test','-q','-p','ph2d-shape-gpu','--release','--','--include-ignored','--test-threads=1'],cwd=R,env=env,capture_output=True,text=True)
    out=p.stdout+p.stderr
    ok=sum(int(x) for x in re.findall(r'(\d+) passed',out)); fa=sum(int(x) for x in re.findall(r'(\d+) failed',out))
    nc='could not compile' in out or 'error[' in out
    return p.returncode,ok,fa,nc,out
rc,ok,fa,nc,out=corre()
print('LIMPA: rc=%d passaram %d falharam %d'%(rc,ok,fa))
if rc!=0 or fa or ok==0: print(out[-2000:]); sys.exit(4)
res=[]
for m in M:
    if ids and m[0] not in ids: continue
    bk=open(m[2]).read()
    try:
        open(m[2],'w').write(bk.replace(m[3],m[4]))
        rc,ok,fa,nc,out=corre()
    finally:
        open(m[2],'w').write(bk); os.utime(m[2],None)
    v='NAO COMPILA' if nc else ('SANGRA' if fa>0 else ('ZERO TESTES' if ok==0 else 'SOBREVIVE'))
    falhou=re.findall(r'^    (\S+)$',out,re.M)
    print('%s %-40s %s (passaram %d, falharam %d) %s'%(m[0],m[1],v,ok,fa,' '.join(falhou[:3])),flush=True)
    res.append(v)
print('sangram %d de %d'%(res.count('SANGRA'),len(res)))
