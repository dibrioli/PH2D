#!/usr/bin/env python3
"""O VectorCraft como ORÁCULO das booleanas e do Offset (prompt 49, W0).

Passo 1 (da raiz da worktree) — escrever o corpus NOSSO:
    bash scripts/ph2d-run.sh cargo test -p ph2d-vec-boolean --test it writes_the_corpus -- --ignored
Passo 2 — correr o vectorcraft-cli sobre ele e gravar a fixture do gate:
    python3 -I "docs/Vector Module/ferramentas/vectorcraft_2026-10-11/oraculo_vc.py" \\
        > crates/ph2d-vec-boolean/tests/it/fixtures/vectorcraft_oracle.tsv
O gate `oracle_vectorcraft::the_engine_agrees_with_the_vectorcraft_oracle` lê a fixture.
"""
import datetime, json, os, re, subprocess, sys, tempfile
from concurrent.futures import ThreadPoolExecutor

VC = os.environ.get("VC", os.path.expanduser("~/Apps/vectorcraft/squashfs-root/usr/bin/vectorcraft-cli"))
CORPUS = os.environ.get("VC_CORPUS", "target/vectorcraft/corpus")
OPS = ["unite", "minusFront", "intersect", "exclude"]
OFFSETS = ["0.1", "0.4", "-0.1"]  # unidades do documento; o SVG do corpus é ×100
JOINS = ["miter", "round", "bevel"]
TMP = tempfile.mkdtemp(prefix="oraculo_vc_")
PATH_RE = re.compile(r"<path\b([^>]*)>")
ATTR_RE = re.compile(r'([a-z-]+)="([^"]*)"')


def paths_of(svg):
    out = []
    for m in PATH_RE.finditer(svg):
        a = dict(ATTR_RE.findall(m.group(1)))
        out.append((a.get("fill-rule", "nonzero"), " ".join(a.get("d", "").split())))
    return out


def run(kind, case, op, param):
    src = os.path.join(CORPUS, kind, case + ".svg")
    out = os.path.join(TMP, f"{kind}_{case}_{op}_{param}.svg")
    if kind == "bool":
        cmd = [VC, "run", "--in", src, "--cmd", "select.all", "--cmd", f"object.pathfinder.{op}", "--export", out]
    else:
        params = json.dumps({"offset": float(param) * 100.0, "joins": op})
        cmd = [VC, "run", "--in", src, "--cmd", "select.all", "--cmd", "object.path.offsetPath",
               "--params", params, "--export", out]
    p = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
    ds = []
    if p.returncode == 0 and os.path.exists(out):
        ps = paths_of(open(out).read())
        # O Offset dele deixa o original e põe o resultado LOGO A SEGUIR; o Pathfinder substitui.
        ds = ps[1:] if kind == "offset" else ps
        os.remove(out)
    return kind, case, op, param, p.returncode, "|".join(f"{r}:{d}" for r, d in ds if d)


jobs = []
for f in sorted(os.listdir(os.path.join(CORPUS, "bool"))):
    jobs += [("bool", f[:-4], op, "-") for op in OPS]
for f in sorted(os.listdir(os.path.join(CORPUS, "offset"))):
    jobs += [("offset", f[:-4], j, d) for j in JOINS for d in OFFSETS]
with ThreadPoolExecutor(int(os.environ.get("JOBS", "6"))) as ex:
    rows = list(ex.map(lambda a: run(*a), jobs))
info = subprocess.run([VC, "--version"], capture_output=True, text=True).stdout.strip() or "vectorcraft-cli"
print(f"# origem: {info} (storytold, MIT OU Apache-2.0, commit fe8af48f) — {VC}")
print('# comando: python3 -I "docs/Vector Module/ferramentas/vectorcraft_2026-10-11/oraculo_vc.py" (corpus do passo 1)')
print(f"# data: {datetime.date.today().isoformat()}")
print("# tipo\tcaso\top|junção\tparam (d)\trc\tregra:d|regra:d…")
for r in rows:
    print("\t".join(str(x) for x in r))
