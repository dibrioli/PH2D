#!/usr/bin/env python3
"""Prova de mutação da W9 da navegação (O CUSTO À ESCALA: a dominância entre frentes na procura
ponderada e o polimento que já não fica preso na quina; a malha sem uma lista por polígono; a fila do
replaneio; e o ORCA que partia com as paredes a excluírem-se).

Backup -> mutar (assert 1 casamento) -> correr SÓ os testes que OBSERVAM a mutação -> restaurar + touch.

Os quatro controlos (o molde da W7/W8):
  (a) a corrida LIMPA de cada observador é VERDE (e com população > 0);
  (b) cada filtro casa >= 1 teste, e CADA teste observador nomeado aparece na saída como corrido;
  (c) o pré-voo das âncoras casa cada uma exactamente 1x (`MUTA_SO_ANCORAS=1` pára aí);
  (d) o controlo da árvore: checksums dos ficheiros tocados iguais antes/depois (senão sai com 2).
Uma mutação que não compila é defeito do ARNÊS, nunca sangria.

Selectores: `MUTA_SO=M1,M3` corre só estas; `MUTA_G=NAV` corre só este grupo (NAV | NAVMESH | ORCA |
PONTE). Todo comando passa por `bash scripts/ph2d-run.sh` (a fatia de 30 min: corra grupo a grupo).
"""
import hashlib, os, re, subprocess, sys

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'
CUSTO = 'crates/ph2d-nav/src/polyanya_custo.rs'
DOM = 'crates/ph2d-nav/src/polyanya_dominancia.rs'
MESH = 'crates/ph2d-nav/src/mesh.rs'
REFRESH = 'crates/ph2d-nav/src/refresh.rs'
AGENT = 'crates/ph2d-nav/src/agent.rs'
TILES = 'crates/ph2d-navmesh/src/tiles.rs'
LP = 'crates/ph2d-orca/src/lp.rs'
FILA = 'crates/ph2d-physics-ecs/src/bridge/nav_fila.rs'
TAPE = 'crates/ph2d-physics-ecs/src/bridge/tape.rs'

DOMI = 'a_dominancia_corta_nos_e_nunca_encarece_um_caminho'
QUINA = 'a_quina_da_lama_nao_prende_o_polimento'
ORDEM = 'a_fila_serve_os_partidos_depois_os_mais_antigos_e_nunca_salta_a_frente'
SOBREP = 'dois_poligonos_sobrepostos_sao_recusados'
CONTORNA = 'o_caminho_contorna_o_furo_pelo_mais_curto'
EXACTO = 'todo_par_do_anel_bate_o_oraculo'
MOSAICO = 'a_juncao_em_t_da_costura_e_reparada'
UM_POR = 'a_porta_que_abre_um_atalho_serve_os_agentes_um_por_tique'
PARTIDO = 'o_caminho_partido_passa_a_frente_na_fila'
SCRUB = 'um_scrub_para_o_meio_da_fila_devolve_a_mesma_corrida'
AVEZ = 'uma_malha_que_nao_para_de_mudar_serve_todos_a_vez'
ENTALADO = 'entalado_entre_duas_paredes_o_3d_nao_parte'
GRELHA = 'a_grelha_das_paredes_da_o_mesmo_que_a_varredura_inteira'
TROCOS = 'so_os_trocos_que_tocam_a_mudanca_se_percorrem'
AREA = 'a_area_que_mudou_e_o_mosaico_da_pedra'
WALLS = 'crates/ph2d-orca/src/walls.rs'

# grupo -> (comando base, os testes observadores do grupo)
G = {
    'NAV': (['cargo', 'test', '-p', 'ph2d-nav', '--lib'], [ORDEM, SOBREP, CONTORNA, EXACTO, TROCOS]),
    'TILES': (['cargo', 'test', '-p', 'ph2d-navmesh', '--lib'], [AREA]),
    'NAVMESH': (['cargo', 'test', '-p', 'ph2d-navmesh', '--test', 'it'], [DOMI, QUINA, MOSAICO]),
    'ORCA': (['cargo', 'test', '-p', 'ph2d-orca', '--lib'], [ENTALADO, GRELHA]),
    'PONTE': (['cargo', 'test', '-p', 'ph2d-physics-ecs', '--test', 'it'],
              [UM_POR, PARTIDO, SCRUB, AVEZ]),
}

