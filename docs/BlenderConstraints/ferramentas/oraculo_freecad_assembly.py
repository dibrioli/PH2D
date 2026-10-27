"""Oráculo FreeCAD Assembly da cola da W5 — CORRE-SE, nunca se lê o fonte (FreeCAD 1.1.4; triagem em 05 §3.1).

Uso (sem placa, sem interface; o caminho de saída é absoluto):
    W5_SAIDA="$PWD/docs/BlenderConstraints/fixtures/freecad_1_1_4_assembly_cola.json" \
    bash scripts/ph2d-run.sh freecadcmd docs/BlenderConstraints/ferramentas/oraculo_freecad_assembly.py

Cada corpo é um `Part::Feature` feito só dos seus pontos de pino (vértices em coordenadas locais), no plano XY.
Pino = junta `Revolute` (eixo Z), Rígido = junta `Fixed`; a referência é enraizada no componente
(`[corpo, ["VertexN", "VertexN"]]`, o idioma do 1.1). Um condutor é um `GroundedJoint` posto DEPOIS de as juntas
existirem e de cada corpo voltar à pose autorada (o palpite) — a criação da junta pré-alinha os corpos.
As entradas são NOSSAS e viajam na fixture (`casos[].corpos/juntas/condutores`): o gate lê-as dali.
"""

import json
import math
import os

import FreeCAD as App
import JointObject
import Part

Z = App.Vector(0, 0, 1)


def placement(p):
    return App.Placement(App.Vector(p[0], p[1], 0), App.Rotation(Z, p[2]))


def pose(pl):
    yaw = pl.Rotation.toEuler()[0]
    return [pl.Base.x, pl.Base.y, yaw]


def corre(caso):
    doc = App.newDocument(caso["nome"])
    asm = doc.addObject("Assembly::AssemblyObject", "Assembly")
    jg = asm.newObject("Assembly::JointGroup", "Joints")
    corpos = {}
    for c in caso["corpos"]:
        o = asm.newObject("Part::Feature", c["nome"])
        o.Shape = Part.Compound([Part.Vertex(App.Vector(x, y, 0)) for x, y in c["pontos"]])
        corpos[c["nome"]] = o
    doc.recompute()
    for i, j in enumerate(caso["juntas"]):
        o = jg.newObject("App::FeaturePython", f"J{i}")
        JointObject.Joint(o, JointObject.JointTypes.index({"pino": "Revolute", "rigida": "Fixed"}[j["tipo"]]))
        va, vb = f"Vertex{j['ia'] + 1}", f"Vertex{j['ib'] + 1}"
        o.Proxy.setJointConnectors(o, [[corpos[j["a"]], [va, va]], [corpos[j["b"]], [vb, vb]]])
    for c in caso["corpos"]:
        corpos[c["nome"]].Placement = placement(c["autorada"])
    for nome, p in caso["condutores"].items():
        corpos[nome].Placement = placement(p)
        g = jg.newObject("App::FeaturePython", f"G_{nome}")
        JointObject.GroundedJoint(g, corpos[nome])
    codigo = asm.solve()
    saida = {n: pose(o.Placement) for n, o in corpos.items()}
    pontos = {c["nome"]: c["pontos"] for c in caso["corpos"]}
    mundo = lambda n, i: corpos[n].Placement.multVec(App.Vector(pontos[n][i][0], pontos[n][i][1], 0))
    residuo = max((mundo(j["a"], j["ia"]) - mundo(j["b"], j["ib"])).Length for j in caso["juntas"])
    App.closeDocument(doc.Name)
    return {"codigo": codigo, "residuo": residuo, "poses": saida}


