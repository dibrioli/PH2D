#!/usr/bin/env python3
"""Resume as fixtures do «enquadrar tudo» (F8): por cena × partida × ação, o zoom final, as quatro
margens do conteúdo no ecrã (px), o tempo até a câmara assentar e a forma da curva (fracção do zoom
andada a 25/50/75 % do tempo). Uso: python3 -I resume.py <fixtures_dir> [cena...]"""
import json
import math
import sys
from pathlib import Path


def margens(e, vp):
    if not e:
        return None
    return (e["x0"] - vp["x"], vp["x"] + vp["w"] - e["x1"], e["y0"] - vp["y"], vp["y"] + vp["h"] - e["y1"])


def assenta(q, fim):
    """Instante (ms) do 1.º quadro a partir do qual a câmara já é a final, e a fracção andada de ln z."""
    t_ok = None
    for t, x, y, z in q:
        igual = abs(x - fim["x"]) < 1e-6 and abs(y - fim["y"]) < 1e-6 and abs(z - fim["z"]) < 1e-9
        if igual and t_ok is None:
            t_ok = t
        elif not igual:
            t_ok = None
    z0 = q[0][3]
    lz = math.log(fim["z"] / z0) if z0 > 0 and fim["z"] != z0 else None
    frac = []
    if lz and t_ok and t_ok > 0:
        for p in (0.25, 0.5, 0.75):
            alvo = q[0][0] + p * (t_ok - q[0][0])
            k = min(q, key=lambda s: abs(s[0] - alvo))
            frac.append(round(math.log(k[3] / z0) / lz, 3))
    return t_ok, frac


def main():
    d = Path(sys.argv[1])
    so = set(sys.argv[2:])
    for f in sorted(d.glob("*.json")):
        fx = json.loads(f.read_text())
        cena = fx["_cabecalho"]["cena"] if isinstance(fx["_cabecalho"]["cena"], str) else f.stem
        if so and cena not in so:
            continue
        print(f"\n## {cena}")
        for k, r in fx["corridas"].items():
            dep = r["depois"]
            cam = dep["camara"]
            vp = dep["viewport"]
            m = margens(dep.get("tudo_ecra"), vp)
            ms = margens(dep.get("selecao_ecra"), vp)
            t, frac = assenta(r["quadros"], cam)
            fm = lambda m: "—" if not m else "L%.1f R%.1f T%.1f B%.1f" % m
            moveu = (r["antes"]["x"], r["antes"]["y"], r["antes"]["z"]) != (cam["x"], cam["y"], cam["z"])
            print(f"{k:38s} z={cam['z']:.6g} tudo[{fm(m)}] sel[{fm(ms)}] "
                  f"{'MOVEU' if moveu else 'parada'} assenta={t if t is None else round(t)}ms curva={frac}")


if __name__ == "__main__":
    main()
