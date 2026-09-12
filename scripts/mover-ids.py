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
   DENTRO do texto movido são reescritos para o sítio novo.
2. **Fachadas** (`--fachadas`) — um `pub use <outro módulo de ids>::…` no `ids.rs` de um painel sai
   (⛔ zero fachadas, auditoria A5b); e uma fachada de um nome que MUDOU de casa sai sempre (uma
   fachada não se redirecciona: redireccioná-la seria a mesma aresta com outro nome — e na
   fundação seria uma aresta que sobe).
3. **Leitores** — em todo `.rs`, as ligações `use … ids` resolvem-se POR FICHEIRO (`use
   ph2d_editor_core::ids;`, `as core_ids`, `use crate::ids;`, grupos `{…}`) e cada
   `<módulo>::NOME` cujo módulo deixou de exportar o nome passa a `<módulo dono>::NOME`. Só em
   CÓDIGO: comentários e strings não se tocam (HOWTO §2.12).
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
DATA = "2026-09-12"
RE_USE = re.compile(r"\b(pub(?:\s*\(([^)]*)\))?\s+)?use\s+([^;]+);")
IDENT = r"[A-Za-z_][A-Za-z0-9_]*"


def snake(c: str) -> str:
    return c.replace("-", "_")


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
        # módulos: mid → (crate, caminho externo, ficheiro raiz)
        self.mods = {"EC": (EC, "ph2d_editor_core::ids", None), "EC_GS": (EC, "ph2d_editor_core::grid_snap::ids", GS)}
        for c, v in pk.items():
            if c.startswith("ph2d-panel-") or c.startswith("ph2d-tool-"):
                raiz = os.path.join(v["dir"], "src/ids.rs")
                if palco.existe(raiz) or c in self.move.values():
                    self.mods[c] = (c, f"{snake(c)}::ids", raiz)
        self.mods["UPS_TOOL"] = ("ph2d-tool-upscale", "ph2d_tool_upscale::tool::ids", None)
        # onde cada nome VIVE antes: os itens do censo na EC; as definições próprias dos outros
        self.antes = {}
        self.ec_nomes = set()
        for k, v in cj.items():
            if v["pub"] and v["kind"] not in ("impl", "mod") and not v["cfg_test"]:
                self.antes[v["name"]] = "EC"
                self.ec_nomes.add(v["name"])
        self.reexp = collections.defaultdict(dict)  # mid → {nome: mid fonte}
        self.glob = collections.defaultdict(list)  # mid → [mid fonte]
        for mid, (crate, _, raiz) in self.mods.items():
            if raiz and palco.existe(raiz):
                self._le_raiz(mid, crate, raiz)
        up = os.path.join(pk["ph2d-tool-upscale"]["dir"], "src/tool.rs")
        s = palco.le(up) or ""
        i = censo.blank(s).find("pub mod ids")
        if i >= 0:
            for m in re.finditer(r"\bpub\s+const\s+(" + IDENT + ")", censo.blank(s)[i:]):
                self.antes.setdefault(m.group(1), "UPS_TOOL")
        # DEPOIS
        self.depois = dict(self.antes)
        for k, dono in self.move.items():
            v = cj[k]
            if v["pub"] and v["kind"] not in ("impl", "mod") and not v["cfg_test"]:
                self.depois[v["name"]] = dono

    def _le_raiz(self, mid, crate, raiz):
        s = self.p.le(raiz)
        b = censo.blank(s)
        for m in re.finditer(r"\bpub\s+(?:const|static|fn|enum|struct|type)\s+(" + IDENT + ")", b):
            self.antes.setdefault(m.group(1), mid)
        for m in RE_USE.finditer(b):
            if not m.group(1) or (m.group(2) or "").strip():
                continue  # só `pub use` nu é re-exportação para fora
            for mp, nome, al in parse_use(m.group(3)):
                fonte = self.mid(mp, crate)
                if not fonte:
                    continue
                if nome == "*":
                    self.glob[mid].append(fonte)
                else:
                    self.reexp[mid][al or nome] = fonte

    def mid(self, caminho: str, de_crate: str):
        caminho = caminho.strip()
        if de_crate == EC and caminho in ("crate::ids", "crate::screens::hero::ids", "super::super::ids"):
            return "EC"
        if de_crate == EC and caminho == "crate::grid_snap::ids":
            return "EC_GS"
        if caminho in ("crate::ids", "super::ids") and de_crate in self.mods:
            return de_crate
        if caminho == "ph2d_editor_core::screens::hero::ids":
            return "EC"
        for m, (_, ext, _) in self.mods.items():
            if caminho == ext:
                return m
        return None

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

    def caminho(self, mid, de_crate):
        crate, ext, _ = self.mods[mid]
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


