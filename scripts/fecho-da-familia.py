#!/usr/bin/env python3
"""**O FECHO de uma família da shell** — a régua que a W2 pede e que não existia.

    python3 scripts/fecho-da-familia.py motion
    python3 scripts/fecho-da-familia.py vec --extra 'warp_*'
    python3 scripts/fecho-da-familia.py --autoteste

# A pergunta

⛔ **Contar citações mede menos do que o fecho, e erra A FAVOR** (HOWTO §2.12; a
`line/app-motion` pagou-a a **30×**: mediu `46 757` LOC movíveis onde a verdade era `1 573`).
A pergunta que responde é:

> *A partir dos ficheiros que quero mover, que raízes da SHELL continuam alcançáveis?*

# As QUATRO formas de a régua mentir, todas medidas neste repo

1. ⛔⛔ **Branquear strings apaga o grafo de `#[path]`.** Um stripper que troca `"x"` por `""`
   deixa `#[path = "motion_state_demo_router.rs"]` ilegível — medido: `motion_state.rs` tem
   **45** `#[path]` e a regex apanha **0**, logo o ponto fixo corre com **zero arestas duras** e
   devolve um número bonito. A `line/app-vec` pagou o mesmo (`CLAUDE.md` §5: *«branquear strings
   apaga o grafo de `#[path]`, que tem 801 arestas»*).
   ⇒ **quem mede ARESTAS lê por [`sem_comentarios`]; quem conta CITAÇÕES lê por [`limpo`].**
2. ⛔⛔ **Ler PROSA como código** (HOWTO §2.12): um censo a varrer `\bApp\b` acusou **93 de 133**
   ficheiros, todos falsos — casava com o doc-comment que EXPLICA a cura.
3. ⛔ **Ler VISIBILIDADE como dependência**: `pub(in crate::render_loop)` não é uma citação de
   `render_loop`; é o âmbito do item. Sem este filtro, `motion_bridge_tutorial_draw.rs` acusa
   **7** arestas para o laço da shell e não tem nenhuma. *É a §2.13 noutra roupa — visibilidade é
   exactamente o que uma fronteira nova muda por construção.*
4. ⛔⛔ **O acoplamento que viaja por um CAMPO não tem nome de módulo nenhum.** As cenas do Motion
   alcançam o estado por `app.gfx.motion`; nenhuma varredura por `crate::` o vê. Medido em 12/09:
   `app.flip_state` (tipo `crate::flip::state::FlipState`) é uma âncora **inteira** que só a
   travessia por campo encontra.
   ⇒ esta régua resolve cada campo de `App`/`AppGfx` ao **tipo** dele e classifica-o.

⚠️ Um alias `pub(crate) use ph2d_app_vec::glyph as vec_glyph;` em `main.rs` **não** é a shell — é
uma crate com nome velho. Sem os reconhecer, 16 falsas âncoras aparecem (medido).

# O que ela imprime

- **ÂNCORAS** — os ficheiros da shell que ficam alcançáveis (a lista a curar);
- **o PONTO FIXO** — quanto move de facto, dado que umas âncoras não se podem curar (`--preso`);
- **a CADEIA CAUSAL** — porque cada ficheiro caiu, e a causa RAIZ por contagem.
"""

from __future__ import annotations

import argparse
import os
import re
import sys

SRC = "shells/desktop/src"


# ---------------------------------------------------------------- strippers
def limpo(src: str) -> str:
    """Sem comentários **e sem strings** — para contar CITAÇÕES."""
    out, i, n = [], 0, len(src)
    while i < n:
        c = src[i]
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            while i < n and src[i] != "\n":
                i += 1
            continue
        if c == "/" and i + 1 < n and src[i + 1] == "*":
            depth, i = 1, i + 2
            while i < n and depth:
                if src[i] == "/" and i + 1 < n and src[i + 1] == "*":
                    depth, i = depth + 1, i + 2
                elif src[i] == "*" and i + 1 < n and src[i + 1] == "/":
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
            out.append(" ")
            continue
        if c == "r" and i + 1 < n and src[i + 1] in '"#':
            j, h = i + 1, 0
            while j < n and src[j] == "#":
                h, j = h + 1, j + 1
            if j < n and src[j] == '"':
                fim = '"' + "#" * h
                k = src.find(fim, j + 1)
                i = n if k < 0 else k + len(fim)
                out.append('""')
                continue
        if c == '"':
            i += 1
            while i < n:
                if src[i] == "\\":
                    i += 2
                elif src[i] == '"':
                    i += 1
                    break
                else:
                    i += 1
            out.append('""')
            continue
        if c == "'":
            if i + 2 < n and src[i + 1] == "\\":
                j = i + 2
                while j < n and src[j] != "'":
                    j += 1
                i = j + 1
                out.append("''")
                continue
            if i + 2 < n and src[i + 2] == "'":
                i += 3
                out.append("''")
                continue
        out.append(c)
        i += 1
    return "".join(out)


