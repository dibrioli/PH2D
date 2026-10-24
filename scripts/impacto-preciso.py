#!/usr/bin/env python3
"""O conjunto PRECISO de testes que um diff pode mudar — o seletor do `nextest-impacted.sh`.

    python3 scripts/impacto-preciso.py <pacote mudado>...   # imprime o filtro do nextest
    python3 scripts/impacto-preciso.py --auto-teste         # confere a lei contra a workspace
    python3 scripts/impacto-preciso.py --confere <lista.json|-> <pacote mudado>...
        # cada filtro de VIGIA casa ≥ 1 teste na `cargo nextest list --message-format json`
    python3 scripts/impacto-preciso.py --leitores           # o censo, um ficheiro por linha

A lei (a mesma do grafo que o cargo compila): os testes de `P` mudam se um pacote mudado está
  · no fecho NORMAL+BUILD de `P` (o que entra no binário de `P`), ou
  · no fecho normal+build de uma DEV-dependência DIRECTA de `P` (o que só os testes de `P` ligam).
A dev-dependência de uma dependência NUNCA entra no binário de `P`.
⛔ O `rdeps()` do nextest segue as dev-arestas em CADEIA (o grafo do guppy).

Mais os VIGIAS: um ficheiro de teste que lê fonte de OUTRO pacote — pelo `CARGO_MANIFEST_DIR` ou por
`include_str!` — não depende dele no grafo. Quem varre a RAIZ da workspace (`crates/`, `shells/`,
`tools/` como directório de partida) entra a cada diff; quem nomeia pacotes entra quando um deles muda.
⭐ Entra SÓ o ficheiro (o módulo dele no binário de teste), nunca o pacote inteiro: um único vigia na
`ph2d-tool-painter` punha os 2 566 testes dela em qualquer diff. O pacote entra INTEIRO só pelo grafo.
`PH2D_IMPACTO=pacote` volta à regra de 13/10 (o pacote inteiro do leitor, a heurística antiga) — a
régua da medição. Medido: `docs/DevOps/VELOCIDADE_DOS_PORTOES.md` §2.2 e §2.5.
"""

import collections
import json
import os
import re
import subprocess
import sys

KINDS_DO_BINARIO = {None, "build"}
GRUPOS = ("crates", "shells", "tools")
# ⛔ O censo tem PISO: um vigia que deixa de ser visto é um teste que deixa de correr em silêncio.
# Contado em 2026-10-23 (`--leitores`). Quem o baixar diz porquê no commit; quem o subir, mede.
PISO_VIGIAS = 279
PISO_GLOBAIS = 102


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


# ── O léxico: comentários fora, literais à parte ────────────────────────────────────────────────

def lexico(s):
    """`(sem_comentarios, cego)`, do MESMO comprimento que `s`: no primeiro os comentários viram
    espaço; no segundo também o miolo dos literais (chaves e `mod` dentro de texto não contam)."""
    a, b = [], []  # pedaços; os apagados viram espaço (as quebras de linha ficam)
    i, n = 0, len(s)
    branco = lambda t: re.sub(r"[^\n]", " ", t) if "\n" in t else " " * len(t)  # noqa: E731
    while i < n:
        m = RE_LEX.search(s, i)
        if not m:
            break
        j, t = m.start(), m.group()
        a.append(s[i:j])
        b.append(s[i:j])
        if t.startswith("//"):
            k = m.end()
            a.append(branco(t))
            b.append(branco(t))
        elif t == "/*":
            prof, k = 1, j + 2
            while k < n and prof:
                x = RE_BLOCO.search(s, k)
                if not x:
                    k = n
                    break
                prof, k = prof + (1 if x.group() == "/*" else -1), x.end()
            a.append(branco(s[j:k]))
            b.append(branco(s[j:k]))
        elif t == "'":
            c = RE_CHAR.match(s, j)
            k = c.end() if c else j + 1
            a.append(s[j:k])
            b.append(s[j] + branco(s[j + 1:k - 1]) + s[k - 1] if c else s[j])
        else:
            if t.endswith('#"') or t.startswith(("r", "br")):
                fim = '"' + t[t.index("r") + 1:-1]
                e = s.find(fim, m.end())
                e = n if e < 0 else e
                k = min(n, e + len(fim))
            else:
                x = RE_FIM_STR.search(s, m.end())
                e = x.end() - 1 if x else n
                k = min(n, e + 1)
            a.append(s[j:k])
            b.append(t + branco(s[m.end():e]) + s[e:k])
        i = k
    a.append(s[i:])
    b.append(s[i:])
    return "".join(a), "".join(b)


