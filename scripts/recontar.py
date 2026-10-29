#!/usr/bin/env python3
"""RECONTA os números que SOMAM entre linhas (CLAUDE.md §5.0: «se CONTA, nunca se escolhe»).

    python3 scripts/recontar.py              # ✓/✗ por número; exit 1 se algum ✗ ou ADR duplicado
    python3 scripts/recontar.py --proximo    # o próximo ADR livre e a próxima cena por roteador
    python3 scripts/recontar.py --raiz DIR --apenas adr,registos,cenas,schema   # fixtura / recorte

Para cada número: CONTA na fonte (o código) e compara com onde ele está ESCRITO. Puro texto, sem
compilar (< 5 s). Corre-se depois de cada rebase de uma rodada (`/pd-integracao` passo 4).

- **adr**: dois ADRs (não-`amendment`) com o mesmo `NNNN` é o defeito de um rebase de duas linhas;
  o índice derivado confere-se por `scripts/adr-index.sh --check`.
- **registos**: os `reg.register*::<…>` de `register_ecs_components` (seguindo as chamadas
  `…(reg)` da própria crate) contra o `assert_eq!(reg.len(), N)` do ecs e dos espelhos render/script
  (= ecs + os da própria crate).
- **cenas**: o `max_level` de cada `SmokeRouter` (literal ou `const` resolvida na crate) contra o
  maior nível que o CLAUDE.md §5.1 escreve para a mesma variável (`=1..N`, `(até N)`, `1|2|6..N`).
- **schema**: o `PROJECT_SCHEMA` da escada contra a última linha da tripla e os irmãos FLIP/VEC —
  com as constantes e o leitor de `scripts/schema-recount.py` (a lógica não se duplica).
"""
import argparse
import glob
import importlib.util
import os
import re
import subprocess
import sys

AQUI = os.path.dirname(os.path.abspath(__file__))
falhas = []


def ok(msg):
    print(f"  ✓ {msg}")


def ko(msg):
    print(f"  ✗ {msg}")
    falhas.append(msg)


def info(msg):
    print(f"  · {msg}")