def sem_comentarios(src: str) -> str:
    """Sem comentários, **com** as strings — para ler ARESTAS (`#[path = "x.rs"]`)."""
    out, i, n = [], 0, len(src)
    while i < n:
        c = src[i]
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            while i < n and src[i] != "\n":
                i += 1
            continue
        if c == "/" and i + 1 < n and src[i + 1] == "*":
            depth, i = 1, i + 2
            while i < n and depth:
                if src[i] == "/" and i + 1 < n and src[i + 1] == "*":
                    depth, i = depth + 1, i + 2
                elif src[i] == "*" and i + 1 < n and src[i + 1] == "/":
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
            out.append(" ")
            continue
        if c == '"':
            j = i + 1
            while j < n:
                if src[j] == "\\":
                    j += 2
                elif src[j] == '"':
                    j += 1
                    break
                else:
                    j += 1
            out.append(src[i:j])
            i = j
            continue
        out.append(c)
        i += 1
    return "".join(out)


RE_VIS = re.compile(r"\bpub\s*\(\s*in\s+crate::[a-z_0-9:]+\s*\)")
RE_P2 = re.compile(r"\bcrate::([a-z_][a-z_0-9]*)::([a-zA-Z_][A-Za-z_0-9]*)")
RE_P1 = re.compile(r"\bcrate::([A-Za-z_][A-Za-z_0-9]*)\b")
RE_PATH = re.compile(
    r'#\[path\s*=\s*"([^"]+)"\]\s*(?:pub(?:\([a-z: ]+\))?\s+)?mod\s+[a-z_0-9]+\s*;'
)
RE_ALIAS = re.compile(
    r"use\s+(ph2d_[a-z_0-9]+)::([a-zA-Z_0-9:]*)\s+as\s+([a-z_0-9]+)\s*;"
)
RE_CAMPO = re.compile(r"^\s+pub\(crate\)\s+([a-z_0-9]+)\s*:\s*([^,]+),", re.M)


# ---------------------------------------------------------------- autoteste
def autoteste() -> int:
    """⚠️ Os controlos POSITIVOS: uma régua sem controlo mede o próprio buraco."""
    casos = []

    c = limpo('let x = 1; // crate::fantasma::a\n/* crate::outro */ crate::real::b();')
    casos.append(("limpo ve o codigo", "crate::real" in c))
    casos.append(("limpo come a prosa", "fantasma" not in c and "outro" not in c))

    # ⛔ o defeito nº1: `limpo` APAGA o grafo de #[path]
    p = '#[path = "filho.rs"]\n/// doc\npub(crate) mod filho;'
    casos.append(("limpo APAGA a aresta (e por isso nao se usa aqui)",
                  RE_PATH.findall(limpo(p)) == []))
    casos.append(("sem_comentarios VE a aresta",
                  RE_PATH.findall(sem_comentarios(p)) == ["filho.rs"]))
    casos.append(("um #[path] COMENTADO nao conta",
                  RE_PATH.findall(sem_comentarios('// #[path = "f.rs"] mod f;')) == []))

    v = RE_VIS.sub("pub", "pub(in crate::render_loop) fn f() { crate::render_loop::x(); }")
    casos.append(("visibilidade nao e citacao", v.count("crate::render_loop") == 1))

    mau = 0
    for nome, ok in casos:
        print(f"  {'OK  ' if ok else 'FALHA'}  {nome}")
        mau += not ok
    print(f"\n{len(casos) - mau}/{len(casos)} controlos passam")
    return 1 if mau else 0


