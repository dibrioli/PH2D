#!/usr/bin/env python3
"""O conjunto PRECISO de pacotes cujos testes um diff pode mudar — o seletor do `nextest-impacted.sh`.

    python3 scripts/impacto-preciso.py <pacote mudado>...   # imprime o filtro do nextest
    python3 scripts/impacto-preciso.py --auto-teste         # confere a lei contra a workspace

A lei (a mesma do grafo que o cargo compila): os testes de `P` mudam se um pacote mudado está
  · no fecho NORMAL+BUILD de `P` (o que entra no binário de `P`), ou
  · no fecho normal+build de uma DEV-dependência DIRECTA de `P` (o que só os testes de `P` ligam).
A dev-dependência de uma dependência NUNCA entra no binário de `P`.

⛔ O `rdeps()` do nextest segue as dev-arestas em CADEIA (o grafo do guppy): o `ph2d-render` tem a
pintura e o `ph2d-editor-core` como dev-deps, e por isso todo dependente do renderizador — a Luz 2D
na placa, o esqueleto — corria a cada mudança de UI sem que um byte do binário dele mudasse.
Medido em 2026-10-13: `docs/DevOps/VELOCIDADE_DOS_PORTOES.md`.

Mais os LEITORES DE FICHEIROS: um teste que lê, pelo `CARGO_MANIFEST_DIR`, fonte de OUTRO pacote não
depende dele no grafo. O pacote que varre a workspace (`read_dir` sobre `crates/`/`shells/`) entra a
cada diff de código; o que nomeia pacotes por caminho entra quando um deles muda. É a mesma cegueira
que a rede global do `nextest-impacted.sh` cobre por nome — aqui ela é derivada, não listada.
"""

import collections
import json
import os
import re
import subprocess
import sys

KINDS_DO_BINARIO = {None, "build"}


def metadata(root):
    out = subprocess.run(
        ["cargo", "metadata", "--offline", "--format-version", "1"],
        cwd=root, capture_output=True, text=True, check=True,
    ).stdout
    return json.loads(out)


def grafo(meta):
    ws = set(meta["workspace_members"])
    nome = {p["id"]: p["name"] for p in meta["packages"]}
    normal = collections.defaultdict(set)
    dev = collections.defaultdict(set)
    for n in meta["resolve"]["nodes"]:
        if n["id"] not in ws:
            continue
        a = nome[n["id"]]
        for d in n["deps"]:
            if d["pkg"] not in ws:
                continue
            kinds = {k["kind"] for k in d["dep_kinds"]}
            if kinds & KINDS_DO_BINARIO:
                normal[a].add(nome[d["pkg"]])
            if "dev" in kinds:
                dev[a].add(nome[d["pkg"]])
    pacotes = sorted(nome[i] for i in ws)
    pastas = {}
    for p in meta["packages"]:
        if p["id"] in ws:
            pastas[p["name"]] = os.path.relpath(os.path.dirname(p["manifest_path"]), meta["workspace_root"])
    return pacotes, normal, dev, pastas


def fecho(x, g, memo):
    if x in memo:
        return memo[x]
    visto, pilha = {x}, [x]
    while pilha:
        for y in g[pilha.pop()]:
            if y not in visto:
                visto.add(y)
                pilha.append(y)
    memo[x] = visto
    return visto


def alcance(pacotes, normal, dev):
    """Para cada pacote, o que entra em algum binário de teste dele."""
    memo = {}
    out = {}
    for p in pacotes:
        a = set(fecho(p, normal, memo))
        for d in dev[p]:
            a |= fecho(d, normal, memo)
        out[p] = a
    return out


