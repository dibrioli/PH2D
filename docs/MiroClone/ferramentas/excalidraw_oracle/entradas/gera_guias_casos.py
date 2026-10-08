# Gera entradas/guias_casos.json (as cenas e sondas do W5b, corre_guias.mjs). Coordenadas NOSSAS.
# Uso: python3 entradas/gera_guias_casos.py entradas/guias_casos.json
import json, math, sys
OUT = sys.argv[1]
def base(id_, type_, x, y, w, h, ang=0.0, **kw):
    e = {"id": id_, "type": type_, "x": x, "y": y, "width": w, "height": h, "angle": ang,
         "strokeColor": "#1e1e1e", "backgroundColor": "#a5d8ff", "fillStyle": "solid",
         "strokeWidth": 2, "roughness": 0, "groupIds": [], "frameId": None,
         "version": 1, "versionNonce": 1, "seed": 1}
    e.update(kw); return e
def rect_c(id_, cx, cy, w, h, deg=0.0, **kw):
    return base(id_, "rectangle", cx - w / 2, cy - h / 2, w, h, math.radians(deg), **kw)
def rect(id_, x, y, w, h, **kw): return base(id_, "rectangle", x, y, w, h, **kw)
def frame(id_, x, y, w, h):
    return base(id_, "frame", x, y, w, h, name="Moldura", strokeColor="#bbb", backgroundColor="transparent")
def linear(id_, type_, x, y, pts, **kw):
    xs = [p[0] for p in pts]; ys = [p[1] for p in pts]
    return base(id_, type_, x, y, max(xs) - min(xs), max(ys) - min(ys), points=pts,
                backgroundColor="transparent", **kw)
def X(de, a, mais): return {"de": de, "a": a, "mais": mais}