M = [
    # ── item 1: a dominância e o polimento ──
    ('M1 a dominância desligada', 'NAVMESH', CUSTO,
     'self.dominancia = !self.sem_dominancia;', 'self.dominancia = false;', (DOMI,)),
    ('M2 a dominância poda a mais (qualquer corte = o nó inteiro)', 'NAVMESH', DOM,
     '            if let Some(s) = corte(&a, rho, g, w, l, r) {\n                if s >= 1.0 {',
     '            if let Some(s) = corte(&a, rho, g, w, l, r) {\n                if s >= 0.0 {',
     (DOMI,)),
    # (a M3 da 1.ª corrida — «o polimento não tira os pontos a mais» — SOBREVIVEU: a sonda deu os
    # mesmos custos ao dígito sem esse passo, e mais depressa. O passo SAIU; a cura é a gama da M4.)
    ('M4 a gama da refracção volta a ser o que a raiz vê', 'NAVMESH', CUSTO,
     '                range: (ar.plo, ar.phi),\n            };\n            self.boundary_root(mesh, root, &ar, x, j, g, w, lado, true, t);',
     '                range: (left, right),\n            };\n            self.boundary_root(mesh, root, &ar, x, j, g, w, lado, true, t);',
     (QUINA,)),
    # ── item 2: a malha contígua ──
    ('M5 a vizinhança procura a aresta errada', 'NAV', MESH,
     '.find(|&&(d, q, _)| d == u && q as usize != pi)', '.find(|&&(d, q, _)| d == w && q as usize != pi)',
     (CONTORNA, EXACTO)),
    ('M6 a sobreposição deixa de ser recusada', 'NAV', MESH,
     '.any(|&(d, q, _)| d == w && (q as usize) < pi)', '.any(|&(d, q, _)| d == w && (q as usize) < pi && false)',
     (SOBREP,)),
    ('M7 as ilhas não atravessam a vizinhança', 'NAV', MESH,
     'for nb in nbrs[a..b].iter().flatten() {', 'for nb in nbrs[a..a].iter().flatten() {',
     (CONTORNA, EXACTO)),
    ('M8 a montagem esquece as junções em T verticais', 'NAVMESH', TILES,
     '                ring.extend(entre(verticais.get(&a.0), a.1, b.1).map(|y| indice[&(a.0, y)]));',
     '', (MOSAICO,)),
    # ── item 3: a fila ──
    ('M9 a fila serve todos no tique', 'PONTE', FILA,
     'let n = serve(&mut fila, self.nav.orcamento);', 'let n = { serve(&mut fila, self.nav.orcamento); fila.len() };',
     (UM_POR,)),
    ('M10 o partido não passa à frente', 'PONTE', REFRESH,
     '        b.broken\n            .cmp(&a.broken)\n            .then(b.ticks.cmp(&a.ticks))',
     '        std::cmp::Ordering::Equal\n            .then(b.ticks.cmp(&a.ticks))',
     (PARTIDO,)),
    ('M11 quem espera não envelhece', 'PONTE', FILA,
     'rt.owed = rt.owed.saturating_add(1);', 'rt.owed = rt.owed.max(1);', (AVEZ,)),
    ('M12 um caminho novo não salda a dívida', 'PONTE', AGENT,
     '        (rt.owed, rt.broken) = (0, false);\n', '', (SCRUB,)),
    ('M13 todo caminho conta como ainda andável', 'PONTE', FILA,
     'path_still_walkable(m, rt, v.pos, onde.as_deref())',
     'true', (PARTIDO,)),
    ('M14 o scrub não devolve a dívida', 'PONTE', TAPE,
     '            self.nav.agents = m.nav.clone();',
     '            self.nav.agents = m.nav.clone();\n            self.nav.agents.values_mut().for_each(|r| r.owed = 0);',
     (SCRUB,)),
    # ── o ORCA ──
    ('M15 o 3D volta a varrer [n_walls..i]', 'ORCA', LP,
     'for lj in &lines[n_walls.min(i)..i] {', 'for lj in &lines[n_walls..i] {', (ENTALADO,)),
    ('M16 a ordem da fila é a da consulta ao mundo, não a das entidades', 'PONTE', FILA,
     'for (i, (&e, v)) in por_entidade.iter().enumerate() {',
     'for (i, (e, v)) in vez.iter().map(|v| (v.p.entity, v)).enumerate() {',
     (UM_POR,)),
    # ── a grelha das paredes (§17.7) ──
    ('M17 a grelha lê só a célula do ponto', 'ORCA', WALLS,
     'let (ax, ay) = g.celula([pos[0] - range, pos[1] - range]);', 'let (ax, ay) = g.celula(pos);',
     (GRELHA,)),
    ('M18 a grelha não tira as repetidas', 'ORCA', WALLS,
     '        found.dedup_by_key(|x| x.1);\n', '', (GRELHA,)),
    # ── o caminho só se percorre onde a malha mudou (§17.7) ──
    ('M19 o troço percorre-se sempre', 'NAV', REFRESH,
     'if !atalho && toca(a, b) && segment_cost', 'if !atalho && segment_cost', (TROCOS,)),
    ('M20 a área que mudou é sempre a malha inteira', 'TILES', TILES,
     '        let refeitos = self.refeitos.as_ref()?;', '        let refeitos: &Vec<(i64, i64)> = None?;',
     (AREA,)),
]