def leitores(root, pastas):
    """`{pacote: (global, alvos)}` dos que leem fonte fora da própria pasta."""
    por_pasta = {os.path.basename(d): n for n, d in pastas.items()}
    desktop = next((n for n, d in pastas.items() if d == "shells/desktop"), None)
    try:
        r = subprocess.run(["rg", "-l", "CARGO_MANIFEST_DIR", "-g", "*.rs", "crates", "shells", "tools"],
                           cwd=root, capture_output=True, text=True)
        ficheiros = r.stdout.split()
    except FileNotFoundError:
        ficheiros = [os.path.relpath(os.path.join(d, f), root)
                     for top in ("crates", "shells", "tools")
                     for d, _, fs in os.walk(os.path.join(root, top)) if "/target" not in d
                     for f in fs if f.endswith(".rs")]
    pasta_de = sorted(((d + "/", n) for n, d in pastas.items()), key=lambda x: -len(x[0]))
    out = {}
    for f in ficheiros:
        dono = next((n for pre, n in pasta_de if f.startswith(pre)), None)
        if dono is None:
            continue
        s = open(os.path.join(root, f), errors="replace").read()
        if "CARGO_MANIFEST_DIR" not in s or not re.search(r'"\.\./|\.\./\.\.|join\("\.\."\)|\.parent\(\)', s):
            continue
        glob_, alvos = out.get(dono, (False, set()))
        for lit in re.findall(r'"([^"\n]*)"', s):
            for a, b in re.findall(r"(?:crates|tools)/([a-z0-9_-]+)|\.\./([a-z0-9_-]+)", lit):
                x = por_pasta.get(a or b)
                if x and x != dono:
                    alvos.add(x)
            if desktop and re.search(r"shells/desktop|\.\./desktop", lit) and desktop != dono:
                alvos.add(desktop)
        if re.search(r"read_dir|WalkDir|walkdir|glob\(", s) and re.search(r'"crates"|crates/|"shells"', s):
            glob_ = True
        out[dono] = (glob_, alvos)
    return out


def impactados(root, mudados, meta=None):
    meta = meta or metadata(root)
    pacotes, normal, dev, pastas = grafo(meta)
    desconhecidos = [m for m in mudados if m not in pastas]
    if desconhecidos:
        raise SystemExit(f"[impacto-preciso] pacote(s) fora da workspace: {desconhecidos}")
    c = set(mudados)
    alc = alcance(pacotes, normal, dev)
    sel = {p for p in pacotes if alc[p] & c}
    if c:
        for p, (glob_, alvos) in leitores(root, pastas).items():
            if glob_ or alvos & c:
                sel.add(p)
    return sorted(sel)


def auto_teste(root):
    """Fatos da workspace que a lei TEM de dar — cada um falha alto se o seletor mudar de sentido."""
    meta = metadata(root)
    ok = True

    def conta(desc, cond):
        nonlocal ok
        print(f"[impacto-preciso] {'✓' if cond else '✗'} {desc}")
        ok &= cond

    tem = {p["name"] for p in meta["packages"]}
    i18n = set(impactados(root, ["ph2d-i18n"], meta))
    conta("a tabela de strings selecciona quem a liga (ph2d-app-light2d)", "ph2d-app-light2d" in i18n)
    conta("a tabela de strings selecciona-se a si", "ph2d-i18n" in i18n)
    if "ph2d-light2d-gpu" in tem:
        conta("a tabela de strings NÃO selecciona a Luz 2D na placa (não está no binário dela)",
              "ph2d-light2d-gpu" not in i18n)
        ref = set(impactados(root, ["ph2d-light2d-ref"], meta))
        conta("o gabarito selecciona quem o liga só nos TESTES (dev-dep da ph2d-light2d-gpu)",
              "ph2d-light2d-gpu" in ref)
    pintura = set(impactados(root, ["ph2d-tool-painter"], meta))
    conta("a pintura selecciona o renderizador (dev-dep directa dele)", "ph2d-render" in pintura)
    conta("a pintura NÃO selecciona a Luz 2D na placa (dev-dep do renderizador não viaja)",
          "ph2d-light2d-gpu" not in pintura or "ph2d-light2d-gpu" not in tem)
    conta("um diff de código selecciona o leitor global da workspace (ph2d-editor-core)",
          "ph2d-editor-core" in pintura)
    return ok


if __name__ == "__main__":
    raiz = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True,
                          check=True).stdout.strip()
    if sys.argv[1:] == ["--auto-teste"]:
        sys.exit(0 if auto_teste(raiz) else 1)
    sel = impactados(raiz, sys.argv[1:])
    print(" | ".join(f"package(={p})" for p in sel))
