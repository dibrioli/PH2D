#!/usr/bin/env python3
"""Prova de mutação da W6 da navegação (o MUNDO QUE MUDA: os mosaicos, a porta, os verbos, os alvos,
o Inspector e a cena `=3`).

Backup -> mutar (assert 1 casamento) -> correr SÓ o grupo que OBSERVA a mutação -> restaurar + touch.

Os quatro controlos (o molde da W4/W5): a corrida limpa de cada grupo é VERDE e corre > 0 testes; o
pré-voo das âncoras casa cada uma exactamente uma vez (`MUTA_SO_ANCORAS=1` pára aí); uma mutação que
não compila é acusada como defeito do ARNÊS e nunca como sangria; um grupo que corre zero testes aborta.
"""
import os, re, subprocess, sys, shutil

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'

G = {
    'NAVMESH': ['-p', 'ph2d-navmesh'],
    'NAV': ['-p', 'ph2d-nav'],
    'ORCA': ['-p', 'ph2d-orca'],
    'ECS': ['-p', 'ph2d-physics-ecs', '-E', 'test(nav)'],
    'APPC': ['-p', 'ph2d-app-components', '-E', 'test(nav) | test(signal_actions_bridge)'],
    'VERB': ['-p', 'ph2d-ecs', '-E', 'test(signal_actions)'],
    'PANEL': ['-p', 'ph2d-panel-inspector', '-E', 'test(a_seccao_nav_esta_viva)'],
    'EDCORE': ['-p', 'ph2d-editor-core', '-E', 'test(nav_edits)'],
}

T = 'crates/ph2d-navmesh/src/tiles.rs'
MESH = 'crates/ph2d-nav/src/mesh.rs'
W = 'crates/ph2d-orca/src/walls.rs'
BR = 'crates/ph2d-physics-ecs/src/bridge/'
APPC = 'crates/ph2d-app-components/src/'
PI = 'crates/ph2d-panel-inspector/src/'

