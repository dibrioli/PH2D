#!/usr/bin/env python3
"""Prova de mutação da W5 da navegação (o DESVIO: a lei `ph2d-orca`, a ponte, o Inspector e a cena `=2`).

Backup -> mutar (assert 1 casamento) -> correr SÓ o grupo que OBSERVA a mutação -> restaurar + touch.

Os quatro controlos (o molde da W4): a corrida limpa de cada grupo é VERDE e corre > 0 testes; o
pré-voo das âncoras casa cada uma exactamente uma vez (`MUTA_SO_ANCORAS=1` pára aí); uma mutação que
não compila é acusada como defeito do ARNÊS e nunca como sangria; um grupo que corre zero testes aborta.
"""
import os, re, subprocess, sys, shutil

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'

G = {
    'ORCA': ['-p', 'ph2d-orca'],
    'ECS': ['-p', 'ph2d-physics-ecs', '-E', 'test(nav)'],
    'APPC': ['-p', 'ph2d-app-components', '-E', 'test(nav_inspector) | test(nav_smoke)'],
    'PANEL': ['-p', 'ph2d-panel-inspector', '-E', 'test(a_seccao_nav_esta_viva)'],
}

O = 'crates/ph2d-orca/src/'
BR = 'crates/ph2d-physics-ecs/src/bridge/nav.rs'
# ⚠️ O desvio saiu da `nav.rs` para o filho `nav_desvio.rs` no fecho (o tecto de LOC: 788/700).
BD = 'crates/ph2d-physics-ecs/src/bridge/nav_desvio.rs'

