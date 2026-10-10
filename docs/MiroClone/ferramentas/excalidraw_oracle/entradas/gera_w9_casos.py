#!/usr/bin/env python3
"""W9 (fidelidade .excalidraw): gera entradas/w9_casos.json para corre_fidelidade.mjs.

Famílias de casos (entradas NOSSAS, elementos completos ⇒ nada sorteado pelo convertTo…):
  B_*  ligações: seta solta → forma (rectângulo/elipse/losango), o editor re-liga ao EMPURRAR as formas;
       modos orbit/inside/skip e o legado {focus,gap}; fixedPoint no centro, dentro, na borda, fora;
       espessura 1/2/4; formas de cantos vivos e redondos.
  P_*  pontas: setas soltas de comprimento 8..400 × espessura 1/2/4 (ponta `arrow`), e cada tipo de ponta.
  F_*  o fuste: redonda de 2 e 3 pontos, de cantos vivos, linha, rugosidade 0 e 1.
  M_*  à mão: a ferramenta Seta arrastada pelo rato no editor (o que o editor GRAVA ao desenhar).
  T_*  formato: um elemento de cada tipo/propriedade, restore + serialize + svg.
  dono_bv: o ficheiro do dono (smoke 10/10).
Uso: python3 gera_w9_casos.py <saida.json>
"""
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent


def base(i, t, x, y, w, h, **kw):
    e = {
        "id": i, "type": t, "x": x, "y": y, "width": w, "height": h, "angle": 0,
        "strokeColor": "#1e1e1e", "backgroundColor": "transparent", "fillStyle": "solid",
        "strokeWidth": 2, "strokeStyle": "solid", "roughness": 0, "opacity": 100,
        "groupIds": [], "frameId": None, "roundness": None, "seed": 1000 + abs(hash(i)) % 9000,
        "version": 1, "versionNonce": 1, "isDeleted": False, "boundElements": None,
        "updated": 1, "link": None, "locked": False,
    }
    e["seed"] = sum(map(ord, i)) * 7919 % 2**31  # determinístico (hash() é sorteado por processo)
    e.update(kw)
    return e


SHAPES = [("R", "rectangle", 160), ("E", "ellipse", 380), ("D", "diamond", 600)]  # (id, tipo, centro y)


def shapes(round_=False, sw=2):
    out = []
    for sid, t, cy in SHAPES:
        rnd = None
        if round_:
            rnd = {"type": 3} if t == "rectangle" else {"type": 2}
        out.append(base(sid, t, 500, cy - 60, 200, 120, roundness=rnd, strokeWidth=sw))
    return out


def arrow(i, p0, p1, sb=None, eb=None, sw=2, rnd={"type": 2}, head="arrow", pts=None, **kw):
    pts = pts or [p0, p1]
    x0, y0 = pts[0]
    rel = [[p[0] - x0, p[1] - y0] for p in pts]
    xs = [p[0] for p in rel]
    ys = [p[1] for p in rel]
    return base(i, "arrow", x0, y0, max(xs) - min(xs), max(ys) - min(ys), strokeWidth=sw,
                roundness=rnd, points=rel, startBinding=sb, endBinding=eb,
                startArrowhead=None, endArrowhead=head, elbowed=False, **kw)


def link(shapes_, arrows):
    """boundElements recíprocos (o que o Excalidraw grava)."""
    by = {s["id"]: s for s in shapes_}
    for a in arrows:
        for k in ("startBinding", "endBinding"):
            b = a.get(k)
            if b and b["elementId"] in by:
                s = by[b["elementId"]]
                s["boundElements"] = (s["boundElements"] or []) + [{"id": a["id"], "type": "arrow"}]


def caso_ligacao(nome, desc, binding, diag=False, sw=2, round_=False, start=None):
    sh = shapes(round_, sw)
    ars = []
    for sid, _t, cy in SHAPES:
        p0 = start(cy) if start else ([300, cy - 150] if diag else [200, cy])
        ars.append(arrow(f"a{sid}", p0, [500, cy], eb=dict(binding, elementId=sid), sw=sw))
    link(sh, ars)
    return {"nome": nome, "descricao": desc, "elementos": sh + ars,
            "passos": [{"tipo": "restore"}, {"tipo": "svg"}, {"tipo": "montar"},
                       {"tipo": "empurrar", "ids": ["R", "E", "D"]}, {"tipo": "svg"}]}