def ler(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        return f.read()


def sem_comentarios(src):
    """Corta `//…` fora de string (aspas pares antes do `//`); preserva as linhas (ficheiro:linha)."""
    out = []
    for l in src.split("\n"):
        i = l.find("//")
        while i >= 0 and l[:i].count('"') % 2:
            i = l.find("//", i + 2)
        out.append(l if i < 0 else l[:i])
    return "\n".join(out)


def linha_de(src, pos):
    return src.count("\n", 0, pos) + 1


# ───────────────────────────────────────────── ADR
def adr(raiz):
    print("— ADR —")
    d = os.path.join(raiz, "docs/architecture/decisions")
    por_num = {}
    for f in sorted(os.listdir(d)) if os.path.isdir(d) else []:
        m = re.match(r"^(\d{4})-(.+)\.md$", f)
        if m and not m.group(2).startswith("amendment"):
            por_num.setdefault(m.group(1), []).append(f)
    if not por_num:
        ko(f"ADR: nenhum ADR em {d} (sonda cega)")
        return None
    dup = {n: fs for n, fs in por_num.items() if len(fs) > 1}
    for n, fs in sorted(dup.items()):
        ko(f"ADR {n} DUPLICADO: {' · '.join(fs)} — renumere o da linha integrada por último")
    if not dup:
        ok(f"ADR: {len(por_num)} números, nenhum duplicado")
    idx = os.path.join(raiz, "scripts/adr-index.sh")
    if os.path.isfile(idx):
        r = subprocess.run(["bash", idx, "--check"], cwd=raiz, capture_output=True, text=True)
        (ok if r.returncode == 0 else ko)("índice: " + (r.stdout + r.stderr).strip().splitlines()[-1]
                                          if (r.stdout + r.stderr).strip() else "índice: adr-index.sh --check")
    return max(int(n) for n in por_num) + 1


# ───────────────────────────────────────────── registos de componentes
def corpo_da_fn(src, nome):
    m = re.search(rf"\bfn {re.escape(nome)}\s*\(", src)
    if not m:
        return None
    j = src.index("{", m.end())
    d = 0
    for k in range(j, len(src)):
        d += {"{": 1, "}": -1}.get(src[k], 0)
        if d == 0:
            return src[j:k]
    return None


def conta_registos(crate_src, nome, vistos=None):
    """Conta `reg.register*::<` no corpo de `nome`, seguindo `…::f(reg)` definidos na crate."""
    vistos = vistos if vistos is not None else set()
    vistos.add(nome)
    for f in glob.glob(os.path.join(crate_src, "**/*.rs"), recursive=True):
        corpo = corpo_da_fn(ler(f), nome)
        if corpo is None:
            continue
        code = sem_comentarios(corpo)
        n = len(re.findall(r"\breg\.register(?:_default)?\s*::<", code))
        for sub in re.findall(r"(?:\b\w+::)*(\w+)\s*\(\s*reg\s*\)", code):
            if sub not in vistos:
                n += conta_registos(crate_src, sub, vistos) or 0
        return n
    return None


def escrito_len(path):
    src = ler(path)
    m = re.search(r"assert_eq!\(reg\.len\(\),\s*(\d+)\)", src)
    return (int(m.group(1)), linha_de(src, m.start())) if m else (None, None)


def registos(raiz):
    print("— registos de componentes (ecs / render / script) —")
    ecs = conta_registos(os.path.join(raiz, "crates/ph2d-ecs/src"), "register_ecs_components")
    if ecs is None:
        ko("registo ph2d-ecs: `register_ecs_components` não encontrado (sonda cega)")
        return
    alvos = [("ph2d-ecs", "crates/ph2d-ecs/src/scene/registry_tests.rs", ecs)]
    for crate, fn in (("ph2d-render", "register_render_components"),
                      ("ph2d-script", "register_script_components")):
        proprio = conta_registos(os.path.join(raiz, f"crates/{crate}/src"), fn)
        if proprio is None:
            ko(f"registo {crate}: `{fn}` não encontrado (sonda cega)")
            continue
        alvos.append((crate, f"crates/{crate}/src/registry.rs", ecs + proprio))
    for crate, rel, contado in alvos:
        p = os.path.join(raiz, rel)
        escrito, ln = escrito_len(p) if os.path.isfile(p) else (None, None)
        if escrito is None:
            ko(f"registo {crate}: sem `assert_eq!(reg.len(), N)` em {rel} (sonda cega)")
        elif escrito == contado:
            ok(f"registo {crate} = {contado}")
        else:
            ko(f"registo {crate}: escrito {escrito}, contado {contado} ({rel}:{ln})")


# ───────────────────────────────────────────── cenas de smoke
def resolve_const(expr, arquivo, crate_src):
    expr = re.sub(r"\s+as\s+u\d+\s*$", "", expr.strip())
    if re.fullmatch(r"\d+", expr):
        return int(expr)
    segs = expr.split("::")
    nome = segs[-1]
    cands = []
    for f in glob.glob(os.path.join(crate_src, "**/*.rs"), recursive=True):
        m = re.search(rf"^\s*(?:pub(?:\([^)]*\))?\s+)?const {re.escape(nome)}\s*:\s*u\d+\s*=\s*(\d+)\s*;",
                      ler(f), re.M)
        if m:
            cands.append((f, int(m.group(1))))
    if len(cands) > 1:  # o caminho do módulo decide: `smoke::CENAS` → smoke.rs, `demo_router::X` → *demo_router*.rs
        mod = segs[-2] if len(segs) > 1 else None
        if mod is None:
            cands = [c for c in cands if c[0] == arquivo] or cands
        else:
            cands = ([c for c in cands if os.path.basename(c[0]) in (f"{mod}.rs", "mod.rs")
                      and (os.path.basename(c[0]) != "mod.rs" or os.path.basename(os.path.dirname(c[0])) == mod)]
                     or [c for c in cands if mod in os.path.basename(c[0])] or cands)
    return cands[0][1] if len(cands) == 1 else None


def roteadores(raiz):
    """{env: (max_level, ficheiro:linha)} dos `SmokeRouter` declarados nas crates de família."""
    out = {}
    for f in sorted(glob.glob(os.path.join(raiz, "crates/ph2d-app-*/src/**/*.rs"), recursive=True)):
        src = ler(f)
        if "SmokeRouter" not in src and not re.search(r"\br\(\s*\"PH2D_", src):
            continue
        crate_src = f.split("/src/")[0] + "/src"
        code = sem_comentarios(src)
        achados = []
        for m in re.finditer(r"SmokeRouter\s*\{\s*env:\s*([^,]+?),\s*max_level:\s*([^,}]+?)\s*,?\s*\}", code):
            achados.append((m.group(1), m.group(2), m.start()))
        for m in re.finditer(r"(?<![\w:])r\(\s*(\"PH2D_\w+\")\s*(?:,\s*([^)]+?))?\s*\)", code):
            achados.append((m.group(1), m.group(2) or "1", m.start()))
        for env, ml, pos in achados:
            env = env.strip()
            if env == "env":  # o próprio `const fn r(env, …)`
                continue
            if not env.startswith('"'):
                c = re.search(rf"const {re.escape(env.split('::')[-1])}\s*:\s*&str\s*=\s*(\"\w+\")", src)
                env = c.group(1) if c else env
            if ml.strip() == "max_level":
                continue
            rel = os.path.relpath(f, raiz)
            out[env.strip('"')] = (resolve_const(ml, f, crate_src), f"{rel}:{linha_de(code, pos)}", ml.strip())
    return out


def escritos_no_claude(raiz):
    p = os.path.join(raiz, "CLAUDE.md")
    if not os.path.isfile(p):
        return {}
    out = {}
    for ln, l in enumerate(ler(p).split("\n"), 1):
        for m in re.finditer(r"`(PH2D_\w+)=([0-9][0-9.|]*)`(?:\s*\(até (\d+)\))?", l):
            env, faixa, ate = m.groups()
            nums = [int(x) for x in re.findall(r"\d+", faixa)]
            n = int(ate) if ate else max(nums)
            if len(nums) == 1 and not ate and ".." not in faixa and "|" not in faixa:
                continue  # `=30` cita UMA cena (ex.: «Smoke PH2D_BOARD_SMOKE=30»), não o tecto
            out.setdefault(env, []).append((n, f"CLAUDE.md:{ln}"))
        for m in re.finditer(r"`(PH2D_\w+)=<n>`\s*\(até (\d+)\)", l):
            out.setdefault(m.group(1), []).append((int(m.group(2)), f"CLAUDE.md:{ln}"))
    return out


def cenas(raiz):
    print("— cenas de smoke (roteador × CLAUDE.md §5.1) —")
    rot = roteadores(raiz)
    if not rot:
        ko("cenas: nenhum SmokeRouter em crates/ph2d-app-*/src (sonda cega)")
        return rot
    for env, (n, onde, expr) in sorted(rot.items()):
        if n is None:
            ko(f"cenas {env}: max_level `{expr}` não se resolve a um número ({onde})")
    for env, lista in sorted(escritos_no_claude(raiz).items()):
        for escrito, onde in lista:
            if env not in rot:
                info(f"cenas {env} = {escrito} escrito ({onde}) — sem SmokeRouter: roteador na shell, não contado")
            elif rot[env][0] == escrito:
                ok(f"cenas {env} = {escrito}")
            elif rot[env][0] is not None:
                ko(f"cenas {env}: escrito {escrito}, contado {rot[env][0]} ({onde}; roteador {rot[env][1]})")
    print(f"  · {len(rot)} roteadores contados")
    return rot


# ───────────────────────────────────────────── PROJECT_SCHEMA (pela lógica do schema-recount.py)
def schema(raiz):
    print("— PROJECT_SCHEMA (escada × tripla × irmãos) —")
    spec = importlib.util.spec_from_file_location("schema_recount", os.path.join(AQUI, "schema-recount.py"))
    sr = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(sr)
    lad, tri = os.path.join(raiz, sr.LADDER), os.path.join(raiz, sr.TRIPLE)
    if not (os.path.isfile(lad) and os.path.isfile(tri)):
        ko(f"schema: {sr.LADDER} ou {sr.TRIPLE} ausente (sonda cega)")
        return
    s = ler(lad)
    m = re.search(re.escape(sr.CONST) + r"(\d+);", s)
    if not m:
        ko(f"schema: `{sr.CONST}` não está em {sr.LADDER} (sonda cega)")
        return
    p = int(m.group(1))
    t = ler(tri)
    linha = sr.tuple_line(t, p)
    if linha is None:
        ko(f"schema: escada diz {p}, a tripla não tem o degrau ({sr.TRIPLE})")
        return
    maior = max(int(x) for x in re.findall(r"^        \((\d+), \d+, \d+\),$", t, re.M))
    if maior != p:
        ko(f"schema: escrito {p} na escada, a tripla vai até {maior} ({sr.LADDER}:{linha_de(s, m.start())})")
        return
    flip, vec = (int(x) for x in re.findall(r"\d+", linha)[1:])
    irmaos = (("FLIP_SCHEMA_VERSION", "crates/ph2d-flip/src/lib.rs", flip),
              ("VEC_SCENE_SCHEMA_VERSION", "crates/ph2d-vec-scene/src/schema.rs", vec))
    bom = True
    for nome, rel, na_tripla in irmaos:
        q = os.path.join(raiz, rel)
        c = re.search(rf"{nome}: u32 = (\d+);", ler(q)) if os.path.isfile(q) else None
        if c is None:
            ko(f"schema: {nome} não está em {rel} (sonda cega)")
            bom = False
        elif int(c.group(1)) != na_tripla:
            ko(f"schema: tripla diz {nome}={na_tripla}, a const diz {c.group(1)} ({rel})")
            bom = False
    if bom:
        ok(f"PROJECT_SCHEMA = {p} (tripla {linha.strip().rstrip(',')})")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--proximo", action="store_true")
    ap.add_argument("--raiz", default=os.path.dirname(AQUI))
    ap.add_argument("--apenas", default="adr,registos,cenas,schema")
    a = ap.parse_args()
    raiz = os.path.abspath(a.raiz)
    quais = set(a.apenas.split(","))
    prox_adr = adr(raiz) if "adr" in quais else None
    if "registos" in quais:
        registos(raiz)
    rot = cenas(raiz) if "cenas" in quais else {}
    if "schema" in quais:
        schema(raiz)
    if a.proximo:
        print("— próximo —")
        if prox_adr:
            print(f"  ADR livre: {prox_adr:04d}")
        unicos = 0
        for env, (n, onde, _) in sorted(rot.items()):
            if n is not None and n > 1:
                print(f"  {env}={n + 1}   ({onde})")
            elif n == 1:
                unicos += 1
        print(f"  (+ {unicos} roteadores de cena única: a próxima é =2)")
    print(f"✗ recontar: {len(falhas)} divergência(s)" if falhas else "✓ recontar: tudo conta")
    return 1 if falhas else 0


if __name__ == "__main__":
    sys.exit(main())
