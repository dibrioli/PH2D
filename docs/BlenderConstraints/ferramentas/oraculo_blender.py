"""Oráculo Blender das constraints em 2D (plano XY, rotação em Z) — CORRE-SE, nunca se lê o fonte.

Uso (a placa pede-se à partida, o Blender não pede a vez):
    PH2D_GPU=1 bash scripts/ph2d-run.sh env \
        LD_LIBRARY_PATH=/home/enio/Apps/ph2d-motionbricks/.venv/lib/python3.12/site-packages/nvidia/cu13/lib \
        blender -b --factory-startup \
        --python docs/BlenderConstraints/ferramentas/oraculo_blender.py -- "$PWD/<saida.json>"

(o `blender` 5.2.3 do pacman não arranca sem uma `libcudart.so.13`, medido 25/10; o caminho de saída
tem de ser absoluto.)

Cada caso monta uma cena mínima, avalia o depsgraph e grava a pose MUNDIAL 2D de quem é
restringido: x, y, rotação em graus, escala x/y. A saída é a fixture que os gates do módulo leem.
"""

import json
import math
import sys

import bpy
from mathutils import Vector


def limpa():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def vazio(nome, loc=(0.0, 0.0), rot_deg=0.0, esc=(1.0, 1.0), pai=None):
    o = bpy.data.objects.new(nome, None)
    bpy.context.scene.collection.objects.link(o)
    o.location = (loc[0], loc[1], 0.0)
    o.rotation_euler = (0.0, 0.0, math.radians(rot_deg))
    o.scale = (esc[0], esc[1], 1.0)
    if pai is not None:
        o.parent = pai
    return o


def pose2d(m):
    loc, rot, esc = m.decompose()
    eixo_x = m.to_3x3() @ Vector((1.0, 0.0, 0.0))
    return {
        "x": round(loc.x, 6),
        "y": round(loc.y, 6),
        "rot_deg": round(math.degrees(math.atan2(eixo_x.y, eixo_x.x)), 6),
        "sx": round(esc.x, 6),
        "sy": round(esc.y, 6),
    }


def avalia(o):
    bpy.context.view_layer.update()
    dg = bpy.context.evaluated_depsgraph_get()
    return pose2d(o.evaluated_get(dg).matrix_world)


def caso_mistura_rotacao():
    """Copy Rotation a 50 % entre 170° e -170°: arco curto (180°) ou longo (0°)?"""
    out = {}
    for infl in (0.0, 0.25, 0.5, 0.75, 1.0):
        limpa()
        alvo = vazio("alvo", rot_deg=-170.0)
        dono = vazio("dono", rot_deg=170.0)
        c = dono.constraints.new("COPY_ROTATION")
        c.target = alvo
        c.influence = infl
        out[str(infl)] = avalia(dono)
    return out


def caso_child_of():
    """Child Of: ao ligar (com e sem Set Inverse) o dono pula? E segue o alvo depois?"""
    out = {}
    for set_inverse in (False, True):
        limpa()
        avo = vazio("avo", loc=(1.0, -2.0), rot_deg=-20.0, esc=(1.2, 1.2))
        alvo = vazio("alvo", loc=(5.0, 0.0), rot_deg=37.0, esc=(1.5, 1.5), pai=avo)
        dono = vazio("dono", loc=(2.0, 1.0), rot_deg=30.0)
        antes = avalia(dono)
        c = dono.constraints.new("CHILD_OF")
        c.target = alvo
        if set_inverse:
            bpy.context.view_layer.update()
            c.inverse_matrix = alvo.matrix_world.inverted()
        ligado = avalia(dono)
        alvo.location.x += 1.0
        alvo.rotation_euler.z += math.radians(10.0)
        movido = avalia(dono)
        out["set_inverse" if set_inverse else "sem_inverse"] = {
            "antes": antes,
            "ao_ligar": ligado,
            "alvo_movido": movido,
        }
    return out


