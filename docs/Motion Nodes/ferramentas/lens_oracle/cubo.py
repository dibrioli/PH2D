#!/usr/bin/env python3
"""A LUT NOSSA dos oráculos da lente (doc 129 §5.1) — `cubo.py <saida.cube>`.

Um `.cube` de 9³ (pequeno para a fixture viver no repo; 17 e 33 são os tamanhos comuns) (spec *Adobe Cube LUT 1.0*), com mistura ENTRE canais de propósito: uma LUT que
só mexe canal a canal dá a mesma resposta em trilinear e tetraédrico, e o gate não distinguiria as
duas interpolações. A função é um look «teal & orange» com contraste em S.
"""
import sys

N = 9


def look(r, g, b):
    luma = 0.2126 * r + 0.7152 * g + 0.0722 * b
    s = lambda v: v * v * (3.0 - 2.0 * v)
    t = luma - 0.5
    return (
        min(1.0, max(0.0, s(r) + 0.20 * t + 0.10 * (g - b))),
        min(1.0, max(0.0, 0.85 * g + 0.15 * s(luma))),
        min(1.0, max(0.0, s(b) - 0.20 * t + 0.08 * (r - g) * b)),
    )


if __name__ == "__main__":
    with open(sys.argv[1], "w") as f:
        f.write('TITLE "ph2d oraculo da lente"\n')
        f.write(f"LUT_3D_SIZE {N}\n")
        f.write("DOMAIN_MIN 0.0 0.0 0.0\nDOMAIN_MAX 1.0 1.0 1.0\n")
        # Ordem da spec: o VERMELHO varia mais depressa, depois o verde, depois o azul.
        for bi in range(N):
            for gi in range(N):
                for ri in range(N):
                    o = look(ri / (N - 1), gi / (N - 1), bi / (N - 1))
                    f.write(f"{o[0]:.6f} {o[1]:.6f} {o[2]:.6f}\n")
