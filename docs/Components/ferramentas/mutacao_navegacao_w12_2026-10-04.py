#!/usr/bin/env python3
"""Prova de mutação da W12 da navegação (O MOSAICO REFEITO MAIS DEPRESSA: a fusão em convexos sem
`BTreeMap`, a numeração e as restrições da triangulação por ordenação, as junções em T numa grelha
contígua, e as assinaturas por palavra).

O motor e os quatro controlos são os da W10 (mutacao_navegacao_w10_2026-10-03.py), copiados verbatim.
Selectores: `MUTA_SO=F1,T1` · `MUTA_G=FUSAO` (FUSAO | TRIANG | JUNCAO | ASSINATURA).
"""
import hashlib, os, re, subprocess, sys

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'
TR = 'crates/ph2d-navmesh/src/triangulate.rs'
TI = 'crates/ph2d-navmesh/src/tiles.rs'

O_LIB = [
    'a_triangulacao_e_a_fusao_de_agora_sao_as_de_antes_ao_bit',
    'a_assinatura_distingue_os_sinais_trocados',
    'a_montagem_por_blocos_e_a_montagem_inteira_ao_bit',
]
O_IT = [
    'sem_areas_a_construcao_e_a_de_sempre_ao_bit',
    'cada_poligono_sabe_a_area_onde_esta_com_a_primeira_a_mandar',
    'a_fronteira_de_uma_area_e_passagem_e_nunca_parede',
    'cem_cenas_com_areas_nenhuma_recusa',
    'por_mosaicos_e_a_mesma_malha_que_inteira',
    'incremental_e_a_frio_dao_o_mesmo',
    'a_porta_fecha_e_abre',
    'a_juncao_em_t_da_costura_e_reparada',
    'com_areas_por_mosaicos_e_a_mesma_malha_que_inteira',
    'com_areas_incremental_e_a_frio_dao_o_mesmo',
    'o_furo_na_quina_de_quatro_mosaicos_nao_vaza',
    'em_regioes_negativas_com_paredes_no_bordo_por_mosaicos_e_a_inteira',
]

OBS = {
    'LIB': (['cargo', 'test', '-p', 'ph2d-navmesh', '--lib'], O_LIB),
    'IT': (['cargo', 'test', '-p', 'ph2d-navmesh', '--test', 'it'], O_IT),
}

# (nome, grupo, ficheiro, âncora, substituição)
M = [
    # ── FUSAO ──
    # ⛔ EQUIVALENTE (1.ª corrida, saiu): F1 «o par de uma semi-aresta sem conferir que é o oposto» — numa
    # aresta da BORDA o par errado vai a `try_merge`, que exige achar `y → x` no outro anel: nenhum anel
    # tem a aresta invertida de uma borda, logo a tentativa é sempre `None`; numa interior o par é o certo.
    ('F2 a convexidade com os vizinhos do interior trocados', 'FUSAO', TR,
     '        (interior(1), interior(nb - 2))',
     '        (interior(nb - 2), interior(1))'),
    ('F3 o anterior de x é o próprio x', 'FUSAO', TR,
     '    let prev_x = pts[m_k(na - 2) as usize];',
     '    let prev_x = pts[m_k(na - 1) as usize];'),
    ('F4 o anel fundido não morre', 'FUSAO', TR,
     '            vivo[b] = false;\n', ''),
    ('F5 o triângulo que não fundiu sai vazio', 'FUSAO', TR,
     '(if r.is_empty() { tris[i].to_vec() } else { r }, l)',
     '(r, l)'),
    ('F6 um anel fundido lido como o triângulo', 'FUSAO', TR,
     '            if rings[i].is_empty() {\n                &tris[i]',
     '            if !vivo[i] || rings[i].len() < usize::MAX {\n                &tris[i]'),
    # ── TRIANG: a numeração e as restrições ──
    ('T1 os vértices pela ordem do ponto, não da 1.ª aparição', 'TRIANG', TR,
     '    grupos.sort_unstable();\n', ''),
    # ⛔ EQUIVALENTE MEDIDA (1.ª corrida, saiu): T2 «as restrições pela ordem da chave» — o `spade` 2.15 dá a
    # mesma triangulação qualquer que seja a ordem das restrições, em todas as fixturas. A ordenação FICA:
    # é a ordem de antes, e não depender de um pormenor interno da biblioteca numa subida dela.
    ('T3 os donos de uma aresta: só o 1.º', 'TRIANG', TR,
     '        let f = arestas.partition_point(|w| w.0 <= c);',
     '        let f = arestas.partition_point(|w| w.0 <= c).min(i + 1);'),
    # ── JUNCAO: as junções em T ──
    ('J1 o balde da borda de cima fica de fora', 'JUNCAO', TR,
     '        if bx < 0 || by < 0 || bx >= nbx || by >= nby {',
     '        if bx < 0 || by < 0 || bx >= nbx - 1 || by >= nby - 1 {'),
    # ── ASSINATURA ──
    ('S1 a assinatura por palavra sem mistura', 'ASSINATURA', TI,
     '        self.0 ^= z ^ (z >> 31);',
     '        self.0 ^= v ^ (z & 0);'),
    ('S2 a caixa de um círculo só com o canto de baixo', 'ASSINATURA', TI,
     '            &circ[..2]',
     '            &circ[..1]'),
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
        for conj in OBS:
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
