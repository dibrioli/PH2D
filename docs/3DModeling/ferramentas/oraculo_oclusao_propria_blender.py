# ⭐ ORÁCULO DA OCLUSÃO PRÓPRIA — o Blender (Cycles) CORRIDO sobre peças NOSSAS, cada uma SOZINHA.
#
# Pergunta que ele responde (e só esta): sob um céu UNIFORME, sem chão, uma peça CÔNCAVA (peças
# fundidas numa união: o vinco onde uma encosta na outra) — que fracção do céu, ponderada pelo
# cosseno, cada ponto dela vê? Difusa branca, luz directa só: o Combined é a visibilidade. Os passes
# Position e Normal dão o ponto e a normal — o gate `ph2d-contacto::tests::a_oclusao_propria_e_a_do_cycles`
# avalia a nossa lei NESSES pontos.
#
# Corra (o arnês põe o Cycles na fatia da linha):
#   cd <worktree> && bash scripts/ph2d-run.sh blender -b -X --python \
#       docs/3DModeling/ferramentas/oraculo_oclusao_propria_blender.py -- \
#       crates/ph2d-contacto/fixtures/oraculo_oclusao_propria.csv
#
# Convenção: (x, y, z)_nosso = (x, z, -y)_blender.
import os
import sys

import bpy
from mathutils import Vector

LADO = 192
AMOSTRAS = 2048
# As peças (cada uma num render só dela), nosso referencial:
#  L     — duas caixas: (0, 0.1, 0) meia (0.3, 0.1, 0.2) e (-0.2, 0.35, 0) meia (0.1, 0.15, 0.2)
#  bola  — caixa (0, 0.15, 0) meia (0.3, 0.15, 0.3) e esfera (0.05, 0.3, 0.05) raio 0.18
#  toro  — centro (0, 0.1, 0), eixo y, raios 0.3 e 0.1
PECAS = ["L", "bola", "toro"]
DE = (1.0, 1.2, -0.9)
MEIA = 0.5

argv = sys.argv[sys.argv.index("--") + 1 :]
saida = argv[0]


def b(p):
    return (p[0], -p[2], p[1])


def nosso(p):
    return (p[0], p[2], -p[1])


def material():
    mat = bpy.data.materials.new("branca")
    mat.use_nodes = True
    nt = mat.node_tree
    nt.nodes.clear()
    dif = nt.nodes.new("ShaderNodeBsdfDiffuse")
    dif.inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    nt.links.new(dif.outputs["BSDF"], out.inputs["Surface"])
    return mat


def caixa(c, h):
    bpy.ops.mesh.primitive_cube_add(size=2.0, location=b(c))
    o = bpy.context.active_object
    o.scale = (h[0], h[2], h[1])
    bpy.ops.object.transform_apply(scale=True)
    return o


def une(a, outro):
    m = a.modifiers.new("uniao", "BOOLEAN")
    m.operation = "UNION"
    m.solver = "EXACT"
    m.object = outro
    bpy.context.view_layer.objects.active = a
    bpy.ops.object.modifier_apply(modifier=m.name)
    bpy.data.objects.remove(outro, do_unlink=True)
    return a


