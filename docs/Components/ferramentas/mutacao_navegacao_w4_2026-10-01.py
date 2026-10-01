#!/usr/bin/env python3
"""Prova de mutação da W4 da navegação (o Inspector, o registo, a semente, a leitura viva).

Backup -> mutar (assert 1 casamento) -> correr SÓ o grupo que OBSERVA a mutação -> restaurar + touch.

Os quatro controlos: a corrida limpa de cada grupo é VERDE e corre > 0 testes; o pré-voo das
âncoras casa cada uma exactamente uma vez (`MUTA_SO_ANCORAS=1` pára aí); uma mutação que não
compila é acusada como defeito do ARNÊS e nunca como sangria; um grupo que corre zero testes aborta.

⚠️ A população de cada mutação é de quem a OBSERVA, nunca de quem a CONTÉM (a lei que o arnês da
§20 da escultura pagou com um sobrevivente fabricado).
"""
import os, re, subprocess, sys, shutil

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'

# Os grupos — cada um é um filtro de nextest sobre as crates que observam.
G = {
    'ECS': ['-p', 'ph2d-physics-ecs', '-E', 'test(nav::) | test(registers_every_physics_component)'],
    'EDITOR': ['-p', 'ph2d-editor-core', '-E', 'test(nav_edits)'],
    'APPC': ['-p', 'ph2d-app-components', '-E', 'test(nav_inspector)'],
    'PANEL': ['-p', 'ph2d-panel-inspector', '-E', 'test(a_seccao_nav_esta_viva)'],
    'SHELL': ['-p', 'ph2d-host-desktop', '-E',
              'test(component_seed_seam) | test(toda_fila_do_inspector)'],
    # ⭐ A ÁREA ANDÁVEL (o pintor) — e o preço do 8-direcções, que vive na suíte da ponte.
    'APHYS': ['-p', 'ph2d-app-physics', '-E', 'test(uma_parede_da_area)'],
}

BR = 'crates/ph2d-physics-ecs/src/bridge/'
AC = 'crates/ph2d-app-components/src/nav_inspector.rs'
EC = 'crates/ph2d-editor-core/src/nav_edits.rs'
EV = 'crates/ph2d-panel-inspector/src/event_nav.rs'
RL = 'shells/desktop/src/render_loop/'