cenas, casos = {}, []
# ---- G1: alvo A 0°, móvel B rodado
for deg in (15, 30, 45, 37):
    k = f"g1_{deg}"
    cenas[k] = {"elements": [rect("A", 300, 150, 340, 120), rect_c("B", 900, 560, 160, 100, deg)]}
    casos.append({"nome": f"G1_B{deg}", "cena": k,
        "pergunta": f"A 0° (300..640), B 160×100 a {deg}°: o encaixe usa a AABB de B ou os cantos rodados?",
        "sondas": [
            {"rotulo": "a_aabb_esq", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.aabb.l", 3)},
            {"rotulo": "b_canto1_esq", "tipo": "arrasta", "mover": "B", "x": X("cantos.1", "A.aabb.l", 3)},
            {"rotulo": "b2_canto1_esq_menos", "tipo": "arrasta", "mover": "B", "x": X("cantos.1", "A.aabb.l", -3)},
            {"rotulo": "c_centros", "tipo": "arrasta", "mover": "B", "x": X("aabb.c", "A.aabb.c", 3)},
            {"rotulo": "a_aabb_esq_sem_encaixe", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.aabb.l", 3), "snap": False},
            {"rotulo": "d_aabb_topo", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.aabb.r", 60), "y": X("aabb.t", "A.aabb.b", 3)},
            {"rotulo": "e_canto1_topo", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.aabb.r", 60), "y": X("cantos.1", "A.aabb.b", 3)},
        ]})
# ---- G2: alvo A rodado 30°, móvel B 0°
cenas["g2"] = {"elements": [rect_c("A", 450, 250, 200, 120, 30), rect("B", 850, 560, 160, 100)]}
casos.append({"nome": "G2_A30", "cena": "g2",
    "pergunta": "A 200×120 a 30°, B 0°: o alvo é a AABB de A ou os cantos rodados de A?",
    "sondas": [
        {"rotulo": "a_aabb_esq", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.aabb.l", 3)},
        {"rotulo": "b_canto1_de_A", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.cantos.1", 3)},
        {"rotulo": "b2_canto2_de_A", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.cantos.2", 3)},
        {"rotulo": "c_centros", "tipo": "arrasta", "mover": "B", "x": X("aabb.c", "A.aabb.c", 3)},
        {"rotulo": "d_aabb_dir", "tipo": "arrasta", "mover": "B", "x": X("aabb.r", "A.aabb.r", 3)},
    ]})
# ---- G3: os dois a 30°; B posto com a aresta esquerda PARALELA à de A, 3 px ao longo da aresta
th = math.radians(30); u = (math.cos(th), math.sin(th)); v = (-math.sin(th), math.cos(th))
ca = (420, 220); wA, hA, wB, hB = 200, 120, 160, 100
def em(s, t): return [round(ca[0] + u[0] * s + v[0] * t, 3), round(ca[1] + u[1] * s + v[1] * t, 3)]
cenas["g3"] = {"elements": [rect_c("A", *ca, wA, hA, 30), rect_c("B", 900, 560, wB, hB, 30)]}
casos.append({"nome": "G3_os_dois_30", "cena": "g3",
    "pergunta": "A e B a 30°: aparece alguma guia paralela às arestas rodadas, ou só guias de eixo?",
    "sondas": [
        {"rotulo": "a_aresta_esq_colinear_3", "tipo": "arrasta", "mover": "B", "centro_em": em(-wA / 2 + wB / 2 + 3, hA / 2 + hB / 2 + 133)},
        {"rotulo": "b_aresta_inf_colinear_3", "tipo": "arrasta", "mover": "B", "centro_em": em(wA / 2 + wB / 2 + 108, hA / 2 - hB / 2 + 3)},
        {"rotulo": "c_aabb_esq", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.aabb.l", 3)},
        {"rotulo": "d_canto1_com_canto1", "tipo": "arrasta", "mover": "B", "x": X("cantos.1", "A.cantos.1", 3)},
    ]})
# ---- G4: rotação
cenas["g4"] = {"elements": [rect_c("A", 350, 300, 200, 120, 30), rect("B", 620, 330, 160, 100)]}
roda = lambda r, g, **kw: {"rotulo": r, "tipo": "roda", "el": "B", "graus": g, **kw}
casos.append({"nome": "G4_rotacao", "cena": "g4",
    "pergunta": "Rodar B (0°) perto de A (30°) pela pega: encaixa no ângulo de A, em 0, ou nada? E com Shift (passo)?",
    "sondas": [roda("a_28", 28), roda("b_2", 2), roda("c_menos2", -2), roda("d_31_5", 31.5), roda("e_44", 44),
               roda("f_28_sem_encaixe", 28, snap=False), roda("g_2_sem_encaixe", 2, snap=False),
               roda("h_shift_8", 8, shift=True), roda("i_shift_22", 22, shift=True), roda("j_shift_28", 28, shift=True),
               roda("k_shift_37", 37, shift=True), roda("l_shift_50", 50, shift=True), roda("m_shift_100", 100, shift=True)]})
# ---- G5: vãos
cenas["g5"] = {"elements": [rect("A", 200, 300, 100, 80), rect("B", 340, 300, 100, 80), rect("C", 560, 300, 100, 80)]}
cenas["g5_meio"] = {"elements": [rect("A", 200, 300, 100, 80), rect("B", 380, 520, 100, 80), rect("C", 500, 300, 100, 80)]}
cenas["g5_outra_linha"] = {"elements": [rect("A", 200, 300, 100, 80), rect("B", 340, 300, 100, 80), rect("C", 560, 520, 100, 80)]}
casos.append({"nome": "G5_vaos", "cena": "g5",
    "pergunta": "A|40|B e C arrastado para um vão de 43 / 37 / 41 / 46 depois de B: encaixa no vão igual (40)?",
    "sondas": [{"rotulo": f"vao_{g}", "tipo": "arrasta", "mover": "C", "x": X("aabb.l", "B.aabb.r", g)} for g in (43, 37, 41, 46)]
              + [{"rotulo": "vao_43_sem_encaixe", "tipo": "arrasta", "mover": "C", "x": X("aabb.l", "B.aabb.r", 43), "snap": False}]})
casos.append({"nome": "G5b_vao_meio", "cena": "g5_meio",
    "pergunta": "A(200..300) e C(500..600): B (100 de largura) posto 3 px fora do meio (vãos 53/47; o meio é 50/50): encaixa no meio?",
    "sondas": [{"rotulo": "meio_mais3", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.aabb.r", 53), "y": X("aabb.t", "A.aabb.t", 0)}]})
casos.append({"nome": "G5c_vao_outra_linha", "cena": "g5_outra_linha",
    "pergunta": "Vão 43→40 quando C NÃO se sobrepõe em y a A e B (C 220 abaixo): ainda encaixa?",
    "sondas": [{"rotulo": "vao_43_outra_linha", "tipo": "arrasta", "mover": "C", "x": X("aabb.l", "B.aabb.r", 43)}]})
# ---- G6: molduras
cenas["g6"] = {"elements": [
    rect("c1", 520, 250, 60, 60, frameId="F"), rect("c2", 640, 370, 90, 80, frameId="F"), frame("F", 400, 200, 400, 300),
    rect("R", 950, 600, 40, 80), rect("T", 100, 700, 80, 80), rect("U", 490, 60, 60, 60)]}
casos.append({"nome": "G6_molduras", "cena": "g6",
    "pergunta": "Moldura F (400,200,400×300) com c1,c2: é alvo? o que encaixa ao arrastá-la? um filho encaixa em quem? roda?",
    "sondas": [
        {"rotulo": "a_livre_na_esq_da_moldura", "tipo": "arrasta", "mover": "R", "x": X("aabb.l", "F.aabb.l", 3)},
        {"rotulo": "b_livre_na_dir_da_moldura", "tipo": "arrasta", "mover": "R", "x": X("aabb.r", "F.aabb.r", 3)},
        {"rotulo": "c_moldura_pelo_titulo_ao_T", "tipo": "arrasta", "mover": "F", "pega": "titulo", "x": X("aabb.l", "T.aabb.l", 3)},
        {"rotulo": "c2_moldura_topo_ao_R", "tipo": "arrasta", "mover": "F", "pega": "titulo", "y": X("aabb.b", "R.aabb.t", 3)},
        {"rotulo": "d_filho_ao_irmao", "tipo": "arrasta", "mover": "c1", "x": X("aabb.l", "c2.aabb.l", 3)},
        {"rotulo": "e_filho_a_borda_da_moldura", "tipo": "arrasta", "mover": "c1", "x": X("aabb.l", "F.aabb.l", 3)},
        {"rotulo": "f_filho_a_um_de_fora", "tipo": "arrasta", "mover": "c1", "x": X("aabb.l", "U.aabb.l", 3)},
        {"rotulo": "h_roda_a_moldura", "tipo": "roda", "el": "F", "graus": 30},
        {"rotulo": "i_roda_um_filho", "tipo": "roda", "el": "c2", "graus": 30},
    ]})
# ---- G7: lineares e traço livre como alvos
cenas["g7"] = {"elements": [
    linear("L", "line", 100, 150, [[0, 0], [120, 110], [200, 50]]),
    linear("S", "arrow", 400, 330, [[0, 0], [180, 70]]),
    linear("L2", "line", 500, 700, [[0, 0], [200, 30], [40, 60]]),
    linear("D", "freedraw", 760, 120, [[0, 0], [25, -20], [70, 10], [25, 60], [0, 90], [70, 120]], pressures=[], simulatePressure=True),
    rect("B", 900, 600, 10, 10)]}
casos.append({"nome": "G7_lineares", "cena": "g7",
    "pergunta": "Linha, seta e traço livre servem de alvo? (pontas, vértice do meio, caixa)",
    "sondas": [
        {"rotulo": "a_linha_ponta_esq", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "L.aabb.l", 3)},
        {"rotulo": "b_linha_vertice_meio", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "L.ponto.1", 3)},
        {"rotulo": "c_linha_dir", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "L.aabb.r", 3)},
        {"rotulo": "c2_linha_centro_aabb", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "L.aabb.c", 3)},
        {"rotulo": "c3_linha2_vertice_meio_e_extremo", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "L2.ponto.1", 3)},
        {"rotulo": "c4_linha2_ponta_final_interior", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "L2.ponto.2", 3)},
        {"rotulo": "d_seta_ponta_esq", "tipo": "arrasta", "mover": "B", "x": X("aabb.r", "S.aabb.l", -3), "y": X("aabb.t", "S.aabb.b", 120)},
        {"rotulo": "e_seta_ponta_dir", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "S.aabb.r", 3), "y": X("aabb.t", "S.aabb.b", 120)},
        {"rotulo": "f_tracolivre_esq", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "D.aabb.l", 3)},
        {"rotulo": "g_tracolivre_dir", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "D.aabb.r", 3)},
        {"rotulo": "h_tracolivre_vertice_meio", "tipo": "arrasta", "mover": "B", "x": X("aabb.r", "D.ponto.1", -3)},
    ]})
# ---- G8: tolerância (px de ecrã), zoom 1 e 2
cenas["g8_z1"] = {"elements": [rect("A", 200, 150, 200, 120), rect("B", 600, 400, 60, 60)]}
cenas["g8_z2"] = {"elements": [rect("A", 100, 60, 120, 80), rect("B", 400, 250, 40, 40)]}
ds = [d for d in range(-15, 16) if d != 0]
for z in (1, 2):
    casos.append({"nome": f"G8_tolerancia_zoom{z}", "cena": f"g8_z{z}", "zoom": z,
        "pergunta": f"Zoom {z}: a que distância (px de ECRÃ) da esquerda de A o encaixe deixa de acontecer?",
        "sondas": [{"rotulo": f"d{d:+d}", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.aabb.l", d)} for d in ds]
                  + [{"rotulo": "d+3_sem_encaixe", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.aabb.l", 3), "snap": False}]})
# ---- G0: o atalho Alt+S liga e desliga
casos.append({"nome": "G0_atalho_alt_s", "cena": "g8_z1",
    "pergunta": "Alt+S liga/desliga o encaixe a objectos? (o mesmo arrasto a 3 px com o modo desligado, ligado por Alt+S, desligado por Alt+S)",
    "sondas": [
        {"rotulo": "a_desligado", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.aabb.l", 3), "snap": False},
        {"rotulo": "b_alt_s_1x", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.aabb.l", 3), "snap": False, "teclas_antes": ["Alt+S"]},
        {"rotulo": "c_alt_s_2x", "tipo": "arrasta", "mover": "B", "x": X("aabb.l", "A.aabb.l", 3), "snap": False, "teclas_antes": ["Alt+S", "Alt+S"]},
    ]})
json.dump({"cenas": cenas, "casos": casos}, open(OUT, "w"), indent=1, ensure_ascii=False)
open(OUT, "a").write("\n")
print(len(casos), "casos", sum(len(c["sondas"]) for c in casos), "sondas")
