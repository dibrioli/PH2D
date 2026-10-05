#!/usr/bin/env python3
"""Prova de mutação da W15 da navegação (O TIQUE DEPOIS DA PORTA, COM LAMA: a procura em fatias, toda
procura paga do orçamento, as procuras a meio em paralelo, e «mudou» pelo conteúdo — plano 30 §23).

O motor e os quatro controlos são os da W14, copiados verbatim (cada grupo corre só os observadores
dele, `OBS_DO_GRUPO`; o controlo limpo corre todos).
Selectores: `MUTA_SO=F1,V2` · `MUTA_G=FATIAS` (FATIAS | VEZ | PARALELO | SCRUB).
"""
import hashlib, os, re, subprocess, sys

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'
FA = 'crates/ph2d-nav/src/polyanya_fatias.rs'
PL = 'crates/ph2d-nav/src/plano.rs'
LK = 'crates/ph2d-nav/src/link.rs'
AG = 'crates/ph2d-nav/src/agent.rs'
RF = 'crates/ph2d-nav/src/refresh.rs'
NV = 'crates/ph2d-physics-ecs/src/bridge/nav.rs'
FI = 'crates/ph2d-physics-ecs/src/bridge/nav_fila.rs'
ML = 'crates/ph2d-physics-ecs/src/bridge/nav_malha.rs'
TP = 'crates/ph2d-physics-ecs/src/bridge/tape.rs'

OBS = {
    'NAV': (['cargo', 'test', '-p', 'ph2d-nav', '--lib'],
            ['refresh::tests::a_fila_reparte_pelos_partidos_depois_os_mais_antigos_e_nunca_salta_a_frente']),
    'MESH_IT': (['cargo', 'test', '-p', 'ph2d-navmesh', '--test', 'it'],
                ['fatias::a_procura_em_fatias_e_a_procura_inteira_ao_bit',
                 # (2.ª corrida) F3 SOBREVIVIA: a procura «inteira» de referência passa pelo mesmo código.
                 'dominancia::a_dominancia_corta_nos_e_nunca_encarece_um_caminho']),
    'IT': (['cargo', 'test', '-p', 'ph2d-physics-ecs', '--test', 'it'],
           ['nav_fatias::uma_procura_que_nao_cabe_para_a_meio_e_acaba_no_mesmo_caminho',
            'nav_fatias::um_scrub_a_meio_de_uma_procura_em_fatias_devolve_a_mesma_corrida',
            'nav_fatias::quem_segue_um_alvo_que_salta_espera_pela_vez',
            'nav_fatias::o_passo_em_paralelo_da_o_mesmo_com_uma_thread_e_com_oito',
            'nav_fatias::uma_malha_que_nunca_para_nao_deixa_a_procura_sem_acabar',
            # (2.ª corrida) S1/S2 SOBREVIVIAM: o gate do scrub de antes tinha a fixtura vazia (o orçamento
            # `1` deixava os guardas sem caminho no tique do replay) — este CONTÉM o fenómeno.
            'nav_mundo::um_scrub_numa_corrida_com_a_porta_a_alternar_devolve_a_mesma_corrida',
            # (2.ª corrida) V6, P2 e a ordem do paralelo não tinham régua.
            'nav_fatias::com_poucas_faixas_a_procura_mais_adiantada_acaba_primeiro',
            'nav_fatias::uma_procura_que_nao_comecou_nao_conta_como_recomeco',
            'nav_fatias::uma_procura_a_meio_numa_malha_que_mudou_recomeca',
            'nav_mundo::um_scrub_para_o_meio_da_fila_devolve_a_mesma_corrida',
            'nav_mundo::o_caminho_partido_passa_a_frente_na_fila',
            'nav_mundo::a_porta_que_abre_um_atalho_serve_os_agentes_um_por_tique',
            'nav_mundo::quem_nasce_com_a_fila_cheia_espera_pelo_tique_seguinte',
            'nav_nascer::os_que_nascem_juntos_procuram_pela_fila',
            'nav_nascer::na_lama_a_vez_de_quem_nasce_conta_o_trabalho']),
}
OBS_DO_GRUPO = {
    'FATIAS': ['MESH_IT', 'IT'],
    'VEZ': ['NAV', 'IT'],
    'PARALELO': ['IT'],
    'SCRUB': ['IT'],
}