M = [
    # ── os mosaicos (ph2d-navmesh) ──────────────────────────────────────────────────────────────
    # ⚠️ A 1.ª M1 (a ordem fixa dos extremos no corte) SOBREVIVEU e era EQUIVALENTE: os dois mosaicos
    # cortam o MESMO anel no mesmo sentido. A ordem saiu do código, e a M1 é agora a do cruzamento.
    ('M1 o cruzamento arredonda para baixo', 'NAVMESH', T,
     '        (2 * n + d) / (2 * d)', '        n / d'),
    ('M2 o cruzamento sai do pedaço', 'NAVMESH', T,
     '        Apoio::Original(p, q) => (p, q),', '        Apoio::Original(_, _) => (a, b),'),
    ('M3 sem a reparação das junções em T', 'NAVMESH', T,
     '                out.extend(meio.iter().map(|q| indice[q]));', '                let _ = &meio;'),
    ('M4 a folga do balde é o raio', 'NAVMESH', T,
     '            let folga = (2.0 * r * SCALE).ceil() as i64 + 4;',
     '            let folga = (r * SCALE).ceil() as i64 + 4;'),
    ('M5 o mosaico não assina os obstáculos', 'NAVMESH', T,
     '                    quem.iter().for_each(|&i| h.u64(assin[i]));', '                    let _ = quem;'),
    ('M6 o atalho global nunca deixa actualizar', 'NAVMESH', T,
     '        if self.sig == Some(h.0) {', '        if self.sig.is_some() {'),
    ('M7 diz que mudou sem mudar', 'ECS', T,
     '        let mudou = stats.rebuilt > 0 || !self.mosaicos.is_empty();', '        let mudou = true;'),
    ('M8 nenhum vértice se funde', 'NAVMESH', T,
     '                if !(vx || hy) {', '                if true {'),
    # ── a NavMesh (ph2d-nav) ────────────────────────────────────────────────────────────────────
    ('M9 a malha sobreposta passa', 'NAV', MESH,
     '                    .any(|&q| (q as usize) < pi && edge_in(q, u, w).is_some())',
     '                    .any(|&q| false && (q as usize) < pi && edge_in(q, u, w).is_some())'),
    ('M10 o vizinho pela aresta errada', 'NAV', MESH,
     '                    .find_map(|&q| edge_in(q, w, u).map(|j| (q, j)));',
     '                    .find_map(|&q| edge_in(q, u, w).map(|j| (q, j)));'),
    ('M11 a grelha regista o polígono errado', 'NAV', MESH,
     '                grid.items[cursor[c] as usize] = pi as u32;', '                grid.items[cursor[c] as usize] = 0;'),
    # ── as paredes do desvio (ph2d-orca) ────────────────────────────────────────────────────────
    ('M12 fica a ÚLTIMA continuação', 'ORCA', W,
     '            if starting_at[para as usize] == u32::MAX {', '            if true {'),
    # ── a ponte: o obstáculo, as malhas ─────────────────────────────────────────────────────────
    ('M13 o cinemático parado não recorta', 'ECS', BR + 'nav_malha.rs',
     '                let parado = self.world.body_velocity(handle)? == [0.0, 0.0]',
     '                let parado = false && self.world.body_velocity(handle)? == [0.0, 0.0]'),
    ('M14 o que anda recorta', 'ECS', BR + 'nav_malha.rs',
     '                if !parado {', '                if false && !parado {'),
    ('M15 o personagem vira parede', 'ECS', BR + 'nav_malha.rs',
     '            if b.rest.is_sensor || movers.contains(&e) {', '            if b.rest.is_sensor {'),
    ('M16 a rotação não conta para «parado»', 'ECS', BR + 'nav_malha.rs',
     '                    && self.world.body_angvel(handle)? == 0.0;', '                    && true;'),
    ('M17 todos esquecem o caminho', 'ECS', BR + 'nav.rs',
     '            if v.chave.is_some_and(|k| mudou.contains(&k)) {', '            if true {'),
    # ── as ordens ───────────────────────────────────────────────────────────────────────────────
    ('M18 o Stop liga', 'ECS', BR + 'nav_ordens.rs',
     '                PedidoDeNavegacao::Para => ordem.ligado = Some(false),',
     '                PedidoDeNavegacao::Para => ordem.ligado = Some(true),'),
    ('M19 o Start não troca o alvo', 'ECS', BR + 'nav_ordens.rs',
     '                    ordem.alvo = alvo;\n', ''),
    ('M20 a fita não grava', 'ECS', BR + 'nav_ordens.rs',
     '                o.fita.insert(tick, fila.clone());', '                let _ = &fila;'),
    ('M21 a ordem fora do anel', 'ECS', BR + 'tape.rs',
     '            self.nav.ordens.em_vigor = m.nav_ordens.clone();\n', ''),
    ('M22 o recomeço não limpa as ordens', 'ECS', BR + 'rewind.rs',
     '        self.nav.ordens.em_vigor.clear();\n', ''),
    ('M23 a ordem não manda no Active', 'ECS', BR + 'nav.rs',
     '            if !ordem.ligado.unwrap_or(p.active) {', '            if !p.active {'),
    # ── os alvos ────────────────────────────────────────────────────────────────────────────────
    ('M24 o próprio conta como a tag mais perto', 'ECS', BR + 'nav_alvo.rs',
     '                    .filter(|(e, _)| *e != agente)\n', ''),
    ('M25 o mais LONGE', 'ECS', BR + 'nav_alvo.rs',
     '                    .min_by(|a, b| a.0.total_cmp(&b.0));', '                    .max_by(|a, b| a.0.total_cmp(&b.0));'),
    ('M26 a ronda fechada não dá a volta', 'ECS', BR + 'nav_alvo.rs',
     '            r.ponto = (r.ponto + 1) % n;', '            r.ponto = (r.ponto + 1).min(n - 1);'),
    ('M27 a árvore nunca chega', 'ECS', BR + 'nav_alvo.rs',
     '        if self.nav.arvore != *tree {', '        if false {'),
    ('M28 a ronda fora do anel', 'ECS', BR + 'tape.rs',
     '            self.nav.rondas = m.nav_rondas.clone();\n', ''),
    # ── a família: a rota, os verbos, o Inspector, a cena ───────────────────────────────────────
    ('M29 a curva não se parte', 'APPC', APPC + 'nav_rota.rs',
     '    if tol <= 0.0 || b - a <= tol {', '    if true {'),
    ('M30 a rota fica em coordenadas locais', 'APPC', APPC + 'nav_rota.rs',
     '    let ponto = |s: f64| afim.apply(arco.frame_at(s).0);', '    let ponto = |s: f64| arco.frame_at(s).0;'),
    ('M31 o Stop anuncia um Start', 'APPC', APPC + 'signal_actions_bridge.rs',
     '        SignalVerb::StopNavigation => ph2d_physics_ecs::PedidoDeNavegacao::Para,',
     '        SignalVerb::StopNavigation => ph2d_physics_ecs::PedidoDeNavegacao::Anda(0),'),
    ('M32 aceita um alvo sem agente', 'APPC', APPC + 'signal_actions_bridge.rs',
     '    sim.world().get::<ph2d_physics_ecs::NavAgent>(fx.target)?;\n', ''),
    ('M33 a entrega esquece a navegação', 'APPC', APPC + 'signal_actions_bridge.rs',
     '        physics.pede_navegacao(alvo, pedido);', '        let _ = (alvo, pedido);'),
    ('M34 o nome da patrulha escreve um objecto', 'APPC', APPC + 'nav_inspector.rs',
     '            Some(if em_patrulha {', '            Some(if false {'),
    ('M35 um objecto sem forma é forma', 'APPC', APPC + 'nav_inspector.rs',
     '        .is_some_and(|(e, _)| world.get::<ph2d_ecs::VecPathRef>(e).is_some())',
     '        .is_some_and(|(_e, _)| true)'),
    ('M36 a tag escolhida não escreve', 'APPC', APPC + 'nav_inspector.rs',
     '        NavFieldEdit::AlvoTag(t) => a.target = NavTarget::NearestTagged(*t),',
     '        NavFieldEdit::AlvoTag(_) => {}'),
    ('M37 a porta da cena nunca arma', 'APPC', APPC + 'nav_smoke_guarda.rs',
     '                transitions: vec![passo(0, PERSEGUIR, 1), passo(1, SALVO, 2)],',
     '                transitions: vec![passo(0, PERSEGUIR, 1)],'),
    ('M38 o guarda da cena não desiste', 'APPC', APPC + 'nav_smoke_guarda.rs',
     '        a.on_no_path = PERDEU.to_owned();', '        a.on_no_path = String::new();'),
    # ── o verbo, o painel, a queixa ─────────────────────────────────────────────────────────────
    ('M39 as tags dos verbos trocadas', 'VERB', 'crates/ph2d-ecs/src/signal_actions.rs',
     '        SignalVerb::StartNavigation,\n        SignalVerb::StopNavigation,\n    ];',
     '        SignalVerb::StopNavigation,\n        SignalVerb::StartNavigation,\n    ];'),
    ('M40 o Start não lê o nome', 'VERB', 'crates/ph2d-ecs/src/signal_actions.rs',
     '            SignalVerb::StartNavigation => ArgKind::ObjectName,', '            SignalVerb::StartNavigation => ArgKind::None,'),
    ('M41 as opções da tag não são registadas', 'PANEL', PI + 'populate_nav.rs',
     '        .chain(ids::INSP_NAV_TAG_OPT)\n', ''),
    ('M42 o despacho lê a opção ao lado', 'PANEL', PI + 'event_nav.rs',
     '            if let Some(o) = crate::sections::nav_tag_row::nav_tag_options().get(i) {',
     '            if let Some(o) = crate::sections::nav_tag_row::nav_tag_options().get(i + 1) {'),
    # ⚠️ A 1.ª M43 (um `if false` no braço do `match`) NÃO COMPILAVA — o `match` deixava de ser
    # exaustivo. A que fica tira o registo do clique do chip.
    ('M43 o chip da tag não é clicável', 'PANEL', PI + 'sections/nav_tag_row.rs',
     '    hit_index.register(ids::INSP_NAV_TAG_PICK, linha.control);\n', ''),
    ('M44 a forma perdida nunca se queixa', 'EDCORE', 'crates/ph2d-editor-core/src/nav_edits.rs',
     '            NavAlvoModo::Patrulha if self.alvo_perdido => return Some(AgentQueixa::SemForma),',
     '            NavAlvoModo::Patrulha if false && self.alvo_perdido => return Some(AgentQueixa::SemForma),'),
    ('M45 a tag por escolher não se queixa', 'EDCORE', 'crates/ph2d-editor-core/src/nav_edits.rs',
     '            NavAlvoModo::Tag if self.alvo_tag == 0 => return Some(AgentQueixa::SemAlvo),',
     '            NavAlvoModo::Tag if false && self.alvo_tag == 0 => return Some(AgentQueixa::SemAlvo),'),
]

