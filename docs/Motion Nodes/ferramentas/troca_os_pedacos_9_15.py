#!/usr/bin/env python3
"""troca_os_pedacos_9_15.py — os binários da rodada do doc 121 §9.15, de seguida, a partir da árvore com
TODOS os pedaços ligados (o commit `pedacos(...)`): cada binário liga só os seus pedaços (as constantes
dos outros a `false`) e as ablações mutilam o `base`. Cada troca tem a contagem da âncora conferida;
no fim (e numa falha) a árvore volta ao commit por `git checkout` e o `git diff` tem de sair vazio.

uso (dentro da worktree):  python3 "docs/Motion Nodes/ferramentas/troca_os_pedacos_9_15.py" <dir-dos-binarios> [nome…]
"""
import re
import shutil
import subprocess
import sys
from pathlib import Path

WGSL = "crates/ph2d-shape-gpu/src/contorno.wgsl"
RS = "crates/ph2d-shape-gpu/src/contorno.rs"
PASS = "crates/ph2d-shape-gpu/src/pass.rs"
FICHEIROS = [WGSL, RS, PASS]

# Os pedaços: as constantes que cada um liga (ficheiro, nome).
PEDACOS = {
    "A1a": [(WGSL, "AJUSTE_NA_CONTAGEM")],
    "A1b": [(WGSL, "TOTAL_NO_PERCURSO")],
    "B1": [(WGSL, "ARESTAS_COMPACTAS"), (RS, "ARESTAS_COMPACTAS")],
    "B2": [(WGSL, "JUNTA_UMA_POR_TROCO")],
    "c2": [(RS, "MEDE_NO_INICIO")],
    "D": [(PASS, "PREFIXO_POR_SUBGRUPO")],
}

# As ablações, sobre o `base` (todos os pedaços desligados): (ficheiro, âncora, troca, contagem).
ABLACOES = {
    # E1 — o percurso tracejado sem emitir pedaço nenhum (as voltas ficam).
    "E1": [(WGSL, "                emite_pedaco(it, lin, t, caneta, tr, p);\n", "", 1),
           (WGSL, "            emite_pedaco(it, lin, t, caneta, tr, p);\n        }\n    }\n}\n",
            "        }\n    }\n}\n", 1)],
    # E2 — o ajuste `= 1` (sem a volta dele).
    "E2": [(WGSL, "                ajuste = ajuste_do_tracejado(cp.eixo_rg.x, cp.eixo_rg.y, cp.lin, cp.t, caneta);\n",
            "                ajuste = 1.0;\n", 1)],
    # E3 — a escrita sem gravar arestas (só o cursor e a caixa).
    "E3": [(WGSL, "        contorno_rw[base_saida + cursor] = vec4<f32>(p0, p1);\n", "", 1)],
    # C1 — a contagem sem o arco (a corda).
    "C1": [(WGSL, "            let len = arco(it, cp.lin, cp.t);\n",
            "            let len = comprimento(cp.lin, cp.t, it.a, it.b);\n", 1)],
    # P0 — o pedaço sem as duas arestas de ponta (o `cs_deposita` sem elas).
    "P0": [(WGSL, "    let q0 = q.xy + m0;\n    let q1 = q.zw + m1;\n    let q2 = q.zw - m1;\n    let q3 = q.xy - m0;\n"
                  "    let s = positivo(q0, q1, q2);\n    if !faixa0 {\n        aresta(q3, q0, s);\n    }\n"
                  "    aresta(q0, q1, s);\n    aresta(q2, q3, s);\n    if !faixa1 {\n        aresta(q1, q2, s);\n    }\n",
            "    let q0 = q.xy + m0;\n    let q1 = q.zw + m1;\n    let q2 = q.zw - m1;\n    let q3 = q.xy - m0;\n"
            "    let s = positivo(q0, q1, q2);\n    aresta(q0, q1, s);\n    aresta(q2, q3, s);\n", 1)],
    # V0 — o `cs_varre` sem o prefixo (o tecto da alavanca do (d), §9.14).
    "V0": [(WGSL, "    prefixo[li] = depositos_do_pixel(g, viva);\n    for (var d = 1u; d < PIXELS_DA_CELULA; d *= 2u) {\n"
                  "        workgroupBarrier();\n        var t = vec3<i32>(0);\n        if p >= d {\n"
                  "            t = prefixo[li - d];\n        }\n        workgroupBarrier();\n        prefixo[li] += t;\n    }\n",
            "    prefixo[li] = depositos_do_pixel(g, viva);\n", 1)],
}

TODOS = list(PEDACOS)
BINARIOS = {
    "base": ([], []),
    "E1": ([], ["E1"]), "E2": ([], ["E2"]), "E3": ([], ["E3"]), "C1": ([], ["C1"]),
    "P0": ([], ["P0"]), "V0": ([], ["V0"]),
    "A1a": (["A1a"], []), "A1b": (["A1b"], []), "A1": (["A1a", "A1b"], []),
    "B1": (["B1"], []), "B2": (["B2"], []), "D": (["D"], []), "c2": (["c2"], []),
    "F": (TODOS, []),
}


def troca(texto, ancora, nova, n, onde):
    achadas = texto.count(ancora)
    assert achadas == n, f"{onde}: a âncora casa {achadas}× (esperado {n}): {ancora[:60]!r}"
    return texto.replace(ancora, nova)


def aplica(ligados, ablacoes):
    textos = {f: Path(f).read_text() for f in FICHEIROS}
    for nome, consts in PEDACOS.items():
        if nome in ligados:
            continue
        for f, const in consts:
            padrao = re.compile(rf"const {const}: bool = true;")
            achadas = len(padrao.findall(textos[f]))
            assert achadas == 1, f"{f}: a constante {const} casa {achadas}×"
            textos[f] = padrao.sub(f"const {const}: bool = false;", textos[f])
    for a in ablacoes:
        for f, ancora, nova, n in ABLACOES[a]:
            textos[f] = troca(textos[f], ancora, nova, n, f"{a} em {f}")
    for f, t in textos.items():
        Path(f).write_text(t)


def compila():
    r = subprocess.run(
        ["bash", "scripts/ph2d-run.sh", "cargo", "test", "-p", "ph2d-app-motion", "--lib", "--release", "--no-run"],
        capture_output=True, text=True)
    saida = r.stdout + r.stderr
    if r.returncode != 0:
        print(saida[-4000:], file=sys.stderr)
        raise SystemExit(f"a compilação falhou (exit {r.returncode})")
    bins = re.findall(r"target/release/deps/ph2d_app_motion-[0-9a-f]+", saida)
    assert bins, "nenhum binário na saída do cargo"
    return bins[-1]


def main():
    destino = Path(sys.argv[1])
    destino.mkdir(parents=True, exist_ok=True)
    nomes = sys.argv[2:] or list(BINARIOS)
    assert subprocess.run(["git", "diff", "--quiet", "--", *FICHEIROS]).returncode == 0, "a árvore não está limpa"
    try:
        for nome in nomes:
            ligados, ablacoes = BINARIOS[nome]
            subprocess.run(["git", "checkout", "--", *FICHEIROS], check=True)
            aplica(ligados, ablacoes)
            b = compila()
            shutil.copy2(b, destino / nome)
            print(f"{nome}: {b} -> {destino / nome}", flush=True)
    finally:
        subprocess.run(["git", "checkout", "--", *FICHEIROS], check=True)
        assert subprocess.run(["git", "diff", "--quiet", "--", *FICHEIROS]).returncode == 0, "a árvore não voltou"
        print("árvore reposta", flush=True)


if __name__ == "__main__":
    main()