def caso_aponta():
    """Os três 'apontar' do Blender no plano: dão o mesmo ângulo?"""
    out = {}
    for tipo in ("DAMPED_TRACK", "TRACK_TO", "LOCKED_TRACK"):
        limpa()
        alvo = vazio("alvo", loc=(3.0, 4.0))
        dono = vazio("dono", rot_deg=10.0)
        c = dono.constraints.new(tipo)
        c.target = alvo
        if tipo == "DAMPED_TRACK":
            c.track_axis = "TRACK_X"
        elif tipo == "TRACK_TO":
            c.track_axis = "TRACK_X"
            c.up_axis = "UP_Z"
        else:
            c.track_axis = "TRACK_X"
            c.lock_axis = "LOCK_Z"
        out[tipo] = avalia(dono)
    return out


def caso_limite_angulo():
    """Limit Rotation Z em [-45°, 45°]: para que borda vai cada rotação?"""
    out = {}
    for r in (30.0, 60.0, 120.0, 170.0, -170.0, -100.0):
        limpa()
        dono = vazio("dono", rot_deg=r)
        c = dono.constraints.new("LIMIT_ROTATION")
        c.use_limit_z = True
        c.min_z = math.radians(-45.0)
        c.max_z = math.radians(45.0)
        out[str(r)] = avalia(dono)
    return out


def caso_distancia():
    """Limit Distance Inside, raio 2: dono a 5 do alvo."""
    limpa()
    alvo = vazio("alvo")
    dono = vazio("dono", loc=(5.0, 0.0))
    c = dono.constraints.new("LIMIT_DISTANCE")
    c.target = alvo
    c.distance = 2.0
    c.limit_mode = "LIMITDIST_INSIDE"
    return avalia(dono)


def caso_ordem():
    """Copy Location + Limit Location: a ordem da pilha muda o resultado?"""
    out = {}
    for ordem in (("COPY_LOCATION", "LIMIT_LOCATION"), ("LIMIT_LOCATION", "COPY_LOCATION")):
        limpa()
        alvo = vazio("alvo", loc=(8.0, 1.0))
        dono = vazio("dono")
        for tipo in ordem:
            c = dono.constraints.new(tipo)
            if tipo == "COPY_LOCATION":
                c.target = alvo
            else:
                c.use_max_x = True
                c.max_x = 3.0
        out[" > ".join(ordem)] = avalia(dono)
    return out


def caso_ik_por_ultimo():
    """Osso 2 com IK e, ABAIXO dele, um Copy Rotation do alvo: o IK corre por último?"""
    limpa()
    arm_data = bpy.data.armatures.new("rig")
    arm = bpy.data.objects.new("rig", arm_data)
    bpy.context.scene.collection.objects.link(arm)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.mode_set(mode="EDIT")
    b1 = arm_data.edit_bones.new("b1")
    b1.head, b1.tail = (0, 0, 0), (1, 0, 0)
    b2 = arm_data.edit_bones.new("b2")
    b2.head, b2.tail = (1, 0, 0), (2, 0, 0)
    b2.parent = b1
    b2.use_connect = True
    for b in (b1, b2):
        b.roll = 0.0
    bpy.ops.object.mode_set(mode="OBJECT")
    alvo = vazio("alvo", loc=(1.2, 0.9))
    rodador = vazio("rodador", rot_deg=90.0)
    pb2 = arm.pose.bones["b2"]
    ik = pb2.constraints.new("IK")
    ik.target = alvo
    ik.chain_count = 2
    copia = pb2.constraints.new("COPY_ROTATION")
    copia.target = rodador
    bpy.context.view_layer.update()
    dg = bpy.context.evaluated_depsgraph_get()
    ev = arm.evaluated_get(dg)
    res = {}
    for n in ("b1", "b2"):
        m = ev.matrix_world @ ev.pose.bones[n].matrix
        cauda = ev.matrix_world @ ev.pose.bones[n].tail
        p = pose2d(m)
        p["cauda_x"] = round(cauda.x, 6)
        p["cauda_y"] = round(cauda.y, 6)
        res[n] = p
    return res


# ─── Onda 2: ossos ────────────────────────────────────────────────────────────────────────────────
# O comprimento do osso no Blender é o eixo Y local; aqui grava-se a cabeça, a cauda, o rumo do osso
# (cabeça → cauda, graus) e o factor de comprimento (|cauda − cabeça| / comprimento de repouso).
# ⚠️ O Blender 5.2 NÃO tem «Softness» na IK (é do Spine, que só corre com interface): essa lei não
# tem oráculo corrível — ver docs/BlenderConstraints/01_plano.md §5, onda 2.