# `MUTA_SO=M2,M3` corre só estas.
so = set(os.environ['MUTA_SO'].split(',')) if os.environ.get('MUTA_SO') else None
# ⭐ Corre só estes grupos (`MUTA_G=ECS,APPC`) — a fatia tem prazo de 30 min.
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


# pré-voo das âncoras (antes do controlo: uma âncora morta não gasta uma corrida limpa)
for nome, g, f, old, new in M:
    n = open(os.path.join(ROOT, f)).read().count(old)
    if n != 1:
        sys.exit(f'ÂNCORA MORTA ({n}x): {nome}')
print(f'pré-voo: as {len(M)} âncoras casam 1x')
if os.environ.get('MUTA_SO_ANCORAS'):
    sys.exit(0)

# controlo: cada grupo limpo verde e com população
for g in sorted(grupos):
    ran, fail, comp, rc, out = corre(g)
    print(f'LIMPO {g}: {ran} testes, {fail} vermelhos, rc={rc}')
    sys.stdout.flush()
    if fail or ran == 0 or not comp or rc != 0:
        print(out[-3000:])
        sys.exit(f'o controlo limpo do grupo {g} não está verde')

sang = 0
tot = 0
maus = 0
for nome, g, f, old, new in M:
    if so and nome.split(' ')[0] not in so:
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