RE_LEX = re.compile(r'//[^\n]*|/\*|(?<![\w])b?r#*"|(?<![\w])b?"|"|\'')
RE_BLOCO = re.compile(r"/\*|\*/")
RE_CHAR = re.compile(r"'(?:\\u\{[0-9a-fA-F]+\}|\\.|[^\\'\n])'")
RE_FIM_STR = re.compile(r'(?<!\\)(?:\\\\)*"')


def literais(codigo):
    """Os literais do código, menos as MENSAGENS (`expect("crates/")` não é um caminho)."""
    return [m.group(1) for m in re.finditer(r'"((?:[^"\\\n]|\\.)*)"', codigo)
            if not re.search(r"\b(?:expect|panic!|unreachable!|e?println!)\s*\(\s*$", codigo[max(0, m.start() - 40):m.start()])
            # `.join("crates").join("ph2d-x")`: o pedaço do meio é lido pela cadeia (`_pontos`), não aqui
            and not re.match(r"\s*\)\s*\.\s*join\s*\(", codigo[m.end():m.end() + 40])]


# ── A árvore de módulos: que ficheiro é que módulo de que binário de teste ───────────────────────

COM_TESTES = "<binários com algum #[test]>"
RE_MOD = re.compile(r"((?:#\[[^\]]*\]\s*)*)(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+(?:r#)?(\w+)\s*([;{])")


_MEMO = {}


def modulos(meta, root):
    if ("mods", root) not in _MEMO:
        _MEMO[("mods", root)] = _modulos(meta, root)
    return _MEMO[("mods", root)]


def _modulos(meta, root):
    """`{ficheiro relativo: [(pacote, binary_id, caminho do módulo)]}` de todo alvo que o nextest corre."""
    ws = set(meta["workspace_members"])
    out = collections.defaultdict(list)
    com_testes = set()
    out[COM_TESTES] = com_testes
    for p in meta["packages"]:
        if p["id"] not in ws:
            continue
        for t in p["targets"]:
            k = set(t["kind"])
            if k & {"lib", "rlib", "proc-macro", "dylib", "cdylib", "staticlib"}:
                bid = p["name"]
            elif "test" in k:
                bid = f'{p["name"]}::{t["name"]}'
            elif "bin" in k:
                bid = f'{p["name"]}::bin/{t["name"]}'
            else:
                continue
            visto = set()

            def anda(f, caminho, dono_da_pasta):
                f = os.path.normpath(f)
                if f in visto or not os.path.isfile(f):
                    return
                visto.add(f)
                out[os.path.relpath(f, root)].append((p["name"], bid, "::".join(caminho)))
                cru = open(f, errors="replace").read()
                if RE_TEM_TESTE.search(cru):
                    com_testes.add(bid)
                if not re.search(r"\bmod\s+(?:r#)?\w+\s*;", cru):
                    return  # uma folha: nenhum ficheiro abaixo dela
                sem, cego = lexico(cru)
                pasta = os.path.dirname(f)
                base = pasta if dono_da_pasta else os.path.join(pasta, os.path.splitext(os.path.basename(f))[0])
                eventos = [(m.start(3), "mod", m) for m in RE_MOD.finditer(cego)]
                usadas = {m.start(3) for _, _, m in eventos if m.group(3) == "{"}
                eventos += [(x.start(), x.group(), None) for x in re.finditer(r"[{}]", cego) if x.start() not in usadas]
                pilha = []
                for pos, ev, m in sorted(eventos, key=lambda e: e[0]):
                    if ev == "{":
                        pilha.append(None)
                    elif ev == "}":
                        if pilha:
                            pilha.pop()
                    elif m.group(3) == "{":
                        pilha.append(m.group(2))
                    else:
                        dentro = [x for x in pilha if x]
                        attrs = sem[m.start(1):m.end(1)]
                        pa = re.search(r'#\[\s*path\s*=\s*"([^"]+)"\s*\]', attrs)
                        nome = m.group(2)
                        if pa and not dentro:
                            anda(os.path.join(pasta, pa.group(1)), caminho + [nome], True)
                        elif pa:
                            anda(os.path.join(base, *dentro, pa.group(1)), caminho + dentro + [nome], True)
                        else:
                            d = os.path.join(base, *dentro)
                            for c, dp in ((os.path.join(d, nome + ".rs"), False),
                                          (os.path.join(d, nome, "mod.rs"), True)):
                                if os.path.isfile(c):
                                    anda(c, caminho + dentro + [nome], dp)
                                    break

            anda(t["src_path"], [], True)
    return out