def rig(ossos, escala_obj=(1.0, 1.0)):
    """`ossos` = [(nome, cabeça, cauda, pai)] ligados; devolve o objecto armadura."""
    arm_data = bpy.data.armatures.new("rig")
    arm = bpy.data.objects.new("rig", arm_data)
    bpy.context.scene.collection.objects.link(arm)
    arm.scale = (escala_obj[0], escala_obj[1], 1.0)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.mode_set(mode="EDIT")
    for nome, cabeca, cauda, pai in ossos:
        b = arm_data.edit_bones.new(nome)
        b.head = (cabeca[0], cabeca[1], 0.0)
        b.tail = (cauda[0], cauda[1], 0.0)
        b.roll = 0.0
        if pai is not None:
            b.parent = arm_data.edit_bones[pai]
            b.use_connect = True
    bpy.ops.object.mode_set(mode="OBJECT")
    return arm


def osso2d(arm, nome):
    bpy.context.view_layer.update()
    dg = bpy.context.evaluated_depsgraph_get()
    ev = arm.evaluated_get(dg)
    pb = ev.pose.bones[nome]
    cab = ev.matrix_world @ pb.head
    cau = ev.matrix_world @ pb.tail
    m = ev.matrix_world @ pb.matrix
    d = cau - cab
    return {
        "cabeca_x": round(cab.x, 6),
        "cabeca_y": round(cab.y, 6),
        "cauda_x": round(cau.x, 6),
        "cauda_y": round(cau.y, 6),
        "rumo_deg": round(math.degrees(math.atan2(d.y, d.x)), 6),
        "comprimento": round(d.length / pb.bone.length, 6),
        "largura": round((m.to_3x3() @ Vector((1.0, 0.0, 0.0))).length, 6),
        "fundo": round((m.to_3x3() @ Vector((0.0, 0.0, 1.0))).length, 6),
    }


# Braço dobrado para CIMA em repouso (o lado da dobra é o do repouso, sem pólo): b1 1,0, b2 1,0.
BRACO = [
    ("b1", (0.0, 0.0), (1.0, 0.0), None),
    ("b2", (1.0, 0.0), (1.0 + math.cos(math.radians(30.0)), math.sin(math.radians(30.0))), "b1"),
]


def caso_ik_dois_ossos():
    """IK de 2 ossos sem pólo: a cauda chega ao alvo? De que lado dobra? E fora de alcance?"""
    out = {}
    alvos = {"perto": (1.2, 0.9), "baixo": (1.5, -0.6), "fora": (3.0, 1.0), "colado": (0.3, 0.1)}
    for espelho in (False, True):
        for nome, (ax, ay) in alvos.items():
            limpa()
            sx = -1.0 if espelho else 1.0
            arm = rig(BRACO, escala_obj=(sx, 1.0))
            alvo = vazio("alvo", loc=(ax * sx, ay))
            ik = arm.pose.bones["b2"].constraints.new("IK")
            ik.target = alvo
            ik.chain_count = 2
            ik.use_stretch = False
            chave = ("espelho." if espelho else "") + nome
            out[chave] = {
                "alvo": [ax * sx, ay],
                "b1": osso2d(arm, "b1"),
                "b2": osso2d(arm, "b2"),
            }
    return out


def caso_ik_estica():
    """IK com Stretch (os dois ossos com `ik_stretch` 1): fora de alcance, quem estica e quanto?"""
    out = {}
    for nome, (ax, ay) in {"fora": (3.0, 1.0), "perto": (1.2, 0.9)}.items():
        limpa()
        arm = rig(BRACO)
        alvo = vazio("alvo", loc=(ax, ay))
        for n in ("b1", "b2"):
            arm.pose.bones[n].ik_stretch = 1.0
        ik = arm.pose.bones["b2"].constraints.new("IK")
        ik.target = alvo
        ik.chain_count = 2
        ik.use_stretch = True
        out[nome] = {"alvo": [ax, ay], "b1": osso2d(arm, "b1"), "b2": osso2d(arm, "b2")}
    return out


