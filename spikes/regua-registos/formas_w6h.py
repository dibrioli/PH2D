#!/usr/bin/env python3
"""As formas de diagnóstico da W6h §6 sobre o módulo do PADRÃO (o `v4.wgsl` que o `medida_luz2d_regua_registos`
grava). Cada forma é um `.wgsl` + o `.txt` da régua de registos; a mesma pasta serve à régua do custo
(`PH2D_CANTOS_CUSTO_MODULOS=<saida>`). Nenhuma é de mesmo bit salvo `x0`: são para localizar o custo.

    python3 -I spikes/regua-registos/formas_w6h.py <pasta com v4.wgsl e v4.txt> <saida>
"""
import re
import sys
from pathlib import Path

ENTRADA, SAIDA = Path(sys.argv[1]), Path(sys.argv[2])
SAIDA.mkdir(parents=True, exist_ok=True)
src = (ENTRADA / "v4.wgsl").read_text()
txt = (ENTRADA / "v4.txt").read_text()
assert "modulo v4.wgsl" in txt

# O meio e o ramo da cunha dele (de `let e = cantos_tabela[` ao `else if d < 0.0` que fecha o `if e != 0u`).
k = src.index("fn meio(pc: vec2<f32>) -> Meio {")
k2 = src.index("\n}\n", k) + 3
meio = src[k:k2]
i = meio.index("            let e = cantos_tabela[")
fim = "            } else if d < 0.0 {\n                return m;\n            }\n"
ramo = meio[i : meio.index(fim, i) + len(fim)]
assert meio.count(ramo) == 1 and src.count(ramo) == 1
linha_e = ramo.splitlines(True)[0]
SO_D = "            if d < 0.0 {\n                return m;\n            }\n"
atlas = (
    "                let k = e - 1u;\n"
    "                let a = vec2<i32>(i32(k % 128u), i32(k / 128u)) * 8 + (t & vec2<i32>(7));\n"
    "                let w = textureLoad(cantos, a, 0);\n"
)
assert atlas in ramo

# Um irmão `meio_sem` (o meio SEM o ramo) e `num_corpo_sem`, mortos até alguém os chamar.
sem = meio.replace("fn meio(", "fn meio_sem(").replace(ramo, SO_D)
NC = "fn num_corpo(pc: vec2<f32>) -> bool {\n    return meio(pc).tau < 1.0;\n}"
assert src.count(NC) == 1
base = (src[:k2] + sem + src[k2:]).replace(
    NC, NC + "\n\nfn num_corpo_sem(pc: vec2<f32>) -> bool {\n    return meio_sem(pc).tau < 1.0;\n}"
)


def funcoes(s):
    return [(m.start(), m.group(1)) for m in re.finditer(r"\nfn (\w+)\(", s)]


def troca(s, chamada, nova, dentro):
    """Troca as chamadas `chamada(` cuja função envolvente satisfaz `dentro(nome)` por `nova(`."""
    fs = funcoes(s)
    out, last, n = [], 0, 0
    for m in re.finditer(r"\b" + chamada + r"\(", s):
        if s[m.start() - 3 : m.start()] == "fn ":
            continue
        if dentro([f for p, f in fs if p < m.start()][-1]):
            out += [s[last : m.start()], nova + "("]
            last, n = m.end(), n + 1
    out.append(s[last:])
    assert n > 0, (chamada, nova)
    return "".join(out)


formas = {
    "x0_head": src,
    "x9_sem_ramo": src.replace(ramo, SO_D),
    # o ramo só na marcha (os outros sítios do meio sem ele) · só fora dela
    "x4_ramo_so_na_marcha": troca(base, "meio", "meio_sem", lambda f: f not in ("marchar_ate", "meio_sem")),
    "x5_ramo_fora_da_marcha": troca(base, "meio", "meio_sem", lambda f: f == "marchar_ate"),
    # o num_corpo sem o ramo (o predicado de geometria)
    "ga_num_corpo": troca(base, "meio", "meio_sem", lambda f: f == "num_corpo"),
}
# o num_corpo sem o ramo só num chamador de cada vez
for nome, g in {
    "h_na_fronteira": {"na_fronteira"},
    "h_na_fronteira_fina": {"na_fronteira_fina"},
    "h_pe_na_face": {"pe_na_face"},
    "h_pe_de_um_lado": {"pe_de_um_lado"},
    "h_pe": {"pe_na_borda", "cruzamento", "normal_secante", "saida_na_direcao", "fora_do_opaco"},
}.items():
    formas[nome] = troca(base, "num_corpo", "num_corpo_sem", lambda f, g=g: f in g)
# as peças do ramo no num_corpo: só a tabela · + o atlas (a decisão sempre `d < 0`)
for nome, corpo in {
    "gt_tabela": linha_e + "            if e == 0xffffffffu || d < 0.0 {\n                return m;\n            }\n",
    "gs_atlas": linha_e
    + "            if e != 0u {\n"
    + atlas
    + "                if w.x == -7.0 || d < 0.0 {\n                    return m;\n                }\n"
    + "            } else if d < 0.0 {\n                return m;\n            }\n",
}.items():
    m2 = meio.replace("fn meio(", "fn meio_nc(").replace(ramo, corpo)
    s = (src[:k2] + m2 + src[k2:]).replace(NC, NC.replace("meio(pc)", "meio_nc(pc)"))
    formas[nome] = s

for nome, s in formas.items():
    (SAIDA / f"{nome}.wgsl").write_text(s)
    (SAIDA / f"{nome}.txt").write_text(txt.replace("modulo v4.wgsl", f"modulo {nome}.wgsl"))
print(" ".join(sorted(formas)))