# ────────────────────────────────────────────────────────────── a reescrita de UM leitor
def reescreve_leitor(mundo: Mundo, f, cf, s, precisa):
    cod = censo.blank(s)
    usos = []
    liga = {}
    for m in RE_USE.finditer(cod):
        vis_arg = (m.group(2) or "").strip()
        vis = "" if not m.group(1) else (f"pub({vis_arg}) " if vis_arg else "pub ")
        itens = parse_use(m.group(3))
        usos.append((m.start(), m.end(), vis, itens))
        for mp, nome, al in itens:
            mid = mundo.mid(f"{mp}::{nome}" if mp else nome, cf)
            if mid:
                liga[al or nome] = mid
    raiz_de_cf = mundo.mods.get(cf, (None, None, None))[2]
    edits, n = [], 0

    prefixos = {ext: mid for mid, (_, ext, _) in mundo.mods.items()}
    prefixos["ph2d_editor_core::screens::hero::ids"] = "EC"
    if cf in mundo.mods:
        prefixos["crate::ids"] = cf
        prefixos["super::ids"] = cf
    if cf == EC:
        prefixos.update({"crate::ids": "EC", "crate::screens::hero::ids": "EC", "crate::grid_snap::ids": "EC_GS"})
    for al, mid in liga.items():
        prefixos.setdefault(al, mid)
    pat = re.compile(r"(?<![A-Za-z0-9_:])(" + "|".join(re.escape(p) for p in sorted(prefixos, key=len, reverse=True)) + r")::(" + IDENT + r")\b")
    dentro = [(a, e) for a, e, _, _ in usos]
    for m in pat.finditer(cod):
        if any(a <= m.start() < e for a, e in dentro):
            continue
        mid, nome = prefixos[m.group(1)], m.group(2)
        if not mundo.conhecido(nome) or not mundo.exporta(mid, nome, False) or mundo.exporta(mid, nome, True):
            continue
        dest = mundo.depois[nome]
        edits.append((m.start(), m.end(), mundo.caminho(dest, cf) + "::" + nome))
        precisa(mundo.mods[dest][0])
        n += 1

    for a, e, vis, itens in usos:
        manter, novos, mudou = [], [], False
        resto = cod[:a] + cod[e:]
        for mp, nome, al in itens:
            mid = mundo.mid(mp, cf) if mp else None
            if not (mid and nome != "*" and mundo.conhecido(nome) and mundo.exporta(mid, nome, False) and not mundo.exporta(mid, nome, True)):
                manter.append((mp, nome, al))
                continue
            mudou = True
            n += 1
            dest = mundo.depois[nome]
            usado_nu = re.search(r"(?<![A-Za-z0-9_:])" + re.escape(al or nome) + r"\b", resto)
            if vis == "pub ":
                # ⛔ uma fachada nunca se redirecciona — ela morre; se o nome é usado nu aqui, entra um `use` privado
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
            grupos.setdefault((mundo.caminho(dest, cf), v), []).append((nome, al))
        for (cam, v), ns in grupos.items():
            linhas += render_use([(cam, x, al) for x, al in ns], v)
        if linhas:
            edits.append((a, e, ("\n" + ind).join(linhas)))
        else:
            edits.append(span_da_linha_com_comentario(s, a, e))

    out = aplica(s, edits)
    return limpa_aliases(mundo, out, cf), n


