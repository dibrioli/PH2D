"""Os materiais da régua E3-B / W3: UMA lista para os dois lados — o `cycles.py` (o Principled com o MESMO
valor) e a sonda `medida_oleo_real_oraculo` (`PH2D_OLEO_VARIANTES`, o cartão Real Oil `mat_*`).

    python3 docs/Painter/49_realshader/oraculo/materiais_e3.py [nome,…]   # imprime o PH2D_OLEO_VARIANTES

W3 (27/10): os valores são os ABSOLUTOS do cartão (os 8 do Principled simplificado, `MaterialCartao::CAMPOS`),
sobre a célula NEUTRA. Saíram Spec tint, Anisotropy e Translucency (a Subsurface inclui a volta do papel).
`subsuperficie` = Subsurface Weight; `raio_subsuperficie` = px de tela → Subsurface Scale do Blender em células
(÷ `ratio`), com o Subsurface Radius de omissão `(1; 0,2; 0,1)`. `_luz` = a variante de luz do `cycles.py`
(de omissão `studio_sol10`, a de fábrica: Estúdio a 0,6).
"""

import sys

MATERIAIS = {
    "neutro": {},
    "metal1": {"metal": 1.0},
    "metal05": {"metal": 0.5},
    "rug05": {"rugosidade": 0.14},
    "rug2": {"rugosidade": 0.56},
    "esp0": {"nivel_ior": 0.0},
    "esp2": {"nivel_ior": 1.0},
    "verniz1": {"verniz": 1.0},
    "verniz1_r10": {"verniz": 1.0, "verniz_rugosidade": 0.3},
    "veludo1": {"veludo": 1.0},
    "sss05_r4": {"subsuperficie": 0.5, "raio_subsuperficie": 4.0},
    "sss1_r4": {"subsuperficie": 1.0, "raio_subsuperficie": 4.0},
    "sss05_r16": {"subsuperficie": 0.5, "raio_subsuperficie": 16.0},
    "sss1_r16": {"subsuperficie": 1.0, "raio_subsuperficie": 16.0},
    "sss025_r4": {"subsuperficie": 0.25, "raio_subsuperficie": 4.0},
    "sss075_r4": {"subsuperficie": 0.75, "raio_subsuperficie": 4.0},
    "sss1_r1": {"subsuperficie": 1.0, "raio_subsuperficie": 1.0},
    "sss001_r4": {"subsuperficie": 0.01, "raio_subsuperficie": 4.0},
    "neutro_hdri": {"_luz": "hdri_studio"},
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
    print(variantes(sys.argv[1].split(",") if len(sys.argv) > 1 else None))
