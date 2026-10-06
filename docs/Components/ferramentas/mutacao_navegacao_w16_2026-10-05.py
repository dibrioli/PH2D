#!/usr/bin/env python3
"""Prova de mutação da W16 (plano 30 §25: o desenho cabe no corpo, o caminho contorna um corpo que anda,
o alvo à vista sem procura, e a régua da guarda `sem_zona`).

O motor e os quatro controlos são os da W15, copiados verbatim (cada grupo corre só os observadores
dele, `OBS_DO_GRUPO`; o controlo limpo corre todos).
Selectores: `MUTA_SO=A1,B2` · `MUTA_G=DESENHO` (DESENHO | CONTORNO | VISTA | SCRUB).
"""
import hashlib, os, re, subprocess, sys

ROOT = '/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components'
AT = 'crates/ph2d-render/src/atlas/mod.rs'
LAVA = 'crates/ph2d-app-components/src/nav_smoke_lava.rs'
SD = 'crates/ph2d-app-components/src/smoke_desenho.rs'
LIB = 'crates/ph2d-app-components/src/lib.rs'
DV = 'crates/ph2d-physics-ecs/src/bridge/nav_desvio.rs'
FI = 'crates/ph2d-physics-ecs/src/bridge/nav_fila.rs'
AG = 'crates/ph2d-nav/src/agent.rs'

OBS = {
    'RENDER': (['cargo', 'test', '-p', 'ph2d-render', '--lib'],
               ['atlas::tests::the_disc_tile_covers_the_inscribed_circle_and_nothing_outside_it',
                'atlas::tests::white_tile_is_opaque_white_and_never_collides_with_import_keys']),
    'COMP': (['cargo', 'test', '-p', 'ph2d-app-components', '--lib'],
             ['smoke_desenho_e_corpo_tests::nenhuma_cena_de_smoke_desenha_fora_do_corpo',
              'smoke_desenho_e_corpo_tests::o_censo_ve_um_quadrado_sobre_uma_bola_e_aceita_o_disco',
              'smoke_desenho_e_corpo_tests::todo_roteador_que_a_shell_le_esta_na_familia']),
    'ECS_LIB': (['cargo', 'test', '-p', 'ph2d-physics-ecs', '--lib'],
                ['bridge::nav::tests::o_caminho_contorna_um_corpo_pela_tangente_do_lado_mais_curto']),
    'NAV': (['cargo', 'test', '-p', 'ph2d-nav', '--lib'],
            ['agent::tests::o_alvo_a_vista_e_so_a_recta_que_nenhum_caminho_bate']),
    'IT': (['cargo', 'test', '-p', 'ph2d-physics-ecs', '--test', 'it'],
           ['nav_desvio_largo::um_corpo_largo_que_vem_de_frente_e_contornado',
            'nav_desvio_largo::uma_capsula_que_anda_e_um_torniquete_que_roda_sao_contornados_pela_forma',
            'nav_fatias::quem_persegue_um_alvo_a_vista_nao_espera_a_vez',
            'nav_mundo::um_scrub_com_a_zona_de_outra_porta_no_fim_devolve_a_mesma_corrida',
            'nav_mundo::um_scrub_numa_corrida_com_a_porta_a_alternar_devolve_a_mesma_corrida']),
}
OBS_DO_GRUPO = {
    'DESENHO': ['RENDER', 'COMP'],
    'CONTORNO': ['ECS_LIB', 'IT'],
    'VISTA': ['NAV', 'IT'],
    'SCRUB': ['IT'],
}

# (nome, grupo, ficheiro, âncora, substituição)
M = [
    ('A1 o disco desenha o quadrado', 'DESENHO', AT,
     '            let a = (r - (dx * dx + dy * dy).sqrt() + 0.5).clamp(0.0, 1.0);',
     '            let a = 1.0_f32;'),
    ('A2 o herói da cena da lava volta ao quadrado', 'DESENHO', LAVA,
     '            crate::smoke_desenho::disco(RAIO_HEROI, HEROI_RGBA),',
     '            ph2d_render::Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [RAIO_HEROI * 2.0, RAIO_HEROI * 2.0], HEROI_RGBA),'),
    ('A3 o «Rumo» sai do disco', 'DESENHO', SD,
     '            Transform::from_translation(Vec2::new(0.35 * raio, 0.0)),',
     '            Transform::from_translation(Vec2::new(0.6 * raio, 0.0)),'),
    ('A4 o roteador da arma fora da família', 'DESENHO', LIB,
     '        r("PH2D_WEAPON_SMOKE", weapon_smoke::CENAS),\n',
     ''),
    ('A5 as peças reservadas sem o disco', 'DESENHO', AT,
     '        self.insert(gpu, DISC_TILE_KEY, DISC_TILE_PX, DISC_TILE_PX, &px)?;\n',
     ''),
    ('B1 o empate vai pela esquerda', 'CONTORNO', DV,
     '    let d = if perto(dir_) >= perto(esq) { dir_ } else { esq };',
     '    let d = if perto(dir_) > perto(esq) { dir_ } else { esq };'),
    ('B2 a tangente sem a folga do desvio', 'CONTORNO', DV,
     '                    .find_map(|(pts, folga)| contorna(p.pos, p.dir, ate, p.raio + folga, pts))',
     '                    .find_map(|(pts, _folga)| contorna(p.pos, p.dir, ate, p.raio, pts))'),
    ('B3 o troço até ao infinito, não ao canto', 'CONTORNO', DV,
     '    let fim = [pos[0] + dir[0] * ate, pos[1] + dir[1] * ate];',
     '    let fim = [pos[0] + dir[0] * 1e3, pos[1] + dir[1] * 1e3];'),
    ('B4 o primeiro vértice e não o extremo', 'CONTORNO', DV,
     '            .reduce(|a, b| if cruz(a, b) * sinal > 0.0 { b } else { a })',
     '            .reduce(|a, _| a)'),
    ('B5 aponta ao vértice, sem engordar', 'CONTORNO', DV,
     '        let s = (r / l).min(1.0) * sinal;',
     '        let s = 0.0 * sinal;'),
    ('B6 o troço que atravessa o polígono não conta', 'CONTORNO', DV,
     '            return 0.0;\n        }\n        menor = menor',
     '        }\n        menor = menor'),
    ('B7 o contorno desligado por omissão', 'CONTORNO', FI,
     '            contorno: true,',
     '            contorno: false,'),
    ('V1 os atalhos não contam', 'VISTA', AG,
     '    q.links.is_empty()\n        && q.costs',
     '    q.costs'),
    ('V2 uma área mais barata que 1 não conta', 'VISTA', AG,
     '        && q.costs.iter().all(|&c| c >= 1.0)\n',
     ''),
    ('V3 a lama na recta não conta', 'VISTA', AG,
     '.is_some_and(|c| c <= dist(a, b) + EPS)',
     '.is_some()'),
    ('V4 a recta não se instala', 'VISTA', AG,
     '        if recta {\n            (pronto, rt.last_work)',
     '        if false {\n            (pronto, rt.last_work)'),
    ('V5 o alvo à vista desligado por omissão', 'VISTA', FI,
     '            a_vista: true,',
     '            a_vista: false,'),
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
