#!/usr/bin/env python3
"""Dá NOME aos ids que a sonda `censo_dos_controlos` imprime em hexadecimal.

Lê toda a árvore `crates/` e junta: (1) os literais `hash_node_id("slug")` com o nome da const; (2) os
slugs DERIVADOS `format!("prefixo.{k}")` dentro de `hash_node_id_runtime`, expandidos para k = 0..255.
Uso: python3 nomes_dos_ids.py < target/censo_todos.log > target/censo_nomeado.tsv
"""
import os, re, sys

FNV_OFF, FNV_PRIME = 0xcbf29ce484222325, 0x100000001b3

def fnv(s: str) -> int:
    h = FNV_OFF
    for b in s.encode():
        h ^= b
        h = (h * FNV_PRIME) & 0xFFFFFFFFFFFFFFFF
    return h or 1

nomes = {}
lit = re.compile(r'(?:pub\s+)?const\s+([A-Z0-9_]+)\s*:\s*NodeId\s*=\s*hash_node_id\(\s*"([^"]+)"')
lit_solto = re.compile(r'hash_node_id\(\s*"([^"]+)"')
fmt = re.compile(r'hash_node_id_runtime\(\s*&format!\(\s*"([^"]*)\{[^}]*\}([^"]*)"')
for raiz, _, fs in os.walk('crates'):
    for f in fs:
        if not f.endswith('.rs'):
            continue
        t = open(os.path.join(raiz, f), encoding='utf-8', errors='replace').read()
        for m in lit_solto.finditer(t):
            nomes.setdefault(fnv(m.group(1)), m.group(1))
        for m in lit.finditer(t):
            nomes[fnv(m.group(2))] = m.group(1)
        for m in fmt.finditer(t):
            for k in range(256):
                slug = f'{m.group(1)}{k}{m.group(2)}'
                nomes.setdefault(fnv(slug), slug)

hexa = re.compile(r'0x[0-9a-f]{16}')
for linha in sys.stdin:
    if not linha.startswith('CENSO'):
        continue
    linha = hexa.sub(lambda m: nomes.get(int(m.group(0), 16), m.group(0)), linha)
    sys.stdout.write(linha)