def caso_ik_cadeia():
    """Cadeia de 3 ossos: a cauda chega? (o solver dele é iterativo; o PH2D compara a CHEGADA e os
    comprimentos, não o ângulo de cada osso — uma cadeia de 3 tem infinitas soluções)."""
    limpa()
    arm = rig(
        [
            ("c1", (0.0, 0.0), (1.0, 0.0), None),
            ("c2", (1.0, 0.0), (1.7, 0.3), "c1"),
            ("c3", (1.7, 0.3), (2.2, 0.6), "c2"),
        ]
    )
    alvo = vazio("alvo", loc=(1.0, 1.4))
    ik = arm.pose.bones["c3"].constraints.new("IK")
    ik.target = alvo
    ik.chain_count = 3
    ik.use_stretch = False
    return {"alvo": [1.0, 1.4], **{n: osso2d(arm, n) for n in ("c1", "c2", "c3")}}


def caso_stretch_to():
    """Stretch To num osso de comprimento 1 (repouso 1): alvo a 2 e a 0,5 — e o volume por modo."""
    out = {}
    for volume in ("VOLUME_XZX", "VOLUME_X", "VOLUME_Z", "NO_VOLUME"):
        for dist in (2.0, 0.5):
            limpa()
            arm = rig([("s", (0.0, 0.0), (1.0, 0.0), None)])
            ang = math.radians(25.0)
            alvo = vazio("alvo", loc=(dist * math.cos(ang), dist * math.sin(ang)))
            c = arm.pose.bones["s"].constraints.new("STRETCH_TO")
            c.target = alvo
            c.rest_length = 1.0
            c.volume = volume
            c.bulge = 1.0
            out[f"{volume}.{dist}"] = osso2d(arm, "s")
    return out


def caminho_poly(pontos):
    cu = bpy.data.curves.new("caminho", "CURVE")
    cu.dimensions = "2D"
    cu.use_path = True
    sp = cu.splines.new("POLY")
    sp.points.add(len(pontos) - 1)
    for p, (x, y) in zip(sp.points, pontos):
        p.co = (x, y, 0.0, 1.0)
    o = bpy.data.objects.new("caminho", cu)
    bpy.context.scene.collection.objects.link(o)
    return o


def caminho_bezier_meio_circulo(raio):
    """Meio círculo de (r,0) a (−r,0) por (0,r), Bézier de 3 pontos; resolução alta para a medida
    não depender da poligonal do Blender."""
    cu = bpy.data.curves.new("caminho", "CURVE")
    cu.dimensions = "2D"
    cu.use_path = True
    cu.resolution_u = 64
    sp = cu.splines.new("BEZIER")
    sp.bezier_points.add(2)
    k = 0.5522847498 * raio
    pts = [((raio, 0.0), (raio, -k), (raio, k)), ((0.0, raio), (k, raio), (-k, raio)), ((-raio, 0.0), (-raio, k), (-raio, -k))]
    for b, (co, h1, h2) in zip(sp.bezier_points, pts):
        b.co = (co[0], co[1], 0.0)
        b.handle_left = (h1[0], h1[1], 0.0)
        b.handle_right = (h2[0], h2[1], 0.0)
    o = bpy.data.objects.new("caminho", cu)
    bpy.context.scene.collection.objects.link(o)
    return o


def caso_follow_path():
    """Follow Path por % (posição fixa), orientado pela tangente (+X para a frente)."""
    out = {}
    for forma in ("L", "meio_circulo"):
        for f in (0.0, 0.25, 0.5, 0.75, 1.0):
            limpa()
            cam = caminho_poly([(0.0, 0.0), (4.0, 0.0), (4.0, 3.0)]) if forma == "L" else caminho_bezier_meio_circulo(2.0)
            dono = vazio("dono")
            c = dono.constraints.new("FOLLOW_PATH")
            c.target = cam
            c.use_fixed_location = True
            c.offset_factor = f
            c.use_curve_follow = True
            c.forward_axis = "FORWARD_X"
            c.up_axis = "UP_Z"
            out[f"{forma}.{f}"] = avalia(dono)
    return out


