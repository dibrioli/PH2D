"""A referência de POSE e TEMPO do boneco de fábrica (plano 06 §2) — corre-se no Blender, sem interface:

    PH2D_GPU=1 bash scripts/ph2d-run.sh <blender> -b -P docs/GameEngineFuncional/ferramentas/boneco_blender.py -- <saída>

Monta o boneco verde de capacete do Kenney Pixel Platformer (CC0) como uma hierarquia de objectos (o rig:
raiz na sola → anca → tronco (squash & stretch) → cabeça; anca → coxa → canela, por perna), anima os quatro
ciclos com chaves Bézier a 30 q/s e, em cada desenho da folha, (1) renderiza de LADO em ortográfica um PNG
de referência e (2) grava as juntas em `poses.json`, que o `boneco_svg.py` lê para desenhar o vetor.

Unidades: 1 = a altura de uma célula da folha (o sprite de 1 m). Eixos do Blender: X para a frente (o boneco
olha para +X), Z para cima, a câmara em -Y. A sola fica no z = 0 e a célula vai de z = -SOLA_MARGEM a
1 - SOLA_MARGEM.
"""

import json
import math
import os
import sys

import bpy

SAIDA = sys.argv[sys.argv.index("--") + 1] if "--" in sys.argv else "/tmp/boneco_ref"
FPS = 30
CELULA_PX = 192
SOLA_MARGEM = 0.05  # da sola ao fundo da célula (o estique do salto chega a 0,03 do topo)

# Proporções do tile_0000 do Kenney (o capacete domina), com as pernas de 0,14 para 0,20 da célula: com as
# do pixel (3 de 22) a corrida não se lê de lado.
COXA = 0.10
CANELA = 0.10
PERNA_R = 0.042  # meia-grossura
ANCA_Z = COXA + CANELA + PERNA_R  # o pé do vetor é a ponta redonda da canela: a sola fica no z = 0
TRONCO = (0.30, 0.12)  # largura, altura (centro 0,05 acima da anca)
CABECA_PIVO = 0.27  # da anca ao pivô da cabeça
CAPACETE_R = 0.27
CABECA = (0.34, 0.28)

# Paleta do tile_0000 (histograma do PNG do pacote).
VERDE = (0x5A, 0xD2, 0x8C)
VERDE_ESCURO = (0x34, 0x65, 0x47)
VERDE_MEIO = (0x34, 0x99, 0x5E)
VIDRO = (0xBE, 0xDE, 0xF1)

# Os ciclos: chaves (quadro, pose) e os quadros que viram desenhos da folha. Pose: anca (dz, inclinação para a
# frente em graus), tronco (sx, sz), cabeça (graus), pernas (coxa para a frente, joelho dobrado) — R é a perna
# do lado da câmara.
def pose(dz=0.0, lean=0.0, sx=1.0, sz=1.0, head=0.0, r=(0.0, 0.0), l=(0.0, 0.0), neck=0.0):
    return dict(dz=dz, lean=lean, sx=sx, sz=sz, head=head, r=r, l=l, neck=neck)


def espelho(p):
    q = dict(p)
    q["r"], q["l"] = p["l"], p["r"]
    return q


def pe(x, alto=0.0):
    """Perna de APOIO: o pé em `x`, plantado no chão (ou `alto` acima dele) — coxa e canela saem do IK de dois ossos."""
    return ("pe", x, alto)


def ik(dz, x, alto=0.0):
    h = ANCA_Z + dz - PERNA_R - alto
    d = min(math.hypot(x, h), COXA + CANELA - 1e-6)
    base = math.degrees(math.atan2(x, h))
    meio = math.degrees(math.acos(d / (COXA + CANELA)))
    return (base + meio, 2 * meio)


CONTACTO = pose(dz=-0.016, lean=5, head=-2, r=pe(0.07), l=(-32, 40))
BAIXO = pose(dz=-0.040, lean=6, sx=1.05, sz=0.94, head=1, r=pe(0.025), l=(-14, 76))
PASSAGEM = pose(dz=-0.006, lean=5, head=2, r=pe(-0.045), l=(28, 96))
CIMA = pose(dz=0.030, lean=4, sx=0.97, sz=1.05, head=0, r=(-36, 6), l=(46, 46))