# ── Os vigias: quem lê fonte alheia, e de quem ─────────────────────────────────────────────────

RE_TEM_TESTE = re.compile(r"#\[\s*(?:\w+::)*(?:test|rstest|test_case)\b|proptest!")
RE_VARRE = re.compile(r"\bread_dir\b|WalkDir|walkdir|\bglob\(")


def _cadeia(cod, j):
    """Os passos de caminho a partir de `j`: `[('sobe', n) | ('lit', texto) | ('var',)]`."""
    passos = []
    while True:
        m = re.match(r"\s*\)?\s*\.\s*(\w+)\s*\(", cod[j:])
        if not m:
            return passos
        k, prof = j + m.end(), 1
        while k < len(cod) and prof:
            prof += {"(": 1, ")": -1}.get(cod[k], 0)
            k += 1
        arg = cod[j + m.end():k - 1].strip()
        met = m.group(1)
        if met == "parent":
            passos.append(("sobe", 1))
        elif met in ("and_then", "map") and "parent" in arg:
            passos.append(("sobe", arg.count("parent")))
        elif met == "nth" and arg.isdigit():
            passos.append(("sobe", int(arg)))
        elif met in ("join", "push"):
            lit = re.fullmatch(r'"((?:[^"\\]|\\.)*)"', arg)
            if not lit:
                if passos and passos[-1][0] == "lit":
                    passos[-1] = ("lit", passos[-1][1] + "/{var}")
                else:
                    passos.append(("var",))
                return passos
            t = lit.group(1)
            if passos and passos[-1][0] == "lit":  # `.join("crates").join("ph2d-x")` é UM caminho
                passos[-1] = ("lit", os.path.normpath(passos[-1][1] + "/" + t))
            else:
                while t.startswith("../") or t == "..":
                    passos.append(("sobe", 1))
                    t = t[3:] if t.startswith("../") else ""
                if t:
                    passos.append(("lit", t))
        elif met not in ("expect", "unwrap", "unwrap_or_else", "canonicalize", "to_path_buf",
                         "as_path", "to_owned", "into", "clone", "as_ref", "into_owned", "ancestors"):
            return passos
        j = k


RE_BASE = re.compile(r'env!\(\s*"CARGO_MANIFEST_DIR"\s*\)|var(?:_os)?\(\s*"CARGO_MANIFEST_DIR"\s*\)')
RE_NOME = re.compile(r"\b(\w+)\b(?:\(\))?")


