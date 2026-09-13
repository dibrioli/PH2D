#!/usr/bin/env python3
"""**O CENSO dos ids de widget da fundação** — quem LÊ cada item de `ph2d-editor-core/src/ids/`,
e para onde ele pode descer.

    python3 scripts/censo-ids.py                 # a tabela
    python3 scripts/censo-ids.py --json x.json   # item → classe, dono, leitores (para o mover)
    python3 scripts/censo-ids.py --lista DESCE   # os nomes de uma classe
    python3 scripts/censo-ids.py --autoteste

# A pergunta (auditoria de arquitectura 2026-09-12, A5b)

Um id que só UM painel lê (e as crates que dependem desse painel) não tem razão para morar na
fundação que 60 crates recompilam. O que decide é o LEITOR, e o leitor lê-se no CÓDIGO:

1. ⛔ **prosa não é leitura** (HOWTO §2.12) — comentários e strings saem antes de contar;
2. ⛔ **um `pub use` não é leitura** — o `ids.rs` de cada painel re-exporta dezenas de ids que
   ninguém usa por aquele caminho; contá-lo como leitor faria o painel «ler» tudo o que publica;
3. ⛔ **a citação DENTRO de `ids/` prende** — uma tabela `const` que nomeia `A` obriga `A` a ficar
   onde a tabela o vê: se a tabela FICA, `A` fica; se ela desce para `P`, `A` pode ficar ou descer
   para `P`, nunca para outro sítio;
4. ⛔ **camadas** — de um painel só dependem famílias, a composição e os registos (gate
   `architecture_no_dependency_climbs_a_layer`); um leitor numa FERRAMENTA ou numa FOLHA prende o
   id acima do painel;
5. ⛔ **a CERCA com a `line/render-loop`** — um leitor em `shells/desktop/src/render_loop/**`,
   `app_state.rs`, `vec_*.rs` ou `ph2d-app-vec/src/state.rs` prende o id nesta rodada.

# A unidade

A régua lê TOKENS de identificador. Um nome de id é SCREAMING_CASE ou `snake_case_id` e raramente
é homónimo; os homónimos (o mesmo nome definido fora de `ids/`) são impressos à parte, porque um
leitor falso só erra no sentido CONSERVADOR (prende um id que podia descer), e o sentido perigoso
— descer um id que alguém lê — falha ALTO no compilador.
"""

from __future__ import annotations

import argparse
import collections
import fnmatch
import json
import os
import re
import subprocess
import sys

IDS_DIR = "crates/ph2d-editor-core/src/ids/"
EC = "ph2d-editor-core"
# ⛔ A CERCA ACABOU: as duas linhas (`line/render-loop` e `line/editor-core`) integraram-se em 13/09, e os
# ids que ela prendia desceram na mesma integração. Uma rodada futura com linhas paralelas volta a
# declará-la aqui, com os globs dos ficheiros da outra linha.
CERCA: list[str] = []
RE_IDENT = re.compile(r"\b[A-Za-z_][A-Za-z0-9_]*\b")