def limpa_aliases(mundo, s, cf):
    """Um `use` PRIVADO de módulo de ids que ficou sem `alias::` no ficheiro sai.

    ⛔ Nunca um `pub(crate) use` / `pub(super) use`: esse existe para OUTROS ficheiros (medido: o
    `ph2d-panel-inspector/src/sections/mod.rs` liga `ids` para os irmãos, e um dia apagá-lo por não
    ter uso no próprio ficheiro partiu 40 sítios). Se ele ficar mesmo sem uso, o `rustc` avisa."""
    cod = censo.blank(s)
    edits = []
    for m in RE_USE.finditer(cod):
        if m.group(1):
            continue
        itens = parse_use(m.group(3))
        vivos = []
        for mp, nome, al in itens:
            mid = mundo.mid(f"{mp}::{nome}" if mp else nome, cf)
            if mid and not re.search(r"(?<![A-Za-z0-9_:])" + re.escape(al or nome) + r"::", cod[: m.start()] + cod[m.end():]):
                continue
            vivos.append((mp, nome, al))
        if len(vivos) != len(itens):
            linha_ini = s.rfind("\n", 0, m.start()) + 1
            ind = s[linha_ini:m.start()] if s[linha_ini:m.start()].strip() == "" else ""
            if vivos:
                edits.append((m.start(), m.end(), ("\n" + ind).join(render_use(vivos, ""))))
            else:
                edits.append(span_da_linha_com_comentario(s, m.start(), m.end()))
    return aplica(s, edits)


def span_da_linha_com_comentario(s, a, e):
    """O `use` apagado leva a LINHA inteira e o comentário `//` colado por cima dele — senão a nota que
    explicava a fachada fica a explicar a linha seguinte (é a armadilha do comentário de `Cargo.toml`
    do HOWTO §2.18, um nível abaixo)."""
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
        if cod[: m.start()].count("{") != cod[: m.start()].count("}"):
            continue
        corpo = re.sub(r"\s+", " ", s[m.start(3): m.end(3)])
        if re.match(r"(super|crate::ids|crate::screens)", corpo) or "ph2d_tool_registry" in corpo or corpo.startswith("ph2d_a11y::NodeId"):
            continue
        if corpo.startswith("crate::"):
            corpo = "ph2d_editor_core::" + corpo[len("crate::"):]
        out.append((corpo, [al or nome for _, nome, al in parse_use(corpo)]))
    return out


def reescreve_movido(mundo: Mundo, corpo, dono, irmaos):
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
            edits.append((m.start(), m.end(), f"super::{nome}"))
        elif nome in mundo.depois:
            edits.append((m.start(), m.end(), f"{mundo.caminho(mundo.depois[nome], dono)}::{nome}"))
        elif nome in ("hash_node_id", "hash_node_id_runtime"):
            edits.append((m.start(), m.end(), f"ph2d_tool_registry::{nome}"))
        elif nome == "NodeId":
            edits.append((m.start(), m.end(), "ph2d_a11y::NodeId"))
    return aplica(corpo, edits)