def cena(nome):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    s = bpy.context.scene
    s.render.engine = "CYCLES"
    s.cycles.device = "CPU"
    s.cycles.samples = AMOSTRAS
    s.cycles.use_denoising = False
    s.cycles.use_adaptive_sampling = False
    s.cycles.max_bounces = 0
    s.cycles.filter_width = 0.01
    s.cycles.seed = 11
    s.render.resolution_x = LADO
    s.render.resolution_y = LADO
    s.render.resolution_percentage = 100
    s.render.film_transparent = True
    vl = s.view_layers[0]
    vl.use_pass_position = True
    vl.use_pass_normal = True
    s.render.image_settings.media_type = "MULTI_LAYER_IMAGE"
    s.render.image_settings.file_format = "OPEN_EXR_MULTILAYER"
    s.render.image_settings.color_depth = "32"
    w = bpy.data.worlds.new("uniforme")
    w.use_nodes = True
    w.node_tree.nodes["Background"].inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
    w.node_tree.nodes["Background"].inputs["Strength"].default_value = 1.0
    s.world = w
    if nome == "L":
        o = une(caixa((0.0, 0.1, 0.0), (0.3, 0.1, 0.2)), caixa((-0.2, 0.35, 0.0), (0.1, 0.15, 0.2)))
        alvo = (0.0, 0.2, 0.0)
    elif nome == "bola":
        a = caixa((0.0, 0.15, 0.0), (0.3, 0.15, 0.3))
        bpy.ops.mesh.primitive_uv_sphere_add(segments=256, ring_count=128, radius=0.18,
                                             location=b((0.05, 0.3, 0.05)))
        bpy.ops.object.shade_smooth()
        o = une(a, bpy.context.active_object)
        alvo = (0.0, 0.2, 0.0)
    else:
        bpy.ops.mesh.primitive_torus_add(major_radius=0.3, minor_radius=0.1, major_segments=256,
                                         minor_segments=96, location=b((0.0, 0.1, 0.0)))
        bpy.ops.object.shade_smooth()
        o = bpy.context.active_object
        alvo = (0.0, 0.1, 0.0)
    o.data.materials.clear()
    o.data.materials.append(material())
    cam_d = bpy.data.cameras.new("cam")
    cam_d.type = "ORTHO"
    cam_d.ortho_scale = 2.0 * MEIA
    cam = bpy.data.objects.new("cam", cam_d)
    d = Vector(DE).normalized()
    cam.location = Vector(b(alvo)) + Vector(b(tuple(d * 6.0)))
    cam.rotation_euler = (-Vector(b(tuple(d)))).to_track_quat("-Z", "Y").to_euler()
    s.collection.objects.link(cam)
    s.camera = cam
    return s


import OpenImageIO as oiio

linhas = []
for k, nome in enumerate(PECAS):
    s = cena(nome)
    exr = os.path.join(os.path.dirname(os.path.abspath(saida)), f"_oraculo_propria_{nome}.exr")
    s.render.filepath = exr
    bpy.ops.render.render(write_still=True)
    inp = oiio.ImageInput.open(exr)
    passes = {}
    sub = 0
    while inp.seek_subimage(sub, 0):
        spec = inp.spec()
        px = inp.read_image(sub, 0, 0, spec.nchannels, oiio.FLOAT)
        for i, n in enumerate(spec.channelnames):
            passes[n] = (px, i)
        sub += 1
    inp.close()
    os.remove(exr)

    def canal(sufixo):
        c = [n for n in passes if n.endswith(sufixo)]
        if len(c) != 1:
            raise SystemExit(f"canal {sufixo}: {c}")
        return passes[c[0]]

    def le(c, i, j):
        (px, q) = c
        return float(px[j][i][q])

    comb, alfa = canal("Combined.R"), canal("Combined.A")
    pos = [canal(f"Position.{e}") for e in "XYZ"]
    nrm = [canal(f"Normal.{e}") for e in "XYZ"]
    for j in range(LADO):
        for i in range(LADO):
            if le(alfa, i, j) < 0.999:
                continue
            # Um pixel em dois (xadrez), e um em quatro dos que veem o céu todo.
            if (i + j) % 2:
                continue
            v = le(comb, i, j)
            if v > 0.995 and (i % 2 or j % 2):
                continue
            p = nosso(tuple(le(c, i, j) for c in pos))
            n = nosso(tuple(le(c, i, j) for c in nrm))
            linhas.append(f"{nome},{i},{j},{p[0]:.5f},{p[1]:.5f},{p[2]:.5f},{n[0]:.5f},{n[1]:.5f},{n[2]:.5f},{v:.5f}")

with open(saida, "w") as f:
    f.write(f"# ORÁCULO: Blender {bpy.app.version_string} Cycles CPU, {AMOSTRAS} amostras, sem denoise, "
            f"luz directa só (max_bounces 0), filtro 0,01 px, difusa branca, céu uniforme 1.\n")
    f.write("# Gerado por docs/3DModeling/ferramentas/oraculo_oclusao_propria_blender.py — NÃO editar à mão.\n")
    f.write(f"# CENA lado={LADO} pecas={PECAS} de={DE} meia={MEIA} (as peças: ver o cabeçalho do script)\n")
    f.write("# Cada peça sozinha; p e n no MUNDO nosso; vis = a fracção do céu vista.\n")
    f.write("peca,i,j,x,y,z,nx,ny,nz,vis\n")
    f.write("\n".join(linhas) + "\n")
print(f"ORACULO: {len(linhas)} linhas em {saida}")
