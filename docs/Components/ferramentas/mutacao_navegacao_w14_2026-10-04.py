#!/usr/bin/env python3
"""Prova de mutação da W14 da navegação (A VELOCIDADE EM CENAS GRANDES: o trabalho da procura na fila,
os mosaicos em paralelo, os vizinhos por anéis, e o corpo largo que anda em polígono com velocidade).

O motor e os quatro controlos são os da W10/W13, copiados verbatim — com UMA mudança declarada: cada
grupo corre só os observadores dele (`OBS_DO_GRUPO`); o controlo limpo corre todos.
Selectores: `MUTA_SO=T1,V2` · `MUTA_G=TRABALHO` (TRABALHO | PARALELO | VIZINHOS | MOVEL).
"""
import hashlib, os, re, subprocess, sys

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'
PY = 'crates/ph2d-nav/src/polyanya.rs'
DO = 'crates/ph2d-nav/src/polyanya_dominancia.rs'
TR = 'crates/ph2d-nav/src/polyanya_trabalho.rs'
CU = 'crates/ph2d-nav/src/polyanya_custo.rs'
AG = 'crates/ph2d-nav/src/agent.rs'
NV = 'crates/ph2d-physics-ecs/src/bridge/nav.rs'
TI = 'crates/ph2d-navmesh/src/tiles.rs'
VZ = 'crates/ph2d-orca/src/vizinhos.rs'
CR = 'crates/ph2d-orca/src/crowd.rs'
DE = 'crates/ph2d-physics-ecs/src/bridge/nav_desvio.rs'

OBS = {
    'NAV': (['cargo', 'test', '-p', 'ph2d-nav', '--lib'],
            ['agent::tests::a_ultima_procura_guarda_o_trabalho_e_nao_os_nos']),
    'MESH_IT': (['cargo', 'test', '-p', 'ph2d-navmesh', '--test', 'it'],
                ['dominancia::o_trabalho_sem_lama_sao_os_nos_e_na_lama_pesa_o_que_custa',
                 'mosaicos::incremental_e_a_frio_dao_o_mesmo']),
    'MESH_LIB': (['cargo', 'test', '-p', 'ph2d-navmesh', '--lib'],
                 ['tiles::tests::os_mosaicos_feitos_em_paralelo_sao_os_de_uma_thread_ao_bit',
                  'tiles::tests::a_montagem_por_blocos_e_a_montagem_inteira_ao_bit']),
    'ORCA_LIB': (['cargo', 'test', '-p', 'ph2d-orca', '--lib'],
                 ['vizinhos_tests::os_vizinhos_por_aneis_sao_os_da_varrida']),
    'ORCA_IT': (['cargo', 'test', '-p', 'ph2d-orca', '--test', 'it'],
                ['oraculo_do_godot_largo::o_corpo_largo_em_poligono_contorna_e_em_discos_prende_como_no_godot',
                 'banco_de_cenarios::ninguem_se_sobrepoe_ninguem_entra_na_parede_e_todos_chegam',
                 # (2.ª corrida) L2 SOBREVIVIA: os desfechos não mudam; a lei é a do referencial.
                 'movel::um_corpo_que_anda_e_o_mesmo_parado_no_referencial_dele']),
    'IT': (['cargo', 'test', '-p', 'ph2d-physics-ecs', '--test', 'it'],
           ['nav_nascer::na_lama_a_vez_de_quem_nasce_conta_o_trabalho',
            'nav_desvio_largo::um_corpo_largo_que_vem_de_frente_e_contornado',
            'nav_desvio_largo::uma_capsula_que_anda_e_um_torniquete_que_roda_sao_contornados_pela_forma',
            'nav_desvio_corpos::um_corpo_composto_desvia_se_pela_forma_inteira']),
}
OBS_DO_GRUPO = {
    'TRABALHO': ['NAV', 'MESH_IT', 'IT'],
    'PARALELO': ['MESH_LIB', 'MESH_IT'],
    'VIZINHOS': ['ORCA_LIB'],
    'MOVEL': ['ORCA_IT', 'IT'],
}

