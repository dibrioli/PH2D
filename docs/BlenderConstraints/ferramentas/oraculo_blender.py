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
    }
    with open(saida, "w", encoding="utf-8") as f:
        json.dump(dados, f, indent=1, ensure_ascii=False)
    print("ORACULO_OK", saida)


main()
