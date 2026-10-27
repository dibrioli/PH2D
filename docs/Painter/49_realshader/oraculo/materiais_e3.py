"""Os materiais da régua E3-B (W2, 27/10): UMA lista para os dois lados — o `cycles.py` (o Principled com o
MESMO valor) e a sonda `medida_oleo_real_oraculo` (`PH2D_OLEO_VARIANTES`, o cartão Real Oil `mat_*`).

    python3 docs/Painter/49_realshader/oraculo/materiais_e3.py      # imprime o PH2D_OLEO_VARIANTES

Os valores são os do CARTÃO (multiplicadores para rugosidade/specular/rugosidade do verniz, sobre a célula
NEUTRA: rugosidade 0,28 × m, Specular 0,5 × m, verniz 0,03 × m). `_luz` = a variante de luz do `cycles.py`
(de omissão `studio_sol10`, a de fábrica: Estúdio a 0,6).
"""

MATERIAIS = {
    "neutro": {},
    "metal1": {"metal": 1.0},
    "metal05": {"metal": 0.5},
    "rug05": {"rugosidade": 0.5},
    "rug2": {"rugosidade": 2.0},
    "esp0": {"especular": 0.0},
    "esp2": {"especular": 2.0},
    "tint1": {"tinta_reflexo": 1.0},
    "verniz1": {"verniz": 1.0},
    "verniz1_r10": {"verniz": 1.0, "verniz_rugosidade": 10.0},
    "veludo1": {"veludo": 1.0},
    "aniso08": {"anisotropia": 0.8},
    "transl1": {"translucidez": 1.0},
    "neutro_hdri": {"_luz": "hdri_studio"},
    "transl1_hdri": {"translucidez": 1.0, "_luz": "hdri_studio"},
}

# A luz da sonda que corresponde a cada `_luz` do Cycles.
AF_DA_LUZ = {"studio_sol10": "", "hdri_studio": "forca_ambiente=1"}


def luz(m):
    return MATERIAIS[m].get("_luz", "studio_sol10")


def variantes(so=None):
    partes = []
    for nome, m in MATERIAIS.items():
        if so and nome not in so:
            continue
        af = [f"mat_{k}={v}" for k, v in m.items() if not k.startswith("_")]
        if AF_DA_LUZ[luz(nome)]:
            af.append(AF_DA_LUZ[luz(nome)])
        partes.append(f"{nome}:{','.join(af)}")
    return ";".join(partes)


if __name__ == "__main__":
    print(variantes())