def _pontos(cod):
    """Onde o ficheiro chega a partir do `CARGO_MANIFEST_DIR`: `[(níveis acima, literal|'{var}'|None)]`.
    Segue `let x = …` e `fn f() -> … { … }` (até três saltos de nome); a DEFINIÇÃO de um nome não é
    um ponto de chegada — os USOS dele é que são."""
    defs = [(m.start(2), m.end(2), m.group(1), m.group(2))
            for m in re.finditer(r"\blet\s+(?:mut\s+)?(\w+)\s*(?::[^=;]+)?=\s*([^;]*)", cod)]
    defs += [(m.start(2), m.end(2), m.group(1), m.group(2))
             for m in re.finditer(r"\bfn\s+(\w+)\s*\(\s*\)\s*->[^{]*\{([^{}]*)\}", cod)]
    nomes = {}
    for _ in range(3):
        for _, _, nome, expr in defs:
            nivel = _nivel_de(expr, nomes)
            if nivel is not None:
                nomes[nome] = nivel
    spans = [(a, b) for a, b, nome, _ in defs if nome in nomes]
    partidas = [(m.end(), 0) for m in RE_BASE.finditer(cod)]
    partidas += [(m.end(), nomes[m.group(1)]) for m in RE_NOME.finditer(cod)
                 if m.group(1) in nomes and not re.search(r"\b(?:let(?:\s+mut)?|fn)\s+$", cod[max(0, m.start() - 12):m.start()])]
    out = []
    for j, n0 in partidas:
        if any(a <= j <= b for a, b in spans):
            continue
        n, lit = n0, None
        for p in _cadeia(cod, j):
            if p[0] == "sobe":
                n += p[1]
            elif p[0] == "lit":
                lit = p[1]
            else:
                lit = "{var}"
            if lit and lit.startswith(".."):  # `.join("crates").join("..")` — volta a subir
                n, lit = n + 1, None
        out.append((n, lit))
    return out


def _nivel_de(expr, nomes):
    m = RE_BASE.search(expr)
    n = 0
    if not m:
        m = next((x for x in RE_NOME.finditer(expr) if x.group(1) in nomes), None)
        if not m:
            return None
        n = nomes[m.group(1)]
    for p in _cadeia(expr, m.end()):
        if p[0] != "sobe":
            return None
        n += p[1]
    return n


def _pacote_do_caminho(rel, pasta_de):
    rel = os.path.normpath(rel)
    if rel.startswith(".."):
        return None
    return next((n for pre, n in pasta_de if (rel + "/").startswith(pre)), None)


def vigias(root, pastas):
    if ("vigias", root) not in _MEMO:
        _MEMO[("vigias", root)] = _vigias(root, pastas)
    return _MEMO[("vigias", root)]


def _vigias(root, pastas):
    """`{ficheiro: (dono, global, alvos)}` — cada ficheiro que lê fonte FORA da pasta do seu pacote."""
    pasta_de = sorted(((d + "/", n) for n, d in pastas.items()), key=lambda x: -len(x[0]))
    por_pasta = {os.path.basename(d): n for n, d in pastas.items()}
    r = subprocess.run(["rg", "-l", r"CARGO_MANIFEST_DIR|include_(str|bytes)!", "-g", "*.rs", *GRUPOS],
                       cwd=root, capture_output=True, text=True)
    if r.returncode not in (0, 1):
        raise SystemExit(f"[impacto-preciso] ✗ o `rg` falhou: {r.stderr.strip()}")
    out = {}
    for f in sorted(r.stdout.split()):
        dono = next((n for pre, n in pasta_de if f.startswith(pre)), None)
        if dono is None:
            continue
        pasta = pastas[dono]
        fundo = pasta.count("/") + 1  # níveis do pacote até à raiz
        sem, _ = lexico(open(os.path.join(root, f), errors="replace").read())
        glob_, alvos = False, set()

        def nomeia(rel_raiz):
            nonlocal glob_
            alvo = _pacote_do_caminho(rel_raiz, pasta_de)
            partes = os.path.normpath(rel_raiz).split("/")
            if alvo:
                alvos.add(alvo)
            elif partes[0] in GRUPOS and (len(partes) == 1 or any(
                    d.startswith(f"{partes[0]}/{partes[1]}") for d in pastas.values())):
                glob_ = True  # `crates/`, `crates/<var>`, `crates/ph2d-panel-` — a pasta de grupo, por prefixo
            # (`shells/game`, que não é pacote nem prefixo de nenhum, não lê fonte de ninguém)

        # 1. `include_str!`/`include_bytes!`: relativo ao ficheiro, ou `concat!(env!(…), "/…")`.
        for m in re.finditer(r'include_(?:str|bytes)!\(\s*(?:concat!\(\s*env!\(\s*"CARGO_MANIFEST_DIR"\s*\)\s*,\s*"([^"]+)"|"([^"]+)")', sem):
            if m.group(1):
                nomeia(os.path.join(pasta, m.group(1).lstrip("/")))
            else:
                nomeia(os.path.join(os.path.dirname(f), m.group(2)))
        if "CARGO_MANIFEST_DIR" in sem:
            # 2. As cadeias a partir da pasta do pacote: subir ao GRUPO (`crates/`) sem nomear um filho
            #    é varrer a workspace; subir à raiz e nomear um caminho é nomear um pacote.
            for nivel, lit in _pontos(sem):
                if nivel == 0:
                    continue
                if nivel > fundo:
                    continue
                if lit and "{" in lit:
                    lit = lit.split("{")[0].rstrip("/") or None  # `ph2d-x/src/{var}`: o prefixo nomeia
                if lit is None:
                    if nivel <= fundo - 1:  # o GRUPO (`crates/`) sem nomear um filho: varre
                        glob_ = True
                    continue
                nomeia(os.path.join(pasta, *([".."] * nivel), lit))
            # 3. Os literais de caminho (sem espaço: uma mensagem não é um caminho).
            for lit in literais(sem):
                if not lit or re.search(r"\s", lit):
                    continue
                t = re.sub(r"^(?:\.\./|\./|/)+", "", lit.replace("\\\\", "/"))
                t = re.sub(r"^.*/\.\./\.\./?", "", t) if "/../.." in t else t
                if re.fullmatch(r"(?:crates|shells|tools)/?(?:\*)?", t) or re.search(r"(?:^|/)(?:crates|shells|tools)/\{", t):
                    glob_ = True
                    continue
                if t.split("/")[0] in GRUPOS:
                    nomeia(t)
                elif lit.startswith("../") and t.split("/")[0] in por_pasta:
                    alvos.add(por_pasta[t.split("/")[0]])
        alvos.discard(dono)
        if glob_ or alvos:
            out[f] = (dono, glob_, alvos)
    return out


