#!/usr/bin/env python3
"""Prova de mutação da W18 (plano 30 §27: a cena da lama `=5` e o 5.º motivo de replaneio — os custos ou
os atalhos mudaram desde que o caminho foi planeado).

O motor e os quatro controlos são os da W15/W16, copiados verbatim (cada grupo corre só os observadores
dele, `OBS_DO_GRUPO`; o controlo limpo corre todos).
Selectores: `MUTA_SO=L1,C2` · `MUTA_G=LAMA` (LAMA | CUSTO | CANTO).
"""
import hashlib, os, re, subprocess, sys

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'
LAMA = 'crates/ph2d-app-components/src/nav_smoke_lama.rs'
NS = 'crates/ph2d-app-components/src/nav_smoke.rs'
AG = 'crates/ph2d-nav/src/agent.rs'
FI = 'crates/ph2d-physics-ecs/src/bridge/nav_fila.rs'
NAV = 'crates/ph2d-physics-ecs/src/bridge/nav.rs'

OBS = {
    'COMP': (['cargo', 'test', '-p', 'ph2d-app-components', '--lib'],
             ['nav_smoke_lama::tests::a_leve_atravessa_se_a_pesada_contorna_se',
              'nav_smoke_lama::tests::com_o_cost_em_2_os_da_direita_cortam_pela_lama',
              'nav_smoke_lama::tests::a_cena_tem_as_pecas_que_o_roteiro_nomeia',
              'nav_smoke::tests::o_cenas_conta_os_niveis_do_roteador',
              'nav_smoke_lama::tests::nenhum_corredor_volta_atras_para_um_canto',
              'smoke_desenho_e_corpo_tests::nenhuma_cena_de_smoke_desenha_fora_do_corpo']),
    'IT': (['cargo', 'test', '-p', 'ph2d-physics-ecs', '--test', 'it'],
           ['nav_custo::mexer_no_custo_refaz_o_caminho_de_quem_anda']),
}
OBS_DO_GRUPO = {
    'LAMA': ['COMP'],
    'CUSTO': ['IT', 'COMP'],
    'CANTO': ['COMP'],
}

# (nome, grupo, ficheiro, âncora, substituição)
M = [
    ('L1 a pesada custa o mesmo que a leve', 'LAMA', LAMA,
     'pub const CUSTO_PESADA: f32 = 10.0;',
     'pub const CUSTO_PESADA: f32 = 2.0;'),
    ('L2 a faixa sem a área de custo', 'LAMA', LAMA,
     '            NavCostArea {\n                cost: custo,\n                forbidden: false,\n            },\n',
     ''),
    ('L3 o roteador sem a cena 5', 'LAMA', NS,
     '    if nivel == 5 {',
     '    if nivel == 55 {'),
    ('L4 a shell abre a secção do agente', 'LAMA', NS,
     '        if self.lama.is_some() {',
     '        if false {'),
    ('L5 o CENAS fica em 4', 'LAMA', NS,
     'pub const CENAS: u32 = 5;',
     'pub const CENAS: u32 = 4;'),
    ('L6 a lama desenhada maior que o corpo', 'LAMA', LAMA,
     '            Sprite::atlas(WHITE_TILE_KEY, [h[0] * 2.0, h[1] * 2.0], cor),',
     '            Sprite::atlas(WHITE_TILE_KEY, [h[0] * 2.0, h[1] * 2.6], cor),'),
    ('C1 sem o 5.º motivo', 'CUSTO', AG,
     '        || vez.refazer\n        || custos_mudaram;',
     '        || vez.refazer;'),
    ('C2 o caminho instalado não guarda os custos', 'CUSTO', AG,
     '        rt.custos_do_caminho = vez.custos;\n',
     ''),
    ('C3 a assinatura devolvida é zero', 'CUSTO', FI,
     '        (por_malha, h.0)',
     '        (por_malha, 0)'),
    ('C4 a ponte não passa a assinatura', 'CUSTO', NAV,
     '                custos: assinatura_dos_custos,',
     '                custos: 0,'),
    ('K1 o canto só no passo do executor (o R2 do report do dono)', 'CANTO', AG,
     'pub const ALCANCE_DO_CANTO: f64 = 0.1;',
     'pub const ALCANCE_DO_CANTO: f64 = 0.0;'),
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
