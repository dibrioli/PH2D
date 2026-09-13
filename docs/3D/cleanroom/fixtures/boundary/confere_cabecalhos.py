#!/usr/bin/env python3
"""A RÉGUA das fixtures do pincel de contorno — deriva a tabela de excepções do README.

⭐ Existe porque «uma lista de excepções sem a régua ao lado não é auditável»: quem
acrescentar um traço herda a régua que não vê, e a lista envelhece em silêncio.
⚠️ Uma grandeza NOVA no cabeçalho é uma coluna nova nesta régua, no MESMO commit.

    python3 confere_cabecalhos.py            # imprime a tabela (markdown)
    python3 confere_cabecalhos.py --check N  # sai !=0 se a soma das excepções não for N

Não lê nem escreve nada do alvo; só os cabeçalhos das fixtures desta pasta.
"""
import gzip
import glob
import sys
import collections
import os

# O parágrafo «O traço» do README, escrito como dados. Cada chave aqui é uma
# grandeza que o parágrafo fixa SEM ressalva; o que ele já ressalva fica de fora.
BASE = {
    'raio': '0.25',
    'forca': '1',
    'pressao': '1',
    'curva': 'smooth',
    'passos': '8',
    'inverter': '0',
    'simetria_x': '0',
    'esbatimento_da_simetria': '0',
    'mascara': 'nenhuma',
    'alvo': 'geometria',
    'deslocamento_da_origem': '0',
}
# ⚠️ 'superficie', 'modo' e 'queda_no_contorno' NÃO entram: são o eixo do corpus
# (é para variar essas três que o corpus existe), e o README diz isso por escrito.

# O arrasto do parágrafo é «0,1 para BAIXO na vista», logo o literal DEPENDE da vista —
# compará-lo com um só literal acusaria as 11 peças de vista frontal por construção da
# régua, e não por serem excepção. ⇒ a régua resolve a vista primeiro.
ARRASTO_POR_VISTA = {
    'topo': '0.00000000 -0.10000000 0.00000000',
    'frente': '0.00000000 0.00000000 -0.10000000',
}


def header(path):
    h = {}
    with gzip.open(path, 'rt') as g:
        for line in g:
            if line.startswith('#'):
                continue
            if line.startswith(('c ', 'd ', 'caminho', 'passo ', 'vertices')):
                break
            k, _, v = line.strip().partition(' ')
            h[k] = v
    return h


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    files = sorted(glob.glob(os.path.join(here, '*.deformado.txt.gz'))) + \
        sorted(glob.glob(os.path.join(here, '*.porpasso.txt.gz')))
    if not files:
        print('✗ nenhuma fixture encontrada — a régua estaria a medir ZERO', file=sys.stderr)
        return 2
    exc = collections.defaultdict(list)
    missing = []
    for p in files:
        h = header(p)
        name = os.path.basename(p).split('.')[0]
        esperado = dict(BASE)
        vista = h.get('vista')
        if vista not in ARRASTO_POR_VISTA:
            missing.append((name, 'vista'))
        else:
            esperado['arrasto'] = ARRASTO_POR_VISTA[vista]
        for k, v in esperado.items():
            got = h.get(k)
            if got is None:
                missing.append((name, k))
            elif got != v:
                exc[k].append((name, got))
    if missing:
        print('✗ cabeçalho sem grandeza que a régua mede (a régua está à frente do corpus):',
              file=sys.stderr)
        for n, k in missing[:10]:
            print('   %s: %s' % (n, k), file=sys.stderr)
        return 2

    total = sum(len(v) for v in exc.values())
    print('| grandeza | quantas | quais |')
    print('|---|---|---|')
    for k in list(BASE) + ['arrasto']:
        if exc[k]:
            quais = ' · '.join('`%s` (`%s`)' % (a, b) for a, b in exc[k])
            esperado = BASE.get(k, '0,1 para baixo NA VISTA')
            print('| **%s** ≠ `%s` | `%d` | %s |' % (k, esperado, len(exc[k]), quais))
    print()
    print('fixtures varridas: %d · soma das excepções: %d' % (len(files), total))

    if '--check' in sys.argv:
        want = int(sys.argv[sys.argv.index('--check') + 1])
        if total != want:
            print('✗ soma das excepções %d, README diz %d — actualize o README' % (total, want),
                  file=sys.stderr)
            return 1
        print('✓ soma das excepções bate com o README')
    return 0


if __name__ == '__main__':
    sys.exit(main())