def leitores_por_pacote(root, pastas):
    """⛔ A regra de 13/10, VERBATIM — só para `PH2D_IMPACTO=pacote` (a régua da medição W4)."""
    por_pasta = {os.path.basename(d): n for n, d in pastas.items()}
    desktop = next((n for n, d in pastas.items() if d == "shells/desktop"), None)
    r = subprocess.run(["rg", "-l", "CARGO_MANIFEST_DIR", "-g", "*.rs", *GRUPOS],
                       cwd=root, capture_output=True, text=True)
    pasta_de = sorted(((d + "/", n) for n, d in pastas.items()), key=lambda x: -len(x[0]))
    out = {}
    for f in r.stdout.split():
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


def filtros_do_vigia(f, dono, mods, root):
    """Os filtros que seleccionam SÓ os testes de `f`: `[(binary_id, prefixo|None)]`; `None` se o
    ficheiro não está em binário de teste nenhum (aí o pacote entra inteiro — o lado seguro)."""
    sitios = [(b, c) for p, b, c in mods.get(f, []) if p == dono]
    if not sitios:
        return None
    tem = bool(RE_TEM_TESTE.search(lexico(open(os.path.join(root, f), errors="replace").read())[0]))
    # Sem `#[test]` é um AJUDANTE: quem o chama vive noutro módulo do mesmo binário ⇒ o binário inteiro;
    # num binário sem teste nenhum (as ferramentas `tools/*-sync`) não há o que correr.
    return sorted({(b, c if (c and tem) else None) for b, c in sitios
                   if tem or b in mods[COM_TESTES]})


def expressao(bid, prefixo):
    return f"binary_id(={bid})" if prefixo is None else f"(binary_id(={bid}) & test(/^{prefixo}::/))"