# ---------------------------------------------------------------- stripper que PRESERVA offsets
def blank(src: str, keep_strings: bool = False) -> str:
    """Comentários (e strings, salvo `keep_strings`) viram ESPAÇOS — as quebras de linha e os
    offsets ficam, para que um span medido no texto limpo corte o ficheiro original."""
    out = list(src)
    i, n = 0, len(src)

    def apaga(a: int, b: int) -> None:
        for k in range(a, min(b, n)):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        c = src[i]
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            j = src.find("\n", i)
            j = n if j < 0 else j
            apaga(i, j)
            i = j
            continue
        if c == "/" and i + 1 < n and src[i + 1] == "*":
            depth, j = 1, i + 2
            while j < n and depth:
                if src[j] == "/" and j + 1 < n and src[j + 1] == "*":
                    depth, j = depth + 1, j + 2
                elif src[j] == "*" and j + 1 < n and src[j + 1] == "/":
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            apaga(i, j)
            i = j
            continue
        if c in "rb" and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_")):
            j = i
            if src[j] == "b" and j + 1 < n and src[j + 1] == "r":
                j += 1
            if src[j] == "r" and j + 1 < n and src[j + 1] in '"#':
                k, h = j + 1, 0
                while k < n and src[k] == "#":
                    h, k = h + 1, k + 1
                if k < n and src[k] == '"':
                    fim = '"' + "#" * h
                    e = src.find(fim, k + 1)
                    e = n if e < 0 else e + len(fim)
                    if not keep_strings:
                        apaga(k + 1, e - len(fim))
                    i = e
                    continue
        if c == '"':
            j = i + 1
            while j < n:
                if src[j] == "\\":
                    j += 2
                elif src[j] == '"':
                    break
                else:
                    j += 1
            if not keep_strings:
                apaga(i + 1, j)
            i = j + 1
            continue
        if c == "'":
            # char literal (não lifetime): 'x' · '\n' · '\u{..}'
            if i + 2 < n and src[i + 1] == "\\":
                j = src.find("'", i + 2)
                if j > 0 and j - i <= 12:
                    apaga(i + 1, j)
                    i = j + 1
                    continue
            if i + 2 < n and src[i + 2] == "'":
                apaga(i + 1, i + 2)
                i += 3
                continue
        i += 1
    return "".join(out)


# ---------------------------------------------------------------- itens de topo de um ficheiro
RE_HEAD = re.compile(
    r"^(?P<attrs>(?:\s*#\s*!?\[[^\]]*\]\s*)*)\s*"
    r"(?P<vis>pub(?:\s*\([^)]*\))?\s+)?"
    r"(?P<kind>const\s+fn|const|static|fn|enum|struct|type|impl|mod|use|trait|macro_rules!)\s*"
    r"(?P<rest>.*)$",
    re.S,
)


def items(src: str):
    """→ [(kind, name, vis, start, end, is_cfg_test)] dos itens de TOPO. `start` inclui os
    doc-comments e atributos acima do item; `end` é exclusivo e cobre o `;` ou a `}` final."""
    b = blank(src)
    n = len(b)
    out = []
    depth = 0
    item_start = 0  # onde começou o texto do item corrente (fim do anterior)
    i = 0
    while i < n:
        c = b[i]
        if c in "{([":
            depth += 1
        elif c in "})]":
            depth -= 1
            if c == "}" and depth == 0:
                out.append((item_start, i + 1))
                item_start = i + 1
        elif c == ";" and depth == 0:
            out.append((item_start, i + 1))
            item_start = i + 1
        i += 1
    res = []
    for a, e in out:
        texto = b[a:e]
        m = RE_HEAD.match(texto)
        if not m:
            continue
        kind = re.sub(r"\s+", " ", m.group("kind"))
        rest = m.group("rest")
        attrs = m.group("attrs") or ""
        cfg_test = bool(re.search(r"cfg\s*\(\s*test\s*\)", attrs))
        if kind == "impl":
            mm = re.match(r"(?:<[^>]*>\s*)?([A-Za-z_][A-Za-z0-9_]*)", rest)
            name = mm.group(1) if mm else "?"
        elif kind == "use":
            name = ""
        else:
            mm = re.match(r"([A-Za-z_][A-Za-z0-9_]*)", rest)
            name = mm.group(1) if mm else "?"
        # o início REAL do item inclui os doc-comments do ORIGINAL imediatamente acima: recua do
        # primeiro caractere não-branco do texto limpo até à linha a seguir ao item anterior.
        primeiro = a + (len(texto) - len(texto.lstrip()))
        linha_ini = src.rfind("\n", 0, primeiro) + 1
        # sobe por linhas de doc/atributo/comentário contíguas
        k = linha_ini
        while k > a:
            prev_fim = k - 1
            prev_ini = src.rfind("\n", 0, prev_fim) + 1
            if prev_ini < a:
                break
            linha = src[prev_ini:prev_fim].strip()
            if linha.startswith("//") or linha.startswith("#["):
                k = prev_ini
            else:
                break
        res.append((kind, name, m.group("vis") or "", k, e, cfg_test))
    return res


