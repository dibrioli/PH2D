#!/usr/bin/env python3
"""O oráculo Krita 6 da BATERIA do EXAME: cada preset pinta bateria.json por eventos de caneta, num Krita PRÓPRIO.

Isolamento: perfil próprio (XDG_CONFIG/DATA/CACHE em ~/.cache/ph2d-oraculos/krita — o kritarc do dono não é tocado),
servidor X próprio (Xvfb num display livre a partir de :87; DISPLAY/WAYLAND_DISPLAY do dono removidos). O plugin
`ph2dbateria` (escrito aqui no perfil) corre `krita_bateria_dentro.py` quando a janela nasce. A pintura é por
`QWindowSystemInterface::handleTabletEvent` (pressão real, timestamps nossos); ver o docstring de dentro.

Depois: cada PNG exportada (RGBA) é composta sobre o papel e gravada RGB em fixtures/krita_bateria/, com
manifesto.json (sha256, px com tinta, origem do preset).

uso: python3 -I krita_bateria.py [--presets "b) Basic-5 Size,c) Pencil-2"] [--passo-ms 10] [--dt-ms 10]
                                  [--cru DIR] [--sem-fixtures] [--base "b) Basic-1"]
"""
import argparse
import hashlib
import json
import os
import subprocess
import sys
import time

import numpy as np
from PIL import Image

AQUI = os.path.dirname(os.path.abspath(__file__))
PERFIL = os.path.expanduser("~/.cache/ph2d-oraculos/krita")

PLUGIN_DESKTOP = """[Desktop Entry]
Type=Service
ServiceTypes=Krita/PythonPlugin
X-KDE-Library=ph2dbateria
X-Python-2-Compatible=false
Name=PH2D bateria
Comment=Oraculo da bateria do exame (PH2D) - corre o script de PH2D_KRITA_SCRIPT
"""
PLUGIN_INIT = """import os, runpy, traceback
from krita import Krita, Extension
from PyQt6.QtCore import QTimer

class Bateria(Extension):
    def setup(self):
        Krita.instance().notifier().windowCreated.connect(lambda: QTimer.singleShot(1500, self.corre))
    def corre(self):
        alvo = os.environ.get("PH2D_KRITA_SCRIPT")
        if not alvo:
            return
        try:
            runpy.run_path(alvo, run_name="__ph2d__")
        except Exception:
            traceback.print_exc()
            os._exit(3)
    def createActions(self, window):
        pass

Krita.instance().addExtension(Bateria(Krita.instance()))
"""


def prepara_perfil():
    for d in ("config", "data/krita/pykrita/ph2dbateria", "cache"):
        os.makedirs(os.path.join(PERFIL, d), exist_ok=True)
    open(os.path.join(PERFIL, "data/krita/pykrita/ph2dbateria.desktop"), "w").write(PLUGIN_DESKTOP)
    open(os.path.join(PERFIL, "data/krita/pykrita/ph2dbateria/__init__.py"), "w").write(PLUGIN_INIT)
    rc = os.path.join(PERFIL, "config/kritarc")
    txt = open(rc).read() if os.path.exists(rc) else ""
    if "enable_ph2dbateria=true" not in txt:
        open(rc, "a").write("\n[python]\nenable_ph2dbateria=true\n")