AGACHADO = pose(dz=-0.082, lean=6, sx=1.12, sz=0.80, head=-2, neck=-0.14, r=pe(0.06), l=pe(-0.05))
ARRASTA = pose(dz=-0.084, lean=8, sx=1.12, sz=0.80, head=-2, neck=-0.14, r=pe(0.07), l=pe(-0.06))
ARRASTA_PASSA = pose(dz=-0.078, lean=8, sx=1.10, sz=0.80, head=0, neck=-0.14, r=pe(0.0), l=pe(0.0, 0.03))

CICLOS = {
    # Respirar: 1 s, seis desenhos.
    "idle": dict(
        chaves=[
            (0, pose(dz=-0.004, r=pe(0.045), l=pe(-0.045))),
            (15, pose(dz=-0.018, sx=1.03, sz=0.96, head=-2, r=pe(0.045), l=pe(-0.045))),
            (30, pose(dz=-0.004, r=pe(0.045), l=pe(-0.045))),
        ],
        desenhos=[0, 5, 10, 15, 20, 25],
        fim=30,
        repete=True,
    ),
    # Correr: dois passos em 16 quadros (0,53 s) — contacto, baixo, passagem, cima, e o mesmo do outro pé.
    "run": dict(
        chaves=[
            (0, CONTACTO),
            (2, BAIXO),
            (4, PASSAGEM),
            (6, CIMA),
            (8, espelho(CONTACTO)),
            (10, espelho(BAIXO)),
            (12, espelho(PASSAGEM)),
            (14, espelho(CIMA)),
            (16, CONTACTO),
        ],
        desenhos=[0, 2, 4, 6, 8, 10, 12, 14],
        fim=16,
        repete=True,
    ),
    # Saltar (já no ar, a subir): o estique da largada, as pernas a encolher, o ápice — uma vez, fica no último.
    "jump": dict(
        chaves=[
            (0, pose(dz=0.035, lean=4, sx=0.90, sz=1.13, head=-3, r=(-18, 6), l=(-28, 12))),
            (3, pose(dz=0.030, lean=2, sx=0.95, sz=1.07, head=-1, r=(18, 58), l=(-6, 70))),
            (7, pose(dz=0.024, lean=0, sx=1.0, sz=1.0, head=1, r=(36, 92), l=(12, 98))),
            (12, pose(dz=0.020, lean=-2, sx=1.02, sz=0.98, head=2, r=(30, 84), l=(8, 92))),
        ],
        desenhos=[0, 3, 7, 12],
        fim=16,
        repete=False,
    ),
    # Cair: pernas a pender e a pedalar devagar, o corpo um pouco esticado — um laço de 12 quadros.
    "fall": dict(
        chaves=[
            (0, pose(dz=0.02, lean=-3, sx=0.97, sz=1.04, head=-3, r=(16, 22), l=(-14, 30))),
            (6, pose(dz=0.024, lean=-4, sx=0.96, sz=1.05, head=-4, r=(6, 10), l=(-22, 16))),
            (12, pose(dz=0.02, lean=-3, sx=0.97, sz=1.04, head=-3, r=(16, 22), l=(-14, 30))),
        ],
        desenhos=[0, 3, 6, 9],
        fim=12,
        repete=True,
    ),
    # Agachado (plano 10): a anca desce, a cabeça afunda no tronco e o corpo achata até o topo do capacete
    # ficar a ~0,50 da sola (o corpo agachado); os pés não saem da linha. Respira devagar.
    "crouch": dict(
        chaves=[
            (0, AGACHADO),
            (12, pose(dz=-0.086, lean=6, sx=1.14, sz=0.78, head=-3, neck=-0.14, r=pe(0.06), l=pe(-0.05))),
            (24, AGACHADO),
        ],
        desenhos=[0, 6, 12, 18],
        fim=24,
        repete=True,
    ),
    # A andar agachado: dois passos arrastados (o pé de trás sobe pouco e passa por baixo), sem quase subir.
    "crouch_walk": dict(
        chaves=[
            (0, ARRASTA),
            (4, ARRASTA_PASSA),
            (8, espelho(ARRASTA)),
            (12, espelho(ARRASTA_PASSA)),
            (16, ARRASTA),
        ],
        desenhos=[0, 4, 8, 12],
        fim=16,
        repete=True,
    ),
}