# (nome, grupo, ficheiro, âncora, substituição)
M = [
    ('T1 as raízes de fronteira não contam', 'TRABALHO', PY,
     '                    self.stats.pending += 1;\n', ''),
    ('T2 as frentes comparadas não contam', 'TRABALHO', DO,
     '            self.stats.compared += 1;\n', ''),
    ('T3 o trabalho sem os pesos', 'TRABALHO', TR,
     '        self.expanded\n            + (OITAVOS_POR_PENDENTE',
     '        self.expanded\n            + 0 * (OITAVOS_POR_PENDENTE'),
    ('T4 o agente guarda os nós', 'TRABALHO', AG,
     '        let antes = search.stats.work();\n        let planeado = plan(mesh, search, q, pos, t);\n        rt.last_work = search.stats.work() - antes;',
     '        let antes = search.stats.expanded;\n        let planeado = plan(mesh, search, q, pos, t);\n        rt.last_work = search.stats.expanded - antes;'),
    ('T5 a 2.ª passagem conta os nós', 'TRABALHO', NV,
     '            let antes = search.stats.work();\n',
     '            let antes = search.stats.work() - search.stats.work() + search.stats.expanded + (search.stats.work() - search.stats.work());\n'),
    ('T6 uma tabela sem nada abaixo de 1 é uniforme', 'TRABALHO', CU,
     '        let uniforme = costs.iter().all(|&c| c == 1.0)',
     '        let uniforme = costs.iter().all(|&c| c >= 1.0)'),
    ('P1 os mosaicos entram pela ordem trocada', 'PARALELO', TI,
     '            for ((k, _), (m, falhou)) in faltam.into_iter().zip(feitos) {',
     '            for ((k, _), (m, falhou)) in faltam.into_iter().rev().zip(feitos) {'),
    ('V1 o minorante sem a folga da célula', 'VIZINHOS', VZ,
     '        ((r as f64 + folga / self.lado - 1e-6) * self.lado, tudo)',
     '        ((r as f64 + 1.0) * self.lado + 0.0 * folga, tudo)'),
    ('V2 pára a meio alcance', 'VIZINHOS', CR,
     '                if tudo || n == 0 || fora >= self.cell {',
     '                if tudo || n == 0 || fora >= 0.5 * self.cell {'),
    ('V3 o anel sem a última linha das colunas', 'VIZINHOS', VZ,
     '            for y in y0 + 1..y1 {',
     '            for y in y0 + 1..y1 - 1 {'),
    ('V4 pára com o n-ésimo EMPATADO', 'VIZINHOS', CR,
     '                    if found[n - 1].0 < fora * fora {',
     '                    if found[n - 1].0 <= (fora + self.cell) * (fora + self.cell) {'),
    ('L1 as linhas do móvel sem a velocidade dele', 'MOVEL', CR,
     '                    point: add(l.point, u),',
     '                    point: add(l.point, scale(u, 0.0)),'),
    ('L2 a velocidade do agente não é a relativa', 'MOVEL', CR,
     '                    vel: sub(a.vel, u),',
     '                    vel: sub(a.vel, scale(u, 0.0)),'),
    ('L3 sem a folga do que ele anda', 'MOVEL', CR,
     '                let folga = len(u) * tau_m;',
     '                let folga = 0.0 * len(u) * tau_m;'),
    ('L4 os polígonos fora do desvio', 'MOVEL', DE,
     '                if !poligonos.is_empty() {',
     '                if false && !poligonos.is_empty() {'),
    ('L5 a velocidade de um ponto sem a rotação', 'MOVEL', CR,
     '            self.vel[0] - self.omega * r[1],\n            self.vel[1] + self.omega * r[0],',
     '            self.vel[0] - 0.0 * self.omega * r[1],\n            self.vel[1] + 0.0 * self.omega * r[0],'),
    ('L6 o octógono vira losango (inscrito)', 'MOVEL', DE,
     '    let t = 2.0_f64.sqrt() - 1.0;',
     '    let t = 0.0 * (2.0_f64.sqrt() - 1.0);'),
]

so = set(os.environ['MUTA_SO'].split(',')) if os.environ.get('MUTA_SO') else None
grupos = set(os.environ['MUTA_G'].split(',')) if os.environ.get('MUTA_G') else {m[1] for m in M}


def corre(conj, filtros):
    base = OBS[conj][0]
    c = ['bash', 'scripts/ph2d-run.sh'] + base + ['--'] + filtros
    r = subprocess.run(c, cwd=ROOT, capture_output=True, text=True)
    out = r.stdout + r.stderr
    compila = not ('error[' in out or 'could not compile' in out)
    passed = sum(int(x) for x in re.findall(r'(\d+) passed', out))
    fail = sum(int(x) for x in re.findall(r'(\d+) failed', out))
    corridos = {f for f in filtros if re.search(r'test [^\n]*' + re.escape(f) + r'(?: - should panic)? \.\.\. ', out)}
    return passed + fail, fail, compila, r.returncode, out, corridos