casos = []
ORB = lambda fp: {"mode": "orbit", "fixedPoint": fp}  # noqa: E731
for nome, desc, b, kw in [
    ("B_orbit_centro", "orbit, fixedPoint no centro, seta horizontal", ORB([0.5, 0.5]), {}),
    ("B_orbit_centro_diag", "orbit, centro, seta diagonal (passa junto ao canto sup. esq.)", ORB([0.5, 0.5]), {"diag": True}),
    ("B_orbit_dentro", "orbit, fixedPoint dentro fora do centro (0.75,0.3)", ORB([0.75, 0.3]), {}),
    ("B_orbit_dentro_diag", "orbit, (0.75,0.3), diagonal", ORB([0.75, 0.3]), {"diag": True}),
    ("B_orbit_borda_frente", "orbit, fixedPoint no meio da face que olha a seta (0,0.5)", ORB([0.0, 0.5]), {}),
    ("B_orbit_borda_tras", "orbit, fixedPoint no meio da face de trás (1,0.5)", ORB([1.0, 0.5]), {}),
    ("B_orbit_fora", "orbit, fixedPoint FORA da caixa (-0.25,0.5)", ORB([-0.25, 0.5]), {}),
    ("B_inside_dentro", "inside, (0.75,0.3)", {"mode": "inside", "fixedPoint": [0.75, 0.3]}, {}),
    ("B_inside_centro", "inside, centro", {"mode": "inside", "fixedPoint": [0.5, 0.5]}, {}),
    ("B_skip_dentro", "skip, (0.75,0.3)", {"mode": "skip", "fixedPoint": [0.75, 0.3]}, {}),
    ("B_legado_gap1", "0.18.1: {focus:0, gap:1}", {"focus": 0, "gap": 1}, {}),
    ("B_legado_focus05_gap10", "0.18.1: {focus:0.5, gap:10}", {"focus": 0.5, "gap": 10}, {}),
    ("B_legado_focus05_gap10_diag", "0.18.1: {focus:0.5, gap:10}, diagonal", {"focus": 0.5, "gap": 10}, {"diag": True}),
    ("B_orbit_centro_sw1", "orbit, centro, strokeWidth 1", ORB([0.5, 0.5]), {"sw": 1}),
    ("B_orbit_centro_sw4", "orbit, centro, strokeWidth 4", ORB([0.5, 0.5]), {"sw": 4}),
    ("B_orbit_centro_diag_sw1", "orbit, centro, diagonal, strokeWidth 1", ORB([0.5, 0.5]), {"diag": True, "sw": 1}),
    ("B_orbit_centro_diag_sw4", "orbit, centro, diagonal, strokeWidth 4", ORB([0.5, 0.5]), {"diag": True, "sw": 4}),
    ("B_orbit_centro_diag_redondo", "orbit, centro, diagonal, formas de cantos redondos", ORB([0.5, 0.5]), {"diag": True, "round_": True}),
    ("B_ambos_orbit_e_legado", "a ligação com AS DUAS gramáticas: {mode orbit, fixedPoint (0.75,0.3), focus 0.2, gap 3} (a escrita que serve as duas versões?)",
     {"mode": "orbit", "fixedPoint": [0.75, 0.3], "focus": 0.2, "gap": 3}, {}),
    ("B_orbit_centro_curta", "orbit, centro, seta CURTA (começa a 30 da face)", ORB([0.5, 0.5]), {"start": lambda cy: [470, cy]}),
]:
    casos.append(caso_ligacao(nome, desc, b, **kw))

# As duas pontas presas (R → E, vertical), orbit no centro e no ponto do dono.
sh = shapes()
a = arrow("aRE", [600, 220], [600, 320], sb=dict(ORB([0.5, 0.5]), elementId="R"), eb=dict(ORB([0.5, 0.5]), elementId="E"))
a2 = arrow("aED", [650, 440], [560, 540], sb=dict(ORB([0.75, 0.9]), elementId="E"), eb=dict(ORB([0.3, 0.4]), elementId="D"))
link(sh, [a, a2])
casos.append({"nome": "B_duas_pontas", "descricao": "R→E orbit centro/centro; E→D orbit em pontos interiores",
              "elementos": sh + [a, a2],
              "passos": [{"tipo": "restore"}, {"tipo": "svg"}, {"tipo": "montar"},
                         {"tipo": "empurrar", "ids": ["R", "E", "D"]}, {"tipo": "svg"}]})

