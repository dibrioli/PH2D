#!/usr/bin/env python3
"""Prova de mutação da ponte da navegação (W3). Backup -> mutar (assert 1 casamento) -> testar -> restaurar + touch."""
import os, re, subprocess, sys, shutil, time
ROOT='/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'
B='crates/ph2d-physics-ecs/src/bridge/'
M=[
 ('M1 a porta dos controladores não chama a navegação', B+'controllers.rs','        self.drive_nav_agents(sim);\n',''),
 ('M2 o seed não devolve a memória dos agentes', B+'tape.rs','            self.nav.agents = m.nav.clone();\n','            let _ = &m.nav;\n'),
 ('M3 o rebuild_from_rest não esquece os agentes', B+'rewind.rs','        self.nav.agents.clear();\n',''),
 ('M4 a porta dos sinais não lê a navegação', B+'signals.rs','        for ev in self.nav_events() {','        for ev in self.nav_events().iter().take(0) {'),
 ('M5 nunca publica', B+'nav.rs','            events.append(tick_events);','            tick_events.clear();'),
 ('M6 o replay também publica', B+'nav.rs','        if publicar {\n            let NavWorld {','        if true {\n            let NavWorld {'),
 ('M7 o raio derivado é zero', B+'nav.rs','                raio_que_envolve(&body.rest)\n','                0.0\n'),
 ('M8 a guarda dos controlos invertida', B+'nav.rs','            if mover.default_controls || world','            if !mover.default_controls || world'),
 ('M9 desligado não zera a intenção', B+'nav.rs','                self.player_input.insert(p.entity, PlayerInput::default());\n',''),
 ('M10 a mudança de obstáculos não esquece os caminhos', B+'nav.rs','                rt.forget_path();\n            }\n        }\n        let dt','            }\n        }\n        let dt'),
 ('M11 o alvo nomeado lê só o Transform', B+'nav.rs','        if let Some(b) = self.bodies.get(&e)\n            && let Some(p)','        if let Some(b) = self.bodies.get(&e).filter(|_| false)\n            && let Some(p)'),
 ('M12 a camada fora da máscara também bloqueia', B+'nav.rs','            if d.layer < 8 && r.layers & (1u8 << d.layer) != 0 {','            if true {'),
]
so=os.environ.get('MUTA_SO')
def corre():
    cmds=[['bash','scripts/ph2d-run.sh','cargo','test','-p','ph2d-physics-ecs','--test','it','nav::'],
          ['bash','scripts/ph2d-run.sh','cargo','test','-p','ph2d-physics-ecs','--lib','bridge::nav']]
    ran=fail=0; compila=True; rcs=[]
    for c in cmds:
        r=subprocess.run(c,cwd=ROOT,capture_output=True,text=True)
        out=r.stdout+r.stderr; rcs.append(r.returncode)
        if 'error[' in out or 'could not compile' in out: compila=False
        for m in re.finditer(r'test result: \w+\. (\d+) passed; (\d+) failed',out):
            ran+=int(m.group(1))+int(m.group(2)); fail+=int(m.group(2))
    return ran,fail,compila,rcs
# controlo: limpo verde
ran,fail,comp,rcs=corre()
print(f'LIMPO: {ran} testes, {fail} vermelhos, rc={rcs}'); sys.stdout.flush()
if fail or ran==0 or not comp: sys.exit('o controlo limpo não está verde')
# pré-voo das âncoras
for nome,f,old,new in M:
    n=open(os.path.join(ROOT,f)).read().count(old)
    if n!=1: sys.exit(f'ÂNCORA MORTA ({n}x): {nome}')
print('pré-voo: todas as âncoras casam 1x')
if os.environ.get('MUTA_SO_ANCORAS'): sys.exit(0)
sang=0; tot=0
for nome,f,old,new in M:
    if so and not nome.startswith(so+' '): continue
    p=os.path.join(ROOT,f); bak=p+'.muta'
    shutil.copy2(p,bak)
    try:
        s=open(p).read(); assert s.count(old)==1
        open(p,'w').write(s.replace(old,new))
        ran,fail,comp,rcs=corre()
    finally:
        shutil.move(bak,p); os.utime(p,None)
    tot+=1
    if not comp: v='NÃO COMPILA (lê-se como sangrar — REESCREVER)'
    elif ran==0: v='ZERO testes (arnês) — ABORTO'
    elif fail>0: v='SANGRA'; sang+=1
    else: v='SOBREVIVEU'
    print(f'{nome}: {v} ({fail}/{ran})'); sys.stdout.flush()
print(f'== {sang} de {tot} sangram')
