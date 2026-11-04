#!/usr/bin/env python3
"""Tempo por job e passo de corridas do CI, com o MÍNIMO por grupo (A×B em corridas reais).

    python3 scripts/ci-passos.py 41791251163                 # uma corrida
    python3 scripts/ci-passos.py A:111 A:222 B:333 B:444     # mínimo por passo em cada variante

Método: docs/DevOps/MEDIR_VELOCIDADE.md §CI. Só mostra passos ≥ 0,5 min.
"""
import collections
import datetime as d
import json
import subprocess
import sys


def t(s):
    return d.datetime.fromisoformat(s.replace("Z", "+00:00"))


def jobs(run):
    o = subprocess.run(["gh", "run", "view", run, "--json", "jobs"], capture_output=True, text=True, check=True).stdout
    return json.loads(o)["jobs"]


def main():
    grupos = collections.OrderedDict()
    for a in sys.argv[1:]:
        g, _, r = a.rpartition(":")
        grupos.setdefault(g or r, []).append(r)
    tab = collections.defaultdict(dict)  # (job, passo) -> {grupo: [min...]}
    for g, runs in grupos.items():
        for r in runs:
            for j in jobs(r):
                if not (j.get("startedAt") and j.get("completedAt")) or j.get("conclusion") == "skipped":
                    continue
                m = (t(j["completedAt"]) - t(j["startedAt"])).total_seconds() / 60
                tab[(j["name"], "· job inteiro")].setdefault(g, []).append(m)
                for s in j["steps"]:
                    if s.get("startedAt") and s.get("completedAt") and s.get("conclusion") != "skipped":
                        m = (t(s["completedAt"]) - t(s["startedAt"])).total_seconds() / 60
                        tab[(j["name"], s["name"][:60])].setdefault(g, []).append(m)
    nomes = list(grupos)
    print("min".rjust(7) * len(nomes) + "  job · passo  (" + " ".join(nomes) + ")")
    for (job, passo), v in sorted(tab.items()):
        if max(min(x) for x in v.values()) < 0.5:
            continue
        cols = "".join(f"{min(v[g]):7.1f}" if g in v else "      -" for g in nomes)
        print(f"{cols}  {job[:34]} · {passo}")


if __name__ == "__main__":
    main()
