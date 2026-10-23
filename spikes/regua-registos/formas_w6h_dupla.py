#!/usr/bin/env python3
"""As formas de diagnóstico da W6h §10 (as duas pipelines da fina) sobre o módulo do PADRÃO (o `v4.wgsl` que o
`medida_luz2d_regua_registos` grava): onde vai o custo das peças. Só `d0_igual` é de mesmo bit; as outras
saltam peças (a luz fica errada) para as localizar. A pasta de saída serve à régua do custo
(`PH2D_CANTOS_CUSTO_MODULOS=<saida>`, com `PH2D_CANTOS_CUSTO_SO=0,4,6`).

    python3 -I spikes/regua-registos/formas_w6h_dupla.py <pasta com v4.wgsl> <saida>
"""
import sys
from pathlib import Path

ENTRADA, SAIDA = Path(sys.argv[1]), Path(sys.argv[2])
SAIDA.mkdir(parents=True, exist_ok=True)
src = (ENTRADA / "v4.wgsl").read_text()


def troca(s, velho, novo):
    assert s.count(velho) == 1, velho
    return s.replace(velho, novo)


LINHAS = "    let nb = blocos_da_captura();\n    soma_prefixa(wid.x * nb.x, 1u, nb.x, lid, true);\n"
COLUNAS = "    let nb = blocos_da_captura();\n    soma_prefixa(wid.x, nb.x, nb.y, lid, false);\n"
MARCA = "    let gr = grupos_da_regiao();\n    let g = gid.x;\n    if g >= gr.x * gr.y {\n        return;\n    }\n    let nb = blocos_da_captura();\n"
MAPA = "    if fina_marcas_lidas.grupos[wid.y * grupos_da_regiao().x + wid.x] != 0u {\n        return;\n    }\n"
DMAX = "        for (var ty = t0.y; ty <= t1.y; ty++) {"


def sem_sat(s):
    return troca(troca(s, LINHAS, "    return;\n"), COLUNAS, "    return;\n")


def marca_vazia(s):
    return troca(s, MARCA, MARCA + "    return;\n")


formas = {
    "d0_igual": src,
    "d1_sem_sat": sem_sat(src),
    "d2_marca_vazia": marca_vazia(src),
    "d3_so_enxuta": marca_vazia(sem_sat(src)),
    "d4_enxuta_sem_mapa": troca(marca_vazia(sem_sat(src)), MAPA, ""),
    "d5_marca_sem_dmax": troca(src, DMAX, "        for (var ty = t0.y; ty < t0.y; ty++) {"),
}
for nome, f in formas.items():
    (SAIDA / f"{nome}.wgsl").write_text(f)
    print(nome, len(f))
