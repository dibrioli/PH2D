#!/usr/bin/env python3
"""Prova de mutação da W11 da navegação (AS PAREDES DO DESVIO POR MOSAICOS: a `ParedesPorBlocos` da
`ph2d-orca` em duas camadas L1/L2 e a concatenação, a busca de `Walls::near` por bloco, as faixas de
paredes da `ph2d-nav`, e a `ParedesDaMalha` da ponte que as liga).

Backup -> mutar (assert 1 casamento) -> correr SÓ os testes que OBSERVAM a mutação -> restaurar + touch.
O motor e os quatro controlos são os da W10 (mutacao_navegacao_w10_2026-10-03.py), copiados verbatim:
  (a) a corrida LIMPA de cada observador é VERDE; (b) cada teste observador nomeado aparece como corrido;
  (c) o pré-voo das âncoras casa cada uma exactamente 1x (`MUTA_SO_ANCORAS=1` pára aí);
  (d) checksums dos ficheiros tocados iguais antes/depois (senão sai com 2).

Selectores: `MUTA_SO=B1,B3` · `MUTA_G=ESCRITA` (ESCRITA | JUNTA | BORDA | L1 | BUSCA | NAV | PONTE).
Todo comando passa por `bash scripts/ph2d-run.sh`.
"""
import hashlib, os, re, subprocess, sys

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'
BO = 'crates/ph2d-orca/src/blocos.rs'
WA = 'crates/ph2d-orca/src/walls.rs'
NB = 'crates/ph2d-nav/src/blocos.rs'
NJ = 'crates/ph2d-nav/src/blocos_junta.rs'
DE = 'crates/ph2d-physics-ecs/src/bridge/nav_desvio.rs'
TI = 'crates/ph2d-navmesh/src/tiles.rs'

O_ORCA = [
    'as_paredes_por_blocos_sao_a_construcao_inteira_ao_bit',
    'so_se_refaz_o_bloco_que_mudou_e_os_vizinhos',
    'uma_parede_fora_do_bloco_e_recusada',
    'blocos_que_nao_formam_uma_grelha_sao_recusados',
    'sem_blocos_nao_ha_paredes',
    'a_grelha_das_paredes_da_o_mesmo_que_a_varredura_inteira',
]
O_NAV = ['as_recusas_sao_as_da_porta_inteira_com_o_indice_da_malha_montada']
O_TILES = ['a_montagem_por_blocos_e_a_montagem_inteira_ao_bit']
O_PONTE = ['as_paredes_da_ponte_por_mosaicos_sao_as_da_malha_inteira']
# a ponte em ponta: os módulos da navegação dos testes de integração (o desvio, o mundo que muda).
O_PONTE_IT = [
    'nav_mundo::um_personagem_parado_nao_vira_parede',
    'nav_mundo::so_os_agentes_da_malha_que_mudou_refazem_o_caminho',
    'nav_mundo::a_porta_que_desliza_e_para_fecha_o_caminho_e_abrir_devolve_o',
    'nav_mundo::o_caminho_partido_passa_a_frente_na_fila',
    'nav_mundo::uma_porta_a_andar_nao_recorta',
    'nav_mundo::uma_porta_a_rodar_no_sitio_nao_recorta',
    'nav_mundo::a_porta_que_abre_um_atalho_serve_os_agentes_um_por_tique',
    'nav_mundo::uma_malha_que_nao_para_de_mudar_serve_todos_a_vez',
    'nav_mundo::um_scrub_para_o_meio_da_fila_devolve_a_mesma_corrida',
    'nav_desvio::uma_porta_comprida_a_andar_desvia_se_pela_forma',
    'nav_desvio::o_perseguidor_chega_ao_heroi_que_nao_se_desvia',
    'nav_desvio::quem_nao_desvia_obriga_o_outro_a_desviar_por_inteiro',
    'nav_desvio::um_scrub_devolve_a_mesma_multidao',
    'nav_desvio::o_desvio_nao_empurra_contra_a_parede',
    'nav_desvio::um_corpo_que_nao_e_agente_tambem_se_evita',
    'nav_desvio::frente_a_frente_os_dois_cruzam_se_e_chegam',
    'nav_desvio::oito_pela_porta_passam_todos',
]