# ---------------------------------------------------------------- a medicao
def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("familia", nargs="?", help="p.ex. motion, vec, flip")
    ap.add_argument("--extra", action="append", default=[],
                    help="prefixo de ficheiro que TAMBEM e' da familia (p.ex. 'warp_')")
    ap.add_argument("--preso", action="append", default=[],
                    help="ancora que NAO se pode curar (arvore de outra linha)")
    ap.add_argument("--curavel", action="append", default=[],
                    help="ancora que ESTA linha cura")
    ap.add_argument("--autoteste", action="store_true")
    a = ap.parse_args()

    if a.autoteste:
        return autoteste()
    if not a.familia:
        ap.error("diga a familia, ou --autoteste")
    if not os.path.isdir(SRC):
        print(f"⛔ corra da RAIZ da arvore (nao existe {SRC}/)", file=sys.stderr)
        return 2

    fam = a.familia
    todos = sorted(
        os.path.relpath(os.path.join(dp, n), SRC)
        for dp, _, ns in os.walk(SRC)
        for n in ns
        if n.endswith(".rs")
    )
    existe = set(todos)

    def e_da_familia(p: str) -> bool:
        b = os.path.basename(p)
        if p.startswith(f"{fam}/") or p == f"{fam}.rs":
            return True
        if p.startswith("render_loop/") and b.startswith(f"{fam}_"):
            return True
        return any(b.startswith(e.rstrip("*")) for e in a.extra)

    S = {p for p in todos if e_da_familia(p)}
    if not S:
        print(f"⛔ zero ficheiros para a familia «{fam}» — o censo perdeu o sujeito", file=sys.stderr)
        return 2

    cru, lim = {}, {}
    for p in todos:
        raw = open(os.path.join(SRC, p), encoding="utf-8", errors="replace").read()
        cru[p] = sem_comentarios(raw)
        lim[p] = RE_VIS.sub("pub", limpo(raw))

    main_rs = lim["main.rs"]
    mods_topo = set(re.findall(
        r"^\s*(?:pub(?:\([a-z: ]+\))?\s+)?mod\s+([a-z_0-9]+)\s*;", main_rs, re.M))
    alias = {v: f"{k}::{seg}" for k, seg, v in RE_ALIAS.findall(main_rs)}

    def resolve(s1: str, s2: str) -> str | None:
        for c in (f"{s1}/{s2}.rs", f"{s1}/{s2}/mod.rs", f"{s1}.rs", f"{s1}/mod.rs"):
            if c in existe:
                return c
        return None

    # ---- os campos de `App`/`AppGfx` que a familia toca, resolvidos ao TIPO
    tipos: dict[str, str] = {}
    # ⚠️ O `AppGfx`, os tipos das ferramentas de imagem e o `HeroLive` mudaram-se para irmãos do
    # `app_state.rs` (`line/loc-caps`, 2026-09-13): sem os ler, todo `gfx.<campo>` deixava de
    # resolver ao tipo e a régua errava A FAVOR. ⚠️ E lêem-se pela ORDEM que tinham no ficheiro
    # único: o `setdefault` fica com a PRIMEIRA declaração de um nome, e `physics` existe nas duas
    # structs (`AppGfx` antes da `App`) — outra ordem trocava-lhe o tipo em silêncio.
    campos_src = "\n".join(
        lim.get(f, "")
        for f in (
            "app_state_gfx.rs",
            "app_state_image_tools.rs",
            "app_state_hero_live.rs",
            "app_state.rs",
        )
    )
    for m in RE_CAMPO.finditer(campos_src):
        tipos.setdefault(m.group(1), m.group(2).strip())
    RE_FD = re.compile(r"\b(?:app|self|gfx)\s*\.\s*([a-z_0-9]+)")
    campos: dict[str, set[str]] = {}
    for p in S:
        for m in RE_FD.finditer(lim[p]):
            if m.group(1) in tipos:
                campos.setdefault(m.group(1), set()).add(p)

    # ---- as ancoras
    ancoras: dict[str, dict[str, int]] = {}

    # ⚠️ `ancoras` guarda só o que sai de S; `nomeia` guarda TUDO o que cada ficheiro
    # resolve — incluindo dentro de S. O ponto fixo precisa das duas: um ficheiro de S
    # que fique preso arrasta quem o nomeia, e essa aresta não é uma âncora.
    nomeia: dict[str, set[str]] = {}

    def anota(alvo: str, p: str) -> None:
        nomeia.setdefault(p, set()).add(alvo)
        if alvo in S or alvo == "::APP::":
            return
        ancoras.setdefault(alvo, {}).setdefault(p, 0)
        ancoras[alvo][p] += 1

    for p in sorted(S):
        t, vistos = lim[p], []
        for m in RE_P2.finditer(t):
            vistos.append(m.span())
            s1, s2 = m.group(1), m.group(2)
            if s1 in alias:
                continue
            alvo = resolve(s1, s2)
            if alvo and alvo != p:
                anota(alvo, p)
        for m in RE_P1.finditer(t):
            if any(x <= m.start() < y for x, y in vistos):
                continue
            n = m.group(1)
            if n in alias:
                continue
            if n in mods_topo:
                alvo = resolve(n, "\0")
                if alvo and alvo != p:
                    anota(alvo, p)
            elif n in ("App", "AppGfx"):
                anota("::APP::", p)
    # o acoplamento por CAMPO: o TIPO e' que prende
    for c, fs in campos.items():
        ty = tipos[c]
        m = re.search(r"crate::([a-z_0-9]+)::([a-zA-Z_0-9]+)", ty)
        if m:
            alvo = resolve(m.group(1), m.group(2))
            if alvo:
                for p in fs:
                    if alvo != p:
                        anota(alvo, p)

    # ---- as arestas DURAS (`#[path]`); a declaracao num ficheiro-RAIZ re-aloja-se
    raizes = {f"{fam}.rs", "render_loop/mod.rs", "main.rs"}
    dur: dict[str, set[str]] = {}
    for p in todos:
        if p in raizes:
            continue
        d = os.path.dirname(p)
        for m in RE_PATH.finditer(cru[p]):
            c = os.path.normpath(os.path.join(d, m.group(1)))
            if c in existe:
                dur.setdefault(p, set()).add(c)
                dur.setdefault(c, set()).add(p)

    def loc(ps) -> int:
        return sum(len(open(os.path.join(SRC, p), encoding="utf-8",
                            errors="replace").read().splitlines()) for p in ps)

    print(f"família «{fam}»:  S = {len(S)} ficheiros / {loc(S)} LOC")
    print(f"resto da shell:   {len(todos) - len(S)} ficheiros")
    print(f"alias-para-crate reconhecidos em main.rs: {len(alias)}")
    print()
    print(f"=== ÂNCORAS — raízes da shell alcançáveis a partir de S: {len(ancoras)} ===")
    for alvo in sorted(ancoras, key=lambda k: -sum(ancoras[k].values())):
        f = ancoras[alvo]
        n = "(crate::App / crate::AppGfx)" if alvo == "::APP::" else alvo
        tam = "" if alvo == "::APP::" else f"{loc([alvo]):6d} LOC"
        print(f"  {n:44s} {tam:>10s}  {sum(f.values()):4d} usos / {len(f):3d} ficheiros")
    print()
    assert tipos, ("⛔ o censo de campos leu ZERO campos de app_state.rs — "
                   "perdeu o sujeito (HOWTO §2.7: um censo sem piso fica verde a varrer nada)")
    print("=== os campos de App/AppGfx que a família toca, por TIPO ===")
    print("    (⛔ SHELL = âncora que NENHUMA varredura por `crate::` vê)")
    for c in sorted(campos, key=lambda k: -len(campos[k])):
        ty = tipos[c]
        dono = ("⛔ SHELL" if ty.startswith("crate::") and not ty.startswith(f"crate::{fam}")
                else "⭐ da família" if ty.startswith(f"crate::{fam}") else "✓ crate irmã")
        print(f"  .{c:20s} {len(campos[c]):3d} f   {ty[:44]:44s} {dono}")

    # ---- o ponto fixo
    presos_cfg = set(a.preso)
    curaveis = set(a.curavel)
    movivel, porque, ronda = set(S), {}, 0
    while True:
        ronda += 1
        fora = {}
        for p in movivel:
            causa = None
            for alvo in nomeia.get(p, ()):
                if alvo == "::APP::":
                    continue
                if alvo in presos_cfg:
                    causa = ("nomeia PRESO", alvo)
                    break
                if alvo not in movivel and alvo not in curaveis:
                    causa = ("nomeia", alvo)
                    break
            if causa is None:
                for alvo in dur.get(p, ()):
                    if alvo not in movivel and alvo not in curaveis:
                        causa = ("#[path] com", alvo)
                        break
            if causa:
                fora[p] = (ronda,) + causa
        if not fora:
            break
        porque.update(fora)
        movivel -= set(fora)

    print()
    print(f"=== PONTO FIXO ({ronda} rondas) ===")
    print(f"  MOVE   {len(movivel):4d} ficheiros / {loc(movivel):6d} LOC")
    print(f"  FICA   {len(S - movivel):4d} ficheiros / {loc(S - movivel):6d} LOC")
    if porque:
        from collections import Counter

        raiz = Counter((t, alvo) for _, t, alvo in porque.values())
        print("\n  cadeia causal (as 2 primeiras rondas):")
        for p, (r, t, alvo) in sorted(porque.items(), key=lambda kv: kv[1][0]):
            if r <= 2:
                print(f"    r{r}  {p:54s} {t} {alvo}")
        print("\n  causa RAIZ, por contagem:")
        for (t, alvo), n in raiz.most_common(8):
            print(f"    {n:4d} ficheiros   {t} {alvo}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
