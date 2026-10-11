#!/usr/bin/env python3
# ⭐ CORRE O ORÁCULO DAS ALÇAS DAS FORMAS — o Inkscape 1.4.4 (GPL: caixa preta, só a SAÍDA é gravada)
# numa tela INVISÍVEL (Xvfb), com o rato sintético (xdotool) a arrastar as alças das formas NOSSAS.
# Cada família (rectângulo, estrela, polígono, arco, espiral — alinhadas e rodadas) abre um documento
# com UMA forma; cada experiência = seleccionar com a ferramenta da forma → premir a alça onde a
# LEI prevista a põe → arrastar (com ou sem Shift/Ctrl) → soltar → Ctrl+S → ler os atributos → Ctrl+Z.
# Saída: `fixtures/<família>.json`, com cabeçalho (versão, calibração tela→documento, a forma de origem).
#
# uso:  python3 -I corre_inkscape.py [familia ...]      (sem argumentos: todas)
# requer: inkscape, xorg-server-xvfb, xdotool.
#
# ⚠️ MEDIDO (11/10) para o roteiro funcionar: a janela nasce minúscula (990×276) — `windowsize` e a
# tecla `5` (encaixar a página); o Ctrl+S sobre um `.svg` grava sem diálogo; os nós desenham-se com a
# forma seleccionada pela ferramenta DELA (r · * · e · i), e a selecção sobrevive à troca de ferramenta.
import json, math, os, shutil, subprocess, sys, time, xml.etree.ElementTree as ET

AQUI = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(AQUI, "fixtures")
DISP = ":96"
TMP = "/dev/shm/oraculo_alcas.%d" % os.getpid()
NS = {"sodipodi": "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd",
      "inkscape": "http://www.inkscape.org/namespaces/inkscape"}
SOD = "{%s}" % NS["sodipodi"]
INK = "{%s}" % NS["inkscape"]

CAB = ('<?xml version="1.0" encoding="UTF-8"?>\n<svg xmlns="http://www.w3.org/2000/svg" '
       'xmlns:sodipodi="%s" xmlns:inkscape="%s" width="800" height="600" viewBox="0 0 800 600">\n'
       '<sodipodi:namedview id="nv" inkscape:document-units="px"/>\n' % (NS["sodipodi"], NS["inkscape"]))
EST = 'style="fill:#88aaff;stroke:#000000;stroke-width:2"'


def x(*a):
    subprocess.run(["xdotool", *map(str, a)], env={**os.environ, "DISPLAY": DISP}, check=False)


# ── as formas NOSSAS (o que cada família abre) e a ferramenta que lhes mostra os nós ───────────────
def rect(rot):
    t = ' transform="rotate(%g 400 300)"' % rot if rot else ""
    return ('<rect id="F" x="300" y="240" width="200" height="120" rx="10" ry="10"%s %s/>' % (t, EST),
            "r", {"cx": 400, "cy": 300, "rot": rot})


def star(rot, flat=False, sides=5):
    t = ' transform="rotate(%g 400 300)"' % rot if rot else ""
    a1 = -math.pi / 2
    a2 = a1 + math.pi / sides
    return (('<path id="F" sodipodi:type="star" sodipodi:sides="%d" sodipodi:cx="400" sodipodi:cy="300" '
             'sodipodi:r1="100" sodipodi:r2="45" sodipodi:arg1="%.9f" sodipodi:arg2="%.9f" '
             'inkscape:flatsided="%s" inkscape:rounded="0" inkscape:randomized="0" d="M 0,0"%s %s/>')
            % (sides, a1, a2, "true" if flat else "false", t, EST), "asterisk", {"cx": 400, "cy": 300, "rot": rot})


def arc(rot, kind="slice"):
    t = ' transform="rotate(%g 400 300)"' % rot if rot else ""
    return (('<path id="F" sodipodi:type="arc" sodipodi:cx="400" sodipodi:cy="300" sodipodi:rx="120" '
             'sodipodi:ry="80" sodipodi:start="0" sodipodi:end="4.712389" sodipodi:arc-type="%s" '
             'd="M 0,0"%s %s/>') % (kind, t, EST), "e", {"cx": 400, "cy": 300, "rot": rot})


def spiral(rot):
    t = ' transform="rotate(%g 400 300)"' % rot if rot else ""
    return (('<path id="F" sodipodi:type="spiral" sodipodi:cx="400" sodipodi:cy="300" sodipodi:expansion="1" '
             'sodipodi:revolution="3" sodipodi:radius="110" sodipodi:argument="0" sodipodi:t0="0.1" '
             'd="M 0,0"%s style="fill:none;stroke:#000000;stroke-width:2"/>') % t, "i",
            {"cx": 400, "cy": 300, "rot": rot})


