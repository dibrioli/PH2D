#!/usr/bin/env python3
"""Resume as fixtures do oráculo numa tabela: o que o Xournal++ devolveu para cada traço nosso, em
GEOMETRIA (não em pontos) — linha: extremos; polígono: cantos e ângulos; círculo: centro, raio e
resíduo do ajuste. É o que o gate do PH2D compara por passo. Uso: resume_fixtures.py [fixtures_dir]"""
import glob
import math
import os
import sys

FIX = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(__file__), "fixtures")


def carrega(p):
    hdr, ins, outs = {}, [], []
    for l in open(p, encoding="utf-8"):
        l = l.strip()
        if l.startswith("# ") and ":" in l and not l.startswith("# ---"):
            k, v = l[2:].split(":", 1)
            hdr[k.strip()] = v.strip()
        elif l.startswith("in "):
            ins.append(tuple(map(float, l.split()[1:])))
        elif l.startswith("out "):
            outs.append(tuple(map(float, l.split()[1:])))
    return hdr, ins, outs


def ajusta_circulo(pts):
    """Kåsa (mínimos quadrados algébricos): centro, raio, resíduo rms (unidades de página)."""
    n = len(pts)
    sx = sum(x for x, _ in pts) / n
    sy = sum(y for _, y in pts) / n
    u = [(x - sx, y - sy) for x, y in pts]
    suu = sum(a * a for a, _ in u); svv = sum(b * b for _, b in u); suv = sum(a * b for a, b in u)
    suuu = sum(a ** 3 for a, _ in u); svvv = sum(b ** 3 for _, b in u)
    suvv = sum(a * b * b for a, b in u); svuu = sum(b * a * a for a, b in u)
    det = suu * svv - suv * suv
    if abs(det) < 1e-9:
        return None
    r1 = (suuu + suvv) / 2; r2 = (svvv + svuu) / 2
    uc = (r1 * svv - r2 * suv) / det; vc = (suu * r2 - suv * r1) / det
    r = math.sqrt(uc * uc + vc * vc + (suu + svv) / n)
    cx, cy = uc + sx, vc + sy
    rms = math.sqrt(sum((math.hypot(x - cx, y - cy) - r) ** 2 for x, y in pts) / n)
    return cx, cy, r, rms


def angulos(pts):
    """Ângulos internos (graus) de uma polilinha fechada de cantos (último == primeiro)."""
    c = pts[:-1] if pts[0] == pts[-1] else pts
    out = []
    for i in range(len(c)):
        a, b, d = c[i - 1], c[i], c[(i + 1) % len(c)]
        v1 = (a[0] - b[0], a[1] - b[1]); v2 = (d[0] - b[0], d[1] - b[1])
        dot = v1[0] * v2[0] + v1[1] * v2[1]
        n1 = math.hypot(*v1); n2 = math.hypot(*v2)
        out.append(math.degrees(math.acos(max(-1, min(1, dot / (n1 * n2))))) if n1 and n2 else 0.0)
    return out


print(f"{'traço':28s} {'in':>4s} {'out':>4s}  veredito do oráculo")
for p in sorted(glob.glob(os.path.join(FIX, "*.txt"))):
    hdr, ins, outs = carrega(p)
    nome = os.path.basename(p)[:-4]
    rec = hdr.get("reconhecido", "").startswith("sim")
    if not rec:
        v = "cru"
    elif len(outs) == 2:
        (ax, ay), (bx, by) = outs
        v = f"LINHA {ax:.1f},{ay:.1f} → {bx:.1f},{by:.1f} · comprimento {math.hypot(bx-ax, by-ay):.1f}"
    elif len(outs) <= 9:
        fech = "fechado" if outs[0] == outs[-1] else "aberto"
        ang = " ".join(f"{a:.0f}°" for a in angulos(outs))
        lados = len(outs) - 1 if fech == "fechado" else len(outs) - 1
        v = f"POLÍGONO {lados} lados {fech} · ângulos {ang}"
    else:
        c = ajusta_circulo(outs)
        fech = "fechado" if math.hypot(outs[0][0]-outs[-1][0], outs[0][1]-outs[-1][1]) < 1e-6 else f"aberto ({math.hypot(outs[0][0]-outs[-1][0], outs[0][1]-outs[-1][1]):.1f})"
        v = f"CÍRCULO centro {c[0]:.1f},{c[1]:.1f} raio {c[2]:.2f} rms {c[3]:.3f} · {len(outs)} pts {fech}" if c else "?"
    print(f"{nome:28s} {len(ins):4d} {len(outs):4d}  {v}")