OBS = {
    'ORCA': (['cargo', 'test', '-p', 'ph2d-orca', '--lib'], O_ORCA),
    'NAV': (['cargo', 'test', '-p', 'ph2d-nav', '--lib'], O_NAV),
    'TILES': (['cargo', 'test', '-p', 'ph2d-navmesh', '--lib'], O_TILES),
    'PONTE': (['cargo', 'test', '-p', 'ph2d-physics-ecs', '--lib'], O_PONTE),
    'PONTE_IT': (['cargo', 'test', '-p', 'ph2d-physics-ecs', '--test', 'it'], O_PONTE_IT),
}

# (nome, grupo, ficheiro, âncora, substituição)
M = [
    # ── ESCRITA: poe / tira (quem se refaz, quem se recose) ──
    ('B1 poe: a comparação ignora o conteúdo', 'ESCRITA', BO,
     '                .all(|(x, y)| mesmo_ponto(x.0, y.0) && mesmo_ponto(x.1, y.1))',
     '                .all(|(x, _)| mesmo_ponto(x.0, x.0))'),
    ('B2 poe: refaz sempre (o igual não sai cedo)', 'ESCRITA', BO,
     '        {\n            return;\n        }\n        let dentro',
     '        {\n        }\n        let dentro'),
    ('B3 poe: só suja o próprio', 'ESCRITA', BO,
     '        self.contas.l1 += 1;\n        self.sujos.extend((0..9).map(|d| vizinho(k, d)));',
     '        self.contas.l1 += 1;\n        self.sujos.insert(k);'),
    ('B4 tira: não suja os vizinhos', 'ESCRITA', BO,
     '        if self.blocos.remove(&k).is_some() {\n            self.sujos.extend((0..9).map(|d| vizinho(k, d)));',
     '        if self.blocos.remove(&k).is_some() {\n            self.sujos.insert(k);'),
    ('B5 poe: a parede fora do rectângulo passa', 'ESCRITA', BO,
     '                dentro(de) && dentro(para),',
     '                dentro(de) || dentro(para) || k.0 == k.0,'),
    # ── JUNTA: a concatenação e a busca ──
    ('B6 um Fora com a base do próprio', 'JUNTA', BO,
     '                Ref::Fora(d, j) => base[&vizinho(k, d)] + j,',
     '                Ref::Fora(_, j) => b0 + j,'),
    ('B7 a L2 não escreve o dir', 'JUNTA', BO, '                w.dir[i] = p.dir;\n', ''),
    ('B8 a L2 não escreve o convex', 'JUNTA', BO, '                w.convex[i] = p.convex;\n', ''),
    ('B9 a L2 não escreve o next', 'JUNTA', BO, '                w.next[i] = global(p.next);\n', ''),
    ('B10 a L2 não escreve o prev', 'JUNTA', BO, '                w.prev[i] = global(p.prev);\n', ''),
    ('B11 a grelha do bloco com a base 0', 'JUNTA', BO,
     '                blocos[iy * nx + ix] = Some((base[k], Arc::clone(g)));',
     '                blocos[iy * nx + ix] = Some((0, Arc::clone(g)));'),
    ('B12 um bloco fora da grelha passa', 'JUNTA', BO,
     '                xs.get(ix + 1) == Some(&b.hi[0]) && ys.get(iy + 1) == Some(&b.hi[1]),',
     '                xs.get(ix).is_some() && ys.get(iy).is_some(),'),
    # ── BORDA: a L2 (a borda com os vizinhos) ──
    ('B13 os vizinhos pela ordem inversa', 'BORDA', BO,
     '            (x0..=x1)\n                .flat_map(|x| (y0..=y1).map(move |y| (x * 3 + y) as u8))',
     '            (x0..=x1)\n                .rev()\n                .flat_map(|x| (y0..=y1).rev().map(move |y| (x * 3 + y) as u8))'),
    ('B14 lados: só o próprio bloco', 'BORDA', BO,
     '(if v == lo { 0 } else { 1 }, if v == hi { 2 } else { 1 })',
     '(1 + 0 * usize::from(v == lo && lo == hi), 1)'),
    ('B15 sem seguinte na borda = a entrada 0', 'BORDA', BO,
     '                    acha(de, true).unwrap_or(eu)',
     '                    acha(de, true).unwrap_or(Ref::Local(0))'),
    ('B16 o anterior na borda procura quem COMEÇA', 'BORDA', BO,
     '                    acha(para, false).unwrap_or(eu)',
     '                    acha(para, true).unwrap_or(eu)'),
    ('B17 o seguinte de dentro de um pendente = ele próprio', 'BORDA', BO,
     '                    Ref::Local(b.l1.next[j as usize])',
     '                    Ref::Local(j)'),
    # ── L1: os pontos de dentro, a borda, a grelha ──
    ('B18 a ÚLTIMA entrada de cada ponto em vez da 1.ª', 'L1', BO,
     '            d.dedup_by_key(|x| x.0);',
     '            d.reverse();\n            d.dedup_by_key(|x| x.0);\n            d.reverse();'),
    ('B19 sem seguinte de dentro = a entrada 0', 'L1', BO,
     '                } else {\n                    k\n                };',
     '                } else {\n                    0\n                };'),
    ('B20 a borda só nos lados x', 'L1', BO,
     '    move |p| p[0] == lo[0] || p[0] == hi[0] || p[1] == lo[1] || p[1] == hi[1]',
     '    move |p| p[0] == lo[0] || p[0] == hi[0]'),
    ('B21 o ponto da borda onde começa E acaba perde o «acaba»', 'L1', BO,
     '                    (x.0, x.1, y.1)',
     '                    (x.0, x.1, u32::MAX)'),
    ('B22 o dir de dentro pelo `de`, não pelo seguinte', 'L1', BO,
     '                let (pn, pp) = (ends[nx as usize].1, ends[pv as usize].1);',
     '                let (pn, pp) = (de, ends[pv as usize].1);'),
    ('B23 a grelha do bloco com o segmento num ponto só', 'L1', BO,
     'Grade::new(ends.len(), |i| (ends[i].1, ends[i].0))',
     'Grade::new(ends.len(), |i| (ends[i].1, ends[i].1))'),
    # ── BUSCA: Walls::near por bloco ──
    ('W1 só o 1.º bloco de cada eixo', 'BUSCA', WA,
     '        for by in iy0..iy1 {\n            for bx in ix0..ix1 {',
     '        for by in iy0..iy1.min(iy0 + 1) {\n            for bx in ix0..ix1.min(ix0 + 1) {'),
    ('W2 o índice local sem a base', 'BUSCA', WA,
     '                            let i = base + i;',
     '                            let i = i + base * 0;'),
    # ── NAV: as faixas das paredes ──
    ('N1 a faixa perde a última parede do bloco', 'NAV', NJ,
     '            paredes: inicio..walls.len(),',
     '            paredes: inicio..walls.len().max(inicio + 1) - 1,'),
    ('N2 o rectângulo da faixa é o hi', 'NAV', NJ,
     '            lo: bl.peca.lo,',
     '            lo: bl.peca.hi,'),
    ('N3 uma malha recusada guarda as faixas velhas', 'NAV', NB,
     '        self.faixas.clear();\n', ''),
    # ── PONTE: a ParedesDaMalha ──
    # (2.ª corrida) a 1.ª P1 — `p.montadas = None` apagado em `nav_malha.rs` — SOBREVIVEU: nenhum teste
    # olhava as paredes depois de a malha mudar. Cura: a frescura passou para DENTRO da `ParedesDaMalha`
    # (a versão da malha), que o gate da ponte exerce.
    ('P1 as paredes montadas nunca se refazem depois da 1.ª', 'PONTE', DE,
     '        if self.versao != Some(tm.versao()) {',
     '        if self.versao.is_none() {'),
    ('P3 a malha muda e a versão não sobe', 'PONTE', TI,
     '            self.versao += 1;\n', ''),
    ('P2 os mosaicos que saíram ficam', 'PONTE', DE,
     '            .retem(|k| faixas.binary_search_by(|f| f.chave.cmp(&k)).is_ok());',
     '            .retem(|_| !faixas.is_empty() || faixas.is_empty());'),
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