# ─── Onda 3: poderosas ────────────────────────────────────────────────────────────────────────────
# ⚠️ O Blender 5.2 NÃO tem mola (Spring) nas constraints, e a Physics do Spine só corre com interface:
# a lei da mola é a da casa (`ph2d-node-motion-spring::law::passo`) e o gate é o determinismo
# (`a_spring_rewinds_to_the_same_bit`), ver docs/BlenderConstraints/03_W3_poderosas.md §1.


def caso_transformation():
    """Transformation: um canal do alvo, numa faixa, mapeado para um canal do dono, noutra."""
    out = {}
    # posição x do alvo [0, 2] → rotação z do dono [0°, 90°]
    for extrapola in (False, True):
        for ax in (-1.0, 0.0, 0.5, 1.0, 2.0, 3.0):
            limpa()
            dono = vazio("dono", loc=(1.0, 0.5), rot_deg=10.0)
            alvo = vazio("alvo", loc=(ax, 0.0))
            c = dono.constraints.new("TRANSFORM")
            c.target = alvo
            c.map_from = "LOCATION"
            c.from_min_x = 0.0
            c.from_max_x = 2.0
            c.map_to = "ROTATION"
            c.map_to_z_from = "X"
            c.to_min_z_rot = 0.0
            c.to_max_z_rot = math.radians(90.0)
            c.use_motion_extrapolate = extrapola
            c.mix_mode_rot = "REPLACE"
            out[f"loc_rot.{'extrapola' if extrapola else 'apara'}.{ax}"] = avalia(dono)
    # o mesmo, somando à rotação autorada (10°)
    for ax in (0.5, 1.0):
        limpa()
        dono = vazio("dono", loc=(1.0, 0.5), rot_deg=10.0)
        alvo = vazio("alvo", loc=(ax, 0.0))
        c = dono.constraints.new("TRANSFORM")
        c.target = alvo
        c.map_from = "LOCATION"
        c.from_min_x = 0.0
        c.from_max_x = 2.0
        c.map_to = "ROTATION"
        c.map_to_z_from = "X"
        c.to_max_z_rot = math.radians(90.0)
        c.mix_mode_rot = "ADD"
        out[f"loc_rot.soma.{ax}"] = avalia(dono)
    # rotação z do alvo [-90°, 90°] → posição y do dono [0, 3]
    for ar in (-170.0, -90.0, 0.0, 45.0, 120.0):
        limpa()
        dono = vazio("dono", loc=(1.0, 0.5))
        alvo = vazio("alvo", rot_deg=ar)
        c = dono.constraints.new("TRANSFORM")
        c.target = alvo
        c.map_from = "ROTATION"
        c.from_min_z_rot = math.radians(-90.0)
        c.from_max_z_rot = math.radians(90.0)
        c.map_to = "LOCATION"
        c.map_to_y_from = "Z"
        c.to_min_y = 0.0
        c.to_max_y = 3.0
        c.mix_mode = "REPLACE"
        out[f"rot_loc.{ar}"] = avalia(dono)
    # escala x do alvo [1, 2] → escala y do dono [1, 3]
    for asx in (0.5, 1.5, 2.0):
        limpa()
        dono = vazio("dono")
        alvo = vazio("alvo", esc=(asx, 1.0))
        c = dono.constraints.new("TRANSFORM")
        c.target = alvo
        c.map_from = "SCALE"
        c.from_min_x_scale = 1.0
        c.from_max_x_scale = 2.0
        c.map_to = "SCALE"
        c.map_to_y_from = "X"
        c.to_min_y_scale = 1.0
        c.to_max_y_scale = 3.0
        c.mix_mode_scale = "REPLACE"
        out[f"esc_esc.{asx}"] = avalia(dono)
    return out


def acao_de_teste():
    """Uma acção de objecto: quadro 1 → x 0, rot 0°; quadro 21 → x 4, rot 90°, interpolação linear."""
    fonte = vazio("fonte")
    for quadro, x, r in ((1, 0.0, 0.0), (21, 4.0, 90.0)):
        fonte.location = (x, 0.0, 0.0)
        fonte.rotation_euler = (0.0, 0.0, math.radians(r))
        fonte.keyframe_insert("location", frame=quadro)
        fonte.keyframe_insert("rotation_euler", frame=quadro)
    ad = fonte.animation_data
    acao, slot = ad.action, ad.action_slot
    from bpy_extras import anim_utils

    for fc in anim_utils.action_get_channelbag_for_slot(acao, slot).fcurves:
        for k in fc.keyframe_points:
            k.interpolation = "LINEAR"
    fonte.animation_data_clear()
    bpy.data.objects.remove(fonte)
    return acao, slot