def braco(l_chao, l_manivela, l_biela, l_balancim):
    """Quadrilátero articulado: chão O2=(0,0)–O4=(l_chao,0); poses autoradas com a manivela a 0°, ramo de cima."""
    ax, ay = l_manivela, 0.0
    d = math.hypot(l_chao - ax, -ay)
    a = (l_biela**2 - l_balancim**2 + d**2) / (2 * d)
    h = math.sqrt(max(l_biela**2 - a**2, 0.0))
    ux, uy = (l_chao - ax) / d, -ay / d
    bx, by = ax + a * ux - h * uy, ay + a * uy + h * ux
    corpos = [
        {"nome": "chao", "pontos": [[0, 0], [l_chao, 0]], "autorada": [0, 0, 0]},
        {"nome": "manivela", "pontos": [[0, 0], [l_manivela, 0]], "autorada": [0, 0, 0]},
        {"nome": "biela", "pontos": [[0, 0], [l_biela, 0]],
         "autorada": [ax, ay, math.degrees(math.atan2(by - ay, bx - ax))]},
        {"nome": "balancim", "pontos": [[0, 0], [l_balancim, 0]],
         "autorada": [l_chao, 0, math.degrees(math.atan2(by, bx - l_chao))]},
    ]
    juntas = [
        {"tipo": "pino", "a": "chao", "ia": 0, "b": "manivela", "ib": 0},
        {"tipo": "pino", "a": "manivela", "ia": 1, "b": "biela", "ib": 0},
        {"tipo": "pino", "a": "biela", "ia": 1, "b": "balancim", "ib": 1},
        {"tipo": "pino", "a": "chao", "ia": 1, "b": "balancim", "ib": 0},
    ]
    return corpos, juntas


def casos():
    out = []
    cadeia = [
        {"nome": "p1", "pontos": [[0, 0], [2, 0]], "autorada": [0, 0, 0]},
        {"nome": "p2", "pontos": [[0, 0], [2, 0]], "autorada": [2, 0, 0]},
        {"nome": "p3", "pontos": [[0, 0], [2, 0]], "autorada": [4, 0, 0]},
    ]
    out.append({"nome": "cadeia_dois_pinos", "corpos": cadeia,
                "juntas": [{"tipo": "pino", "a": "p1", "ia": 1, "b": "p2", "ib": 0},
                           {"tipo": "pino", "a": "p2", "ia": 1, "b": "p3", "ib": 0}],
                "condutores": {"p1": [0, 0, 30]}})
    rigida = [
        {"nome": "a", "pontos": [[1, 0.5]], "autorada": [1, 2, 0]},
        {"nome": "b", "pontos": [[0, 0]], "autorada": [2, 2.5, 0]},
    ]
    out.append({"nome": "rigida", "corpos": rigida,
                "juntas": [{"tipo": "rigida", "a": "a", "ia": 0, "b": "b", "ib": 0}],
                "condutores": {"a": [3, -1, 40]}})
    corpos, juntas = braco(4.0, 1.0, 3.5, 3.0)
    for ang in (0, 30, 60, 90):
        out.append({"nome": f"quadrilatero_manivela_{ang}", "corpos": corpos, "juntas": juntas,
                    "condutores": {"chao": [0, 0, 0], "manivela": [0, 0, ang]}})
    bal = corpos[3]["autorada"][2]
    for d in (0, 5, 15, 25):  # curso do balancim com estas medidas: 101,4°–141,4° (colineares)
        out.append({"nome": f"quadrilatero_balancim_{d}", "corpos": corpos, "juntas": juntas,
                    "condutores": {"chao": [0, 0, 0], "balancim": [4.0, 0, bal + d]}})
    corpos, juntas = braco(4.0, 1.0, 1.0, 0.5)
    out.append({"nome": "laco_impossivel", "corpos": corpos, "juntas": juntas,
                "condutores": {"chao": [0, 0, 0], "manivela": [0, 0, 30]}})
    return out


resultado = []
for c in casos():
    r = corre(c)
    resultado.append({**c, "freecad": r})
    print(c["nome"], r["codigo"], f"{r['residuo']:.2e}", {k: [round(v, 6) for v in p] for k, p in r["poses"].items()})

fixture = {
    "cabecalho": {
        "oraculo": "FreeCAD 1.1.4 Assembly (freecadcmd, OndselSolver/MbD), LGPL — corrido, nunca lido",
        "gerador": "docs/BlenderConstraints/ferramentas/oraculo_freecad_assembly.py",
        "unidades": "poses [x, y, graus]; pontos de pino em coordenadas locais do corpo",
        "juntas": "pino = Revolute (eixo Z), rigida = Fixed; condutor = GroundedJoint na pose dada",
        "palpite": "cada corpo começa na pose autorada do caso (o mesmo palpite do nosso solver)",
        "codigo": "AssemblyObject.solve(): 0 ok, -3 conflito, -4 sobre-restrito, -1 erro do solver — MEDIDO: dá 0 num laço que não fecha; quem decide é o residuo",
        "residuo": "maior distância no mundo entre os dois pontos de cada junta, depois do solve",
        "data": "2026-10-27",
    },
    "casos": resultado,
}
with open(os.environ["W5_SAIDA"], "w") as f:
    json.dump(fixture, f, indent=1)
