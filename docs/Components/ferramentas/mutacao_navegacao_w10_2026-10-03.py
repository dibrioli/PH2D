#!/usr/bin/env python3
"""Prova de mutação da W10 da navegação (A MALHA POR BLOCOS: a montagem em três camadas L1/L2/L3 da
`MalhaPorBlocos`, a junção linear, a grelha de localização por bloco, e o `Tiles` que a alimenta).

Backup -> mutar (assert 1 casamento) -> correr SÓ os testes que OBSERVAM a mutação -> restaurar + touch.

Os quatro controlos (o molde da W7/W8/W9):
  (a) a corrida LIMPA de cada observador é VERDE (e com população > 0);
  (b) cada filtro casa >= 1 teste, e CADA teste observador nomeado aparece na saída como corrido;
  (c) o pré-voo das âncoras casa cada uma exactamente 1x (`MUTA_SO_ANCORAS=1` pára aí);
  (d) o controlo da árvore: checksums dos ficheiros tocados iguais antes/depois (senão sai com 2).
Uma mutação que não compila é defeito do ARNÊS, nunca sangria.

A população de testes é a de quem OBSERVA: o código mutado vive na `ph2d-nav` (e uma mutação no
`tiles.rs`), mas quem o exerce em ponta são os três conjuntos de OBS (nav --lib, navmesh --lib,
navmesh --test it). Cada mutação corre os três (os que ela alcança) e o placar diz QUAIS ficaram
vermelhos; só «nenhum vermelho em nenhum» é SOBREVIVEU.

Selectores: `MUTA_SO=M1,M3` corre só estas; `MUTA_G=ESCRITA` corre só este grupo de mutações
(ESCRITA | COSTURA | JUNTA | GRELHA). Todo comando passa por `bash scripts/ph2d-run.sh` (a fatia de 30 min:
corra grupo a grupo).
"""
import hashlib, os, re, subprocess, sys

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'
BL = 'crates/ph2d-nav/src/blocos.rs'
JU = 'crates/ph2d-nav/src/blocos_junta.rs'
GR = 'crates/ph2d-nav/src/grelha.rs'
TILES = 'crates/ph2d-navmesh/src/tiles.rs'

# os observadores, por conjunto de testes
O_NAV = [
    'as_recusas_sao_as_da_porta_inteira_com_o_indice_da_malha_montada',
    'um_indice_fora_e_as_areas_que_nao_batem_sao_recusados',
    'uma_peca_fora_do_rectangulo_e_um_erro_de_quem_chama',
    'um_ponto_de_costura_repetido_na_peca_e_um_vertice_so',
    # (2.ª corrida) as fixturas das quatro sobreviventes da 1.ª: M3 · M5 · M9 · M15.
    'tirar_um_bloco_desliga_o_vizinho',
    'um_vizinho_com_os_mesmos_pontos_de_lado_que_deixa_de_ligar_devolve_a_parede',
    'dois_pontos_cosidos_numa_aresta_que_desce_vao_pela_ordem_dela',
    'um_ponto_a_tolerancia_da_fronteira_ve_o_bloco_do_outro_lado',
    'a_localizacao_ve_os_dois_lados_de_uma_aresta_partilhada',
]
O_TILES = ['a_montagem_por_blocos_e_a_montagem_inteira_ao_bit']
O_IT = [
    'incremental_e_a_frio_dao_o_mesmo',
    'com_areas_incremental_e_a_frio_dao_o_mesmo',
    'a_juncao_em_t_da_costura_e_reparada',
    'por_mosaicos_e_a_mesma_malha_que_inteira',
]

# conjunto de observadores -> (comando base, os testes)
OBS = {
    'NAV': (['cargo', 'test', '-p', 'ph2d-nav', '--lib'], O_NAV),
    'TILES': (['cargo', 'test', '-p', 'ph2d-navmesh', '--lib'], O_TILES),
    'IT': (['cargo', 'test', '-p', 'ph2d-navmesh', '--test', 'it'], O_IT),
}