def impactados(root, mudados, meta=None, modo=None):
    """`(pacotes inteiros, [(pacote, binary_id, prefixo, ficheiro)])`."""
    meta = meta or metadata(root)
    modo = modo or os.environ.get("PH2D_IMPACTO", "preciso")
    pacotes, normal, dev, pastas = grafo(meta)
    desconhecidos = [m for m in mudados if m not in pastas]
    if desconhecidos:
        raise SystemExit(f"[impacto-preciso] pacote(s) fora da workspace: {desconhecidos}")
    c = set(mudados)
    alc = alcance(pacotes, normal, dev)
    sel = {p for p in pacotes if alc[p] & c}
    if not c:
        return sorted(sel), []
    if modo == "pacote":
        for p, (glob_, alvos) in leitores_por_pacote(root, pastas).items():
            if glob_ or alvos & c:
                sel.add(p)
        return sorted(sel), []
    mods = modulos(meta, root)
    filtros = set()
    for f, (dono, glob_, alvos) in vigias(root, pastas).items():
        if dono in sel or not (glob_ or alvos & c):
            continue
        fs = filtros_do_vigia(f, dono, mods, root)
        if fs is None:
            print(f"[impacto-preciso] AVISO: {f} não está em binário de teste nenhum — o pacote {dono} entra inteiro",
                  file=sys.stderr)
            sel.add(dono)
            continue
        filtros |= {(dono, b, pre, f) for b, pre in fs}
    return sorted(sel), sorted(x for x in filtros if x[0] not in sel)


def filtro(sel, filtros):
    partes = [f"package(={p})" for p in sel]
    por_binario = collections.defaultdict(set)
    for _, b, pre, _ in filtros:
        por_binario[b].add(pre)
    for b, pres in sorted(por_binario.items()):
        if None in pres:
            partes.append(expressao(b, None))
        else:  # um termo por binário: `test(/^(?:a|b::c)::/)`
            partes.append(f"(binary_id(={b}) & test(/^(?:{'|'.join(sorted(pres))})::/))")
    return " | ".join(partes)


# ── Os controlos ───────────────────────────────────────────────────────────────────────────────

def lista_do_nextest(fonte):
    """`[(pacote, binary_id, nome do teste, corre)]` — TODOS os listados; `corre` = não é `#[ignore]`.
    (Com `-E` a lista traz também os que o filtro não casou: o casamento aqui é o estrutural.)"""
    dados = json.load(sys.stdin if fonte == "-" else open(fonte))
    return [(s["package-name"], bid, nome, not caso.get("ignored", False))
            for bid, s in dados["rust-suites"].items() for nome, caso in s["testcases"].items()]


def casa(testes, b, pre):
    return sum(1 for _, bb, n, _ in testes if bb == b and (pre is None or n.startswith(pre + "::")))


def confere(root, fonte, mudados):
    """Filtro que casa ZERO é um portão verde que não correu: vermelho alto."""
    testes = lista_do_nextest(fonte)
    if mudados:
        _, filtros = impactados(root, mudados)
    else:  # sem diff: TODOS os vigias da workspace, cada filtro que algum diff pode emitir
        meta = metadata(root)
        _, _, _, pastas = grafo(meta)
        mods = modulos(meta, root)
        filtros = []
        for f, (dono, _, _) in vigias(root, pastas).items():
            filtros += [(dono, b, pre, f) for b, pre in (filtros_do_vigia(f, dono, mods, root) or [])]
    zeros = [(f, b, pre) for _, b, pre, f in filtros if casa(testes, b, pre) == 0]
    for f, b, pre in zeros:
        print(f"[impacto-preciso] ✗ o filtro do vigia {f} casa ZERO testes: {expressao(b, pre)}")
    print(f"[impacto-preciso] {'✓' if not zeros else '✗'} {len(filtros) - len(zeros)}/{len(filtros)} filtros de vigia casam ≥ 1 teste")
    return not zeros