so = set(os.environ['MUTA_SO'].split(',')) if os.environ.get('MUTA_SO') else None
grupos = set(os.environ['MUTA_G'].split(',')) if os.environ.get('MUTA_G') else set(G)


def corre(grupo, filtros):
    base = G[grupo][0]
    c = ['bash', 'scripts/ph2d-run.sh'] + base + ['--'] + filtros
    r = subprocess.run(c, cwd=ROOT, capture_output=True, text=True)
    out = r.stdout + r.stderr
    compila = not ('error[' in out or 'could not compile' in out)
    passed = sum(int(x) for x in re.findall(r'(\d+) passed', out))
    fail = sum(int(x) for x in re.findall(r'(\d+) failed', out))
    corridos = {f for f in filtros if re.search(r'test [^\n]*' + re.escape(f) + r' \.\.\. ', out)}
    return passed + fail, fail, compila, r.returncode, out, corridos


def sha(f):
    return hashlib.sha256(open(os.path.join(ROOT, f), 'rb').read()).hexdigest()


tocados = sorted({m[2] for m in M})
antes = {f: sha(f) for f in tocados}

# (c) pré-voo
for nome, g, f, old, new, obs in M:
    n = open(os.path.join(ROOT, f)).read().count(old)
    if n != 1:
        sys.exit(f'ÂNCORA MORTA ({n}x): {nome}')
    assert set(obs) <= set(G[g][1]), f'observador fora do grupo: {nome}'
print(f'pré-voo: as {len(M)} âncoras casam 1x')
if os.environ.get('MUTA_SO_ANCORAS'):
    sys.exit(0)

# (a)+(b) controlo limpo: cada observador corre e é verde
for g in sorted(grupos):
    obs = G[g][1]
    ran, fail, comp, rc, out, corridos = corre(g, list(obs))
    print(f'LIMPO {g}: {ran} testes, {fail} vermelhos, rc={rc}, observadores corridos {len(corridos)}/{len(obs)}')
    sys.stdout.flush()
    if fail or ran == 0 or not comp or rc != 0 or corridos != set(obs):
        print(out[-3000:])
        sys.exit(f'o controlo limpo do grupo {g} não está verde/completo')

sang = tot = maus = 0
for nome, g, f, old, new, obs in M:
    if so and nome.split(' ')[0] not in so:
        continue
    if g not in grupos:
        continue
    p = os.path.join(ROOT, f)
    s = open(p).read()
    assert s.count(old) == 1
    try:
        open(p, 'w').write(s.replace(old, new))
        ran, fail, comp, rc, out, corridos = corre(g, list(obs))
    finally:
        open(p, 'w').write(s)   # restaura por substituição exacta
        os.utime(p, None)       # o cargo guarda o build da mutação pelo mtime
    tot += 1
    if not comp:
        v = 'NÃO COMPILA (defeito do ARNÊS — REESCREVER)'
        maus += 1
    elif ran == 0 or corridos != set(obs):
        v = 'ZERO testes / observador não correu (arnês) — ABORTO'
        maus += 1
    elif fail > 0:
        vermelhos = ','.join(sorted(set(re.findall(r'test [^\n]*?(\w+) \.\.\. FAILED', out))))
        v = f'SANGRA [{vermelhos}]'
        sang += 1
    else:
        v = 'SOBREVIVEU'
    print(f'{nome} [{g}]: {v} ({fail}/{ran})')
    sys.stdout.flush()

depois = {f: sha(f) for f in tocados}
if depois != antes:
    print('ÁRVORE MUTADA: checksums diferem', [f for f in tocados if antes[f] != depois[f]])
    sys.exit(2)
print('árvore: checksums iguais antes/depois')
parcial = f' (PARCIAL: MUTA_SO={so} MUTA_G={sorted(grupos)})' if (so or grupos != set(G)) else ''
print(f'== {sang} de {tot} sangram{parcial}; {maus} defeitos de arnês')
sys.exit(0 if sang == tot and maus == 0 else 1)