# (nome, grupo, ficheiro, âncora, substituição)
M = [
    # ── ESCRITA: poe / tira / monta / liga (quem se recose, quem se religa) ──
    ('M1 poe: um lado mudado não suja os vizinhos', 'ESCRITA', BL,
     '                            self.sujos.insert((k.0 + ex, k.1 + ey));',
     '                            let _ = (ex, ey);'),
    ('M2 mesmo_lado é sempre verdadeiro', 'ESCRITA', BL,
     '        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.0 == y.0)',
     '        let _ = (a, b);\n        true'),
    ('M3 tira: só suja o próprio k', 'ESCRITA', BL,
     '        if self.blocos.remove(&k).is_some() {\n            self.sujos.extend((0..9).map(|d| vizinho(k, d)));',
     '        if self.blocos.remove(&k).is_some() {\n            self.sujos.insert(k);'),
    ('M4 monta: L3 sem o lado de frente do vizinho', 'ESCRITA', BL,
     '                    l3.insert((v, frente));',
     '                    let _ = frente;'),
    ('M5 liga: um Fora anterior não volta a Parede', 'ESCRITA', BL,
     '                b.l2.lig[pos] = Lig::Parede;',
     '                let _ = pos;'),
    ('M22 tiles: os mosaicos que saíram da região não são tirados', 'ESCRITA', TILES,
     '        for &k in self.mosaicos.keys() {\n            self.blocos.tira(k);\n        }\n',
     ''),
    # ── COSTURA: o dono dos pontos, os anéis cosidos, a validação ──
    ('M6 alias: sem o candidato da quina (esq && baixo)', 'COSTURA', BL,
     '                    (esq && baixo, -1, -1, DIR, q[1]),\n', ''),
    ('M7 alias: sem o break (ganha o último candidato)', 'COSTURA', BL,
     '                        r = Vref::lado(d, lado, j);\n                        break;',
     '                        r = Vref::lado(d, lado, j);'),
    ('M8 costura: ignora os pontos de lado do vizinho de frente', 'COSTURA', BL,
     '                let deles: &[(f64, u32)] = n_frente.map_or(&[], |n| &n.l1.lados[frente]);',
     '                let deles: &[(f64, u32)] = &[];'),
    ('M9 costura: não inverte os pontos quando ca > cb', 'COSTURA', BL,
     '                if ca > cb {\n                    pontos.reverse();',
     '                if false && ca > cb {\n                    pontos.reverse();'),
    ('M10 costura: a fatia inclui a ponta de baixo', 'COSTURA', BL,
     '                    let a = v.partition_point(|e| e.0 <= lo);',
     '                    let a = v.partition_point(|e| e.0 < lo);'),
    ('M17 canon devolve i (repetidos ignorados)', 'COSTURA', BL,
     '            .map_or(i, |k| self.repetidos[k].1)',
     '            .map_or(i, |k| {\n                let _ = k;\n                i\n            })'),
    ('M18 valida: sem a convexidade', 'COSTURA', BL,
     '            if side_dist(a, b, c) < -EPS {',
     '            if false && side_dist(a, b, c) < -EPS {'),
    ('M19 L1::de: sem o assert do rectângulo', 'COSTURA', BL,
     '                q[0] >= p.lo[0] && q[0] <= p.hi[0] && q[1] >= p.lo[1] && q[1] <= p.hi[1],',
     '                true,'),
    ('M20 liga_dentro: BAIXO/CIMA nunca vão às bordas', 'COSTURA', BL,
     '                Some(DIR)\n            } else if pu[1] == pv[1] && pu[1] == p.lo[1] {\n'
     '                Some(BAIXO)\n            } else if pu[1] == pv[1] && pu[1] == p.hi[1] {\n'
     '                Some(CIMA)\n            } else {',
     '                Some(DIR)\n            } else {'),
    ('M21 slot de fora: Vref::lado(d, lado, j) (lista do lado errado)', 'COSTURA', BL,
     'l.slots.push(Vref::lado(d, frente, j));',
     'l.slots.push(Vref::lado(d, lado, j));'),
    # ── JUNTA: a passagem linear final ──
    ('M11 junta: sem a união de componentes através de Fora', 'JUNTA', JU,
     '                        pai[ca.max(cb) as usize] = ca.min(cb);',
     '                        let _ = (ca, cb);'),
    # M12 («a parede só marca o canto u») SAIU: é um mutante EQUIVALENTE — numa malha sem sobreposição a
    # ponta `v` de uma parede é a ponta `u` da parede seguinte do mesmo contorno, que a marca (1.ª corrida:
    # sobreviveu; nenhum gate o pode ver porque nenhuma malha válida o distingue).
    ('M13 junta: Fora traduz com pbase[b]', 'JUNTA', JU,
     '                        (Some(pbase[o] + q), e)',
     '                        (Some(pbase[b] + q), e)'),
    ('M14 junta: a numeração ignora os alias', 'JUNTA', JU,
     '            mapa[mbase[b] + i] = if o == b && j == i as u32 {',
     '            mapa[mbase[b] + i] = if true {'),
    # ── GRELHA: a localização por bloco ──
    ('M15 faixa: f < x em vez de f <= x', 'GRELHA', GR,
     '    xs[1..n].partition_point(|&f| f <= x)',
     '    xs[1..n].partition_point(|&f| f < x)'),
    ('M16 candidatos: só a 1.ª sonda EPS', 'GRELHA', GR,
     '        for dx in [-EPS, EPS] {\n            for dy in [-EPS, EPS] {',
     '        for dx in [-EPS] {\n            for dy in [-EPS] {'),
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
