#!/usr/bin/env python3
"""**A MUDANÇA DE CASA dos ids de widget** — lê o censo (`scripts/censo-ids.py --json`) e muda cada
item que DESCE para a crate dona; mata as re-exportações dos painéis; reescreve todo leitor.

    python3 scripts/censo-ids.py --json target/prova/censo.json      # SEMPRE antes (offsets frescos)
    python3 scripts/mover-ids.py target/prova/censo.json --plano
    python3 scripts/mover-ids.py target/prova/censo.json --aplicar [--dono ph2d-tool-painter …] [--fachadas]

# O que ele faz

1. **Definições** — os itens de um ficheiro de `ph2d-editor-core/src/ids/` que descem para `O`
   viram `crates/O/src/ids/<assunto>.rs`, com o texto **verbatim** (o valor de um id é o slug, e o
   slug não se toca) e os imports GERADOS dos tokens de código. Os caminhos `super::`/`crate::`
   DENTRO do texto movido são reescritos para o sítio novo. Um import que só um `#[cfg(test)] mod`
   usa vai para DENTRO dele.
2. **Fachadas** (`--fachadas`) — um `pub use <outro módulo de ids>::…` no `ids.rs` de um painel sai
   (⛔ zero fachadas, auditoria A5b); e uma fachada de um nome que MUDOU de casa sai sempre (uma
   fachada não se redirecciona: seria a mesma aresta com outro nome — e na fundação, uma que sobe).
3. **Leitores** — em todo `.rs`, as ligações `use … ids` resolvem-se POR FICHEIRO, **herdando** as do
   módulo pai quando o ficheiro abre com `use super::*;` (o prelúdio `pub(crate) use …::ids;` do
   `sections/mod.rs` do Inspector serve 40 ficheiros assim), e cada `<módulo>::NOME` cujo módulo
   deixou de exportar o nome passa a `<módulo dono>::NOME`. `crate::ids` tem um ANTES e um DEPOIS
   numa crate que ganha o seu próprio módulo de ids. Só em CÓDIGO: comentários e strings não se
   tocam (HOWTO §2.12). Em `tests/`, `benches/` e `examples/` a `crate` é o binário de teste: o
   caminho é sempre o externo.
4. **Cargo** — acrescenta a dependência que o caminho novo exige (normal no produto, dev num
   teste), e só se a lei de camadas a permite; senão ABORTA **antes de escrever um byte**.

⛔ Ele não decide donos (o censo decide) e não formata (`cargo fmt` a seguir). O compilador é o
oráculo do resto: um caminho que ele não soube reescrever falha ALTO.
"""

from __future__ import annotations

import argparse
import collections
import importlib.util
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("censo", os.path.join(HERE, "censo-ids.py"))
censo = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(censo)

EC = "ph2d-editor-core"
IDS = "crates/ph2d-editor-core/src/ids/"
GS = "crates/ph2d-editor-core/src/grid_snap/ids.rs"
DATA = "2026-09-13"
RE_USE = re.compile(r"\b(pub(?:\s*\(([^)]*)\))?\s+)?use\s+([^;]+);")
IDENT = r"[A-Za-z_][A-Za-z0-9_]*"


def snake(c: str) -> str:
    return c.replace("-", "_")


def externo(f: str) -> bool:
    """Em `tests/`, `benches/`, `examples/` e `build.rs` a `crate` NÃO é a biblioteca."""
    # ⚠️ `src/tool/paint/tests/*.rs` do Painter é módulo da BIBLIOTECA: só conta o `tests/` fora de `src/`
    return ("/src/" not in f and bool(re.search(r"(^|/)(tests|benches|examples)/", f))) or f.endswith("/build.rs")


# ────────────────────────────────────────────────────────────── escrita TRANSACCIONAL
class Palco:
    """Tudo em memória; `grava()` só no fim — um `assert` a meio não deixa a árvore partida."""

    def __init__(self, root):
        self.root = root
        self.novo = {}
        self.apagar = set()

    def le(self, f):
        if f in self.novo:
            return self.novo[f]
        if f in self.apagar:
            return None
        p = os.path.join(self.root, f)
        return open(p, encoding="utf-8").read() if os.path.isfile(p) else None

    def escreve(self, f, s):
        self.apagar.discard(f)
        self.novo[f] = s

    def apaga(self, f):
        self.novo.pop(f, None)
        self.apagar.add(f)

    def existe(self, f):
        return self.le(f) is not None

    def grava(self):
        for f in sorted(self.apagar):
            p = os.path.join(self.root, f)
            if os.path.isfile(p):
                os.remove(p)
        for f, s in sorted(self.novo.items()):
            p = os.path.join(self.root, f)
            os.makedirs(os.path.dirname(p), exist_ok=True)
            open(p, "w", encoding="utf-8").write(s)