def caso_action():
    """Action: a rotação z do alvo [0°, 90°] escolhe o quadro [1, 21] da acção que o dono toca."""
    out = {}
    for ar in (-45.0, 0.0, 45.0, 90.0, 135.0):
        limpa()
        acao, slot = acao_de_teste()
        dono = vazio("dono")
        alvo = vazio("alvo", rot_deg=ar)
        c = dono.constraints.new("ACTION")
        c.target = alvo
        c.action = acao
        c.action_slot = slot
        c.transform_channel = "ROTATION_Z"
        # ⚠️ Medido 25/10: `min`/`max` de um canal de ROTAÇÃO vão em GRAUS (em radianos a faixa
        # vira [0°, 1,57°]); o espaço LOCAL de um objecto não lê a rotação dele (dá 0).
        c.target_space = "WORLD"
        c.min = 0.0
        c.max = 90.0
        c.frame_start = 1
        c.frame_end = 21
        out[str(ar)] = avalia(dono)
    return out


def caso_floor():
    """Floor: o dono não passa o plano do alvo (+Y é cima), com desvio; o tecto; o chão rodado."""
    out = {}
    for nome, (dx, dy), eixo, ar, usa_rot in (
        ("abaixo", (1.0, -1.0), "FLOOR_Y", 0.0, False),
        ("acima", (1.0, 2.0), "FLOOR_Y", 0.0, False),
        ("tecto", (1.0, 2.0), "FLOOR_NEGATIVE_Y", 0.0, False),
        ("rodado", (1.0, -1.0), "FLOOR_Y", 30.0, True),
        ("rodado_sem_rot", (1.0, -1.0), "FLOOR_Y", 30.0, False),
    ):
        limpa()
        dono = vazio("dono", loc=(dx, dy), rot_deg=15.0)
        alvo = vazio("alvo", loc=(0.0, 0.0), rot_deg=ar)
        c = dono.constraints.new("FLOOR")
        c.target = alvo
        c.floor_location = eixo
        c.offset = 0.5
        c.use_rotation = usa_rot
        out[nome] = avalia(dono)
    return out


def caso_pivot():
    """Pivot: a rotação autorada do dono acontece à volta do alvo, não da origem dele."""
    out = {}
    for nome, rot in (("noventa", 90.0), ("zero", 0.0), ("menos_45", -45.0)):
        limpa()
        dono = vazio("dono", loc=(2.0, 0.0), rot_deg=rot)
        alvo = vazio("alvo", loc=(1.0, 1.0))
        c = dono.constraints.new("PIVOT")
        c.target = alvo
        c.rotation_range = "ALWAYS_ACTIVE"
        out[nome] = avalia(dono)
    return out


def caso_armature():
    """Multi-parent: o dono segue dois ossos com pesos (constraint Armature), com e sem volume."""
    out = {}
    for volume in (False, True):
        for w1 in (0.5, 0.3):
            limpa()
            arm = rig([("p1", (0.0, 0.0), (1.0, 0.0), None), ("p2", (0.0, 0.0), (1.0, 0.0), None)])
            arm.pose.bones["p1"].rotation_mode = "XYZ"
            arm.pose.bones["p2"].rotation_mode = "XYZ"
            # Os ossos do Blender têm o comprimento em Y local: rodar em Y local é rodar no plano?
            # Não — o osso deitado em +X tem o Y local no +X mundial; a rotação no plano é a do X
            # local... medimos pela pose final, não pelo eixo: roda-se o osso pelo seu Z mundial.
            arm.pose.bones["p1"].matrix = (
                __import__("mathutils").Matrix.Rotation(math.radians(90.0), 4, "Z")
                @ arm.pose.bones["p1"].matrix
            )
            bpy.context.view_layer.update()
            dono = vazio("dono", loc=(2.0, 0.0))
            c = dono.constraints.new("ARMATURE")
            for nome, w in (("p1", w1), ("p2", 1.0 - w1)):
                t = c.targets.new()
                t.target = arm
                t.subtarget = nome
                t.weight = w
            c.use_deform_preserve_volume = volume
            out[f"{'volume' if volume else 'linear'}.{w1}"] = avalia(dono)
    return out


