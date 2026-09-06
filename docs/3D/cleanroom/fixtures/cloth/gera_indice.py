#!/usr/bin/env python3
"""Gera `indice.json` a partir dos cabeçalhos dos `*.deformado.txt.gz` deste diretório.
Uma entrada por ficheiro, com as mesmas chaves do cabeçalho. O índice é DERIVADO: nunca se
escreve à mão, e regenera-se sempre que um traço é acrescentado. Exit 0 = escrito.
(Harness — não carrega algoritmo nenhum.)"""
import gzip, json, glob, os, sys
TEXTO = {'superficie', 'modo', 'falloff_da_forca', 'area', 'curva'}
INTEIRO = {'pino', 'passos', 'movidos'}
def cabecalho(path):
    h = {}
    with gzip.open(path, 'rt') as f:
        for line in f:
            if line.startswith('#'):
                continue
            t = line.split()
            if t[0] in ('caminho', 'vertices', 'c', 'd', 'v'):
                break
            h[t[0]] = t[1] if t[0] in TEXTO else (int(t[1]) if t[0] in INTEIRO else float(t[1]))
    return h
def main(d):
    entradas = [[os.path.basename(f)[:-len('.deformado.txt.gz')], cabecalho(f)]
                for f in sorted(glob.glob(os.path.join(d, '*.deformado.txt.gz')))]
    with open(os.path.join(d, 'indice.json'), 'w') as g:
        json.dump(entradas, g, indent=1)
        g.write('\n')
    print(f'indice.json: {len(entradas)} entradas para {len(entradas)} ficheiros')
    sys.exit(0)
if __name__ == '__main__':
    main(sys.argv[1] if len(sys.argv) > 1 else os.path.dirname(os.path.abspath(__file__)))