def xvfb():
    n = 87
    while os.path.exists(f"/tmp/.X11-unix/X{n}") or os.path.exists(f"/tmp/.X{n}-lock"):
        n += 1
    p = subprocess.Popen(["Xvfb", f":{n}", "-screen", "0", "2560x1600x24", "-nolisten", "tcp"],
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    for _ in range(100):
        if os.path.exists(f"/tmp/.X11-unix/X{n}"):
            return p, f":{n}"
        time.sleep(0.1)
    p.kill()
    raise RuntimeError("Xvfb não arrancou")


def corre_krita(job_path, display, prazo_s):
    env = {k: v for k, v in os.environ.items() if k not in ("WAYLAND_DISPLAY", "DISPLAY")}
    env.update(DISPLAY=display, QT_QPA_PLATFORM="xcb", XDG_CONFIG_HOME=f"{PERFIL}/config",
               XDG_DATA_HOME=f"{PERFIL}/data", XDG_CACHE_HOME=f"{PERFIL}/cache",
               PH2D_KRITA_SCRIPT=os.path.join(AQUI, "krita_bateria_dentro.py"), PH2D_KRITA_JOB=job_path)
    with open(os.path.join(PERFIL, "krita_stderr.txt"), "a") as err:
        try:
            return subprocess.run(["krita", "--nosplash"], env=env, stdout=err, stderr=err, timeout=prazo_s).returncode
        except subprocess.TimeoutExpired:
            return "prazo"


def compoe(cru_png, papel):
    im = np.asarray(Image.open(cru_png).convert("RGBA")).astype(np.float64) / 255.0
    a = im[..., 3:4]
    rgb = im[..., :3] * a + np.array(papel) / 255.0 * (1 - a)
    out = np.clip(np.round(rgb * 255), 0, 255).astype(np.uint8)
    tinta = int((np.abs(out.astype(int) - np.array(papel)).max(axis=2) > 2).sum())
    return out, tinta


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--presets", default="")
    ap.add_argument("--passo-ms", type=int, default=10)
    ap.add_argument("--dt-ms", type=int, default=10)
    ap.add_argument("--cru", default=os.path.join(PERFIL, "saida"))
    ap.add_argument("--saida", default=os.path.join(AQUI, "..", "fixtures", "krita_bateria"))
    ap.add_argument("--sem-fixtures", action="store_true")
    ap.add_argument("--base", default="", help="preset das faixas da cena sobre (omissão: o próprio)")
    ap.add_argument("--prazo-s", type=int, default=4 * 3600)
    a = ap.parse_args()
    bateria = os.path.join(AQUI, "bateria.json")
    bat = json.load(open(bateria))
    prepara_perfil()
    os.makedirs(a.cru, exist_ok=True)
    job = {"bateria": bateria, "saida": a.cru, "presets": [p for p in a.presets.split(",") if p] or None,
           "passo_ms": a.passo_ms, "dt_ms": a.dt_ms, "paira": 10,
           "base": a.base or None}
    job_path = os.path.join(a.cru, "job.json")
    json.dump(job, open(job_path, "w"), ensure_ascii=False)
    proc, disp = xvfb()
    try:
        for tentativa in range(5):  # retoma depois de um crash: o dentro salta o que já tem as 3 PNG
            rc = corre_krita(job_path, disp, a.prazo_s)
            log = open(os.path.join(a.cru, "krita_log.txt")).read() if os.path.exists(
                os.path.join(a.cru, "krita_log.txt")) else ""
            print(f"krita rc={rc} tentativa={tentativa}", flush=True)
            if log.rstrip().endswith("FIM"):
                break
    finally:
        proc.terminate()
        proc.wait()
    if a.sem_fixtures:
        return
    mapa = json.load(open(os.path.join(a.cru, "krita_presets.json")))
    os.makedirs(a.saida, exist_ok=True)
    mpath = os.path.join(a.saida, "manifesto.json")
    man = json.load(open(mpath)) if os.path.exists(mpath) else {"pinceis": {}}
    man.update(bateria_sha256=hashlib.sha256(open(bateria, "rb").read()).hexdigest(), passo_ms=a.passo_ms,
               dt_ms=a.dt_ms, krita=subprocess.run(["krita_version"], capture_output=True, text=True).stdout.strip())
    falta = []
    for nome, info in mapa.items():
        ent = {"ficheiro": info["ficheiro"], "origem": info["origem"], "base_sobre": a.base or nome}
        for cena in bat["cenas"]:
            cru = os.path.join(a.cru, f"{info['ficheiro']}_{cena['nome']}.png")
            if not os.path.exists(cru):
                falta.append(f"{nome}/{cena['nome']}")
                continue
            img, tinta = compoe(cru, bat["papel"])
            dst = os.path.join(a.saida, f"{info['ficheiro']}_{cena['nome']}.png")
            Image.fromarray(img, "RGB").save(dst, optimize=True)
            ent[cena["nome"]] = {"px_com_tinta": tinta,
                                 "sha256": hashlib.sha256(open(dst, "rb").read()).hexdigest()[:16]}
        man["pinceis"][nome] = ent
    man["falta"] = falta
    json.dump(man, open(mpath, "w"), indent=1, ensure_ascii=False, sort_keys=True)
    print(f"{len(mapa)} presets, {len(falta)} fotos em falta")


if __name__ == "__main__":
    sys.exit(main())