# (nome, grupo, ficheiro, âncora, substituição)
M = [
    ('F1 a pausa quando o trabalho CHEGA ao tecto', 'FATIAS', FA,
     '            if ate != u64::MAX && self.stats.work() > ate {',
     '            if ate != u64::MAX && self.stats.work() >= ate {'),
    ('F2 a procura nunca pára', 'FATIAS', FA,
     '            if ate != u64::MAX && self.stats.work() > ate {',
     '            if false && ate != u64::MAX && self.stats.work() > ate {'),
    ('F3 a ponderada continua sem a dominância', 'FATIAS', FA,
     '                Fase::Ponderada { geral } => {\n                    search.dominancia = !search.sem_dominancia;',
     '                Fase::Ponderada { geral } => {\n                    search.dominancia = false;'),
    ('F4 o troço dos atalhos acaba e não relaxa', 'FATIAS', LK,
     '                if let (Ok(p), Some((u, _))) = (r, self.u) {\n                    self.relaxa(u, v, p);',
     '                if let (Ok(p), Some((u, _))) = (r, self.u) {\n                    let _ = (u, v, p);'),
    ('F5 o plano conta com as contagens de quem emprestou', 'FATIAS', PL,
     '        let emprestadas = std::mem::take(&mut search.stats);',
     '        let emprestadas = search.stats;'),
    ('F6 o scrub refaz até ao trabalho (e não trabalho − 1)', 'FATIAS', AG,
     '                    && let Some(r) = p.run(mesh, q, a.trabalho - 1)',
     '                    && let Some(r) = p.run(mesh, q, a.trabalho)'),
    ('V1 a condução sem tecto', 'VEZ', NV,
     '                    reserva.unwrap_or(0).saturating_add(livre)',
     '                    u64::MAX - 0 * reserva.unwrap_or(0).saturating_add(livre)'),
    ('V2 a série não desconta a maior fatia', 'VEZ', NV,
     '        let orcamento = self.nav.orcamento.saturating_sub(maior);',
     '        let orcamento = self.nav.orcamento.saturating_sub(0 * maior);'),
    ('V6 o recomeço sem trabalho também conta', 'VEZ', AG,
     '        if a.trabalho > 0 {\n            rt.recomecos = rt.recomecos.saturating_add(1);',
     '        if a.trabalho >= 0 {\n            rt.recomecos = rt.recomecos.saturating_add(1);'),
    ('V7 o recomeço não dobra a fatia', 'VEZ', AG,
     '    pode.saturating_mul(1u64.checked_shl(recomecos.min(63)).unwrap_or(u64::MAX))',
     '    pode.saturating_mul(1u64.checked_shl(0 * recomecos.min(63)).unwrap_or(u64::MAX))'),
    ('P1 o passo em paralelo não corre', 'PARALELO', FI,
     '        let corre = (*paralelas).min(fila.len());',
     '        let corre = 0 * (*paralelas).min(fila.len());'),
    ('P2 o paralelo avança procuras de outras entradas', 'PARALELO', FI,
     '                (entradas.get(&k) == Some(&a.entradas))',
     '                (entradas.get(&k).is_some() || a.entradas == 0)'),
    ('P4 o paralelo não dá a fatia à mais adiantada', 'PARALELO', FI,
     '                    .then_some(((a.trabalho, rt.broken, rt.owed), v.p.entity, k))',
     '                    .then_some(((0 * a.trabalho, rt.broken, rt.owed), v.p.entity, k))'),
    ('P3 a resposta do paralelo perde-se', 'PARALELO', FI,
     '            if let Some(r) = r {\n                prontos.insert(e, r);',
     '            if let Some(r) = r {\n                let _ = (e, r);'),
    ('S1 o scrub não devolve as assinaturas das malhas', 'SCRUB', TP,
     '            self.nav.sinais = m.nav_sinais.clone();\n',
     ''),
    ('S2 «mudou» é o que a actualização diz', 'SCRUB', ML,
     '            if antes != agora {',
     '            if base != agora {'),
    ('S3 o scrub guarda as procuras a meio do futuro', 'SCRUB', TP,
     '            self.nav.planos.clear();\n',
     ''),
    ('S4 as entradas não contam a malha', 'SCRUB', FI,
     '                [regiao.to_bits(), u64::from(raio), evita, s]',
     '                [regiao.to_bits(), u64::from(raio), evita, 0 * s]'),
    ('S5 a zona do que mudou depois de um scrub', 'SCRUB', FI,
     "                .filter(|_| v.chave.is_none_or(|k| !self.nav.sem_zona.contains(&k)))\n",
     ''),
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