def srgb_linear(c):
    def f(v):
        v /= 255.0
        return v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4

    return (f(c[0]), f(c[1]), f(c[2]), 1.0)


def limpa():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def vazio(nome, pai=None, loc=(0, 0, 0)):
    o = bpy.data.objects.new(nome, None)
    bpy.context.scene.collection.objects.link(o)
    o.parent = pai
    o.location = loc
    o.rotation_mode = "XYZ"
    return o


def malha(nome, op, pai, cor, loc, escala, **kw):
    op(location=(0, 0, 0), **kw)
    o = bpy.context.active_object
    o.name = nome
    o.scale = escala
    o.parent = pai
    o.location = loc
    o.color = srgb_linear(cor)
    return o


def rig():
    raiz = vazio("raiz")
    anca = vazio("anca", raiz, (0, 0, ANCA_Z))
    tronco = vazio("tronco", anca)
    malha("corpo", bpy.ops.mesh.primitive_cube_add, tronco, VERDE_MEIO,
          (0, 0, 0.05), (TRONCO[0] / 2, 0.08, TRONCO[1] / 2))
    cabeca = vazio("cabeca", tronco, (0, 0, CABECA_PIVO))
    malha("capacete", bpy.ops.mesh.primitive_uv_sphere_add, cabeca, VIDRO,
          (0, 0.05, 0.03), (CAPACETE_R,) * 3, segments=48, ring_count=24)
    malha("rosto", bpy.ops.mesh.primitive_cube_add, cabeca, VERDE,
          (0.02, -0.25, 0), (CABECA[0] / 2, 0.01, CABECA[1] / 2))
    pernas = {}
    for lado, y, cor in (("r", -0.10, VERDE_MEIO), ("l", 0.10, VERDE_ESCURO)):
        coxa = vazio(f"coxa_{lado}", anca, (0, y, 0))
        malha(f"coxa_{lado}_m", bpy.ops.mesh.primitive_cylinder_add, coxa, cor,
              (0, 0, -COXA / 2), (PERNA_R, PERNA_R, COXA / 2), vertices=24)
        canela = vazio(f"canela_{lado}", coxa, (0, 0, -COXA))
        malha(f"canela_{lado}_m", bpy.ops.mesh.primitive_cylinder_add, canela, cor,
              (0, 0, -CANELA / 2), (PERNA_R, PERNA_R, CANELA / 2), vertices=24)
        pe = vazio(f"pe_{lado}", canela, (0, 0, -CANELA))
        pernas[lado] = (coxa, canela, pe)
    return dict(raiz=raiz, anca=anca, tronco=tronco, cabeca=cabeca, pernas=pernas)


def aplica(r, p):
    r["anca"].location = (0, 0, ANCA_Z + p["dz"])
    r["anca"].rotation_euler = (0, math.radians(p["lean"]), 0)
    r["tronco"].scale = (p["sx"], 1, p["sz"])
    r["cabeca"].location = (0, 0, CABECA_PIVO + p["neck"])
    r["cabeca"].rotation_euler = (0, math.radians(p["head"]), 0)
    for lado in ("r", "l"):
        perna = p[lado]
        frente, joelho = ik(p["dz"], *perna[1:]) if perna[0] == "pe" else perna
        coxa, canela, _ = r["pernas"][lado]
        # rotação +Y leva o topo para +X e o pé para -X: a coxa à frente é negativa (medida contra a
        # vertical do MUNDO, por isso desfaz a inclinação da anca), o joelho dobra para trás.
        coxa.rotation_euler = (0, math.radians(-frente - p["lean"]), 0)
        canela.rotation_euler = (0, math.radians(joelho), 0)