def auto_teste(root):
    """Fatos da workspace que a lei TEM de dar — cada um falha alto se o seletor mudar de sentido."""
    meta = metadata(root)
    ok = True

    def conta(desc, cond):
        nonlocal ok
        print(f"[impacto-preciso] {'✓' if cond else '✗'} {desc}")
        ok &= cond

    tem = {p["name"] for p in meta["packages"]}
    i18n = set(impactados(root, ["ph2d-i18n"], meta, "preciso")[0])
    conta("a tabela de strings selecciona quem a liga (ph2d-app-light2d)", "ph2d-app-light2d" in i18n)
    conta("a tabela de strings selecciona-se a si", "ph2d-i18n" in i18n)
    if "ph2d-light2d-gpu" in tem:
        conta("a tabela de strings NÃO selecciona a Luz 2D na placa (não está no binário dela)",
              "ph2d-light2d-gpu" not in i18n)
        ref = set(impactados(root, ["ph2d-light2d-ref"], meta, "preciso")[0])
        conta("o gabarito selecciona quem o liga só nos TESTES (dev-dep da ph2d-light2d-gpu)",
              "ph2d-light2d-gpu" in ref)
    pintura = impactados(root, ["ph2d-tool-painter"], meta, "preciso")[0]
    conta("a pintura selecciona o renderizador (dev-dep directa dele)", "ph2d-render" in pintura)
    conta("a pintura NÃO selecciona a Luz 2D na placa (dev-dep do renderizador não viaja)",
          "ph2d-light2d-gpu" not in pintura or "ph2d-light2d-gpu" not in tem)
    conta("um diff em ph2d-tool-painter selecciona-a INTEIRA", "ph2d-tool-painter" in pintura)
    conta("… e a ph2d-editor-core INTEIRA pelo grafo (a pintura é dev-dep directa dela)", "ph2d-editor-core" in pintura)
    if "ph2d-app-board" in tem:
        quadro, qf = impactados(root, ["ph2d-app-board"], meta, "preciso")
        conta("um diff de código fora do grafo dela selecciona os VIGIAS da ph2d-editor-core, não o pacote",
              "ph2d-editor-core" not in quadro and any(p == "ph2d-editor-core" for p, *_ in qf))
        conta("um diff só no Quadro NÃO selecciona o pacote inteiro ph2d-tool-painter",
              "ph2d-tool-painter" not in quadro)
        conta("… mas selecciona o vigia global dela (tests/it/macro_portas.rs)",
              any(f == "crates/ph2d-tool-painter/tests/it/macro_portas.rs" for *_, f in qf))
    antigo = set(impactados(root, ["ph2d-tool-painter"], meta, "pacote")[0])
    conta("PH2D_IMPACTO=pacote volta à regra de 13/10 (o leitor global entra INTEIRO)", "ph2d-editor-core" in antigo)
    _, _, _, pastas = grafo(meta)
    v = vigias(root, pastas)
    globais = sum(1 for _, g, _ in v.values() if g)
    conta(f"o censo dos vigias não cai: {len(v)} ficheiros (piso {PISO_VIGIAS}), {globais} globais (piso {PISO_GLOBAIS})",
          len(v) >= PISO_VIGIAS and globais >= PISO_GLOBAIS)
    mods = modulos(meta, root)
    sem_binario = [f for f, (d, _, _) in v.items() if filtros_do_vigia(f, d, mods, root) is None]
    conta(f"todo vigia mora num binário de teste (sem: {sem_binario[:3]}{'…' if len(sem_binario) > 3 else ''})",
          not sem_binario)
    return ok


if __name__ == "__main__":
    raiz = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True,
                          check=True).stdout.strip()
    a = sys.argv[1:]
    if a == ["--auto-teste"]:
        sys.exit(0 if auto_teste(raiz) else 1)
    if a[:1] == ["--confere"]:
        sys.exit(0 if confere(raiz, a[1], a[2:]) else 1)
    if a == ["--leitores"]:
        meta = metadata(raiz)
        mods = modulos(meta, raiz)
        for f, (d, g, al) in vigias(raiz, grafo(meta)[3]).items():
            fs = filtros_do_vigia(f, d, mods, raiz)
            print(f"{'G' if g else 'A'}\t{d}\t{f}\t{','.join(sorted(al)) or '-'}\t"
                  f"{' | '.join(expressao(b, p) for b, p in fs) if fs else '(binário sem testes)' if fs == [] else 'package(=' + d + ')'}")
        sys.exit(0)
    sel, filtros = impactados(raiz, a)
    print(filtro(sel, filtros))
