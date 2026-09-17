#!/usr/bin/env python3
"""Confere as fixturas de `silhueta/` — a familia cuja malha e' GERADA POR FORMULA e que publica um
SUBCONJUNTO DECLARADO dos vertices.

Ela responde a tres perguntas que um `grep` nao responde:

1. **a formula do cabecalho re-deriva as posicoes de repouso publicadas?** Sem isto, o cabecalho
   podia descrever uma malha e o ficheiro trazer outra, e nenhum gate notaria — a prosa nao e'
   executavel e as chaves `malha_*` existem exactamente para o ser.
2. **os indices publicados sao os que o cabecalho declara?** Um subconjunto e' uma promessa sobre
   QUE vertices estao no ficheiro; se a promessa e a lista divergirem, quem escrever um gate sobre
   «a fila do traco» le uma fila com buracos e chama-lhe medicao.
3. **cada ficheiro tem o cabecalho inteiro** (as chaves que enquadram o traco) e os blocos `r`/`n`
   com o mesmo tamanho.

Uso:  python3 confere_a_formula.py            (corre sobre esta pasta)
Sai `0` em verde e `1` com a lista do que falhou.
"""
import glob, gzip, math, os, sys

AQUI = os.path.dirname(os.path.abspath(__file__))
TOL = 2e-6          # os valores sao escritos com %.9g a partir de float32 do alvo
OBRIGATORIAS = ['fixture', 'familia', 'o_que_ela_fixa', 'oraculo', 'pincel_de_origem', 'unidades',
                'vista', 'superficie', 'malha_gerada_por_formula', 'malha_tipo',
                'subconjunto_publicado', 'vista_px_por_unidade', 'pixel_do_pen_down_no_mundo',
                'um_pixel_para_a_direita', 'raio_efectivo_objecto', 'caminho_do_traco', 'cursor',
                'normais_de_vertice', 'pressao', 'modificador', 'pincel', 'forca', 'curva',
                'espacamento_pct_do_diametro', 'espacamento_medido_em', 'espacamento_adaptativo',
                'espacamento_segue_pressao', 'atenuacao_por_espacamento', 'acumula', 'direccao',
                'automascara', 'textura', 'mascara', 'simetria', 'vertices', 'vertices_publicados']


def ler(path):
    cab, rest, publicados = {}, {}, []
    with gzip.open(path, 'rt') as f:
        for ln in f:
            if ln.startswith('#'):
                k, _, v = ln[1:].strip().partition(':')
                cab[k.strip()] = v.strip()
            elif ln.startswith('r '):
                p = ln.split()
                i = int(p[1]) if len(p) == 5 else len(rest)
                rest[i] = tuple(float(x) for x in p[-3:])
                publicados.append(i)
    return cab, rest, publicados


def repouso_por_formula(cab, i):
    t = cab['malha_tipo']
    r = float(cab.get('malha_raio', 1.0))
    if t == 'cupula':
        nu, nv = int(cab['malha_celulas_u']), int(cab['malha_celulas_v'])
        umax = math.radians(float(cab['malha_meio_angulo_graus']))
        L = float(cab['malha_comprimento'])
        j, k = divmod(i, nu + 1)
        u = -umax + 2 * umax * k / nu
        return (r * math.sin(u), -L / 2 + L * j / nv, r * math.cos(u))
    if t == 'esfera':
        nr, ns = int(cab['malha_aneis']), int(cab['malha_segmentos'])
        if i == 0:
            return (0.0, 0.0, r)
        if i == 1:
            return (0.0, 0.0, -r)
        a, s = divmod(i - 2, ns)
        phi = math.pi * (a + 1) / nr
        th = 2 * math.pi * s / ns
        return (r * math.sin(phi) * math.cos(th), r * math.sin(phi) * math.sin(th), r * math.cos(phi))
    n, lado = int(cab['malha_celulas']), float(cab['malha_lado'])
    j, k = divmod(i, n + 1)
    x = -lado / 2 + lado * k / n
    y = -lado / 2 + lado * j / n
    if t == 'rampa_x':
        return (x, y, float(cab['malha_inclinacao']) * x)
    if t == 'plano_caixa_dupla':
        return (x, y, -1.0 if (x < -0.99 and y < -0.99) else (1.0 if (x > 0.99 and y > 0.99) else 0.0))
    raise ValueError(t)


def indices_declarados(cab):
    t = cab['malha_tipo']
    txt = cab['subconjunto_publicado']
    if t == 'cupula':
        nu, nv = int(cab['malha_celulas_u']), int(cab['malha_celulas_v'])
        j0 = int(txt.split('j = ')[1].split(',')[0])
        cols = [int(x) for x in txt.split('estacoes i = ')[1].split(',')]
        idx = {j0 * (nu + 1) + i for i in range(nu + 1)}
        idx |= {j * (nu + 1) + i for i in cols for j in range(nv + 1)}
        return idx
    if t == 'esfera':
        nr, ns = int(cab['malha_aneis']), int(cab['malha_segmentos'])
        r = float(cab.get('malha_raio', 1.0))
        aneis = [int(x) for x in txt.split('aneis a = ')[1].split(',')]
        idx = {2 + (a - 1) * ns for a in range(1, nr)} | {0}
        for a in aneis:
            phi = math.pi * a / nr
            k = min(ns // 2 - 1, max(3, int(0.35 / (2 * math.pi * r * math.sin(phi) / ns))
                                     if math.sin(phi) > 1e-6 else 3))
            idx |= {2 + (a - 1) * ns + (s % ns) for s in range(-k, k + 1)}
        return idx
    n = int(cab['malha_celulas'])
    return set(range((n + 1) ** 2))


def main():
    ficheiros = sorted(glob.glob(os.path.join(AQUI, '*.txt.gz')))
    if len(ficheiros) < 20:
        print('PISO DE POPULACAO: %d ficheiros, esperava >= 20 — o censo esta a varrer quase nada'
              % len(ficheiros))
        return 1
    mau = []
    for path in ficheiros:
        nome = os.path.basename(path)
        cab, rest, publicados = ler(path)
        faltam = [k for k in OBRIGATORIAS if k not in cab]
        if faltam:
            mau.append('%s: cabecalho sem %s' % (nome, ','.join(faltam)))
            continue
        if len(publicados) != len(set(publicados)):
            mau.append('%s: indices repetidos no bloco r' % nome)
        if len(rest) != int(cab['vertices_publicados']):
            mau.append('%s: %d linhas r contra vertices_publicados=%s'
                       % (nome, len(rest), cab['vertices_publicados']))
        decl = indices_declarados(cab)
        if set(rest) != decl:
            mau.append('%s: o conjunto publicado (%d) nao e o declarado (%d); %d a mais, %d a menos'
                       % (nome, len(rest), len(decl), len(set(rest) - decl), len(decl - set(rest))))
        pior, onde = 0.0, None
        for i, p in rest.items():
            q = repouso_por_formula(cab, i)
            d = max(abs(a - b) for a, b in zip(p, q))
            if d > pior:
                pior, onde = d, i
        if pior > TOL:
            mau.append('%s: a formula do cabecalho erra %.3e no vertice %s (tolerancia %g)'
                       % (nome, pior, onde, TOL))
    for m in mau:
        print('✗', m)
    print('%d ficheiros conferidos, %d problemas' % (len(ficheiros), len(mau)))
    return 1 if mau else 0


if __name__ == '__main__':
    sys.exit(main())