# P — pontas: comprimento × espessura (ponta arrow), cada seta numa linha.
els = []
row = 0
for sw in (1, 2, 4):
    for L in (8, 15, 20, 30, 40, 50, 60, 80, 100, 150, 200, 300, 400):
        y = 20 + row * 30
        els.append(arrow(f"p_sw{sw}_L{L}", [20, y], [20 + L, y], sw=sw))
        row += 1
casos.append({"nome": "P_comprimento", "descricao": "ponta `arrow`: comprimento 8..400 × espessura 1/2/4, roughness 0",
              "elementos": els, "passos": [{"tipo": "restore"}, {"tipo": "svg"}]})
els = []
HEADS = ["arrow", "bar", "dot", "circle", "circle_outline", "triangle", "triangle_outline", "diamond",
         "diamond_outline", "crowfoot_one", "crowfoot_many", "crowfoot_one_or_many", "cardinality_one",
         "cardinality_many", "cardinality_one_or_many", "cardinality_exactly_one", "cardinality_zero_or_one",
         "cardinality_zero_or_many"]
for i, h in enumerate(HEADS):
    for j, L in enumerate((200, 30)):
        y = 20 + (2 * i + j) * 30
        els.append(arrow(f"k_{h}_L{L}", [20, y], [20 + L, y], head=h))
casos.append({"nome": "P_tipos", "descricao": "cada tipo de ponta, L 200 e 30, strokeWidth 2",
              "elementos": els, "passos": [{"tipo": "restore"}, {"tipo": "serializar"}, {"tipo": "svg"}]})

els = []
row = 0
for h in ("bar", "circle", "triangle", "diamond", "cardinality_one", "cardinality_many", "cardinality_zero_or_one"):
    for L in (10, 16, 20, 24, 30, 40, 50, 60, 100):
        els.append(arrow(f"q_{h}_L{L}", [20, 20 + row * 30], [20 + L, 20 + row * 30], head=h))
        row += 1
casos.append({"nome": "P_tipos_comprimento", "descricao": "cada tipo de ponta × L 10..100 (a lei min(base, L/2)?)",
              "elementos": els, "passos": [{"tipo": "restore"}, {"tipo": "svg"}]})

# F — o fuste.
els = []
for r in (0, 1):
    oy = r * 300
    els += [
        arrow(f"f_r{r}_redonda2", [20, oy + 40], [220, oy + 120], roughness=r),
        arrow(f"f_r{r}_redonda3", [260, oy + 40], [460, oy + 120], pts=[[260, oy + 40], [360, oy + 140], [460, oy + 60]], roughness=r),
        arrow(f"f_r{r}_viva3", [500, oy + 40], [700, oy + 120], pts=[[500, oy + 40], [600, oy + 140], [700, oy + 60]], rnd=None, roughness=r),
        arrow(f"f_r{r}_viva2", [740, oy + 40], [940, oy + 120], rnd=None, roughness=r),
    ]
    ln = arrow(f"f_r{r}_linha3", [980, oy + 40], [1180, oy + 60], pts=[[980, oy + 40], [1080, oy + 140], [1180, oy + 60]], roughness=r, head=None)
    ln["type"] = "line"
    del ln["elbowed"]
    els.append(ln)
casos.append({"nome": "F_fuste", "descricao": "redonda 2/3 pontos, cantos vivos 2/3, linha; roughness 0 e 1",
              "elementos": els, "passos": [{"tipo": "restore"}, {"tipo": "svg"}]})