# ────────────────────────────────────────────────────────────── o modelo de nomes
class Mundo:
    """Que módulo de ids tem que nome, ANTES e DEPOIS."""

    def __init__(self, palco: Palco, pk, cj, donos, fachadas):
        self.p = palco
        self.pk = pk
        self.c = cj
        self.fachadas = fachadas
        self.move = {
            k: v["dono"]
            for k, v in cj.items()
            if v["classe"] in ("DESCE", "TESTE-VAI") and v["dono"] and (not donos or v["dono"] in donos)
        }
        self.donos_novos = set(self.move.values())
        # crates cuja RAIZ liga `ids` à fundação (`use ph2d_editor_core::ids;` no lib.rs): nelas,
        # `crate::ids` é a fundação ANTES e a própria crate DEPOIS.
        self.raiz_ec = set()
        self.mods = {"EC": (EC, "ph2d_editor_core::ids", None), "EC_GS": (EC, "ph2d_editor_core::grid_snap::ids", GS)}
        for c, v in pk.items():
            if c.startswith("ph2d-panel-") or c.startswith("ph2d-tool-"):
                raiz = os.path.join(v["dir"], "src/ids.rs")
                if palco.existe(raiz) or c in self.donos_novos:
                    self.mods[c] = (c, f"{snake(c)}::ids", raiz)
                lib = palco.le(os.path.join(v["dir"], "src/lib.rs")) or ""
                if not palco.existe(raiz) and c in self.donos_novos and re.search(r"(?m)^use ph2d_editor_core::ids;\s*$", lib):
                    self.raiz_ec.add(c)
        self.mods["UPS_TOOL"] = ("ph2d-tool-upscale", "ph2d_tool_upscale::tool::ids", None)
        self.antes = {}
        self.ec_nomes = set()
        for k, v in cj.items():
            if v["pub"] and v["kind"] not in ("impl", "mod") and not v["cfg_test"]:
                self.antes[v["name"]] = "EC"
                self.ec_nomes.add(v["name"])
        self.reexp = collections.defaultdict(dict)
        self.glob = collections.defaultdict(list)
        for mid, (crate, _, raiz) in self.mods.items():
            if raiz and palco.existe(raiz):
                self._le_raiz(mid, crate, raiz)
        up = os.path.join(pk["ph2d-tool-upscale"]["dir"], "src/tool.rs")
        s = palco.le(up) or ""
        i = censo.blank(s).find("pub mod ids")
        if i >= 0:
            for m in re.finditer(r"\bpub\s+const\s+(" + IDENT + ")", censo.blank(s)[i:]):
                self.antes.setdefault(m.group(1), "UPS_TOOL")
        self.depois = dict(self.antes)
        for k, dono in self.move.items():
            v = cj[k]
            if v["pub"] and v["kind"] not in ("impl", "mod") and not v["cfg_test"]:
                self.depois[v["name"]] = dono

    def _le_raiz(self, mid, crate, raiz):
        b = censo.blank(self.p.le(raiz))
        for m in re.finditer(r"\bpub\s+(?:const|static|fn|enum|struct|type)\s+(" + IDENT + ")", b):
            self.antes.setdefault(m.group(1), mid)
        for m in RE_USE.finditer(b):
            if not m.group(1) or (m.group(2) or "").strip():
                continue
            for mp, nome, al in parse_use(m.group(3)):
                fonte = self.mid(mp, crate, antes=True)
                if not fonte:
                    continue
                if nome == "*":
                    self.glob[mid].append(fonte)
                else:
                    self.reexp[mid][al or nome] = fonte

    def mid(self, caminho: str, de_crate: str, antes: bool, f: str = ""):
        caminho = caminho.strip()
        if f and caminho in ("super::ids", "super::super::ids"):
            pai = ficheiro_pai(self.p, f)
            if pai and caminho.startswith("super::super::"):
                pai = ficheiro_pai(self.p, pai)
            return self.ids_de(pai, de_crate, antes) if pai else None
        if f and externo(f) and caminho.startswith("crate::"):
            return None
        if de_crate == EC and caminho in ("crate::ids", "crate::screens::hero::ids"):
            return "EC"
        if de_crate == EC and caminho == "crate::grid_snap::ids":
            return "EC_GS"
        if caminho == "crate::ids" and de_crate in self.mods:
            return "EC" if (antes and de_crate in self.raiz_ec) else de_crate
        if caminho == "ph2d_editor_core::screens::hero::ids":
            return "EC"
        for m, (_, ext, _) in self.mods.items():
            if caminho == ext:
                return m
        return None

    def ids_de(self, pai, de_crate, antes):
        """O que `ids` nomeia DENTRO do ficheiro-módulo `pai`: o `mod ids;` declarado ali, ou uma ligação."""
        if re.search(r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+ids\s*;", censo.blank(self.p.le(pai) or "")):
            base = os.path.dirname(pai) if os.path.basename(pai) in ("mod.rs", "lib.rs", "main.rs") else pai[:-3]
            alvo = os.path.join(base, "ids.rs")
            if alvo == GS:
                return "EC_GS"
            if os.path.join(base, "ids") + "/" == IDS:
                return "EC"
            for m, (_, _, raiz) in self.mods.items():
                if raiz == alvo:
                    return m
            return None
        return ligacoes(self, self.p, pai, de_crate, antes).get("ids")

    def conhecido(self, nome):
        return nome in self.antes

    def exporta(self, mid, nome, depois=True) -> bool:
        onde = (self.depois if depois else self.antes).get(nome)
        if onde == mid:
            return True
        if depois and self.fachadas and self.mods.get(mid, (None, None, None))[0] != EC:
            return False
        fonte = self.reexp.get(mid, {}).get(nome)
        if fonte and self.exporta(fonte, nome, depois):
            return True
        return any(self.exporta(g, nome, depois) for g in self.glob.get(mid, ()))

    def caminho(self, mid, de_crate, f=""):
        crate, ext, _ = self.mods[mid]
        if not (f and externo(f)):
            if mid == de_crate or (mid == "EC" and de_crate == EC):
                return "crate::ids"
            if mid == "EC_GS" and de_crate == EC:
                return "crate::grid_snap::ids"
            if mid == "UPS_TOOL" and de_crate == crate:
                return "crate::tool::ids"
        return ext


def parse_use(tree: str):
    """`use` achatado: [(caminho, nome, alias)]. `nome=='*'` é glob; `a::b` liga o módulo `b`."""
    tree = re.sub(r"\s+", " ", tree).strip()
    m = re.match(r"^(.*?)::\{(.*)\}$", tree)
    if m:
        out = []
        for part in split_top(m.group(2)):
            if part.strip():
                out.extend(parse_use(f"{m.group(1)}::{part.strip()}"))
        return out
    alias = None
    mm = re.match(r"^(.*?)\s+as\s+(" + IDENT + r")$", tree)
    if mm:
        tree, alias = mm.group(1).strip(), mm.group(2)
    if tree.endswith("::self"):
        tree = tree[: -len("::self")]
    segs = tree.split("::")
    return [("::".join(segs[:-1]), segs[-1], alias)]


def split_top(s: str):
    depth, cur, out = 0, [], []
    for ch in s:
        depth += (ch == "{") - (ch == "}")
        if ch == "," and depth == 0:
            out.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    out.append("".join(cur))
    return out


def render_use(itens, vis):
    base = collections.OrderedDict()
    for mp, nome, al in itens:
        base.setdefault(mp, []).append(f"{nome} as {al}" if al else nome)
    out = []
    for mp, ns in base.items():
        corpo = ns[0] if len(ns) == 1 else "{" + ", ".join(ns) + "}"
        out.append(f"{vis}use {mp}::{corpo};" if mp else f"{vis}use {corpo};")
    return out


def aplica(s, edits):
    out, pos = [], 0
    for a, e, t in sorted(edits, key=lambda x: x[0]):
        assert a >= pos, ("edições sobrepostas", a, pos)
        out.append(s[pos:a])
        out.append(t)
        pos = e
    out.append(s[pos:])
    return "".join(out)


def profundidade_zero(cod, pos):
    return cod[:pos].count("{") == cod[:pos].count("}")


def bloco_de(cod, a, e):
    """O `{…}` mais interior que contém `[a, e)` — ou o ficheiro inteiro."""
    depth, pilha = 0, []
    for i, ch in enumerate(cod[:a]):
        if ch == "{":
            pilha.append(i)
        elif ch == "}" and pilha:
            pilha.pop()
    if not pilha:
        return 0, len(cod)
    ini = pilha[-1]
    d = 0
    for j in range(ini, len(cod)):
        if cod[j] == "{":
            d += 1
        elif cod[j] == "}":
            d -= 1
            if d == 0:
                return ini, j + 1
    return ini, len(cod)


def ficheiro_pai(palco: Palco, f: str):
    """Quem declara `f` como módulo: `mod <nome>;` no `mod.rs`/`lib.rs`/`main.rs` do directório que o
    contém (ou no `<dir>.rs` ao lado dele), ou um IRMÃO com `#[path = "f.rs"]`. `None` numa raiz."""
    d, base = os.path.dirname(f), os.path.basename(f)
    stem = base[:-3]
    if stem in ("lib", "main", "build"):
        return None
    nome, onde = (os.path.basename(d), os.path.dirname(d)) if stem == "mod" else (stem, d)
    decl = re.compile(r"(?m)^\s*(?:#\[[^\n]*?\]\s*)*(?:pub(?:\([^)]*\))?\s+)?mod\s+" + re.escape(nome) + r"\s*;")
    for c in [os.path.join(onde, x) for x in ("mod.rs", "lib.rs", "main.rs")] + [onde + ".rs"]:
        s = palco.le(c) if c != f else None
        if s is not None and decl.search(censo.blank(s)):
            return c
    por_path = re.compile(r'#\[path\s*=\s*"' + re.escape(base) + r'"\]')
    try:
        irmaos = sorted(os.listdir(os.path.join(palco.root, d)))
    except OSError:
        irmaos = []
    for x in irmaos:
        c = os.path.join(d, x)
        s = palco.le(c) if (c != f and x.endswith(".rs")) else None
        if s is not None and por_path.search(censo.blank(s, keep_strings=True)):
            return c
    return None


def ligacoes(mundo: Mundo, palco: Palco, f, cf, antes: bool, prof=4):
    """alias → mod-id das ligações de TOPO de `f` — e, se `f` abre com `use super::*;`, as do pai."""
    s = palco.le(f) or ""
    cod = censo.blank(s)
    liga = {}
    herda = False
    for m in RE_USE.finditer(cod):
        if not profundidade_zero(cod, m.start()):
            continue
        for mp, nome, al in parse_use(m.group(3)):
            if mp == "super" and nome == "*":
                herda = True
                continue
            mid = mundo.mid(f"{mp}::{nome}" if mp else nome, cf, antes, f)
            if mid:
                liga[al or nome] = mid
    if herda and prof > 0:
        pai = ficheiro_pai(palco, f)
        if pai:
            for al, mid in ligacoes(mundo, palco, pai, cf, antes, prof - 1).items():
                liga.setdefault(al, mid)
    # a raiz de uma crate que ganha `pub mod ids;`: o alias `ids` do lib.rs é a fundação antes e a crate depois
    if cf in mundo.raiz_ec and f.endswith("/src/lib.rs"):
        liga["ids"] = "EC" if antes else cf
    return liga


# ────────────────────────────────────────────────────────────── a reescrita de UM leitor
def reescreve_leitor(mundo: Mundo, palco: Palco, f, cf, s, precisa):
    cod = censo.blank(s)
    liga_antes = ligacoes(mundo, palco, f, cf, True)
    liga_depois = ligacoes(mundo, palco, f, cf, False)
    usos = [(m.start(), m.end(), m) for m in RE_USE.finditer(cod)]
    edits, n = [], 0

    prefixos = {}
    for mid, (_, ext, _) in mundo.mods.items():
        prefixos[ext] = (mid, mid)
    prefixos["ph2d_editor_core::screens::hero::ids"] = ("EC", "EC")
    if not externo(f):
        if cf == EC:
            prefixos.update({"crate::ids": ("EC", "EC"), "crate::screens::hero::ids": ("EC", "EC"), "crate::grid_snap::ids": ("EC_GS", "EC_GS")})
        elif cf in mundo.mods:
            prefixos["crate::ids"] = (mundo.mid("crate::ids", cf, True, f), mundo.mid("crate::ids", cf, False, f))
    for al in set(liga_antes) | set(liga_depois):
        if al in liga_antes:
            prefixos.setdefault(al, (liga_antes[al], liga_depois.get(al, liga_antes[al])))
    for p in ("super::ids", "super::super::ids"):
        pa = mundo.mid(p, cf, True, f)
        if pa:
            prefixos.setdefault(p, (pa, mundo.mid(p, cf, False, f) or pa))
    # os `use … as core_ids;` DENTRO de uma fn (o `stencil.rs` do Painter tem dois): o de topo ganha
    for m in RE_USE.finditer(cod):
        if profundidade_zero(cod, m.start()):
            continue
        for mp, nome, al in parse_use(m.group(3)):
            cam = f"{mp}::{nome}" if mp else nome
            pa = mundo.mid(cam, cf, True, f)
            if pa:
                prefixos.setdefault(al or nome, (pa, mundo.mid(cam, cf, False, f) or pa))
    pat = re.compile(r"(?<![A-Za-z0-9_:])(" + "|".join(re.escape(p) for p in sorted(prefixos, key=len, reverse=True)) + r")::(" + IDENT + r")\b")
    dentro = [(a, e) for a, e, _ in usos]
    for m in pat.finditer(cod):
        if any(a <= m.start() < e for a, e in dentro):
            continue
        antes, depois = prefixos[m.group(1)]
        nome = m.group(2)
        if not mundo.conhecido(nome) or not mundo.exporta(antes, nome, False) or mundo.exporta(depois, nome, True):
            continue
        dest = mundo.depois[nome]
        edits.append((m.start(), m.end(), mundo.caminho(dest, cf, f) + "::" + nome))
        precisa(mundo.mods[dest][0])
        n += 1

    raiz_de_cf = mundo.mods.get(cf, (None, None, None))[2]
    for a, e, m in usos:
        vis_arg = (m.group(2) or "").strip()
        vis = "" if not m.group(1) else (f"pub({vis_arg}) " if vis_arg else "pub ")
        itens = parse_use(m.group(3))
        manter, novos, mudou = [], [], False
        resto = cod[:a] + cod[e:]
        for mp, nome, al in itens:
            mid_a = mundo.mid(mp, cf, True, f) if mp else None
            mid_d = mundo.mid(mp, cf, False, f) if mp else None
            if mp and not mid_a and mp in prefixos:
                # `use core_ids::{A, B};` — o caminho começa num ALIAS, não num módulo
                mid_a, mid_d = prefixos[mp]
            if not (mid_a and nome != "*" and mundo.conhecido(nome) and mundo.exporta(mid_a, nome, False) and not mundo.exporta(mid_d, nome, True)):
                manter.append((mp, nome, al))
                continue
            mudou = True
            n += 1
            dest = mundo.depois[nome]
            usado_nu = re.search(r"(?<![A-Za-z0-9_:])" + re.escape(al or nome) + r"\b", resto)
            if vis == "pub ":
                if usado_nu and not (f == raiz_de_cf and dest == cf):
                    novos.append((dest, nome, al, ""))
                    precisa(mundo.mods[dest][0])
                continue
            if f == raiz_de_cf and dest == cf:
                continue
            novos.append((dest, nome, al, vis))
            precisa(mundo.mods[dest][0])
        if not mudou:
            continue
        linha_ini = s.rfind("\n", 0, a) + 1
        ind = s[linha_ini:a] if s[linha_ini:a].strip() == "" else ""
        linhas = render_use(manter, vis) if manter else []
        grupos = collections.OrderedDict()
        for dest, nome, al, v in novos:
            grupos.setdefault((mundo.caminho(dest, cf, f), v), []).append((nome, al))
        for (cam, v), ns in grupos.items():
            linhas += render_use([(cam, x, al) for x, al in ns], v)
        edits.append((a, e, ("\n" + ind).join(linhas)) if linhas else span_da_linha_com_comentario(s, a, e))

    return aplica(s, edits), n


def filhos(palco: Palco, f: str):
    """Os ficheiros que `f` declara com `mod x;` — com `#[path]` relativo ao directório de `f`."""
    cod = censo.blank(palco.le(f) or "", keep_strings=True)
    d = os.path.dirname(f)
    sob = d if os.path.basename(f) in ("mod.rs", "lib.rs", "main.rs") else f[:-3]
    out = []
    for m in re.finditer(r"(?m)^[ \t]*((?:#\[[^\n]*?\]\s*)*)(?:pub(?:\([^)]*\))?\s+)?mod\s+(" + IDENT + r")\s*;", cod):
        p = re.search(r'#\[path\s*=\s*"([^"]+)"\]', m.group(1))
        cands = [os.path.normpath(os.path.join(d, p.group(1)))] if p else [os.path.join(sob, m.group(2) + ".rs"), os.path.join(sob, m.group(2), "mod.rs")]
        out += [c for c in cands if palco.existe(c)][:1]
    return out


def herdeiros(palco: Palco, f: str, prof=3):
    """O código dos descendentes que abrem com `use super::*;` — eles vêem os `use` privados de `f`."""
    out = []
    for c in filhos(palco, f) if prof > 0 else ():
        cc = censo.blank(palco.le(c) or "")
        if re.search(r"\buse\s+super::\*\s*;", cc):
            out.append(cc)
            out += herdeiros(palco, c, prof - 1)
    return out


def limpa_aliases(mundo, palco, s, cf, f):
    """Um `use` PRIVADO de módulo de ids que ficou sem `alias::` no seu ESCOPO sai.

    ⛔ Nunca um `pub(…) use` (existe para os irmãos). O ESCOPO de um `use` de TOPO inclui os filhos
    que o herdam por `use super::*;` (o `tool.rs` da ferramenta vectorial liga `ids` para o
    `tool_panel_event.rs`), e por isso isto corre DEPOIS de todos os leitores reescritos."""
    cod = censo.blank(s)
    herd = None
    edits = []
    for m in RE_USE.finditer(cod):
        if m.group(1):
            continue
        ini, fim = bloco_de(cod, m.start(), m.end())
        escopo = cod[ini: m.start()] + cod[m.end(): fim]
        if profundidade_zero(cod, m.start()):
            if herd is None:
                herd = "\n".join(herdeiros(palco, f))
            escopo += "\n" + herd
        itens = parse_use(m.group(3))
        vivos = []
        for mp, nome, al in itens:
            mid = mundo.mid(f"{mp}::{nome}" if mp else nome, cf, False, f)
            if mid and not re.search(r"(?<![A-Za-z0-9_:])(?:super::)*" + re.escape(al or nome) + r"::", escopo):
                continue
            vivos.append((mp, nome, al))
        if len(vivos) != len(itens):
            linha_ini = s.rfind("\n", 0, m.start()) + 1
            ind = s[linha_ini:m.start()] if s[linha_ini:m.start()].strip() == "" else ""
            edits.append((m.start(), m.end(), ("\n" + ind).join(render_use(vivos, ""))) if vivos else span_da_linha_com_comentario(s, m.start(), m.end()))
    return aplica(s, edits)


def span_da_linha_com_comentario(s, a, e):
    """O `use` apagado leva a LINHA inteira e o comentário `//` colado por cima dele — senão a nota que
    explicava a fachada fica a explicar a linha seguinte (a armadilha do HOWTO §2.18, um nível abaixo)."""
    ini = s.rfind("\n", 0, a) + 1
    if s[ini:a].strip():
        return (a, e, "")
    fim = e + 1 if e < len(s) and s[e] == "\n" else e
    while ini > 0:
        prev_ini = s.rfind("\n", 0, ini - 1) + 1
        if s[prev_ini: ini - 1].strip().startswith("//"):
            ini = prev_ini
        else:
            break
    return (ini, fim, "")


# ────────────────────────────────────────────────────────────── as DEFINIÇÕES
def cabecalho(s):
    out = []
    for l in s.splitlines(keepends=True):
        if l.startswith("//!"):
            out.append(l)
        elif l.strip() == "" and not out:
            continue
        else:
            break
    return "".join(out)


def uses_de_topo(s):
    """Os `use` de topo do ficheiro de origem que não são do vocabulário de ids (esses regeneram-se).
    `use crate::x::y` vira `use ph2d_editor_core::x::y`."""
    out = []
    cod = censo.blank(s)
    for m in RE_USE.finditer(cod):
        if not profundidade_zero(cod, m.start()):
            continue
        corpo = re.sub(r"\s+", " ", s[m.start(3): m.end(3)])
        if re.match(r"(super|crate::ids|crate::screens)", corpo) or "ph2d_tool_registry" in corpo or corpo.startswith("ph2d_a11y::NodeId"):
            continue
        if corpo.startswith("crate::"):
            corpo = "ph2d_editor_core::" + corpo[len("crate::"):]
        out.append((corpo, [al or nome for _, nome, al in parse_use(corpo)]))
    return out


def reescreve_movido(mundo: Mundo, corpo, dono, irmaos, proprios=frozenset(), teste=False):
    """⚠️ Num `#[cfg(test)] mod` o `super` é o ficheiro de destino e não a raiz de ids: um irmão que não
    mora no mesmo ficheiro nomeia-se por `crate::ids::`."""
    cod = censo.blank(corpo)
    edits = []
    pat = re.compile(r"(?<![A-Za-z0-9_:])((?:super::)+(?:[a-z_][a-z0-9_]*::)?|crate::ids::|crate::screens::hero::ids::|crate::)(" + IDENT + r")\b")
    for m in pat.finditer(cod):
        pref, nome = m.group(1), m.group(2)
        if pref == "crate::":
            if nome != "ids":
                edits.append((m.start(), m.start() + len(pref), "ph2d_editor_core::"))
            continue
        if nome in irmaos:
            edits.append((m.start(), m.end(), f"crate::ids::{nome}" if (teste and nome not in proprios) else f"super::{nome}"))
        elif nome in mundo.depois:
            edits.append((m.start(), m.end(), f"{mundo.caminho(mundo.depois[nome], dono)}::{nome}"))
        elif nome in ("hash_node_id", "hash_node_id_runtime"):
            edits.append((m.start(), m.end(), f"ph2d_tool_registry::{nome}"))
        elif nome == "NodeId":
            edits.append((m.start(), m.end(), "ph2d_a11y::NodeId"))
    return aplica(corpo, edits)


def tokens(corpo):
    return set(re.findall(r"(?<![A-Za-z0-9_:])" + IDENT + r"\b", censo.blank(corpo)))


def linhas_import(mundo: Mundo, toks, dono, irmaos, proprios, topo, sup="super"):
    out = []
    if "NodeId" in toks:
        out.append("use ph2d_a11y::NodeId;")
    porta = [t for t in ("hash_node_id", "hash_node_id_runtime") if t in toks]
    out += render_use([("ph2d_tool_registry", t, None) for t in porta], "") if porta else []
    grupos = collections.defaultdict(list)
    for t in sorted(toks - proprios):
        if t in irmaos:
            grupos[sup].append(t)
        elif t in mundo.depois and t in mundo.ec_nomes and mundo.depois[t] != dono:
            grupos[mundo.caminho(mundo.depois[t], dono)].append(t)
    for cam in sorted(grupos):
        out += render_use([(cam, t, None) for t in grupos[cam]], "")
    for corpo_use, nomes in topo:
        if any(x in toks for x in nomes):
            out.append(f"use {corpo_use};")
    return out


def move_definicoes(mundo: Mundo, palco: Palco, pk, precisa_do_dono, rel):
    c = mundo.c
    por_src = collections.defaultdict(lambda: collections.defaultdict(list))
    for k, dono in mundo.move.items():
        por_src[c[k]["file"]][dono].append(k)
    irmaos_de = collections.defaultdict(set)
    for k, dono in mundo.move.items():
        if c[k]["pub"] and c[k]["kind"] not in ("impl", "mod") and not c[k]["cfg_test"]:
            irmaos_de[dono].add(c[k]["name"])
    stems = collections.defaultdict(list)
    for src in sorted(por_src):
        s = palco.le(src)
        vivos = {(kind, name): (a, e) for kind, name, _, a, e, _ in censo.items(s)}
        spans = []
        topo = uses_de_topo(s)
        for dono, ks in sorted(por_src[src].items()):
            for k in ks:
                v = c[k]
                assert vivos.get((v["kind"], v["name"])) == (v["start"], v["end"]), f"censo VELHO em {k} — corra censo-ids.py --json"
            ks = sorted(ks, key=lambda k: c[k]["start"])
            relsrc = src[len(IDS):]
            stem = os.path.basename(src)[:-3]
            if stem in stems[dono]:
                stem = relsrc[:-3].replace("/", "_")
            stems[dono].append(stem)
            proprios = {c[k]["name"] for k in ks}
            prod = [reescreve_movido(mundo, s[c[k]["start"]: c[k]["end"]].strip("\n"), dono, irmaos_de[dono]) for k in ks if not c[k]["cfg_test"]]
            teste = [reescreve_movido(mundo, s[c[k]["start"]: c[k]["end"]].strip("\n"), dono, irmaos_de[dono], proprios, True) for k in ks if c[k]["cfg_test"]]
            toks_prod = set().union(*(tokens(x) for x in prod)) if prod else set()
            imp = linhas_import(mundo, toks_prod, dono, irmaos_de[dono], proprios, topo)
            # ⚠️ o que só o módulo de teste usa entra DENTRO dele (no topo seria `unused` num build normal)
            teste2 = []
            for t in teste:
                falta = linhas_import(mundo, tokens(t) - toks_prod, dono, irmaos_de[dono], proprios, topo, sup="crate::ids")
                if falta:
                    mm = re.search(r"use super::\*;\n", t)
                    if mm:
                        pad = re.match(r"[ \t]*", t[t.rfind("\n", 0, mm.start()) + 1:]).group(0)
                        t = t[: mm.end()] + "".join(f"{pad}{x}\n" for x in falta) + t[mm.end():]
                    else:
                        i = t.find("{") + 1
                        t = t[:i] + "\n" + "".join(f"    {x}\n" for x in falta) + t[i:]
                teste2.append(t)
            corpo = "\n\n".join(prod + teste2) + "\n"
            head = cabecalho(s).rstrip("\n")
            head += ("\n//!\n" if head else "") + (
                f"//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/{relsrc}` em {DATA}** (auditoria de arquitectura\n"
                f"//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os\n"
                f"//! carregar.\n"
            )
            destino = os.path.join(pk[dono]["dir"], "src/ids", stem + ".rs")
            if palco.existe(destino):
                # ⚠️ 2.ª PASSAGEM (a integração de 2026-09-13, depois da cerca): o ASSUNTO já desceu e o módulo
                # existe — os itens ACRESCENTAM-SE a ele, nunca num segundo ficheiro com o mesmo nome. Os
                # imports em falta entram depois do último `use` de topo (um duplicado EXACTO não entra; um
                # que colida com um grupo existente falha ALTO no compilador), e o `mod` não se declara outra vez.
                velho = palco.le(destino)
                ja = {l.strip() for l in velho.splitlines()}
                novos = [x for x in imp if x.strip() not in ja]
                if novos:
                    usos = list(re.finditer(r"(?m)^use [^;]*;[ \t]*\n", velho))
                    pos = usos[-1].end() if usos else re.match(r"(?:[ \t]*\n)*(?://![^\n]*\n)*", velho).end()
                    velho = velho[:pos] + "".join(x + "\n" for x in novos) + velho[pos:]
                bloco = (
                    f"// ── Desceu de `ph2d-editor-core/src/ids/{relsrc}` em {DATA} (2.ª passagem: a cerca com a\n"
                    f"//    `line/render-loop` prendia-os na fundação até às duas linhas se integrarem).\n\n"
                ) + corpo
                # ⚠️ ANTES de um `mod tests` de topo: depois dele o clippy reprova (`items_after_test_module`,
                # medido no `ph2d-panel-flip` na integração de 2026-09-13).
                testes = re.search(r"(?m)^#\[cfg\(test\)\]\s*\n(?:pub(?:\([a-z]+\))? )?mod tests\b", velho)
                if testes:
                    texto = velho[: testes.start()].rstrip("\n") + "\n\n" + bloco + "\n" + velho[testes.start():]
                else:
                    texto = velho.rstrip("\n") + "\n\n" + bloco
                stems[dono].remove(stem)
            else:
                texto = head + "\n" + ("\n".join(imp) + "\n\n" if imp else "") + corpo
            palco.escreve(destino, texto)
            for ext in set(re.findall(r"(?<![A-Za-z0-9_:])(ph2d_[a-z0-9_]+)::", censo.blank(texto))):
                if ext.replace("_", "-") in pk:
                    precisa_do_dono(dono, ext.replace("_", "-"))
            spans += [(c[k]["start"], c[k]["end"], "") for k in ks]
            rel.append(f"{src} → {destino} ({len(ks)} itens)")
        s2 = aplica(s, spans)
        if not [x for x in censo.items(s2) if x[0] not in ("use", "mod")]:
            palco.apaga(src)
            apaga_declaracao(palco, src)
        else:
            palco.escreve(src, limpa_imports_mortos(s2))
    return stems


def apaga_declaracao(palco: Palco, src: str):
    """Tira a declaração de um ficheiro que ficou vazio — `mod x;` no `mod.rs`, ou `#[path = "x.rs"]
    mod alias;` num IRMÃO (o `chrome/sculpt3d.rs` declara assim o `sculpt3d_cloth.rs`), com o
    `pub use alias::*;` dele. A doc `///` por cima vai junto."""
    d, base = os.path.dirname(src), os.path.basename(src)
    stem = base[:-3]
    doc = r"(?:[ \t]*///[^\n]*\n)*"
    vis = r"(?:pub(?:\([^)]*\))? )?"
    for nome in sorted(os.listdir(os.path.join(palco.root, d))):
        f = os.path.join(d, nome)
        if f == src or not nome.endswith(".rs"):
            continue
        s = palco.le(f)
        if s is None:
            continue
        m = re.search(r"(?m)^" + doc + r'[ \t]*#\[path\s*=\s*"' + re.escape(base) + r'"\]\s*\n[ \t]*' + vis + r"mod ([a-z_0-9]+);\n", s)
        if m:
            alias = m.group(1)
            s2 = s[: m.start()] + s[m.end():]
            s2 = re.sub(r"(?m)^[ \t]*pub use " + re.escape(alias) + r"::\*;\n", "", s2)
            palco.escreve(f, s2)
            return
        if nome == "mod.rs":
            s2 = re.sub(r"(?m)^" + doc + vis + r"mod " + re.escape(stem) + r";\n", "", s)
            if s2 != s:
                s2 = re.sub(r"(?m)^pub use " + re.escape(stem) + r"::\*;\n", "", s2)
                palco.escreve(f, s2)
                return
    raise AssertionError(f"não achei quem declara {src}")


def limpa_imports_mortos(s):
    """No que FICA na fundação: um `use` de topo sem leitor sai; um cujo único leitor é o módulo de
    teste muda-se para dentro dele."""
    cod = censo.blank(s)
    regs = censo.regioes_teste(cod)
    prod = list(cod)
    for a, e in regs:
        for i in range(a, e):
            if prod[i] != "\n":
                prod[i] = " "
    prod = "".join(prod)
    edits, para_teste = [], []
    for m in RE_USE.finditer(cod):
        if (m.group(1) and not (m.group(2) or "").strip()) or not profundidade_zero(cod, m.start()):
            continue
        itens = parse_use(m.group(3))
        resto_prod = prod[: m.start()] + prod[m.end():]
        vivos, so_teste = [], []
        for mp, n, al in itens:
            nome = al or n
            if n == "*" or re.search(r"(?<![A-Za-z0-9_])" + re.escape(nome) + r"\b", resto_prod):
                vivos.append((mp, n, al))
            elif regs and re.search(r"(?<![A-Za-z0-9_])" + re.escape(nome) + r"\b", cod[regs[0][0]: regs[0][1]]):
                so_teste.append((mp, n, al))
        if len(vivos) != len(itens):
            vis = "" if not m.group(1) else f"pub({m.group(2).strip()}) "
            edits.append((m.start(), m.end(), "\n".join(render_use(vivos, vis))) if vivos else span_da_linha_com_comentario(s, m.start(), m.end()))
            para_teste += so_teste
    if para_teste and regs:
        a, e = regs[0]
        mm = re.search(r"use super::\*;\n", s[a:e])
        pos = a + mm.end() if mm else a + s[a:e].find("{") + 1
        pad = "    "
        edits.append((pos, pos, ("" if mm else "\n") + "".join(f"{pad}{x}\n" for x in render_use(para_teste, ""))))
    return aplica(s, edits)


def raizes(mundo: Mundo, palco: Palco, pk, stems):
    for dono, sts in stems.items():
        if not sts:
            continue  # só acrescentou a módulos que já existiam (2.ª passagem)
        d = pk[dono]["dir"]
        raiz = os.path.join(d, "src/ids.rs")
        bloco = "".join(f"mod {st};\npub use {st}::*;\n" for st in sorted(set(sts)))
        s = palco.le(raiz)
        if s is not None:
            palco.escreve(raiz, s.rstrip("\n") + "\n\n" + bloco)
            continue
        palco.escreve(raiz, DOC_RAIZ.format(data=DATA, dono=snake(dono)) + "\n" + bloco)
        lib = os.path.join(d, "src/lib.rs")
        ls = palco.le(lib)
        if dono in mundo.raiz_ec:
            # o alias `use ph2d_editor_core::ids;` da raiz colide com o módulo novo — sai; as leituras
            # do lib.rs por ele reescrevem-se com o ANTES/DEPOIS de `ligacoes`
            ls = re.sub(r"(?m)^use ph2d_editor_core::ids;\s*\n", "", ls, count=1)
        m = re.search(r"(?m)^(pub(?:\([a-z]+\))? )?mod [a-z_0-9]+;", ls)
        assert m, f"{lib}: onde declarar `pub mod ids;`?"
        palco.escreve(lib, ls[: m.start()] + "pub mod ids;\n" + ls[m.start():])


DOC_RAIZ = """//! Os `NodeId` de widget que esta crate LÊ — declarados AQUI, e não na `ph2d-editor-core`.
//!
//! ⭐ **Desceram da fundação em {data}** (auditoria de arquitectura A5b): em 30 dias, 221 dos 437
//! commits à `ph2d-editor-core/src` tocaram `ids/`, e cada um recompilava as 60 crates que dependem
//! dela. O dono de um id é a crate MAIS BAIXA que todo leitor dele vê (`scripts/censo-ids.py`), e
//! um ficheiro de ids — um ASSUNTO — desce inteiro.
//!
//! ⚠️ **UMA definição e ZERO re-exportações**: quem lê de fora nomeia `{dono}::ids::X`. As colisões
//! de slug vigia-as o censo DERIVADO (`ph2d-editor-core/tests/it/node_id_collisions.rs`), que lê os
//! literais da workspace inteira — o id não precisa de morar ao lado dele.
"""

DOC_FACHADA = """//!
//! ⚠️ **Sem re-exportações desde {data}** (auditoria A5b) — ⛔ NOTA PROVISÓRIA, reescreva-a à mão: o
//! cabeçalho acima descreve a re-exportação que morreu, e só quem lê este ficheiro sabe dizer de
//! onde cada id vem agora (fundação, ferramenta, ou este painel).
"""


def mata_fachadas(mundo: Mundo, palco: Palco):
    tocados = []
    for mid, (crate, _, raiz) in mundo.mods.items():
        if not raiz or not crate.startswith("ph2d-panel-") or not palco.existe(raiz):
            continue
        s = palco.le(raiz)
        cod = censo.blank(s)
        edits = []
        for m in RE_USE.finditer(cod):
            if not m.group(1) or (m.group(2) or "").strip():
                continue
            itens = parse_use(m.group(3))
            if not any(mundo.mid(mp, crate, True) for mp, _, _ in itens):
                continue
            a = m.start()
            ini = s.rfind("\n", 0, a) + 1
            while True:
                prev_fim = ini - 1
                prev_ini = s.rfind("\n", 0, max(prev_fim, 0)) + 1
                if prev_fim > 0 and s[prev_ini:prev_fim].strip().startswith("///"):
                    ini = prev_ini
                else:
                    break
            fim = m.end() + (1 if m.end() < len(s) and s[m.end()] == "\n" else 0)
            vivos = [(mp, n, al) for mp, n, al in itens if not mundo.mid(mp, crate, True)]
            edits.append((ini, fim, "\n".join(render_use(vivos, "pub ")) + "\n" if vivos else ""))
        if not edits:
            continue
        s2 = aplica(s, edits)
        h = cabecalho(s2)
        if h and "Sem re-exportações" not in h:
            s2 = h + DOC_FACHADA.format(data=DATA) + s2[len(h):]
        palco.escreve(raiz, s2)
        tocados.append(raiz)
    return tocados


# ────────────────────────────────────────────────────────────── Cargo
def acrescenta_dep(palco, pk, de, para, dev, rel):
    if de == para:
        return
    deps = pk[de]["deps"]
    if (para, None) in deps or (dev and (para, "dev") in deps):
        return
    sp_de, sp_para = censo.especie(de, pk[de]["dir"]), censo.especie(para, pk[para]["dir"])
    assert censo.permitido(sp_de, sp_para, dev), f"a aresta {de} → {para} (dev={dev}) SOBE uma camada — aborto, nada foi escrito"
    man = os.path.join(pk[de]["dir"], "Cargo.toml")
    s = palco.le(man)
    relp = os.path.relpath(pk[para]["dir"], pk[de]["dir"])
    linha = f'{para} = {{ path = "{relp}" }}\n'
    cab = "[dev-dependencies]" if dev else "[dependencies]"
    i = s.find(cab + "\n")
    if i < 0:
        s = s.rstrip("\n") + f"\n\n{cab}\n{linha}"
    else:
        j = i + len(cab) + 1
        s = s[:j] + linha + s[j:]
    palco.escreve(man, s)
    deps.add((para, "dev" if dev else None))
    rel.append(f"{de} → {para}{' [dev]' if dev else ''}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("censo")
    ap.add_argument("--plano", action="store_true")
    ap.add_argument("--aplicar", action="store_true")
    ap.add_argument("--dono", action="append", default=[])
    ap.add_argument("--fachadas", action="store_true")
    a = ap.parse_args()
    root = os.path.abspath(".")
    palco = Palco(root)
    pk = censo.workspace(root)
    dirs = sorted(((v["dir"], k) for k, v in pk.items()), key=lambda x: -len(x[0]))
    cj = json.load(open(a.censo, encoding="utf-8"))
    mundo = Mundo(palco, pk, cj, set(a.dono), a.fachadas)
    if a.plano:
        for d, n in collections.Counter(mundo.move.values()).most_common():
            print(f"  {d:<30}{n:>5}")
        print(f"itens a mover: {len(mundo.move)} · módulos: {len(mundo.mods)} · raiz liga ids à fundação: {sorted(mundo.raiz_ec)}")
        return 0
    if not a.aplicar:
        return 0
    rel, deps_rel = [], []
    files = sorted(set(censo.git_rs_files(root)))
    stems = move_definicoes(mundo, palco, pk, lambda de, para: acrescenta_dep(palco, pk, de, para, False, deps_rel), rel)
    fach = mata_fachadas(mundo, palco) if a.fachadas else []
    decl = censo.ficheiros_teste_declarados(root, files)
    novos_ids = {f for f in palco.novo if "/src/ids/" in f and not f.startswith(IDS)}
    # ⚠️ os leitores lêem-se ANTES de a raiz ganhar `pub mod ids;` (o alias da raiz ainda está no lib.rs)
    total = 0
    reescritos = {}
    for f in files:
        if f.startswith(IDS) or f in novos_ids or f in palco.apagar:
            continue
        s = palco.le(f)
        if s is None or "ids" not in s:
            continue
        cf = censo.crate_of(f, dirs)
        if cf is None:
            continue
        teste = censo.contexto_teste(f) or f in decl
        novo, n = reescreve_leitor(mundo, palco, f, cf, s, lambda crate, cf=cf, teste=teste: acrescenta_dep(palco, pk, cf, crate, teste, deps_rel))
        if novo != s:
            reescritos[f] = novo
            total += n
    for f, s in reescritos.items():
        palco.escreve(f, s)
    for f in reescritos:
        s = palco.le(f)
        s2 = limpa_aliases(mundo, palco, s, censo.crate_of(f, dirs), f)
        if s2 != s:
            palco.escreve(f, s2)
    raizes(mundo, palco, pk, stems)
    palco.grava()
    print(f"ficheiros escritos: {len(palco.novo)} · apagados: {len(palco.apagar)} · fachadas: {len(fach)} · reescritas: {total}")
    print("arestas novas:", *deps_rel, sep="\n  ")
    print(*rel, sep="\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
