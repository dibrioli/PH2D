#!/usr/bin/env python3
"""Prova de mutação da W19 (plano 30 §28: a cota pela distância às áreas baratas — o heurístico, a saída
cedo, a poda dos atalhos e o «alvo à vista» —, a queixa da área barata mais estreita que o corpo e a
cena `=7`).

O motor e os quatro controlos são os da W15/W16/W18, copiados verbatim (cada grupo corre só os
observadores dele, `OBS_DO_GRUPO`; o controlo limpo corre todos).
Selectores: `MUTA_SO=H1,V2` · `MUTA_G=COTA` (COTA | VISTA | QUEIXA | CENA).
"""
import hashlib, os, re, subprocess, sys

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'
POLY = 'crates/ph2d-nav/src/polyanya.rs'
FATIAS = 'crates/ph2d-nav/src/polyanya_fatias.rs'
COTA = 'crates/ph2d-nav/src/cota.rs'
LINK = 'crates/ph2d-nav/src/link.rs'
AG = 'crates/ph2d-nav/src/agent.rs'
NAV = 'crates/ph2d-physics-ecs/src/bridge/nav.rs'
CUSTO_PONTE = 'crates/ph2d-physics-ecs/src/bridge/nav_custo.rs'
EDITS = 'crates/ph2d-editor-core/src/nav_edits.rs'
PAINEL = 'crates/ph2d-panel-inspector/src/sections/nav_custo.rs'
NS = 'crates/ph2d-app-components/src/nav_smoke.rs'

OBS = {
    'NAVMESH': (['cargo', 'test', '-p', 'ph2d-navmesh', '--test', 'it'],
                ['cota::uma_area_barata_longe_nao_pesa_na_procura',
                 'cota::a_cota_nunca_encarece_um_caminho_e_o_oraculo_confirma',
                 'cota::quando_a_vista_diz_sim_nenhum_caminho_e_mais_barato',
                 'cota::com_atalhos_a_poda_pela_cota_nunca_perde_o_mais_barato',
                 'cota::o_atalho_que_acaba_numa_estrada_barata_nao_e_podado',
                 'cota::com_a_estrada_longe_a_saida_cedo_poupa_a_ponderada',
                 'fatias::a_procura_em_fatias_e_a_procura_inteira_ao_bit',
                 'atalhos::a_chegada_anuncia_se_uma_vez_por_aproximacao',
                 'atalhos::o_teleporte_leva_a_outra_sala_e_sem_ele_e_parcial',
                 'atalhos::o_atalho_so_se_usa_quando_compensa',
                 'atalhos::a_meio_da_porta_nao_se_replaneia_mesmo_com_o_alvo_a_andar',
                 'atalhos::a_porta_de_um_sentido_nao_volta']),
    'NAV': (['cargo', 'test', '-p', 'ph2d-nav', '--lib'],
            ['agent::tests::o_alvo_a_vista_e_so_a_recta_que_nenhum_caminho_bate']),
    'IT': (['cargo', 'test', '-p', 'ph2d-physics-ecs', '--test', 'it'],
           ['nav_custo::uma_area_barata_longe_nao_desliga_o_alvo_a_vista',
            'nav_custo::a_ponte_publica_a_area_barata_mais_estreita_que_o_corpo']),
    'EDCORE': (['cargo', 'test', '-p', 'ph2d-editor-core', '--lib'],
               ['nav_edits::tests::a_area_barata_mais_estreita_que_o_corpo_queixa_se']),
    'PANEL': (['cargo', 'test', '-p', 'ph2d-panel-inspector', '--test', 'it'],
              ['a_seccao_nav_custo_esta_viva::a_area_barata_mais_estreita_que_o_corpo_diz_se_no_painel']),
    'COMP': (['cargo', 'test', '-p', 'ph2d-app-components', '--lib'],
             ['nav_smoke_estreita::tests::a_larga_anda_se_a_estreita_nao_faz_nada',
              'nav_smoke_estreita::tests::a_estreita_queixa_se_e_a_larga_nao',
              'nav_smoke_estreita::tests::a_cena_tem_as_pecas_que_o_roteiro_nomeia',
              'nav_smoke::tests::o_cenas_conta_os_niveis_do_roteador']),
}
OBS_DO_GRUPO = {
    'COTA': ['NAVMESH', 'NAV', 'IT'],
    'VISTA': ['NAV', 'NAVMESH', 'IT'],
    'QUEIXA': ['IT', 'EDCORE', 'PANEL', 'COMP'],
    'CENA': ['COMP'],
}

# (nome, grupo, ficheiro, âncora, substituição)
M = [
    ('H1 o heurístico volta à cota global', 'COTA', POLY,
     '                r.d + self.d_alvo,\n', '                0.0,\n'),
    ('H2 a saída cedo pela cota global', 'COTA', FATIAS,
     'inferior(w, geral.length, d_fora)', 'inferior(w, geral.length, 0.0)'),
    ('H3 a cota esquece a área barata (inadmissível)', 'COTA', COTA,
     '        w * l + (1.0 - w) * d_fora\n', '        l\n'),
    ('H4 a distância às áreas baratas é sempre infinita', 'COTA', COTA,
     '        m2.sqrt()\n', '        f64::INFINITY\n'),
    ('H5 a poda dos atalhos esquece a área barata', 'COTA', LINK,
     '                                self.d[u] + self.d[atual],', '                                f64::INFINITY,'),
    ('V1 o «à vista» volta à regra da W16', 'VISTA', AG,
     '.entre(a, b) + EPS)\n', '.entre(a, b) + EPS && q.costs.iter().all(|&x| x >= 1.0))\n'),
    ('V2 o «à vista» ignora a área barata', 'VISTA', AG,
     'c <= crate::cota::Cota::nova(mesh, q.costs, false).entre(a, b) + EPS',
     'c <= dist(a, b) + EPS'),
    ('Q1 a ponte não publica', 'QUEIXA', NAV,
     '        self.publica_areas_estreitas(sim);\n', ''),
    ('Q2 a cara também se publica', 'QUEIXA', CUSTO_PONTE,
     'if a.forbidden || a.cost >= 1.0 {', 'if a.forbidden || a.cost >= 10.0 {'),
    ('Q3 o raio publicado não é o da malha', 'QUEIXA', CUSTO_PONTE,
     '.map(|&r| r as f32 / RAIO_POR_METRO)', '.map(|&r| r as f32 / RAIO_POR_METRO * 0.5)'),
    ('Q4 a queixa calada no Inspector', 'QUEIXA', EDITS,
     '&& self.too_narrow_for.is_some()', '&& false'),
    ('Q5 o painel pinta outra frase', 'QUEIXA', PAINEL,
     'CostAreaQueixa::MaisEstreitaQueOCorpo => "panel.inspector.nav.area_narrower_than_body"',
     'CostAreaQueixa::MaisEstreitaQueOCorpo => "panel.inspector.nav.area_body_moves"'),
    ('S1 o roteador sem a cena 7', 'CENA', NS, '    if nivel == 7 {', '    if nivel == 77 {'),
    ('S2 a shell abre a secção do agente na 7', 'CENA', NS,
     ' || self.nivel == 7 {', ' {'),
    ('S3 o CENAS fica uma abaixo', 'CENA', NS, 'pub const CENAS: u32 = 7;', 'pub const CENAS: u32 = 6;'),
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