def rotp(p, c, deg):
    a = math.radians(deg)
    dx, dy = p[0] - c[0], p[1] - c[1]
    return (c[0] + dx * math.cos(a) - dy * math.sin(a), c[1] + dx * math.sin(a) + dy * math.cos(a))


# ── as experiências: (nome, alça em coordenadas LOCAIS do documento, destino LOCAL, modificador) ───
def exps_rect():
    rx_knot = (500 - 10, 240)      # o nó do rx: no lado de cima, a rx do canto direito (MEDIDO na foto)
    ry_knot = (500, 240 + 10)      # o nó do ry: no lado direito, a ry do canto de cima
    e = []
    for d in (-20, -40, -70, -95, -140, +15):
        e.append(("rx_x%+d" % d, rx_knot, (rx_knot[0] + d, rx_knot[1]), ""))
    for dy in (25, -30):
        e.append(("rx_x-40_y%+d" % dy, rx_knot, (rx_knot[0] - 40, rx_knot[1] + dy), ""))
    for d in (15, 35, 70):
        e.append(("ry_y%+d" % d, ry_knot, (ry_knot[0], ry_knot[1] + d), ""))
    e.append(("rx_x-40_ctrl", rx_knot, (rx_knot[0] - 40, rx_knot[1]), "ctrl"))
    e.append(("ry_y+35_ctrl", ry_knot, (ry_knot[0], ry_knot[1] + 35), "ctrl"))
    e.append(("rx_x-40_shift", rx_knot, (rx_knot[0] - 40, rx_knot[1]), "shift"))
    return e


def exps_star(sides=5, flat=False):
    a1 = -math.pi / 2
    a2 = a1 + math.pi / sides
    tip = (400 + 100 * math.cos(a1), 300 + 100 * math.sin(a1))
    inner = (400 + 45 * math.cos(a2), 300 + 45 * math.sin(a2))
    e = []
    for r in (60, 130, 170):                                   # a ponta, radial
        e.append(("tip_r%d" % r, tip, (400, 300 - r), ""))
    for deg in (20, 45, -30):                                  # a ponta, à volta do centro (r = 100)
        a = a1 + math.radians(deg)
        e.append(("tip_a%+d" % deg, tip, (400 + 100 * math.cos(a), 300 + 100 * math.sin(a)), ""))
    a = a1 + math.radians(30)
    e.append(("tip_a+30_r140_ctrl", tip, (400 + 140 * math.cos(a), 300 + 140 * math.sin(a)), "ctrl"))
    e.append(("tip_r130_shift", tip, (400, 300 - 130), "shift"))
    if not flat:
        for r in (25, 70, 120):                                # o vale, radial
            e.append(("inner_r%d" % r, inner, (400 + r * math.cos(a2), 300 + r * math.sin(a2)), ""))
        for deg in (15, -20):                                  # o vale, à volta do centro
            a = a2 + math.radians(deg)
            e.append(("inner_a%+d" % deg, inner, (400 + 45 * math.cos(a), 300 + 45 * math.sin(a)), ""))
        a = a2 + math.radians(15)
        e.append(("inner_a+15_r70_ctrl", inner, (400 + 70 * math.cos(a), 300 + 70 * math.sin(a)), "ctrl"))
        e.append(("inner_r70_shift", inner, (400 + 70 * math.cos(a2), 300 + 70 * math.sin(a2)), "shift"))
    return e


def exps_arc():
    start = (400 + 120, 300)                                   # start = 0 → (cx + rx, cy)
    end = (400, 300 - 80)                                      # end = 3π/2 → (cx, cy − ry)
    e = []
    for deg in (30, 77, 135, 200, -40):                        # o início pelo ângulo do ponteiro, NA elipse
        a = math.radians(deg)
        e.append(("start_a%+d" % deg, start, (400 + 120 * math.cos(a), 300 + 80 * math.sin(a)), ""))
    for deg in (40, 77):                                       # o mesmo ângulo de ecrã, LONGE da elipse (×1,6)
        a = math.radians(deg)
        e.append(("start_far_a%+d" % deg, start, (400 + 192 * math.cos(a), 300 + 128 * math.sin(a)), ""))
    a = math.radians(40)
    e.append(("start_in_a+40", start, (400 + 50 * math.cos(a), 300 + 33 * math.sin(a)), ""))
    for deg in (200, 300):
        a = math.radians(deg)
        e.append(("end_a%+d" % deg, end, (400 + 120 * math.cos(a), 300 + 80 * math.sin(a)), ""))
    a = math.radians(37)
    e.append(("start_a+37_ctrl", start, (400 + 120 * math.cos(a), 300 + 80 * math.sin(a)), "ctrl"))
    e.append(("start_a+37_shift", start, (400 + 120 * math.cos(a), 300 + 80 * math.sin(a)), "shift"))
    return e


