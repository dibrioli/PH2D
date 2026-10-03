# ⭐ ORÁCULO DO CONTACTO ENTRE PEÇAS — o Blender (Cycles) CORRIDO sobre uma cena NOSSA.
#
# Pergunta que ele responde (e só esta): sob um céu UNIFORME, sem chão, peças CONVEXAS que se tocam
# (uma caixa; uma esfera pousada em cima dela; outra encostada a uma face; uma terceira a 2 cm de
# outra face) — que fracção do céu, ponderada pelo cosseno, cada ponto vê? Uma peça convexa sozinha vê
# o céu todo, logo o que falta é SÓ o que as vizinhas tapam. Difusa branca, luz directa só
# (max_bounces 0): o passe Combined é a própria visibilidade. Os passes Position e Normal dão o ponto
# e a normal de cada pixel, no mundo — o gate `ph2d-contacto::tests::a_oclusao_entre_pecas_e_a_do_cycles` avalia a nossa lei NESSES pontos.
#
# Corra (o arnês põe o Cycles na fatia da linha):
#   cd <worktree> && bash scripts/ph2d-run.sh blender -b -X --python \
#       docs/3DModeling/ferramentas/oraculo_contacto_blender.py -- \
#       crates/ph2d-contacto/fixtures/oraculo_contacto.csv
#
# Convenção: o Blender é Z-para-cima; o produto é Y-para-cima. (x, y, z)_nosso = (x, z, -y)_blender.
import math
import os
import sys

import bpy
from mathutils import Vector

LADO = 256
AMOSTRAS = 2048
CAIXA = ((0.0, 0.2, 0.0), 0.4)  # (centro nosso, aresta)
# (nome, centro nosso, raio): pousada em cima · encostada à face +x · a 2 cm da face −z.
ESFERAS = [
    ("em_cima", (0.0, 0.6, 0.0), 0.2),
    ("ao_lado", (0.45, 0.25, 0.0), 0.25),
    ("perto", (0.0, 0.2, -0.37), 0.15),
]
# A câmara ortográfica: olha PARA `ALVO` vinda da direcção `DE` (nosso), meia-aresta `MEIA`.
DE = (1.0, 0.8, -1.3)
ALVO = (0.15, 0.3, -0.1)
MEIA = 0.75

argv = sys.argv[sys.argv.index("--") + 1 :]
saida = argv[0]


def b(p):
    return (p[0], -p[2], p[1])


def nosso(p):
    return (p[0], p[2], -p[1])


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
vl.use_pass_object_index = True
s.render.image_settings.media_type = "MULTI_LAYER_IMAGE"
s.render.image_settings.file_format = "OPEN_EXR_MULTILAYER"
s.render.image_settings.color_depth = "32"
w = bpy.data.worlds.new("uniforme")
w.use_nodes = True
w.node_tree.nodes["Background"].inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
w.node_tree.nodes["Background"].inputs["Strength"].default_value = 1.0
s.world = w

mat = bpy.data.materials.new("branca")
mat.use_nodes = True
nt = mat.node_tree
nt.nodes.clear()
dif = nt.nodes.new("ShaderNodeBsdfDiffuse")
dif.inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
out = nt.nodes.new("ShaderNodeOutputMaterial")
nt.links.new(dif.outputs["BSDF"], out.inputs["Surface"])

(c, a) = CAIXA
bpy.ops.mesh.primitive_cube_add(size=a, location=b(c))
bpy.context.active_object.data.materials.append(mat)
bpy.context.active_object.pass_index = 1
for k, (_, c, r) in enumerate(ESFERAS):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=256, ring_count=128, radius=r, location=b(c))
    bpy.ops.object.shade_smooth()
    bpy.context.active_object.data.materials.append(mat)
    bpy.context.active_object.pass_index = 2 + k

cam_d = bpy.data.cameras.new("cam")
cam_d.type = "ORTHO"
cam_d.ortho_scale = 2.0 * MEIA
cam = bpy.data.objects.new("cam", cam_d)
d = Vector(DE).normalized()
cam.location = Vector(b(ALVO)) + Vector(b(tuple(d * 6.0)))
cam.rotation_euler = (-Vector(b(tuple(d)))).to_track_quat("-Z", "Y").to_euler()
s.collection.objects.link(cam)
s.camera = cam
exr = os.path.join(os.path.dirname(os.path.abspath(saida)), "_oraculo_contacto.exr")
s.render.filepath = exr
bpy.ops.render.render(write_still=True)

import OpenImageIO as oiio

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
    k = [n for n in passes if n.endswith(sufixo)]
    if len(k) != 1:
        raise SystemExit(f"canal {sufixo}: {k} de {sorted(passes)}")
    return passes[k[0]]


def le(c, i, j):
    (px, k) = c
    return float(px[j][i][k])


comb = canal("Combined.R")
alfa = canal("Combined.A")
pos = [canal(f"Position.{e}") for e in "XYZ"]
nrm = [canal(f"Normal.{e}") for e in "XYZ"]
idx = canal("Object Index.X")
linhas = []
for j in range(LADO):
    for i in range(LADO):
        if le(alfa, i, j) < 0.999:
            continue
        v = le(comb, i, j)
        # Os pixels que veem o céu todo pesam pouco no gate: um em cada quatro.
        if v > 0.995 and (i % 2 or j % 2):
            continue
        p = nosso(tuple(le(c, i, j) for c in pos))
        n = nosso(tuple(le(c, i, j) for c in nrm))
        o = int(round(le(idx, i, j)))
        linhas.append(f"{i},{j},{o},{p[0]:.5f},{p[1]:.5f},{p[2]:.5f},{n[0]:.5f},{n[1]:.5f},{n[2]:.5f},{v:.5f}")

with open(saida, "w") as f:
    f.write(f"# ORÁCULO: Blender {bpy.app.version_string} Cycles CPU, {AMOSTRAS} amostras, sem denoise, "
            f"luz directa só (max_bounces 0), filtro 0,01 px, difusa branca, céu uniforme 1.\n")
    f.write("# Gerado por docs/3DModeling/ferramentas/oraculo_contacto_blender.py — NÃO editar à mão.\n")
    f.write(f"# CENA lado={LADO} caixa={CAIXA} esferas={ESFERAS} de={DE} alvo={ALVO} meia={MEIA}\n")
    f.write("# obj: 1 = caixa, 2.. = as esferas pela ordem; p e n no MUNDO nosso; vis = a fracção do céu vista.\n")
    f.write("i,j,obj,x,y,z,nx,ny,nz,vis\n")
    f.write("\n".join(linhas) + "\n")
print(f"ORACULO: {len(linhas)} linhas em {saida}")