def imports(mundo: Mundo, corpo, dono, irmaos, proprios, topo):
    cod = censo.blank(corpo)
    toks = set(re.findall(r"(?<![A-Za-z0-9_:])" + IDENT + r"\b", cod))
    out = []
    if "NodeId" in toks:
        out.append("use ph2d_a11y::NodeId;")
    porta = [t for t in ("hash_node_id", "hash_node_id_runtime") if t in toks]
    out += render_use([("ph2d_tool_registry", t, None) for t in porta], "") if porta else []
    grupos = collections.defaultdict(list)
    for t in sorted(toks - proprios):
        if t in irmaos:
            grupos["super"].append(t)
        elif t in mundo.depois and t in mundo.ec_nomes:
            d = mundo.depois[t]
            if d != dono:
                grupos[mundo.caminho(d, dono)].append(t)
    for cam in sorted(grupos):
        out += render_use([(cam, t, None) for t in grupos[cam]], "")
    for corpo_use, nomes in topo:
        if any(x in toks for x in nomes):
            out.append(f"use {corpo_use};")
    return "\n".join(out) + ("\n" if out else "")


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
            corpo = "\n".join(s[c[k]["start"]: c[k]["end"]].strip("\n") + "\n" for k in ks)
            corpo = reescreve_movido(mundo, corpo, dono, irmaos_de[dono])
            proprios = {c[k]["name"] for k in ks}
            imp = imports(mundo, corpo, dono, irmaos_de[dono], proprios, uses_de_topo(s))
            head = cabecalho(s).rstrip("\n")
            head += ("\n//!\n" if head else "") + (
                f"//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/{relsrc}` em {DATA}** (auditoria de arquitectura\n"
                f"//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 43 crates recompilam deixou de os\n"
                f"//! carregar.\n"
            )
            destino = os.path.join(pk[dono]["dir"], "src/ids", stem + ".rs")
            assert not palco.existe(destino), f"{destino} já existe"
            texto = head + "\n" + imp + "\n" + corpo
            palco.escreve(destino, texto)
            for ext in set(re.findall(r"(?<![A-Za-z0-9_:])(ph2d_[a-z0-9_]+)::", censo.blank(texto))):
                if ext.replace("_", "-") in pk:
                    precisa_do_dono(dono, ext.replace("_", "-"))
            spans += [(c[k]["start"], c[k]["end"], "") for k in ks]
            rel.append(f"{src} → {destino} ({len(ks)} itens)")
        s2 = aplica(s, spans)
        if not [x for x in censo.items(s2) if x[0] != "use"]:
            palco.apaga(src)
            pai = os.path.join(os.path.dirname(src), "mod.rs")
            ps = palco.le(pai)
            st = os.path.basename(src)[:-3]
            ps2 = re.sub(r"(?m)^(?:[ \t]*///[^\n]*\n)*(?:pub(?:\([^)]*\))? )?mod " + re.escape(st) + r";\n", "", ps)
            ps2 = re.sub(r"(?m)^pub use " + re.escape(st) + r"::\*;\n", "", ps2)
            assert ps2 != ps, f"não achei a declaração de {st} em {pai}"
            palco.escreve(pai, ps2)
        else:
            palco.escreve(src, limpa_imports_mortos(s2))
    return stems


def limpa_imports_mortos(s):
    cod = censo.blank(s)
    edits = []
    for m in RE_USE.finditer(cod):
        if (m.group(1) and not (m.group(2) or "").strip()) or cod[: m.start()].count("{") != cod[: m.start()].count("}"):
            continue
        itens = parse_use(m.group(3))
        resto = cod[: m.start()] + cod[m.end():]
        vivos = [(mp, n, al) for mp, n, al in itens if n == "*" or re.search(r"(?<![A-Za-z0-9_])" + re.escape(al or n) + r"\b", resto)]
        if len(vivos) != len(itens):
            vis = "" if not m.group(1) else f"pub({m.group(2).strip()}) "
            edits.append((m.start(), m.end(), "\n".join(render_use(vivos, vis)) if vivos else ""))
    return aplica(s, edits)


def raizes(mundo: Mundo, palco: Palco, pk, stems):
    for dono, sts in stems.items():
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
        m = re.search(r"(?m)^(pub(?:\([a-z]+\))? )?mod [a-z_0-9]+;", ls)
        assert m, f"{lib}: onde declarar `pub mod ids;`?"
        palco.escreve(lib, ls[: m.start()] + "pub mod ids;\n" + ls[m.start():])