# M — desenhar à mão no editor (ecrã = cena). Formas como as de B.
for nome, de, a_, desc in [
    ("M_centro_centro", [600, 160], [600, 380], "do centro de R ao centro de E"),
    ("M_borda_borda", [600, 212], [600, 328], "de dentro de R junto à base a dentro de E junto ao topo"),
    ("M_solta_dentro", [330, 380], [560, 370], "de um ponto solto a dentro de E (fora do centro)"),
    ("M_solta_perto_fora", [330, 380], [494, 380], "de um ponto solto a 6 px à esquerda de E"),
    ("M_solta_perto_dentro", [330, 380], [506, 380], "de um ponto solto a 6 px dentro de E"),
    ("M_dentro_dentro", [650, 190], [560, 600], "de dentro de R (fora do centro) a dentro de D (fora do centro)"),
    ("M_solta_centro_losango", [330, 600], [600, 600], "de um ponto solto ao centro de D"),
    ("M_solta_solta", [330, 720], [460, 760], "controlo: nada por perto"),
    ("M_centro_centro_ctrl", [600, 160], [600, 380], "do centro de R ao centro de E com Ctrl (desliga a ligação?)"),
]:
    casos.append({"nome": nome, "descricao": desc, "elementos": shapes(),
                  "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                             {"tipo": "desenhar", "tecla": "a", "de": de, "a": a_, **({"mods": ["Control"]} if nome.endswith("_ctrl") else {})},
                             {"tipo": "empurrar", "ids": ["R", "E", "D"]}, {"tipo": "svg"}]})