def exps_spiral():
    # a espiral de Arquimedes do Inkscape: r(t) = radius·t^exp, θ(t) = 2π·revolution·t + argument
    outer = (400 + 110, 300)                                   # t = 1, θ = 6π → (cx + R, cy)
    t0 = 0.1
    th0 = 2 * math.pi * 3 * t0
    inner = (400 + 110 * t0 * math.cos(th0), 300 + 110 * t0 * math.sin(th0))
    e = []
    for deg, r in ((0, 150), (0, 70), (90, 110), (-90, 110), (45, 160), (180, 110)):
        a = math.radians(deg)
        e.append(("outer_a%+d_r%d" % (deg, r), outer, (400 + r * math.cos(a), 300 + r * math.sin(a)), ""))
    a = math.radians(50)
    e.append(("outer_a+50_r130_ctrl", outer, (400 + 130 * math.cos(a), 300 + 130 * math.sin(a)), "ctrl"))
    e.append(("outer_a+50_r130_shift", outer, (400 + 130 * math.cos(a), 300 + 130 * math.sin(a)), "shift"))
    for dr in (20, 40):
        a = th0
        e.append(("inner_out%d" % dr, inner, (400 + (11 + dr) * math.cos(a), 300 + (11 + dr) * math.sin(a)), ""))
    e.append(("inner_alt_out30", inner, (400 + 41 * math.cos(th0), 300 + 41 * math.sin(th0)), "alt"))
    return e


FAMILIAS = {
    "rect": (rect(0), exps_rect),
    "rect_rot25": (rect(25), exps_rect),
    "star5": (star(0), lambda: exps_star(5)),
    "star5_rot25": (star(25), lambda: exps_star(5)),
    "polygon6": (star(0, flat=True, sides=6), lambda: exps_star(6, flat=True)),
    "arc_slice": (arc(0), exps_arc),
    "arc_slice_rot25": (arc(25), exps_arc),
    "spiral": (spiral(0), exps_spiral),
}


# ── o aparelho ───────────────────────────────────────────────────────────────────────────────────
class Ink:
    def __init__(self, doc):
        self.doc = doc
        self.pid = subprocess.Popen(
            ["inkscape", doc], env={"HOME": TMP + "/home", "DISPLAY": DISP, "PATH": "/usr/bin",
                                    "XDG_CONFIG_HOME": TMP + "/home/.config", "LANG": "C.UTF-8"},
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).pid
        w = ""
        for _ in range(150):
            r = subprocess.run(["xdotool", "search", "--name", " - Inkscape$"], capture_output=True, text=True,
                               env={**os.environ, "DISPLAY": DISP})
            w = r.stdout.split()[0] if r.stdout.split() else ""
            if w:
                break
            time.sleep(0.2)
        if not w:
            raise SystemExit("✗ o Inkscape não abriu")
        time.sleep(2.5)
        x("windowmove", w, 0, 0)
        x("windowsize", w, 1600, 1000)
        time.sleep(1.5)
        x("mousemove", 600, 500)
        x("key", "Escape")
        x("key", "5")                       # encaixar a página
        time.sleep(1.0)

    def save(self):
        m = os.stat(self.doc).st_mtime_ns
        x("key", "ctrl+s")
        for _ in range(60):
            time.sleep(0.1)
            if os.stat(self.doc).st_mtime_ns != m:
                time.sleep(0.3)
                return ET.parse(self.doc).getroot()
        raise RuntimeError("Ctrl+S não gravou")

    def close(self):
        try:
            os.kill(self.pid, 15)
        except ProcessLookupError:
            pass
        time.sleep(1.0)


def drag(a, b, mod=""):
    x("mousemove", round(a[0]), round(a[1]))
    time.sleep(0.15)
    x("mousedown", 1)
    time.sleep(0.1)
    if mod:
        x("keydown", mod)
    n = 12
    for i in range(1, n + 1):
        t = i / n
        x("mousemove", round(a[0] + (b[0] - a[0]) * t), round(a[1] + (b[1] - a[1]) * t))
        time.sleep(0.03)
    time.sleep(0.15)
    x("mouseup", 1)
    if mod:
        x("keyup", mod)
    time.sleep(0.3)