# ---------------------------------------------------------------- a workspace
def git_rs_files(root: str) -> list[str]:
    out = subprocess.run(
        ["git", "ls-files", "*.rs"], cwd=root, capture_output=True, text=True, check=True
    ).stdout.split()
    return [f for f in out if os.path.isfile(os.path.join(root, f))]


def workspace(root: str):
    meta = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=root, capture_output=True, text=True, check=True,
        ).stdout
    )
    pk = {}
    for p in meta["packages"]:
        d = os.path.relpath(os.path.dirname(p["manifest_path"]), root)
        pk[p["name"]] = {
            "dir": d,
            "deps": {(x["name"], x.get("kind")) for x in p["dependencies"]},
        }
    return pk


def especie(nome: str, d: str) -> str:
    """A MESMA classificação do gate `architecture_no_dependency_climbs_a_layer`."""
    if d.startswith("shells/"):
        return "composicao"
    if d.startswith("tools/") or d.startswith("tests/"):
        return "folha"
    if nome.endswith("-registry-init"):
        return "registo"
    if nome.startswith("ph2d-app-"):
        return "folha" if nome == "ph2d-app-host" else "familia"
    if nome.startswith("ph2d-panel-"):
        return "painel"
    if nome.startswith("ph2d-tool-"):
        return "folha" if nome[len("ph2d-tool-"):] in ("registry", "runtime") else "ferramenta"
    return "folha"


def pode_depender_de_painel(sp: str, dev: bool) -> bool:
    return True if dev else sp in ("familia", "composicao", "registo")


def crate_of(f: str, dirs: list[tuple[str, str]]) -> str | None:
    for d, n in dirs:
        if d == "." or f.startswith(d + "/"):
            return n
    return None


RE_TESTFILE = re.compile(r"(^|/)(tests?|benches|examples)/|(^|[/_])tests?\.rs$")


def contexto_teste(f: str) -> bool:
    return bool(RE_TESTFILE.search(f))