DOC_RAIZ = """//! Os `NodeId` de widget que esta crate LÊ — declarados AQUI, e não na `ph2d-editor-core`.
//!
//! ⭐ **Desceram da fundação em {data}** (auditoria de arquitectura A5b): em 30 dias, 221 dos 437
//! commits à `ph2d-editor-core/src` tocaram `ids/`, e cada um recompilava as 43 crates que dependem
//! dela. O dono de um id é a crate MAIS BAIXA que todo leitor dele vê (`scripts/censo-ids.py`), e
//! um ficheiro de ids — um ASSUNTO — desce inteiro.
//!
//! ⚠️ **UMA definição e ZERO re-exportações**: quem lê de fora nomeia `{dono}::ids::X`. As colisões
//! de slug vigia-as o censo DERIVADO (`ph2d-editor-core/tests/it/node_id_collisions.rs`), que lê os
//! literais da workspace inteira — o id não precisa de morar ao lado dele.
"""

DOC_FACHADA = """//!
//! ⚠️ **Sem re-exportações desde {data}** (auditoria A5b). Este módulo dizia que os ids ficavam na
//! `ph2d-editor-core` porque *«o layout, o z-order walk e o `node_id_collisions` os referenciam»*: os
//! dois primeiros são os ids que a fundação LÊ, e esses ficaram lá; o terceiro passou a ser um censo
//! DERIVADO dos literais da workspace. O resto desceu para a crate que o lê — e com UMA definição e
//! zero re-exportações, o «fork da verdade» que a nota temia deixou de poder existir.
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
            if not any(mundo.mid(mp, crate) for mp, _, _ in itens):
                continue
            a = m.start()
            # a doc (`///`) imediatamente acima vai junto
            ini = s.rfind("\n", 0, a) + 1
            while True:
                prev_fim = ini - 1
                prev_ini = s.rfind("\n", 0, max(prev_fim, 0)) + 1
                if prev_fim > 0 and s[prev_ini:prev_fim].strip().startswith("///"):
                    ini = prev_ini
                else:
                    break
            fim = m.end() + (1 if m.end() < len(s) and s[m.end()] == "\n" else 0)
            vivos = [(mp, n, al) for mp, n, al in itens if not mundo.mid(mp, crate)]
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
        print(f"itens a mover: {len(mundo.move)} · módulos: {len(mundo.mods)} · re-exportações lidas: "
              f"{sum(len(v) for v in mundo.reexp.values())} + {sum(len(v) for v in mundo.glob.values())} globs")
        return 0
    if not a.aplicar:
        return 0
    rel, deps_rel = [], []
    stems = move_definicoes(mundo, palco, pk, lambda de, para: acrescenta_dep(palco, pk, de, para, False, deps_rel), rel)
    raizes(mundo, palco, pk, stems)
    fach = mata_fachadas(mundo, palco) if a.fachadas else []
    files = set(censo.git_rs_files(root)) | set(palco.novo)
    decl = censo.ficheiros_teste_declarados(root, sorted(files))
    novos_ids = {f for f in palco.novo if "/src/ids/" in f and not f.startswith(IDS)}
    total = 0
    for f in sorted(files):
        if f.startswith(IDS) or f in novos_ids or f in palco.apagar:
            continue
        s = palco.le(f)
        if s is None or "ids" not in s:
            continue
        cf = censo.crate_of(f, dirs)
        if cf is None:
            continue
        teste = censo.contexto_teste(f) or f in decl
        novo, n = reescreve_leitor(mundo, f, cf, s, lambda crate, cf=cf, teste=teste: acrescenta_dep(palco, pk, cf, crate, teste, deps_rel))
        if novo != s:
            palco.escreve(f, novo)
            total += n
    palco.grava()
    print(f"ficheiros escritos: {len(palco.novo)} · apagados: {len(palco.apagar)} · fachadas: {len(fach)} · reescritas: {total}")
    print("arestas novas:", *deps_rel, sep="\n  ")
    print(*rel, sep="\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
