#!/usr/bin/env python3
"""Prova de mutação da W7 da navegação (o CUSTO POR ÁREA e os ATALHOS: a malha com áreas, o Polyanya
que refracta, os atalhos na lei do agente, a lava/lama/proibida/teleporte na ponte, o Inspector e a
cena `=4`).

Backup -> mutar (assert 1 casamento) -> correr SÓ o grupo que OBSERVA a mutação -> restaurar + touch.

Os quatro controlos (o molde da W4/W5/W6): a corrida limpa de cada grupo é VERDE e corre > 0 testes; o
pré-voo das âncoras casa cada uma exactamente uma vez (`MUTA_SO_ANCORAS=1` pára aí); uma mutação que
não compila é acusada como defeito do ARNÊS e nunca como sangria; um grupo que corre zero testes aborta.
"""
import os, re, subprocess, sys, shutil

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'

G = {
    'NAVMESH': ['-p', 'ph2d-navmesh', '-E', 'test(areas) | test(custo) | test(mosaicos) | test(atalhos)'],
    'ECS': ['-p', 'ph2d-physics-ecs', '-E', 'test(nav_custo)'],
    'APPC': ['-p', 'ph2d-app-components', '-E', 'test(nav_smoke_lava) | test(nav_inspector)'],
    'PANEL': ['-p', 'ph2d-panel-inspector', '-E', 'test(nav)'],
    'EDCORE': ['-p', 'ph2d-editor-core', '-E', 'test(nav_edits)'],
}

TRI = 'crates/ph2d-navmesh/src/triangulate.rs'
LIB = 'crates/ph2d-navmesh/src/lib.rs'
T = 'crates/ph2d-navmesh/src/tiles.rs'
PA = 'crates/ph2d-nav/src/polyanya.rs'
PC = 'crates/ph2d-nav/src/polyanya_custo.rs'
AG = 'crates/ph2d-nav/src/agent.rs'
LK = 'crates/ph2d-nav/src/link.rs'
BR = 'crates/ph2d-physics-ecs/src/bridge/'
HE = 'crates/ph2d-physics-ecs/src/components/health.rs'
APPC = 'crates/ph2d-app-components/src/'

