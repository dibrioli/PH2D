#!/usr/bin/env python3
"""O VectorCraft como ORÁCULO do Shape Builder (prompt 49, W4): as faces do mapa planar dele.

Passo 1 (da raiz da worktree) — o corpus NOSSO (os `bool/` do W0 e os `build/`):
    bash scripts/ph2d-run.sh cargo test -p ph2d-vec-boolean --test it writes_the_corpus -- --ignored
    bash scripts/ph2d-run.sh cargo test -p ph2d-vec-boolean --test it writes_the_build_corpus -- --ignored
Passo 2:
    python3 -I "docs/Vector Module/ferramentas/vectorcraft_2026-10-11/oraculo_vc_build.py" \\
        > crates/ph2d-vec-boolean/tests/it/fixtures/vectorcraft_build_regions.tsv

Saída: `caso  m:área,…` — `m` é a pertinência em bits pela ordem do ficheiro (= ordem de z, fundo
primeiro), a área em unidades do DOCUMENTO (o SVG do corpus é ×100).
"""
import json, os, subprocess
from concurrent.futures import ThreadPoolExecutor

VC = os.environ.get("VC", os.path.expanduser("~/Apps/vectorcraft/squashfs-root/usr/bin/vectorcraft-cli"))
CORPUS = os.environ.get("VC_CORPUS", "target/vectorcraft/corpus")


def regions(rel):
    p = subprocess.run([VC, "run", "--in", os.path.join(CORPUS, rel + ".svg"), "--cmd", "select.all",
                        "--cmd", "shapeBuilder.regions"], capture_output=True, text=True, timeout=300)
    res = [json.loads(l) for l in p.stdout.splitlines() if l.startswith("{")][-1]["result"]
    ids = sorted({s for r in res["regions"] for s in r["sources"]}
                 | {e["source"] for e in res["edges"]} | {l["source"] for l in res["lines"]})
    faces = []
    for r in res["regions"]:
        m = sum(1 << ids.index(s) for s in r["sources"])
        faces.append((m, r["area"] / 1e4))
    return rel, sorted(faces)


cases = [f"{d}/{f[:-4]}" for d in ("bool", "build") for f in sorted(os.listdir(os.path.join(CORPUS, d)))]
with ThreadPoolExecutor(int(os.environ.get("JOBS", "6"))) as ex:
    rows = list(ex.map(regions, cases))
info = subprocess.run([VC, "--version"], capture_output=True, text=True).stdout.strip() or "vectorcraft-cli"
print(f"# origem: {info} (storytold, MIT OU Apache-2.0) — {VC}")
print('# comando: python3 -I "docs/Vector Module/ferramentas/vectorcraft_2026-10-11/oraculo_vc_build.py" (corpus do passo 1)')
print("# caso\tpertinência:área (documento) de cada face do shapeBuilder.regions")
for rel, faces in rows:
    print(rel + "\t" + ",".join(f"{m}:{a:.6f}" for m, a in faces))