M = [
    ('M1 o despacho não publica o agora', 'ECS', BR + 'dispatch.rs',
     '        self.publica_navegacao(sim);\n', ''),
    ('M2 o agora de quem deixou de ser agente fica', 'ECS', BR + 'nav.rs',
     '            w.entity_mut(e).remove::<crate::NavNow>();\n', '            let _ = e;\n'),
    ('M3 o raio autorado é ignorado', 'ECS', BR + 'nav.rs',
     '                radius: if autorado > 0.0 {', '                radius: if autorado > 1e9 {'),
    ('M4 o que falta é sempre zero', 'ECS', BR + 'nav.rs',
     '                remaining: rt.remaining(pos) as f32,',
     '                remaining: rt.remaining(pos) as f32 * 0.0,'),
    ('M5 o NavAgent não é registado', 'ECS', 'crates/ph2d-physics-ecs/src/lib.rs',
     '    reg.register_default::<NavAgent>("ph2d::physics::NavAgent");\n', ''),
    ('M6 a queixa do teclado vem antes da do mover', 'EDITOR', EC,
     '        if !self.has_mover {\n            return Some(AgentQueixa::SemMover);\n        }\n'
     '        if self.mover_reads_keys {\n            return Some(AgentQueixa::MoverLeTeclado);\n        }\n',
     '        if self.mover_reads_keys {\n            return Some(AgentQueixa::MoverLeTeclado);\n        }\n'
     '        if !self.has_mover {\n            return Some(AgentQueixa::SemMover);\n        }\n'),
    ('M7 o nome perdido lê-se como vazio', 'EDITOR', EC,
     '            NavAlvoModo::Objecto if self.alvo_perdido => return Some(AgentQueixa::AlvoPerdido),\n',
     ''),
    ('M8 a região sem paredes não se queixa', 'EDITOR', EC,
     '        if self.obstacle_layers == 0 {', '        if self.obstacle_layers == 0 && false {'),
    ('M9 o painel não sabe que o mover lê o teclado', 'APPC', AC,
     '        mover_reads_keys: mover.is_some_and(|m| m.default_controls),',
     '        mover_reads_keys: mover.is_some_and(|m| m.default_controls && false),'),
    ('M10 dentro de uma região sempre', 'APPC', AC,
     '        p[0] >= a[0] && p[0] <= b[0] && p[1] >= a[1] && p[1] <= b[1]',
     '        p[0] >= a[0] || p[0] <= b[0] || p[1] >= a[1] || p[1] <= b[1]'),
    ('M11 o nome vazio grava o hash de ""', 'APPC', AC,
     '            Some(NavTarget::Named(if t.is_empty() {',
     '            Some(NavTarget::Named(if t.is_empty() && false {'),
    ('M12 o mesmo modo apaga o nome', 'APPC', AC,
     '                (NavAlvoModo::Objecto, t @ NavTarget::Named(_)) => t,\n', ''),
    ('M13 o X escreve no Y', 'APPC', AC,
     '            if matches!(edit, NavFieldEdit::AlvoX(_)) {',
     '            if matches!(edit, NavFieldEdit::AlvoY(_)) {'),
    ('M14 a meia-largura aceita negativos', 'APPC', AC,
     '                NavFieldEdit::HalfW(v) => r.half_extents[0] = v.max(0.0),',
     '                NavFieldEdit::HalfW(v) => r.half_extents[0] = *v,'),
    ('M15 a leitura viva lê só o estado', 'APPC', AC,
     '            restante: n.remaining,', '            restante: n.remaining * 0.0,'),
    ('M16 a camada é posta em vez de trocada', 'PANEL', EV,
     '                NavFieldEdit::ObstacleLayers(r.obstacle_layers ^ (1u8 << i)),',
     '                NavFieldEdit::ObstacleLayers(r.obstacle_layers | (1u8 << i)),'),
    ('M17 o interruptor não inverte', 'PANEL', EV,
     '        push(host, bits, NavFieldEdit::Active(!a.active));',
     '        push(host, bits, NavFieldEdit::Active(a.active));'),
    ('M18 o despacho do painel não chama a navegação', 'PANEL',
     'crates/ph2d-panel-inspector/src/event.rs',
     '    if crate::event_nav::apply_nav_event(host, ev) {',
     '    if false && crate::event_nav::apply_nav_event(host, ev) {'),
    ('M19 a semente não desliga o teclado', 'SHELL', 'crates/ph2d-physics-ecs/src/components/topdown.rs',
     '        l.default_controls = false;\n        if l.direction == DirectionMode::default() {',
     '        if l.direction == DirectionMode::default() {'),
    ('M20 a semente esmaga o FourWay', 'SHELL', 'crates/ph2d-physics-ecs/src/components/topdown.rs',
     '        if l.direction == DirectionMode::default() {\n            l.direction = DirectionMode::Free;',
     '        if true {\n            l.direction = DirectionMode::Free;'),
    ('M21 a semente não está na tabela', 'SHELL', 'crates/ph2d-app-physics/src/physics_seed.rs',
     '    ("ph2d::physics::NavAgent", seed_nav_agent),\n', ''),
    ('M22 o agente não pede o mover', 'SHELL', 'crates/ph2d-component-desc/src/catalog/physics.rs',
     '        "component.nav_agent.name",\n        &["ph2d::physics::TopDownPlayer"],',
     '        "component.nav_agent.name",\n        &[],'),
    ('M23 a fila da navegação não é tirada', 'SHELL', RL + 'fase_hero_commits.rs',
     '                nav_edits: take(&mut pd.nav_edits),',
     '                nav_edits: Vec::new(),'),
    ('M24 a aplicação não chama a porta da navegação', 'SHELL', RL + 'fase_inspector_commits_top20.rs',
     '        | ph2d_app_components::nav_inspector::apply_all(sim, nav)\n',
     '        | ph2d_app_components::nav_inspector::apply_all(sim, &nav[..0])\n'),
    ('M25 a área andável não se publica', 'ECS', BR + 'nav.rs',
     '        for mesh in self.nav.meshes.values() {\n            for &(de, para) in mesh.walls() {',
     '        for mesh in self.nav.meshes.values().take(0) {\n            for &(de, para) in mesh.walls() {'),
    ('M26 o recuo esquecido', 'ECS', BR + 'nav.rs',
     '            agent_radius: f64::from(raio),', '            agent_radius: f64::from(raio) * 0.0,'),
    ('M27 o pintor pica o contorno', 'APHYS', 'crates/ph2d-app-physics/src/overlay/probes.rs',
     '                if n > f64::EPSILON && m.kind != ProbeKind::NavEdge {',
     '                if n > f64::EPSILON && m.kind != ProbeKind::Sensor {'),
]

