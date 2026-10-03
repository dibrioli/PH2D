#!/usr/bin/env python3
"""Prova de mutação da W8 da navegação (a ARENA QUE NAVEGA: o morcego que persegue o herói à volta do
muro, a Salamandra imune ao fogo que atravessa a lava, a lava que o morcego evita, e os rótulos do
Inspector que o tutorial cita).

Backup -> mutar (assert 1 casamento) -> correr SÓ os testes que OBSERVAM a mutação -> restaurar + touch.

Os quatro controlos (o molde da W7):
  (a) a corrida LIMPA de cada observador é VERDE (e com população > 0);
  (b) cada filtro casa >= 1 teste, e CADA teste observador nomeado aparece na saída como corrido
      (um filtro que casa nada imprime `ok` — `SOBREVIVEU` falso);
  (c) o pré-voo das âncoras casa cada uma exactamente 1x (`MUTA_SO_ANCORAS=1` pára aí);
  (d) o controlo da árvore: checksums dos ficheiros tocados iguais antes/depois (senão sai com 2).
Uma mutação que não compila é defeito do ARNÊS, nunca sangria.

Selectores: `MUTA_SO=M1,M3` corre só estas; `MUTA_G=APPC` corre só este grupo (APPC | PANEL).
Todo comando passa por `bash scripts/ph2d-run.sh` (a fatia de 30 min: corra grupo a grupo).
"""
import hashlib, os, re, subprocess, sys, shutil

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'
ARENA = 'crates/ph2d-app-components/src/vida_arena_smoke.rs'
I18N = 'crates/ph2d-i18n/src/inspector_nav.rs'

BAT = ('um_morcego_da_a_volta_ao_muro', 'um_morcego_persegue_morde_e_some')
APPC_CMD = ['cargo', 'test', '-p', 'ph2d-app-components', '--lib']
PANEL_CMD = ['cargo', 'test', '-p', 'ph2d-panel-inspector', '--test', 'it']
TUT = 'o_tutorial_da_navegacao_so_cita_rotulos_que_o_painel_pinta'

# grupo -> (comando base, o filtro comum do controlo limpo, os testes observadores do grupo)
G = {
    'APPC': (APPC_CMD, ['vida_arena'], [
        'um_morcego_da_a_volta_ao_muro', 'um_morcego_persegue_morde_e_some',
        'os_morcegos_esperam_na_borda_da_lava', 'a_salamandra_posta_a_perseguir_atravessa_a_lava',
        'a_lava_queima_enquanto_se_pisa_e_depois']),
    'PANEL': (PANEL_CMD, ['o_tutorial_da_navegacao'], [TUT]),
}

NAV_AG = ('            NavAgent {\n                target: NavTarget::Named(stable_name_id(HEROI)),\n'
          '                ..NavAgent::default()\n            },\n')
M = [
    ('M1 o morcego perde o NavAgent', 'APPC', ARENA, NAV_AG, '', BAT),
    ('M2 a região não vê muros', 'APPC', ARENA,
     '            obstacle_layers: u8::MAX,', '            obstacle_layers: 0,',
     ('um_morcego_da_a_volta_ao_muro',)),
    ('M3 o morcego não evita o dano', 'APPC', ARENA,
     '                target: NavTarget::Named(stable_name_id(HEROI)),\n                ..NavAgent::default()',
     '                target: NavTarget::Named(stable_name_id(HEROI)),\n                avoid_harm: false,\n                ..NavAgent::default()',
     ('os_morcegos_esperam_na_borda_da_lava',)),
    ('M4 a Salamandra fica estática', 'APPC', ARENA,
     '                SALAMANDRA_RGBA,\n            ),\n            RigidBody {\n                kind: BodyKind::Kinematic,',
     '                SALAMANDRA_RGBA,\n            ),\n            RigidBody {\n                kind: BodyKind::Static,',
     ('a_salamandra_posta_a_perseguir_atravessa_a_lava',)),
    ('M5 a Salamandra deixa de ser imune ao fogo', 'APPC', ARENA,
     '                        kind: FOGO.to_owned(),\n                        rate: 0.0,',
     '                        kind: FOGO.to_owned(),\n                        rate: 1.0,',
     ('a_salamandra_posta_a_perseguir_atravessa_a_lava',)),
    ('M6 o muro é sensor (não há muro)', 'APPC', ARENA,
     '                half_y: (MURO_CIMA - MURO_BAIXO) / 2.0,\n            },\n            ..Collider::default()',
     '                half_y: (MURO_CIMA - MURO_BAIXO) / 2.0,\n            },\n            is_sensor: true,\n            ..Collider::default()',
     ('um_morcego_da_a_volta_ao_muro',)),
    ('M7 o morcego chega longe demais', 'APPC', ARENA,
     '                target: NavTarget::Named(stable_name_id(HEROI)),\n                ..NavAgent::default()',
     '                target: NavTarget::Named(stable_name_id(HEROI)),\n                arrive_distance: 1.0,\n                ..NavAgent::default()',
     BAT),
    ('M8 a lava não magoa quem evita', 'APPC', ARENA,
     '            over_time_per_s: LAVA_POR_S,', '            over_time_per_s: 0.0,',
     ('os_morcegos_esperam_na_borda_da_lava', 'a_lava_queima_enquanto_se_pisa_e_depois')),
    ('M9 o rótulo Avoid Harm muda', 'PANEL', I18N,
     '"panel.inspector.nav.avoid_harm" => "Avoid Harm",', '"panel.inspector.nav.avoid_harm" => "Avoid Danger",', (TUT,)),
    ('M10 o rótulo Object muda', 'PANEL', I18N,
     '"panel.inspector.nav.target_object" => "Object",', '"panel.inspector.nav.target_object" => "Objeto",', (TUT,)),
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
print(f'pré-voo: as {len(M)} âncoras casam 1x')
if os.environ.get('MUTA_SO_ANCORAS'):
    sys.exit(0)

# (a)+(b) controlo limpo: cada observador corre e é verde
for g in sorted(grupos):
    obs = G[g][2]
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