def chaveia(r, chaves):
    objs = [r["anca"], r["tronco"], r["cabeca"]] + [o for lado in r["pernas"].values() for o in lado[:2]]
    for o in objs:
        o.animation_data_clear()
    for q, p in chaves:
        aplica(r, p)
        for o in objs:
            o.keyframe_insert("location", frame=q)
            o.keyframe_insert("rotation_euler", frame=q)
            o.keyframe_insert("scale", frame=q)
    # As chaves novas já nascem Bézier com alças auto-clamped (o padrão de fábrica): a entrada e a saída
    # de cada pose suavizam-se sem passar do extremo.


def xz(o):
    m = o.matrix_world
    return [round(m.translation.x, 5), round(m.translation.z, 5)]


def afim_2d(o):
    """A matriz 2D (X, Z) do objecto: colunas = eixos X e Z do objecto no mundo, e a origem."""
    m = o.matrix_world
    return [round(v, 5) for v in (m[0][0], m[2][0], m[0][2], m[2][2], m[0][3], m[2][3])]


def cena():
    s = bpy.context.scene
    s.render.fps = FPS
    s.render.engine = "BLENDER_WORKBENCH"
    s.display.shading.light = "FLAT"
    s.display.shading.color_type = "OBJECT"
    s.view_settings.view_transform = "Standard"  # a cor do objecto sai como foi escrita (sem AgX)
    s.render.film_transparent = True
    s.render.resolution_x = s.render.resolution_y = CELULA_PX
    s.render.image_settings.file_format = "PNG"
    s.render.image_settings.color_mode = "RGBA"
    cam = bpy.data.cameras.new("cam")
    cam.type = "ORTHO"
    cam.ortho_scale = 1.0
    o = bpy.data.objects.new("cam", cam)
    s.collection.objects.link(o)
    o.location = (0, -5, 0.5 - SOLA_MARGEM)
    o.rotation_euler = (math.radians(90), 0, 0)
    s.camera = o


def main():
    limpa()
    cena()
    r = rig()
    s = bpy.context.scene
    os.makedirs(SAIDA, exist_ok=True)
    saida = {
        "cabecalho": {
            "ferramenta": f"Blender {bpy.app.version_string}",
            "script": "docs/GameEngineFuncional/ferramentas/boneco_blender.py",
            "fonte": "Kenney Pixel Platformer 1.2 (CC0), Tiles/Characters/tile_0000.png",
            "fps": FPS,
            "celula": {"px": CELULA_PX, "sola_margem": SOLA_MARGEM},
            "medidas": {"anca_z": ANCA_Z, "coxa": COXA, "canela": CANELA, "perna_r": PERNA_R, "tronco": TRONCO,
                        "cabeca_pivo": CABECA_PIVO, "capacete_r": CAPACETE_R, "cabeca": CABECA},
        },
        "ciclos": {},
    }
    for nome, c in CICLOS.items():
        chaveia(r, c["chaves"])
        desenhos = []
        qs = c["desenhos"] + [c["fim"]]
        for i, q in enumerate(c["desenhos"]):
            s.frame_set(q)
            bpy.context.view_layer.update()
            d = {
                "quadro": q,
                "ms": round((qs[i + 1] - q) * 1000 / FPS),
                "tronco": afim_2d(r["tronco"]),
                "cabeca": afim_2d(r["cabeca"]),
                "pernas": {lado: [xz(o) for o in r["pernas"][lado]] for lado in ("r", "l")},
            }
            desenhos.append(d)
            s.render.filepath = os.path.join(SAIDA, f"{nome}_{i}.png")
            bpy.ops.render.render(write_still=True)
        saida["ciclos"][nome] = {"repete": c["repete"], "desenhos": desenhos}
    with open(os.path.join(SAIDA, "poses.json"), "w") as f:
        json.dump(saida, f, indent=1)
    print("[boneco_blender] ok →", SAIDA)


main()