M = [
    ('M1 o recíproco ignorado', 'ORCA', O + 'lines.rs',
     '        point: add(me.vel, scale(u, share)),', '        point: add(me.vel, scale(u, 1.0 + 0.0 * share)),'),
    ('M2 o contacto usa o horizonte em vez do tique', 'ORCA', O + 'lines.rs',
     '        let inv_dt = 1.0 / dt;', '        let inv_dt = 1.0 / tau;'),
    ('M3 as pernas trocadas', 'ORCA', O + 'lines.rs',
     '            dir = if det(rel_pos, w) > 0.0 {', '            dir = if det(rel_pos, w) < 0.0 {'),
    ('M4 o círculo do corte nunca', 'ORCA', O + 'lines.rs',
     '        if d1 < 0.0 && d1 * d1 > r_sq * w_len_sq {', '        if d1 < 0.0 && d1 * d1 > r_sq * w_len_sq && false {'),
    ('M5 o 1D aperta o lado errado', 'ORCA', O + 'lp.rs',
     '        if den >= 0.0 {\n            t_right = t_right.min(t);', '        if den < 0.0 {\n            t_right = t_right.min(t);'),
    ('M6 sem o 3D', 'ORCA', O + 'lp.rs',
     '    if fail < lines.len() {\n        program3(', '    if fail < lines.len() && false {\n        program3('),
    ('M7 o 3D esquece as paredes', 'ORCA', O + 'lp.rs',
     '        proj.extend_from_slice(&lines[..n_walls]);', '        proj.extend_from_slice(&lines[..0]);'),
    ('M8 o convexo trocado', 'ORCA', O + 'walls.rs',
     '                p == q || left_of(self.point[p], self.point[i], self.point[q]) >= 0.0',
     '                p == q || left_of(self.point[p], self.point[i], self.point[q]) <= 0.0'),
    ('M9 as paredes de costas contam', 'ORCA', O + 'walls.rs',
     '            if abs_sq(sub(b, a)) <= 0.0 || left_of(a, b, pos) >= 0.0 {',
     '            if abs_sq(sub(b, a)) <= 0.0 || left_of(a, b, pos) <= 0.0 {'),
    # ⚠️ A 1.ª M10 (`verts[para]` → `verts[de]`) é EQUIVALENTE e foi trocada: com o `next`/`prev` a
    # acompanhar, ela dá as MESMAS arestas invertidas, só numeradas de outra forma (sobreviveu 0/14).
    # A mutação que vira as paredes do avesso é na PONTE, onde a malha entra no desvio.
    ('M10 a parede da malha entra do avesso', 'ECS', BD,
     '                let w = ph2d_orca::Walls::from_walkable_walls(m.verts(), m.walls());',
     '                let w = ph2d_orca::Walls::from_walkable_walls(m.verts(), &m.walls().iter().map(|&(a, b)| (b, a)).collect::<Vec<_>>());'),
    ('M11 sem a preferência de lado', 'ORCA', O + 'crowd.rs',
     '        if regime == Regime::Free || self.params.side_bias <= 0.0 {',
     '        if regime == Regime::Free || self.params.side_bias <= 1e9 {'),
    ('M12 pela esquerda', 'ORCA', O + 'crowd.rs',
     '        let direita = [a.pref[1] / p, -a.pref[0] / p];', '        let direita = [-a.pref[1] / p, a.pref[0] / p];'),
    ('M13 a fotografia comum (Jacobi)', 'ORCA', O + 'crowd.rs',
     '            self.agents[i].vel = v;\n', '            let _ = &v;\n'),
    ('M14 o alvo não é ignorado', 'ORCA', O + 'crowd.rs',
     '                    if j as usize == i || me.ignores == Some(j) {', '                    if j as usize == i {'),
    ('M15 o alcance só soma os raios', 'ORCA', O + 'crowd.rs',
     '                            ri + rj + ki + sj', '                            ri + rj'),
    ('M16 sem o tecto', 'ORCA', O + 'crowd.rs',
     '                found.truncate(n);\n', ''),
    ('M17 as paredes nunca', 'ORCA', O + 'crowd.rs',
     '            && self.params.time_horizon_walls > 0.0', '            && self.params.time_horizon_walls > 1e9'),
    ('M18 o desvio não corre', 'ECS', BR,
     '        self.desvia(pedidas, dt);\n', '        let _ = (pedidas, dt);\n'),
    ('M19 a intenção ignora a velocidade segura', 'ECS', BD,
     '            let dir = if p.avoidance && p.speed > 0.0 {', '            let dir = if false && p.avoidance && p.speed > 0.0 {'),
    ('M20 os corpos que não são agentes ficam de fora', 'ECS', BD,
     '            if b.kind == BodyKind::Static || b.rest.is_sensor || indice.contains_key(&e) {',
     '            if b.kind == BodyKind::Static || b.rest.is_sensor || indice.contains_key(&e) || true {'),
    ('M21 a velocidade do mover é zero', 'ECS', BD,
     '            return [f64::from(st.velocity[0]), f64::from(st.velocity[1])];',
     '            return [f64::from(st.velocity[0]) * 0.0, f64::from(st.velocity[1]) * 0.0];'),
    ('M22 as paredes da malha não chegam ao desvio', 'ECS', BD,
     '|i| paredes.get(i).copied().flatten().map(|w| (w, 0.0)), dt);',
     '|i| paredes.get(i).copied().flatten().map(|w| (w, 0.0)).filter(|_| false), dt);'),
    ('M23 a ponte não diz quem é o alvo', 'ECS', BD,
     '        corpos[k].ignores = p.alvo.and_then(|a| indice.get(&a).copied());',
     '        corpos[k].ignores = p.alvo.and_then(|a| indice.get(&a).copied()).filter(|_| false);'),
    ('M24 o desligado desvia na mesma', 'ECS', BD,
     '                avoids: p.avoidance,', '                avoids: true || p.avoidance,'),
    ('M25 a aplicação não escreve', 'APPC', 'crates/ph2d-app-components/src/nav_inspector.rs',
     '        NavFieldEdit::Avoidance(b) => a.avoidance = *b,', '        NavFieldEdit::Avoidance(_) => {}'),
    ('M26 o retrato lê uma constante', 'APPC', 'crates/ph2d-app-components/src/nav_inspector.rs',
     '        avoidance: a.avoidance,', '        avoidance: true,'),
    ('M27 o controlo da cena desvia', 'APPC', 'crates/ph2d-app-components/src/nav_smoke_porta.rs',
     '        cinzentos[k] = corpo(world, &format!("Grey {}", k + 1), lb[k], lb[destino], false);',
     '        cinzentos[k] = corpo(world, &format!("Grey {}", k + 1), lb[k], lb[destino], true);'),
    ('M28 o interruptor não inverte', 'PANEL', 'crates/ph2d-panel-inspector/src/event_nav.rs',
     '        push(host, bits, NavFieldEdit::Avoidance(!a.avoidance));', '        push(host, bits, NavFieldEdit::Avoidance(a.avoidance));'),
    ('M29 a caixa não é registada', 'PANEL', 'crates/ph2d-panel-inspector/src/populate_nav.rs',
     '    for id in [ids::INSP_NAV_ACTIVE, ids::INSP_NAV_AVOIDANCE] {', '    for id in [ids::INSP_NAV_ACTIVE] {'),
    ('M30 o despacho escuta o id errado', 'PANEL', 'crates/ph2d-panel-inspector/src/event_nav.rs',
     '        && id == crate::ids::INSP_NAV_AVOIDANCE', '        && id == crate::ids::INSP_NAV_ACTIVE'),
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
