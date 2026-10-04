#!/usr/bin/env python3
"""Prova de mutação da W13 da navegação (O COMPORTAMENTO QUE FALTAVA: o empurrão de um golpe na
velocidade que o desvio vê, as peças de um corpo composto no desvio, e a 1.ª procura de quem não tem
caminho dentro do orçamento da fila, pela ordem das entidades).

O motor e os quatro controlos são os da W10 (mutacao_navegacao_w10_2026-10-03.py), copiados verbatim.
Selectores: `MUTA_SO=K1,N2` · `MUTA_G=DESVIO` (DESVIO | FILA).
"""
import hashlib, os, re, subprocess, sys

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'
DE = 'crates/ph2d-physics-ecs/src/bridge/nav_desvio.rs'
NV = 'crates/ph2d-physics-ecs/src/bridge/nav.rs'
FI = 'crates/ph2d-physics-ecs/src/bridge/nav_fila.rs'

O_IT = [
    'nav_desvio_corpos::um_corpo_empurrado_por_um_golpe_e_visto_a_andar',
    'nav_desvio_corpos::um_corpo_composto_desvia_se_pela_forma_inteira',
    'nav_nascer::os_que_nascem_juntos_procuram_pela_fila',
    'nav_nascer::quem_espera_pela_vez_fica_parado_e_depois_chega',
    'nav_nascer::um_scrub_a_meio_dos_nascimentos_devolve_a_mesma_corrida',
    'nav_mundo::a_porta_que_abre_um_atalho_serve_os_agentes_um_por_tique',
    'nav_mundo::o_caminho_partido_passa_a_frente_na_fila',
    'nav_mundo::um_scrub_para_o_meio_da_fila_devolve_a_mesma_corrida',
    'nav_desvio::um_scrub_devolve_a_mesma_multidao',
    # (2.ª corrida) N2 e N5 SOBREVIVIAM: nenhuma fixtura juntava a fila CHEIA e quem nasce no mesmo tique.
    'nav_mundo::quem_nasce_com_a_fila_cheia_procura_um_e_o_outro_espera',
]

OBS = {
    'IT': (['cargo', 'test', '-p', 'ph2d-physics-ecs', '--test', 'it'], O_IT),
}

# (nome, grupo, ficheiro, âncora, substituição)
M = [
    # (2.ª corrida) a 1.ª K1 tirava só a componente x — e o golpe da fixtura é vertical: SOBREVIVIA. O
    # defeito real é o empurrão fora da velocidade, nas duas.
    ('K1 a velocidade vista sem o empurrão', 'DESVIO', DE,
     '                st.velocity[0] + st.knockback[0],\n                st.velocity[1] + st.knockback[1],',
     '                st.velocity[0] + 0.0 * st.knockback[0],\n                st.velocity[1] + 0.0 * st.knockback[1],'),
    ('C1 as peças fora do desvio', 'DESVIO', DE,
     '                for p in minhas {',
     '                for p in minhas.iter().take(0) {'),
    ('C2 o disco do alvo sem as peças', 'DESVIO', DE,
     '                        r.max(d + f64::from(raio_que_envolve(&p.rest)))',
     '                        r.max(0.0 * (d + f64::from(raio_que_envolve(&p.rest))))'),
    ('N1 quem não tem caminho nunca espera', 'FILA', NV,
     '            if sem && sem_caminho > 0 && gasto >= orcamento && malha.is_some() {',
     '            if sem && sem_caminho > 0 && gasto >= orcamento && malha.is_some() && orcamento == 0 {'),
    ('N2 nem o 1.º sem caminho procura sem folga', 'FILA', NV,
     '            if sem && sem_caminho > 0 && gasto >= orcamento && malha.is_some() {',
     '            if sem && gasto >= orcamento && malha.is_some() {'),
    ('N3 a condução pela ordem das tabelas', 'FILA', NV,
     '        vez.sort_by_key(|v| v.p.entity);\n', ''),
    ('N4 as procuras de fora da fila não contam', 'FILA', NV,
     '                gasto = gasto.saturating_add(search.stats.expanded - nos);',
     '                gasto = gasto.saturating_add(0 * (search.stats.expanded - nos));'),
    ('N5 a fila não devolve o que prometeu', 'FILA', FI,
     '                gasto = gasto.saturating_add(o.nodes);',
     '                gasto = gasto.saturating_add(0 * o.nodes);'),
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
