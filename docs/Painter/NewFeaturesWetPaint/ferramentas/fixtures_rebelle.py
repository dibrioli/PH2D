#!/usr/bin/env python3
"""Copia as fotos do Rebelle (recorte 1000×640 a zoom 100 %) para fixtures/rebelle/ e escreve o
cabeçalho de cada uma a partir dos eventos que a pintaram (os NOSSOS gestos, `gera_eventos.py`).

uso: fixtures_rebelle.py <dir do oráculo> <dir fixtures/rebelle>
Só a saída do Rebelle sobre entradas nossas: nenhuma textura, preset ou ficheiro dele é copiado.
"""
import hashlib
import json
import shutil
import sys
from pathlib import Path

# Sessão de 2026-10-07 (as 12 primeiras) e de 2026-10-11 (13 presets de fábrica com a água e a opacidade
# do próprio preset — sem `water`/`opacity` no SET_BRUSH). ⚠️ «Pastel Oil/Oil Pastel» saiu quase em branco sob
# o Wine (0,83 % de tinta: só as pontas da rampa de pressão) ⇒ entra o vizinho «Oil Pastel 2» (`oilpastel2`).
NOMES = ["oil", "express", "wc", "wcpreset", "wcwet", "ink", "pencil", "penciltilt",
         "pastel", "marker", "airbrush", "oildirty",
         "wcround", "wcwash", "wcflat", "wcliner", "gouache", "gouacheflat", "sumi",
         "dippen", "inkbrush", "oilround", "charcoal", "oilpastel2", "markerbullet"]


def cabecalho(ev_path):
    ev = json.loads(Path(ev_path).read_text())["frames"][0]["events"]
    art = next(e for e in ev if e["event_type"] == "NEW_ARTWORK")
    seed = next((e["seed"] for e in ev if e["event_type"] == "SET_RANDOM_SEED"), None)
    brush = next(e for e in ev if e["event_type"] == "SET_BRUSH")
    sim = [e for e in ev if e["event_type"] == "SIMULATION"]
    extras = {k: v for k, v in brush.items() if k not in ("event_type", "tool", "preset", "size_px", "pressure", "color", "size")}
    # A inclinação viaja no PONTEIRO, não no pincel.
    tilts = {(e["pen_tilt"]["x"], e["pen_tilt"]["y"]) for e in ev if "pen_tilt" in e}
    if tilts - {(0.0, 0.0)}:
        extras["pen_tilt"] = sorted(tilts)
    return {
        "tool": brush["tool"], "preset": brush["preset"], "size_px": brush["size_px"],
        "size_slider": brush.get("size"), "pressure": brush["pressure"], "extras": extras,
        "seed": seed, "paper": art["paper"]["preset"], "canvas": [art["width"], art["height"]],
        "simulation": [e.get("repeats", e.get("steps", e.get("frames"))) for e in sim],
        "eventos": {t: sum(1 for e in ev if e["event_type"] == t) for t in sorted({e["event_type"] for e in ev})},
    }


def main():
    src, dst = Path(sys.argv[1]), Path(sys.argv[2])
    dst.mkdir(parents=True, exist_ok=True)
    linhas = ["# Fotos do Rebelle 8.3.4 sobre os gestos da bancada (sessões de 2026-10-07 e 2026-10-11)", "",
              "Recorte 1000×640 da janela a zoom 100 % (`SAVE` não existe por WebSocket), estado A "
              "(depois dos traços + `SIMULATION`). Gerado por `ferramentas/fixtures_rebelle.py`; "
              "as medidas estão no [03 §2](../../03_tracos_medidos_rebelle_x_ph2d.md). As 12 primeiras linhas são de "
              "07/10; as 13 seguintes (de `wcround` a `markerbullet`), de 11/10: presets de fábrica com a água e a "
              "opacidade do próprio preset. «Pastel Oil/Oil Pastel» saiu quase em branco (só as pontas da rampa "
              "de pressão) e foi substituído por «Oil Pastel 2».", "",
              "| ficheiro | ferramenta | preset | size_px | pressão | extras | semente | papel | sha256 (16) |",
              "|---|---|---|---|---|---|---|---|---|"]
    meta = {}
    for n in NOMES:
        png, ev = src / f"c_ws_{n}_A.png", src / f"ev_{n}_A.json"
        h = cabecalho(ev)
        out = dst / f"{n}.png"
        shutil.copyfile(png, out)
        sha = hashlib.sha256(out.read_bytes()).hexdigest()
        h["sha256"] = sha
        meta[n] = h
        ex = ", ".join(f"{k} {v}" for k, v in h["extras"].items()) or "—"
        linhas.append(f"| `{n}.png` | {h['tool']} | {h['preset']} | {h['size_px']:g} | {h['pressure']} | {ex} | "
                      f"{h['seed']} | {h['paper']} | `{sha[:16]}` |")
    (dst / "cabecalhos.json").write_text(json.dumps(meta, indent=1, ensure_ascii=False) + "\n")
    (dst / "README.md").write_text("\n".join(linhas) + "\n")
    print(f"{len(NOMES)} fotos → {dst}")


if __name__ == "__main__":
    main()