def regioes_teste(src_blank: str) -> list[tuple[int, int]]:
    """Os corpos `#[cfg(test)] mod x { … }` inline."""
    out = []
    for m in re.finditer(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*(?:#\s*\[[^\]]*\]\s*)*mod\s+\w+\s*\{", src_blank):
        i = m.end() - 1
        depth = 0
        for j in range(i, len(src_blank)):
            if src_blank[j] == "{":
                depth += 1
            elif src_blank[j] == "}":
                depth -= 1
                if depth == 0:
                    out.append((m.start(), j + 1))
                    break
    return out


def ficheiros_teste_declarados(root: str, files: list[str]) -> set[str]:
    """Os ficheiros que um `#[cfg(test)] mod x;` (com ou sem `#[path]`) declara."""
    out = set()
    for f in files:
        if "/src/" not in f and not f.startswith("src/"):
            continue
        try:
            s = open(os.path.join(root, f), encoding="utf-8").read()
        except OSError:
            continue
        if "cfg(test)" not in s:
            continue
        b = blank(s, keep_strings=True)
        base = os.path.dirname(f)
        stem = os.path.basename(f)[:-3]
        mod_dir = base if stem in ("mod", "lib", "main") else os.path.join(base, stem)
        for m in re.finditer(
            r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*((?:#\s*\[[^\]]*\]\s*)*)(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;",
            b,
        ):
            attrs, name = m.group(1), m.group(2)
            pm = re.search(r'path\s*=\s*"([^"]+)"', attrs)
            if pm:
                cand = [os.path.normpath(os.path.join(base, pm.group(1)))]
            else:
                cand = [os.path.join(mod_dir, name + ".rs"), os.path.join(mod_dir, name, "mod.rs")]
            for c in cand:
                if os.path.isfile(os.path.join(root, c)):
                    out.add(c)
    return out


def na_cerca(f: str) -> bool:
    return any(fnmatch.fnmatch(f, p) for p in CERCA)


# ---------------------------------------------------------------- o censo
def censo(root: str):
    pk = workspace(root)
    dirs = sorted(((v["dir"], k) for k, v in pk.items()), key=lambda x: -len(x[0]))
    files = git_rs_files(root)
    teste_decl = ficheiros_teste_declarados(root, files)

    # 1. os itens de ids/
    itens = {}  # chave → dict
    por_nome = collections.defaultdict(list)
    for f in sorted(x for x in files if x.startswith(IDS_DIR)):
        s = open(os.path.join(root, f), encoding="utf-8").read()
        for kind, name, vis, a, e, cfg_test in items(s):
            if kind == "use":
                continue
            if kind == "mod" and not cfg_test:
                continue  # as declarações `mod x;` do mod.rs
            chave = f"{f}::{name}" if (not vis or kind in ("impl",) or cfg_test) else name
            if kind == "impl":
                chave = f"{f}::impl {name}"
            if cfg_test:
                chave = f"{f}::#[cfg(test)] {name}"
            itens[chave] = {
                "kind": kind, "name": name, "pub": bool(vis) and vis.strip() == "pub",
                "file": f, "start": a, "end": e, "cfg_test": cfg_test,
                "text": s[a:e],
            }
            if name and kind not in ("impl",) and not cfg_test:
                por_nome[name].append(chave)

    pub_names = {it["name"] for it in itens.values() if it["pub"] and it["kind"] not in ("impl", "mod")}

    # 2. arestas dentro de ids/: quem cita quem
    cita = collections.defaultdict(set)  # citante → citados
    for k, it in itens.items():
        toks = set(RE_IDENT.findall(blank(it["text"])))
        for t in toks:
            if t == it["name"] and it["kind"] not in ("impl",) and not it["cfg_test"]:
                continue
            for alvo in por_nome.get(t, []):
                if alvo == k:
                    continue
                a_it = itens[alvo]
                if not a_it["pub"] and a_it["file"] != it["file"]:
                    continue
                cita[k].add(alvo)
        if it["kind"] == "impl":
            for alvo in por_nome.get(it["name"], []):
                cita[k].add(alvo)
                cita[alvo].add(k)  # o impl viaja COM o tipo, nos dois sentidos

    # 3. leitores fora de ids/
    leitores = collections.defaultdict(set)  # nome → {(crate, ctx, ficheiro)}
    reexport = collections.defaultdict(set)  # nome → {crate}
    homonimos = collections.defaultdict(set)
    re_def = re.compile(r"\b(?:const|static|fn|enum|struct|type)\s+([A-Za-z_][A-Za-z0-9_]*)")
    for f in files:
        if f.startswith(IDS_DIR):
            continue
        cr = crate_of(f, dirs)
        if cr is None:
            continue
        try:
            s = open(os.path.join(root, f), encoding="utf-8").read()
        except (OSError, UnicodeDecodeError):
            continue
        b = blank(s)
        if not (set(RE_IDENT.findall(b)) & pub_names):
            continue
        # `pub use` (re-export) não é leitura
        so_uso = b
        exportados = set()
        for m in re.finditer(r"\bpub(?:\s*\([^)]*\))?\s+use\b[^;]*;", b):
            exportados |= set(RE_IDENT.findall(m.group(0)))
            so_uso = so_uso[: m.start()] + " " * (m.end() - m.start()) + so_uso[m.end():]
        for d in re_def.finditer(so_uso):
            if d.group(1) in pub_names:
                homonimos[d.group(1)].add(f)
        ctx_file = "test" if (contexto_teste(f) or f in teste_decl) else "src"
        regs = regioes_teste(so_uso) if ctx_file == "src" else []
        for m in RE_IDENT.finditer(so_uso):
            t = m.group(0)
            if t not in pub_names:
                continue
            ctx = ctx_file
            if ctx == "src" and any(a <= m.start() < e for a, e in regs):
                ctx = "test"
            leitores[t].add((cr, ctx, f))
        for t in exportados & pub_names:
            if not any(x[2] == f for x in leitores[t]):
                reexport[t].add(cr)

    return pk, itens, cita, leitores, reexport, homonimos


#: A lista À MÃO que esta obra substitui por um censo derivado — ela lê cada id pelo nome e, enquanto
#: existir, mantém vivo como «leitor» um id que o produto já não usa (o `INSP_TRANSFORM_SECTION` de
#: 2026-08-21 foi exactamente isso).
IGNORAR_LEITORES = {"crates/ph2d-editor-core/tests/it/node_id_collisions.rs"}


def permitido(de: str, para: str, dev: bool) -> bool:
    """A MESMA lei do gate `architecture_no_dependency_climbs_a_layer`."""
    if para == "familia":
        return de in ("composicao", "registo")
    if para == "painel" and not dev:
        return de in ("familia", "composicao", "registo")
    if para == "ferramenta" and not dev:
        return de in ("painel", "familia", "composicao", "registo")
    return True


def classificar(pk, itens, cita, leitores, homonimos=None):
    """→ {chave: (classe, dono, motivo)}.

    O DONO de um id é a crate MAIS BAIXA, entre os leitores que são PAINEL ou FERRAMENTA, que todo
    outro leitor pode nomear sem subir uma camada — de preferência pelas arestas que já existem.

    Classes: DESCE (para `dono`) · FICA-EC (a fundação lê, no produto ou num teste unitário) ·
    FICA-EC-TESTE (só um teste de integração da fundação lê) · FICA-PAINEIS (≥2 painéis e nenhuma
    ferramenta que os junte) · FICA-CAMADA (um leitor não pode depender de nenhum candidato) ·
    FICA-SEM-DONO (nenhum painel nem ferramenta lê) · FICA-CERCA · FICA-CITACAO · SO-CITADO · ORFAO."""
    homonimos = homonimos or {}
    sp = {n: especie(n, v["dir"]) for n, v in pk.items()}
    deps = {n: v["deps"] for n, v in pk.items()}

    def aresta(r: str, o: str, dev: bool) -> bool:
        d = deps.get(r, set())
        return (o, None) in d or (dev and (o, "dev") in d)

    citado_por = collections.defaultdict(set)
    for a, bs in cita.items():
        for b in bs:
            citado_por[b].add(a)

    base = {}
    for k, it in itens.items():
        if it["cfg_test"]:
            continue
        lei = set()
        if it["pub"]:
            lei = {
                x for x in leitores.get(it["name"], set())
                if x[2] not in IGNORAR_LEITORES and x[2] not in homonimos.get(it["name"], ())
            }
        if any(c == EC and f.startswith("crates/ph2d-editor-core/src/") for c, _, f in lei):
            base[k] = ("FICA-EC", None, "lido pela editor-core (src)")
            continue
        if any(c == EC for c, _, _ in lei):
            base[k] = ("FICA-EC-TESTE", None, ";".join(sorted({f for c, _, f in lei if c == EC})))
            continue
        if not lei:
            base[k] = ("SO-CITADO" if citado_por.get(k) else "ORFAO", None, "")
            continue
        crates = {c for c, _, _ in lei}
        cands = sorted(c for c in crates if sp.get(c) in ("painel", "ferramenta"))
        if not cands:
            base[k] = ("FICA-SEM-DONO", None, ",".join(sorted(crates)))
            continue
        viaveis = []
        for o in cands:
            novas = set()
            ok = True
            for c, ctx, _ in lei:
                if c == o:
                    continue
                dev = ctx == "test"
                if not permitido(sp.get(c, "folha"), sp[o], dev):
                    ok = False
                    break
                if not aresta(c, o, dev):
                    novas.add((c, o, "dev" if dev else "normal"))
            if ok:
                viaveis.append((len(novas), o, novas))
        if not viaveis:
            paineis = [c for c in cands if sp[c] == "painel"]
            cl = "FICA-PAINEIS" if len(paineis) >= 2 and len(paineis) == len(cands) else "FICA-CAMADA"
            base[k] = (cl, None, ",".join(sorted(crates)))
            continue
        viaveis.sort()
        n_novas, o, novas = viaveis[0]
        cerca = sorted({f for _, _, f in lei if na_cerca(f)})
        if cerca:
            base[k] = ("FICA-CERCA", o, ";".join(cerca))
            continue
        base[k] = ("DESCE", o, ";".join(f"{a}->{b}[{t}]" for a, b, t in sorted(novas)))

    # ⛔ A citação DENTRO de ids/ prende por VISIBILIDADE: se `A` cita `B`, o sítio de `A` tem de ver
    # o sítio de `B` — a EC toda a gente vê; uma crate vê-se a si e às suas dependências normais. Um
    # PRIVADO só se vê no mesmo sítio. Quem viola volta à EC (que todos vêem) e o ponto fixo repete.
    def vis(de: str, para: str) -> bool:
        return de == para or para == "EC" or (para, None) in deps.get(de, set())

    def sitio(k: str) -> str:
        cl, dono, _ = res[k]
        return dono if cl == "DESCE" else "EC"

    res = dict(base)
    # um item SÓ citado vai para a crate mais baixa que todos os citantes vêem (se não for a EC)
    mudou = True
    while mudou:
        mudou = False
        for k, (cl, dono, _) in list(res.items()):
            if cl != "SO-CITADO":
                continue
            cit = [c for c in citado_por.get(k, ()) if c in res and not itens[c]["cfg_test"]]
            if any(res[c][0] == "SO-CITADO" for c in cit):
                continue  # espera o citante resolver-se
            sitios = {sitio(c) for c in cit}
            alvo = None
            for cand in sorted(sitios - {"EC"}):
                if all(vis(s, cand) for s in sitios):
                    alvo = cand
                    break
            if alvo and (itens[k]["pub"] or len(sitios) == 1):
                res[k] = ("DESCE", alvo, "herdado dos citantes")
            else:
                res[k] = ("FICA-CITACAO", None, f"citado de {sorted(sitios)}")
            mudou = True
    for k, (cl, _, _) in list(res.items()):
        if cl == "SO-CITADO":  # ciclo de só-citados: ficam juntos na EC
            res[k] = ("FICA-CITACAO", None, "ciclo de itens só citados")
    mudou = True
    while mudou:
        mudou = False
        for a, alvos in cita.items():
            if a not in res or itens[a]["cfg_test"]:
                continue
            for b in alvos:
                if b not in res or itens[b]["cfg_test"]:
                    continue
                sa, sb = sitio(a), sitio(b)
                privado = not itens[b]["pub"]
                if (privado and sa != sb) or not vis(sa, sb):
                    if sb != "EC":
                        # ⭐ antes de devolver `b` à EC, tente o sítio de QUEM o cita: se todo leitor e
                        # todo outro citante de `b` vê `sa`, `b` desce com a tabela (a tabela na
                        # ferramenta, o elemento que só o painel lê — o painel vê a ferramenta).
                        leitores_b = {c for c, _, f in leitores.get(itens[b]["name"], set())
                                      if f not in IGNORAR_LEITORES} if itens[b]["pub"] else set()
                        outros = [sitio(c) for c in citado_por.get(b, ()) if c in res and not itens[c]["cfg_test"]]
                        if sa != "EC" and all(vis(r, sa) for r in leitores_b) and all(vis(s, sa) for s in outros) \
                                and not (privado and any(s != sa for s in outros)):
                            res[b] = ("DESCE", sa, f"desce com o citante {a}")
                        else:
                            res[b] = ("FICA-CITACAO", None, f"citado por {a} ({sa}), que não vê {sb}")
                    else:
                        res[a] = ("FICA-CITACAO", None, f"cita o privado {b}, que fica")
                    mudou = True
    # ⭐ O ASSUNTO desce inteiro: um ficheiro de ids/ é um assunto (a secção de um painel), e os itens
    # dele que descem vão juntos para a crate mais baixa que TODO leitor de TODOS eles vê — em vez de
    # se espalharem por leitor. Medido 2026-09-12: sem isto, os 16 ficheiros `painter_*` partiam-se
    # item a item entre a ferramenta e o painel de camadas; com isto descem inteiros para a
    # `ph2d-tool-painter`, que é o precedente escrito da casa (a `ph2d-tool-color-equalization` e a
    # `ph2d-tool-equalize-sizes` já são donas dos ids dos seus painéis, ADR-0040 §3.8).
    por_ficheiro = collections.defaultdict(list)
    for k, (cl, dono, _) in res.items():
        if cl == "DESCE":
            por_ficheiro[itens[k]["file"]].append(k)
    for f, ks in por_ficheiro.items():
        donos = sorted({res[k][1] for k in ks})
        if len(donos) < 2:
            continue
        for o in donos:
            ok = True
            for k in ks:
                it = itens[k]
                for c, ctx, fl in (leitores.get(it["name"], set()) if it["pub"] else set()):
                    if fl in IGNORAR_LEITORES or c == EC:
                        continue
                    if not (vis(c, o) or (ctx == "test" and (o, "dev") in deps.get(c, set()))):
                        ok = False
                        break
                if not ok:
                    break
            if ok:
                for k in ks:
                    if res[k][1] != o:
                        res[k] = ("DESCE", o, f"o assunto ({f.rsplit('/', 1)[-1]}) desce inteiro")
                break
    # a visibilidade outra vez (o reagrupamento só junta, mas uma citação de fora do ficheiro pode
    # deixar de ver o sítio novo)
    mudou = True
    while mudou:
        mudou = False
        for a, alvos in cita.items():
            if a not in res or itens[a]["cfg_test"]:
                continue
            for b in alvos:
                if b not in res or itens[b]["cfg_test"]:
                    continue
                sa, sb = sitio(a), sitio(b)
                if (not itens[b]["pub"] and sa != sb) or not vis(sa, sb):
                    if sb != "EC":
                        res[b] = ("FICA-CITACAO", None, f"citado por {a} ({sa}), que não vê {sb} (depois do reagrupamento)")
                    else:
                        res[a] = ("FICA-CITACAO", None, f"cita o privado {b}, que fica")
                    mudou = True
    # os `#[cfg(test)] mod` de ids/: vão com os itens que citam — para o sítio que vê todos os outros
    for k, it in itens.items():
        if not it["cfg_test"]:
            continue
        donos = {res[a][1] for a in cita.get(k, ()) if a in res and res[a][0] == "DESCE"}
        alvo = next((o for o in sorted(donos) if all(vis(o, p) for p in donos)), None)
        if not donos:
            res[k] = ("TESTE-FICA", None, "")
        elif alvo:
            res[k] = ("TESTE-VAI", alvo, "")
        else:
            res[k] = ("TESTE-CONFLITO", None, str(sorted(donos)))
    return res


# ---------------------------------------------------------------- autoteste
def autoteste() -> int:
    casos = []
    s = 'pub const A: NodeId = hash_node_id("a"); // B\n/// doc\npub fn b_id(i: usize) -> NodeId { A }\n'
    it = items(s)
    casos.append(("dois itens de topo", [x[1] for x in it] == ["A", "b_id"]))
    casos.append(("o doc-comment viaja com o item", s[it[1][3]:it[1][4]].startswith("/// doc")))
    casos.append(("o texto limpo come a prosa", "B" not in RE_IDENT.findall(blank(s))))
    casos.append(("o texto limpo come a string", "a" not in RE_IDENT.findall(blank('x("a")'))))
    t = "pub const X: [NodeId; 2] = [A, B];\n#[cfg(test)]\nmod tests { use super::*; #[test] fn t() { let _ = A; } }\n"
    it = items(t)
    casos.append(("o `;` dentro de `[T; N]` não fecha o item", [x[1] for x in it] == ["X", "tests"]))
    casos.append(("o mod de teste é reconhecido", it[1][5] is True))
    casos.append(("lifetime não é char", "a" in RE_IDENT.findall(blank("fn f<'a>(x: &'a str) {}"))))
    casos.append(("raw string sai", "zz" not in RE_IDENT.findall(blank('let s = r#"zz"#;'))))
    b = blank('pub use ph2d_editor_core::ids::{A, B};\nfn f() { B }')
    casos.append(("regiões de teste", regioes_teste(blank("#[cfg(test)]\nmod tests { fn x() {} }")) != []))
    casos.append(("pub use detectável", bool(re.search(r"\bpub(?:\s*\([^)]*\))?\s+use\b[^;]*;", b))))
    casos.append(("espécie de painel", especie("ph2d-panel-vector", "crates/ph2d-panel-vector") == "painel"))
    casos.append(("contrato é folha", especie("ph2d-tool-registry", "crates/ph2d-tool-registry") == "folha"))
    casos.append(("ficheiro de teste", contexto_teste("crates/x/src/foo_tests.rs") and contexto_teste("crates/x/tests/it/a.rs")))
    casos.append(("ficheiro de produto", not contexto_teste("crates/x/src/tester.rs")))
    ok = True
    for nome, v in casos:
        print(("✓ " if v else "✗ ") + nome)
        ok &= v
    return 0 if ok else 1


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json")
    ap.add_argument("--lista")
    ap.add_argument("--autoteste", action="store_true")
    ap.add_argument("--root", default=".")
    a = ap.parse_args()
    if a.autoteste:
        return autoteste()
    root = os.path.abspath(a.root)
    pk, itens, cita, leitores, reexport, homonimos = censo(root)
    res = classificar(pk, itens, cita, leitores, homonimos)

    publicos = [k for k, it in itens.items() if it["pub"] and it["kind"] not in ("impl", "mod")]
    nodeid = [k for k in publicos if re.search(r"\bNodeId\b", itens[k]["text"].split("=")[0]) or itens[k]["kind"] in ("fn", "const fn")]
    print(f"itens de topo em ids/: {len(itens)}  (públicos: {len(publicos)}; com NodeId na assinatura: {len(nodeid)})")
    cont = collections.Counter(res[k][0] for k in itens)
    cont_pub = collections.Counter(res[k][0] for k in publicos)
    print(f"\n{'classe':<18}{'itens':>7}{'públicos':>10}")
    for cl in sorted(cont):
        print(f"{cl:<18}{cont[cl]:>7}{cont_pub.get(cl, 0):>10}")
    por_painel = collections.Counter(res[k][1] for k in publicos if res[k][0] == "DESCE")
    print("\nDESCE, por painel (públicos):")
    for p, n in por_painel.most_common():
        print(f"  {p:<34}{n:>5}")
    cerca = [k for k in publicos if res[k][0] == "FICA-CERCA"]
    print(f"\nFICA-CERCA: {len(cerca)} (dono seria: {dict(collections.Counter(res[k][1] for k in cerca))})")
    por_nome = {n: sorted(fs) for n, fs in homonimos.items()}
    print(f"homónimos (nome de ids/ definido fora): {len(por_nome)}")
    for n, fs in sorted(por_nome.items())[:40]:
        print(f"  {n}: {fs[:3]}")
    if a.lista:
        for k in sorted(itens):
            if res[k][0] == a.lista:
                print(f"{k}\t{res[k][1]}\t{res[k][2]}")
    if a.json:
        dump = {
            k: {
                "classe": res[k][0], "dono": res[k][1], "motivo": res[k][2],
                "kind": it["kind"], "name": it["name"], "pub": it["pub"], "file": it["file"],
                "start": it["start"], "end": it["end"], "cfg_test": it["cfg_test"],
                "cita": sorted(cita.get(k, ())),
                "leitores": sorted({f"{c}|{ctx}|{f}" for c, ctx, f in leitores.get(it["name"], set())}) if it["pub"] else [],
                "reexport": sorted(reexport.get(it["name"], set())),
            }
            for k, it in itens.items()
        }
        with open(a.json, "w", encoding="utf-8") as fh:
            json.dump(dump, fh, indent=1, ensure_ascii=False)
    return 0


if __name__ == "__main__":
    sys.exit(main())