M = [
    # ── a construção com áreas (ph2d-navmesh) ───────────────────────────────────────────────────
    ('M1 a fusão atravessa áreas', 'NAVMESH', TRI,
     '        if a == b || labels[a] != labels[b] {', '        if a == b {'),
    ('M2 a junção em T só a uma unidade', 'NAVMESH', TRI,
     'if p == a || p == b || o * o >= 4 * len2(a, b) {', 'if p == a || p == b || o * o >= len2(a, b) {'),
    ('M3 o ambíguo fica como estava', 'NAVMESH', TRI,
     '            _ => None,\n        }\n    };', '            _ => Some(estado),\n        }\n    };'),
    ('M4 a 1.ª área não manda', 'NAVMESH', LIB,
     '            piece = difference_64(&piece, &reclamado, FillRule::NonZero);', '            let _ = &reclamado;'),
    ('M5 o mosaico não assina as áreas', 'NAVMESH', T,
     '                        quais.iter().for_each(|&i| h.u64(assin_areas[i]));', '                        let _ = &assin_areas;'),
    ('M6 o mosaico esquece as áreas', 'NAVMESH', T,
     '                (corta(&anel, lo, hi), a.id)', '                (Vec::new(), a.id)'),
    # ── a procura que refracta (ph2d-nav) ───────────────────────────────────────────────────────
    ('M7 sem a refracção', 'NAVMESH', PA,
     '        if self.cost(mesh, p) != cw {', '        if false && self.cost(mesh, p) != cw {'),
    ('M8 o canto de custo não dobra', 'NAVMESH', PC,
     '                .any(|&q| self.cost(mesh, q) > cw)', '                .any(|&q| false && self.cost(mesh, q) > cw)'),
    ('M9 sem o deslize', 'NAVMESH', PC,
     '        if slide && w_outro > w_into {', '        if false && slide && w_outro > w_into {'),
    ('M10 sem o polimento de Snell', 'NAVMESH', PC,
     '            polish(mesh, &self.costs, &mut pts);', '            let _ = &pts;'),
    ('M11 o atalho exacto mente', 'NAVMESH', PC,
     '        if c0 <= wmin * p0.length * (1.0 + 1e-12) + EPS {', '        if true {'),
    ('M12 dentro da lama sempre a direito', 'NAVMESH', PA,
     '            && c <= self.wmin', '            && true'),
    ('M13 o leque atravessa a fronteira de custo', 'NAVMESH', PA,
     '            if cq != cp {\n                return;\n            }', '            let _ = (cq, cp);'),
    # ── os atalhos (ph2d-nav) ───────────────────────────────────────────────────────────────────
    ('M14 o teleporte conta duas vezes', 'NAVMESH', AG,
     '            && !h.teleport\n        {\n            crossed = Some(h.link);', '        {\n            crossed = Some(h.link);'),
    ('M15 os atalhos não entram no plano', 'NAVMESH', AG,
     '    if !q.links.is_empty()\n', '    if false\n'),
    ('M16 um sentido volta', 'NAVMESH', LK,
     '        if l.two_way {', '        if true {'),
    ('M17 atravessar é de graça', 'NAVMESH', LK,
     '        andar + self.cost.max(0.0)', '        andar'),
    ('M18 replaneia a meio da porta', 'NAVMESH', AG,
     '    let numa_porta = rt.next >= 1 && rt.hop_at(rt.next - 1).is_some_and(|h| !h.teleport);',
     '    let numa_porta = false;'),
    # ── a ponte (ph2d-physics-ecs) ──────────────────────────────────────────────────────────────
    ('M19 nenhuma zona magoa', 'ECS', HE,
     '        tira && self.fere(alvo) && taxa.fator() > 0.0 && !taxa.absorve', '        false && tira'),
    ('M20 o imune evita', 'ECS', HE,
     '        tira && self.fere(alvo) && taxa.fator() > 0.0 && !taxa.absorve', '        tira && self.fere(alvo) && !taxa.absorve'),
    ('M21 quem se cura com o fogo evita-o', 'ECS', HE,
     '        tira && self.fere(alvo) && taxa.fator() > 0.0 && !taxa.absorve', '        tira && self.fere(alvo) && taxa.fator() > 0.0'),
    ('M22 a caixa Avoid Harm não manda', 'ECS', BR + 'nav_custo.rs',
     '.filter(|_| avoid_harm)', '.filter(|_| true)'),
    ('M23 a proibida não fura', 'ECS', BR + 'nav_malha.rs',
     '                .chain(custos.proibidas.iter())\n', ''),
    ('M24 a área não chega à malha', 'ECS', BR + 'nav_malha.rs',
     '            if malha.update_with_areas(&poligono, &dela, &custos.areas) {', '            if malha.update_with_areas(&poligono, &dela, &[]) {'),
    ('M25 a mais BARATA manda', 'ECS', BR + 'nav_custo.rs',
     '        finitas.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));', '        finitas.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));'),
    ('M26 o teleporte não move o corpo', 'ECS', BR + 'nav.rs',
     '                self.world\n                    .set_body_pose(b.handle, p2[0] as f32, p2[1] as f32, rot, true);', '                let _ = (rot, p2);'),
    ('M27 atravessar não vira facto', 'ECS', BR + 'nav.rs',
     '                    kind: Event::Crossed(id),\n                });', '                    kind: Event::Arrived,\n                });\n                let _ = id;'),
    ('M28 a chave ignora o que o agente evita', 'APPC', BR + 'nav.rs',
     '.map(|r| (r.entity, chave_raio, custo::assinatura(&quem)));', '.map(|r| (r.entity, chave_raio, 0));'),
    # ── a cena ──────────────────────────────────────────────────────────────────────────────────
    ('M29 o controlo também evita', 'APPC', APPC + 'nav_smoke_lava.rs',
     '        CINZENTO_RGBA,\n        false,\n', '        CINZENTO_RGBA,\n        true,\n'),
    ('M30 os espigões do Clipper ficam', 'NAVMESH', TRI,
     '.map(|r| limpa_anel(r))', '.map(|r| r.clone())'),
    ('M31 a chegada reanuncia-se a cada empurrão', 'NAVMESH', AG,
     '        if rt.arrival_told {\n            event = None;\n        }', '        let _ = rt.arrival_told;'),
    ('M32 ido embora não reanuncia', 'NAVMESH', AG,
     '        rt.arrival_told = false;\n', ''),
    # ── o Inspector ─────────────────────────────────────────────────────────────────────────────
    ('M33 Avoid Harm não escreve', 'APPC', APPC + 'nav_inspector.rs',
     '        NavFieldEdit::AvoidHarm(b) => a.avoid_harm = *b,', '        NavFieldEdit::AvoidHarm(_) => {}'),
    ('M34 Forbidden não escreve', 'APPC', APPC + 'nav_inspector.rs',
     '                NavFieldEdit::CostAreaForbidden(b) => c.forbidden = *b,', '                NavFieldEdit::CostAreaForbidden(_) => {}'),
    ('M35 o custo da área aceita 0', 'APPC', APPC + 'nav_inspector.rs',
     '                NavFieldEdit::CostAreaCost(v) => c.cost = v.max(NAV_AREA_COST_MIN),', '                NavFieldEdit::CostAreaCost(v) => c.cost = *v,'),
    ('M36 a saída guarda o nome com espaços', 'APPC', APPC + 'nav_inspector.rs',
     '                    let t = nome.trim();\n', '                    let t = nome.as_str();\n'),
    ('M37 Teleport não escreve', 'APPC', APPC + 'nav_inspector.rs',
     '                NavFieldEdit::LinkTeleport(b) => l.teleport = *b,', '                NavFieldEdit::LinkTeleport(_) => {}'),
    ('M38 o clique em Avoid Harm pede o mesmo', 'PANEL', 'crates/ph2d-panel-inspector/src/event_nav.rs',
     '            Some(NavFieldEdit::AvoidHarm(!info.agent.as_ref()?.avoid_harm))', '            Some(NavFieldEdit::AvoidHarm(info.agent.as_ref()?.avoid_harm))'),
    ('M39 o clique em Forbidden pede o mesmo', 'PANEL', 'crates/ph2d-panel-inspector/src/event_nav.rs',
     '            Some(NavFieldEdit::CostAreaForbidden(!info.cost_area?.forbidden))', '            Some(NavFieldEdit::CostAreaForbidden(info.cost_area?.forbidden))'),
    ('M40 a vida não chega ao espelho', 'APPC', APPC + 'nav_inspector.rs',
     '        has_health: world.get::<Health>(e).is_some(),', '        has_health: true,'),
    ('M41 a frase «sem vida» cala-se', 'EDCORE', 'crates/ph2d-editor-core/src/nav_edits.rs',
     '        self.avoid_harm && !self.has_health', '        false'),
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