def compila_ok(out):
    return not ('error[' in out or 'could not compile' in out)


def corridos_ok(out):
    """Os testes que chegaram a imprimir `ok`/`FAILED` (os que não, ficaram por acabar)."""
    return {n for n in re.findall(r'test [^\n]*?(\w+)(?: - should panic)? \.\.\. (?:ok|FAILED)', out)}


def sha(f):
    return hashlib.sha256(open(os.path.join(ROOT, f), 'rb').read()).hexdigest()


tocados = sorted({m[2] for m in M})
antes = {f: sha(f) for f in tocados}

# (c) pré-voo
for nome, g, f, old, new in M:
    n = open(os.path.join(ROOT, f)).read().count(old)
    if n != 1:
        sys.exit(f'ÂNCORA MORTA ({n}x): {nome}')
    assert old != new, nome
print(f'pré-voo: as {len(M)} âncoras casam 1x')
if os.environ.get('MUTA_SO_ANCORAS'):
    sys.exit(0)

# (a)+(b) controlo limpo: cada observador corre e é verde
for conj in OBS:
    obs = OBS[conj][1]
    ran, fail, comp, rc, out, corridos = corre(conj, list(obs))
    print(f'LIMPO {conj}: {ran} testes, {fail} vermelhos, rc={rc}, observadores corridos {len(corridos)}/{len(obs)}')
    sys.stdout.flush()
    if fail or ran == 0 or not comp or rc != 0 or corridos != set(obs):
        print(out[-3000:])
        sys.exit(f'o controlo limpo do conjunto {conj} não está verde/completo')

sang = tot = maus = 0
for nome, g, f, old, new in M:
    if so and nome.split(' ')[0] not in so:
        continue
    if g not in grupos:
        continue
    p = os.path.join(ROOT, f)
    s = open(p).read()
    assert s.count(old) == 1
    vermelhos, mau, resumo, morte = [], None, [], False
    try:
        open(p, 'w').write(s.replace(old, new))
        for conj in OBS_DO_GRUPO[g]:
            ran, fail, comp, rc, out, corridos = corre(conj, list(OBS[conj][1]))
            if not comp:
                mau = f'NÃO COMPILA em {conj} (defeito do ARNÊS — REESCREVER)'
                break
            if '(signal:' in out and compila_ok(out):
                # o binário morreu por sinal (o tecto de memória/prazo da fatia): a mutação fez o
                # observador não terminar — é uma MORTE observada, marcada à parte (não um «vermelho»
                # limpo); só vale se outro observador ficou vermelho de forma limpa (ver abaixo).
                vermelhos.append(f'{conj}:MORTE-POR-SINAL(ficaram por acabar: '
                                 + ','.join(sorted(set(OBS[conj][1]) - corridos_ok(out))) + ')')
                resumo.append(f'{conj} morto')
                morte = True
                continue
            if ran == 0 or corridos != set(OBS[conj][1]):
                mau = f'ZERO testes / observador não correu em {conj} (arnês) — ABORTO'
                break
            nomes = sorted(set(re.findall(r'test [^\n]*?(\w+)(?: - should panic)? \.\.\. FAILED', out)))
            vermelhos += [f'{conj}:{n}' for n in nomes]
            resumo.append(f'{conj} {fail}/{ran}')
    finally:
        open(p, 'w').write(s)   # restaura por substituição exacta
        os.utime(p, None)       # o cargo guarda o build da mutação pelo mtime
    tot += 1
    if mau:
        v = mau
        maus += 1
    elif vermelhos and morte and not any('MORTE' not in x for x in vermelhos):
        v = 'SÓ MORTE POR SINAL (não é sangria limpa) [' + ', '.join(vermelhos) + ']'
        maus += 1
    elif vermelhos:
        v = 'SANGRA [' + ', '.join(vermelhos) + ']'
        sang += 1
    else:
        v = 'SOBREVIVEU'
    print(f'{nome} [{g}]: {v} ({"; ".join(resumo)})')
    sys.stdout.flush()

depois = {f: sha(f) for f in tocados}
if depois != antes:
    print('ÁRVORE MUTADA: checksums diferem', [f for f in tocados if antes[f] != depois[f]])
    sys.exit(2)
print('árvore: checksums iguais antes/depois')
parcial = f' (PARCIAL: MUTA_SO={so} MUTA_G={sorted(grupos)})' if (so or grupos != {m[1] for m in M}) else ''
print(f'== {sang} de {tot} sangram{parcial}; {maus} defeitos de arnês')
sys.exit(0 if sang == tot and maus == 0 else 1)