def caso_spline_ik():
    """Spline IK: 3 ossos de 1 sobre uma curva mais comprida; o modo de escala decide o comprimento."""
    out = {}
    for forma in ("recta", "meio_circulo", "curta"):
        for modo in ("NONE", "BONE_ORIGINAL", "FIT_CURVE"):
            limpa()
            arm = rig(
                [
                    ("s1", (0.0, 0.0), (1.0, 0.0), None),
                    ("s2", (1.0, 0.0), (2.0, 0.0), "s1"),
                    ("s3", (2.0, 0.0), (3.0, 0.0), "s2"),
                ]
            )
            if forma == "recta":
                cam = caminho_poly([(0.0, 1.0), (5.0, 1.0)])
            elif forma == "curta":
                cam = caminho_poly([(0.0, 1.0), (1.0, 1.0), (1.0, 2.0)])
            else:
                cam = caminho_bezier_meio_circulo(2.0)
            c = arm.pose.bones["s3"].constraints.new("SPLINE_IK")
            c.target = cam
            c.chain_count = 3
            c.use_even_divisions = False
            c.use_chain_offset = False
            c.y_scale_mode = modo
            c.xz_scale_mode = "NONE"
            c.use_curve_radius = False
            out[f"{forma}.{modo}"] = {n: osso2d(arm, n) for n in ("s1", "s2", "s3")}
    return out


def caso_vertice():
    """Prender a um ponto de uma forma: Copy Location com um grupo de um vértice só."""
    limpa()
    me = bpy.data.meshes.new("forma")
    me.from_pydata([(0.0, 0.0, 0.0), (1.0, 0.5, 0.0), (0.0, 1.0, 0.0)], [], [(0, 1, 2)])
    forma = bpy.data.objects.new("forma", me)
    bpy.context.scene.collection.objects.link(forma)
    forma.location = (3.0, -1.0, 0.0)
    forma.rotation_euler = (0.0, 0.0, math.radians(30.0))
    forma.scale = (2.0, 2.0, 1.0)
    g = forma.vertex_groups.new(name="v")
    g.add([1], 1.0, "REPLACE")
    dono = vazio("dono")
    c = dono.constraints.new("COPY_LOCATION")
    c.target = forma
    c.subtarget = "v"
    return {"vertice_local": [1.0, 0.5], "dono": avalia(dono)}


def main():
    saida = sys.argv[sys.argv.index("--") + 1]
    dados = {
        "_cabecalho": {
            "oraculo": "Blender " + bpy.app.version_string,
            "licenca": "GPL — corrido por script, fonte não lido (CLAUDE.md §0.9)",
            "script": "docs/BlenderConstraints/ferramentas/oraculo_blender.py",
            "convencao": "plano XY; rot_deg = ângulo do eixo +X mundial do objecto, graus",
        },
        "mistura_rotacao_copy_rotation": caso_mistura_rotacao(),
        "child_of": caso_child_of(),
        "apontar_3_tipos": caso_aponta(),
        "limit_rotation_z_-45_45": caso_limite_angulo(),
        "limit_distance_inside_2": caso_distancia(),
        "ordem_da_pilha": caso_ordem(),
        "ik_com_copy_rotation_abaixo": caso_ik_por_ultimo(),
        "ik_dois_ossos": caso_ik_dois_ossos(),
        "ik_estica": caso_ik_estica(),
        "ik_cadeia_3": caso_ik_cadeia(),
        "stretch_to": caso_stretch_to(),
        "follow_path": caso_follow_path(),
        "transformation": caso_transformation(),
        "action": caso_action(),
        "floor": caso_floor(),
        "pivot": caso_pivot(),
        "armature": caso_armature(),
        "spline_ik": caso_spline_ik(),
        "vertice": caso_vertice(),
    }
    with open(saida, "w", encoding="utf-8") as f:
        json.dump(dados, f, indent=1, ensure_ascii=False)
    print("ORACULO_OK", saida)


main()