# M2 — a escolha orbit × inside ao DESENHAR: a profundidade dentro de E (borda esquerda x=500, y=380)
# e o tempo parado antes de largar; entrar e voltar à borda; entrar devagar.
for prof in (2, 6, 10, 16, 24, 40):
    for espera in (50, 1500):
        casos.append({"nome": f"M2_prof{prof}_espera{espera}", "descricao": f"solto → {prof} px dentro de E, parado {espera} ms",
                      "elementos": shapes(),
                      "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                                 {"tipo": "desenhar", "tecla": "a", "de": [330, 380], "a": [500 + prof, 380], "espera": espera}]})
casos.append({"nome": "M2_fundo_e_volta", "descricao": "solto → fundo de E (600,380) → volta a 10 px dentro da borda",
              "elementos": shapes(),
              "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                         {"tipo": "desenhar", "tecla": "a", "de": [330, 380], "via": [[600, 380]], "a": [510, 380]}]})
casos.append({"nome": "M2_devagar", "descricao": "solto → 16 px dentro de E, 60 passos de 40 ms",
              "elementos": shapes(),
              "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                         {"tipo": "desenhar", "tecla": "a", "de": [330, 380], "a": [516, 380], "n": 60, "dt": 40}]})
for mod in ("Alt", "Shift", "Control"):
    casos.append({"nome": f"M2_prof16_{mod.lower()}", "descricao": f"solto → 16 px dentro de E com {mod}",
                  "elementos": shapes(),
                  "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                             {"tipo": "desenhar", "tecla": "a", "de": [330, 380], "a": [516, 380], "mods": [mod]}]})
for espera in (0, 100, 200, 400, 700):
    casos.append({"nome": f"M2_prof16_rapido_espera{espera}", "descricao": f"solto → 16 px dentro de E em 4 passos de 5 ms, parado {espera} ms",
                  "elementos": shapes(),
                  "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                             {"tipo": "desenhar", "tecla": "a", "de": [330, 380], "a": [516, 380], "n": 4, "dt": 5, "espera": espera}]})
casos.append({"nome": "M2_prof16_cliques", "descricao": "seta por CLIQUES: clique solto, clique 16 px dentro de E, Escape",
              "elementos": shapes(),
              "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                         {"tipo": "desenhar", "tecla": "a", "de": [330, 380], "a": [516, 380], "clique_clique": True}]})
casos.append({"nome": "M2_prof16_um_salto", "descricao": "arrasto num só movimento até 16 px dentro de E",
              "elementos": shapes(),
              "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                         {"tipo": "desenhar", "tecla": "a", "de": [330, 380], "a": [516, 380], "n": 1}]})
solta = arrow("aS", [330, 380], [460, 380])
casos.append({"nome": "M2_arrasta_ponta", "descricao": "seta solta existente; seleccionada e a PONTA final arrastada a 16 px dentro de E",
              "elementos": shapes() + [solta],
              "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                         {"tipo": "desenhar", "tecla": "v", "de": [400, 380], "a": [400, 380], "n": 1, "sem_escape": True},
                         {"tipo": "desenhar", "manter": True, "de": [460, 380], "a": [516, 380]},
                         {"tipo": "empurrar", "ids": ["R", "E", "D"]}]})
casos.append({"nome": "M2_fora_longe", "descricao": "solto → 20 px FORA de E",
              "elementos": shapes(),
              "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                         {"tipo": "desenhar", "tecla": "a", "de": [330, 380], "a": [480, 380]}]})
casos.append({"nome": "M2_fora_10", "descricao": "solto → 10 px FORA de E",
              "elementos": shapes(),
              "passos": [{"tipo": "restore"}, {"tipo": "montar"},
                         {"tipo": "desenhar", "tecla": "a", "de": [330, 380], "a": [490, 380]}]})
# Espessura da SETA × espessura da FORMA (o vão é de quem?).
for sa, sf in ((4, 1), (1, 4)):
    sh_ = shapes(sw=sf)
    ars_ = [arrow(f"a{sid}", [200, cy], [500, cy], eb=dict(ORB([0.5, 0.5]), elementId=sid), sw=sa) for sid, _t, cy in SHAPES]
    link(sh_, ars_)
    casos.append({"nome": f"B_seta_sw{sa}_forma_sw{sf}", "descricao": f"orbit centro; seta sw {sa}, formas sw {sf}",
                  "elementos": sh_ + ars_,
                  "passos": [{"tipo": "restore"}, {"tipo": "montar"}, {"tipo": "empurrar", "ids": ["R", "E", "D"]}, {"tipo": "svg"}]})
# Ponta: qual comprimento manda numa seta de 3 pontos (o último trecho ou o total)?
els = []
for i, (nm, pts, rnd) in enumerate([
    ("viva_ult20", [[20, 0], [220, 0], [240, 0]], None),
    ("viva_pri20", [[20, 0], [40, 0], [240, 0]], None),
    ("redonda_ult20", [[20, 0], [220, 0], [240, 0]], {"type": 2}),
    ("viva_ult20_dobra", [[20, 0], [220, 0], [220, 20]], None),
]):
    y = 40 + i * 60
    els.append(arrow(f"u_{nm}", None, None, pts=[[p[0], p[1] + y] for p in pts], rnd=rnd))
casos.append({"nome": "P_ultimo_trecho", "descricao": "ponta arrow em setas de 3 pontos: último trecho 20 vs total 220",
              "elementos": els, "passos": [{"tipo": "restore"}, {"tipo": "svg"}]})
casos.append({"nome": "M_nota", "descricao": "a ferramenta Sticky note (N) arrastada (só existe no next)", "elementos": [],
              "passos": [{"tipo": "montar"}, {"tipo": "desenhar", "tecla": "n", "de": [300, 300], "a": [500, 460]},
                         {"tipo": "serializar"}, {"tipo": "svg"}]})
casos.append({"nome": "M_linha", "descricao": "a ferramenta Linha (L) arrastada", "elementos": [],
              "passos": [{"tipo": "montar"}, {"tipo": "desenhar", "tecla": "l", "de": [300, 300], "a": [500, 460]},
                         {"tipo": "serializar"}]})
casos.append({"nome": "M_caneta", "descricao": "a ferramenta Draw (P) arrastada", "elementos": [],
              "passos": [{"tipo": "montar"}, {"tipo": "desenhar", "tecla": "p", "de": [300, 300], "a": [500, 460]},
                         {"tipo": "serializar"}]})
casos.append({"nome": "M_retangulo", "descricao": "a ferramenta Rectângulo (R) arrastada", "elementos": [],
              "passos": [{"tipo": "montar"}, {"tipo": "desenhar", "tecla": "r", "de": [300, 300], "a": [500, 460]},
                         {"tipo": "serializar"}]})

# T — o formato inteiro: um de cada.
T = []
T.append(base("t_hach", "rectangle", 20, 20, 120, 80, backgroundColor="#a5d8ff", fillStyle="hachure", roughness=1))
T.append(base("t_cross", "rectangle", 160, 20, 120, 80, backgroundColor="#b2f2bb", fillStyle="cross-hatch", roughness=2))
T.append(base("t_zig", "ellipse", 300, 20, 120, 80, backgroundColor="#ffc9c9", fillStyle="zigzag", strokeStyle="dashed"))
T.append(base("t_dot", "diamond", 440, 20, 120, 80, strokeStyle="dotted", opacity=50, angle=0.5,
              link="https://example.com", locked=True, customData={"quem": "ph2d"}))
T.append(base("t_rnd", "rectangle", 580, 20, 120, 80, roundness={"type": 3, "value": 12}))
for i, ff in enumerate((1, 2, 3, 5, 6, 7, 8, 9)):
    T.append(base(f"t_txt{ff}", "text", 20 + i * 150, 140, 120, 25, text=f"Olá {ff}", originalText=f"Olá {ff}",
                  fontSize=20, fontFamily=ff, textAlign=["left", "center", "right"][i % 3],
                  verticalAlign="top", containerId=None, lineHeight=1.25, autoResize=True))
box = base("t_box", "rectangle", 20, 200, 160, 80, boundElements=[{"id": "t_box_txt", "type": "text"}])
T.append(box)
T.append(base("t_box_txt", "text", 40, 227, 120, 25, text="dentro", originalText="dentro", fontSize=20, fontFamily=5,
              textAlign="center", verticalAlign="middle", containerId="t_box", lineHeight=1.25, autoResize=True))
lab = arrow("t_lab", [220, 240], [420, 240], boundElements=[{"id": "t_lab_txt", "type": "text"}])
T.append(lab)
T.append(base("t_lab_txt", "text", 290, 228, 60, 25, text="rótulo", originalText="rótulo", fontSize=20, fontFamily=5,
              textAlign="center", verticalAlign="middle", containerId="t_lab", lineHeight=1.25, autoResize=True))
ln = arrow("t_poly", [460, 200], [560, 280], pts=[[460, 200], [560, 200], [560, 280], [460, 200]], head=None,
           backgroundColor="#ffec99", polygon=True)
ln["type"] = "line"
del ln["elbowed"]
T.append(ln)
fd = base("t_free", "freedraw", 600, 200, 100, 40, points=[[0, 0], [30, 20], [60, 10], [100, 40]],
          pressures=[0.2, 0.6, 0.9, 0.4], simulatePressure=False)
T.append(fd)
el = arrow("t_elbow", [740, 200], [900, 280], pts=[[740, 200], [820, 200], [820, 280], [900, 280]], rnd=None)
el["elbowed"] = True
el["fixedSegments"] = [{"start": [80, 0], "end": [80, 80], "index": 2}]
el["startIsSpecial"] = False
el["endIsSpecial"] = False
T.append(el)
T.append(base("t_frame", "frame", 20, 320, 300, 160, name="Moldura"))
T.append(base("t_inframe", "rectangle", 40, 360, 80, 60, frameId="t_frame", groupIds=["g_in", "g_out"]))
T.append(base("t_note", "stickynote", 360, 320, 160, 160, backgroundColor="#ffec99", baseHeight=160,
              boundElements=[{"id": "t_note_txt", "type": "text"}]))
T.append(base("t_note_txt", "text", 370, 330, 140, 25, text="nota", originalText="nota", fontSize=20, fontFamily=5,
              baseFontSize=20, textAlign="center", verticalAlign="middle", containerId="t_note", lineHeight=1.25, autoResize=True))
T.append(base("t_embed", "embeddable", 560, 320, 200, 120, link="https://www.youtube.com/watch?v=dQw4w9WgXcQ"))
casos.append({"nome": "T_formato", "descricao": "um elemento de cada tipo/propriedade", "elementos": T,
              "passos": [{"tipo": "restore"}, {"tipo": "serializar"}, {"tipo": "svg"}]})

# O ficheiro do dono.
bv = json.loads((HERE / "w9_dono_bv.excalidraw").read_text())
casos.append({"nome": "dono_bv", "descricao": "o bv.excalidraw do dono (excalidraw.com, 10/10)", "elementos": bv["elements"],
              "passos": [{"tipo": "restore"}, {"tipo": "svg"}, {"tipo": "montar"},
                         {"tipo": "empurrar", "ids": [e["id"] for e in bv["elements"] if e["type"] != "arrow"]},
                         {"tipo": "svg"}]})

# Desenho à mão: relógio LIVRE (o editor next escolhe orbit × inside por um temporizador; o relógio
# fixo congelava-o) e o estado da ligação lido depois de montar.
ESTADO = {"tipo": "estado", "chaves": ["bindMode", "bindingPreference", "isBindingEnabled", "isMidpointSnappingEnabled"]}
for c in casos:
    if c["nome"].startswith("M"):
        c["relogio"] = "livre"
        i = next((k for k, p in enumerate(c["passos"]) if p["tipo"] == "montar"), None)
        if i is not None:
            c["passos"].insert(i + 1, ESTADO)

Path(sys.argv[1]).write_text(json.dumps({"casos": casos}, ensure_ascii=False, indent=1) + "\n")
print(f"{len(casos)} casos → {sys.argv[1]}")