so = os.environ.get('MUTA_SO')
# ⭐ Corre só estes grupos (`MUTA_G=ECS,APPC`) — a fatia tem prazo de 30 min e a SHELL recompila.
grupos = set(os.environ['MUTA_G'].split(',')) if os.environ.get('MUTA_G') else set(G)


def corre(grupo):
    c = ['bash', 'scripts/ph2d-run.sh', 'cargo', 'nextest', 'run', '--no-fail-fast'] + G[grupo]
    r = subprocess.run(c, cwd=ROOT, capture_output=True, text=True)
    out = r.stdout + r.stderr
    compila = not ('error[' in out or 'could not compile' in out)
    m = re.search(r'(\d+) tests? run: (\d+) passed(?:, (\d+) failed)?', out)
    ran = int(m.group(1)) if m else 0
    fail = int(m.group(3) or 0) if m else 0
    return ran, fail, compila, r.returncode, out


# controlo: cada grupo limpo verde e com população
for g in sorted(grupos):
    ran, fail, comp, rc, out = corre(g)
    print(f'LIMPO {g}: {ran} testes, {fail} vermelhos, rc={rc}')
    sys.stdout.flush()
    if fail or ran == 0 or not comp or rc != 0:
        print(out[-3000:])
        sys.exit(f'o controlo limpo do grupo {g} não está verde')

# pré-voo das âncoras
for nome, g, f, old, new in M:
    n = open(os.path.join(ROOT, f)).read().count(old)
    if n != 1:
        sys.exit(f'ÂNCORA MORTA ({n}x): {nome}')
print(f'pré-voo: as {len(M)} âncoras casam 1x')
if os.environ.get('MUTA_SO_ANCORAS'):
    sys.exit(0)

sang = 0
tot = 0
maus = 0
for nome, g, f, old, new in M:
    if so and not nome.startswith(so + ' '):
        continue
    if g not in grupos:
        continue
    p = os.path.join(ROOT, f)
    bak = p + '.muta'
    shutil.copy2(p, bak)
    try:
        s = open(p).read()
        assert s.count(old) == 1
        open(p, 'w').write(s.replace(old, new))
        ran, fail, comp, rc, out = corre(g)
    finally:
        shutil.move(bak, p)
        os.utime(p, None)
    tot += 1
    if not comp:
        v = 'NÃO COMPILA (defeito do ARNÊS — REESCREVER)'
        maus += 1
    elif ran == 0:
        v = 'ZERO testes (arnês) — ABORTO'
        maus += 1
    elif fail > 0:
        v = 'SANGRA'
        sang += 1
    else:
        v = 'SOBREVIVEU'
    print(f'{nome} [{g}]: {v} ({fail}/{ran})')
    sys.stdout.flush()
parcial = ''
if so or grupos != set(G):
    parcial = f' (PARCIAL: MUTA_SO={so} MUTA_G={sorted(grupos)})'
print(f'== {sang} de {tot} sangram{parcial}; {maus} defeitos de arnês')
sys.exit(0 if sang == tot else 1)