def atributos(root):
    el = [e for e in root.iter() if e.get("id") == "F"]
    if not el:
        return None
    out = {}
    for k, v in el[0].attrib.items():
        k = k.replace(SOD, "sodipodi:").replace(INK, "inkscape:")
        if k in ("d", "style", "id"):
            continue
        out[k] = v
    return out


def calibra(ink):
    # A calibração é uma MEDIÇÃO: o rectângulo desenhado pela ferramenta de rectângulos entre dois
    # pontos de ecrã (encaixe desligado) dá a escala e a origem tela → documento.
    x("key", "Escape"); x("key", "r"); time.sleep(0.3)
    x("key", "percent"); time.sleep(0.2)                          # encaixe desligado
    drag((300, 700), (700, 860))
    root = ink.save()
    novos = [e for e in root.iter() if e.tag.endswith("rect") and e.get("id") != "F"]
    if not novos:
        raise SystemExit("✗ a calibração não desenhou o rectângulo")
    r = novos[-1]
    X, Y, W, H = (float(r.get(k)) for k in ("x", "y", "width", "height"))
    sx, sy = 400.0 / W, 160.0 / H
    ox, oy = 300 - X * sx, 700 - Y * sy
    x("key", "ctrl+z"); time.sleep(0.3)
    x("key", "Escape")
    return sx, sy, ox, oy


def corre(nome):
    (svg, tecla, meta), exps = FAMILIAS[nome]
    doc = os.path.join(TMP, nome + ".svg")
    with open(doc, "w") as f:
        f.write(CAB + svg + "\n</svg>\n")
    ink = Ink(doc)
    try:
        sx, sy, ox, oy = calibra(ink)
        tela = lambda p: (ox + p[0] * sx, oy + p[1] * sy)
        origem = atributos(ink.save())
        rot = meta["rot"]
        c = (meta["cx"], meta["cy"])
        res = []
        for (en, alca, alvo, mod) in exps():
            a, b = rotp(alca, c, rot), rotp(alvo, c, rot)       # os nós seguem a rotação da forma
            x("key", "Escape"); x("key", "F1"); time.sleep(0.2)
            x("key", "ctrl+a"); time.sleep(0.2)                 # a forma é a ÚNICA do documento
            x("key", tecla); time.sleep(0.4)
            drag(tela(a), tela(b), mod)
            depois = atributos(ink.save())
            res.append({"nome": en, "mod": mod, "alca_doc": [round(v, 4) for v in a],
                        "alvo_doc": [round(v, 4) for v in b],
                        "alca_tela": [round(v) for v in tela(a)], "alvo_tela": [round(v) for v in tela(b)],
                        "depois": depois})
            x("key", "ctrl+z"); time.sleep(0.4)
            print("  %-26s %s" % (en, "" if depois != origem else "(SEM EFEITO)"), flush=True)
        controlo = atributos(ink.save())
    finally:
        ink.close()
    with open(os.path.join(FIX, nome + ".json"), "w") as f:
        json.dump({"oraculo": subprocess.run(["inkscape", "--version"], capture_output=True, text=True,
                                              env={"PATH": "/usr/bin", "HOME": TMP + "/home"}).stdout.strip(),
                   "nota": "caixa preta (GPL): so a SAIDA sobre formas nossas; coordenadas em px do documento "
                           "(viewBox 800x600); a alca e o alvo ja rodados como a forma; o arrasto e o rato do "
                           "xdotool em 12 passos; o modificador e premido depois do press",
                   "calibracao_tela": {"sx": sx, "sy": sy, "ox": ox, "oy": oy},
                   "forma": svg, "origem": origem, "controlo_depois_do_ultimo_undo": controlo,
                   "experiencias": res}, f, indent=1, ensure_ascii=False)
    return controlo == origem


def main():
    fams = sys.argv[1:] or list(FAMILIAS)
    if os.path.exists("/tmp/.X96-lock"):
        raise SystemExit("✗ já há um ecrã :96 — pare-o pelo PID")
    os.makedirs(FIX, exist_ok=True)
    os.makedirs(TMP + "/home", exist_ok=True)
    xv = subprocess.Popen(["Xvfb", DISP, "-screen", "0", "1600x1000x24", "-nolisten", "tcp"],
                          stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(1)
    try:
        for f in fams:
            print(f, flush=True)
            ok = corre(f)
            print("  controlo (o último Ctrl+Z repôs a forma):", "sim" if ok else "NÃO", flush=True)
    finally:
        xv.terminate()
        shutil.rmtree(TMP, ignore_errors=True)


if __name__ == "__main__":
    main()
