# ⭐ ORÁCULO DA OCLUSÃO DE UMA MALHA NOSSA — o Cycles CORRIDO sobre a malha que o Render extraiu.
#
# Pergunta (e só esta): sob um céu UNIFORME, sem chão, a malha `.obj` (escrita pela sonda
# `malha_render_contacto::tests::sonda_oclusao_da_malha`) sozinha — que fracção do céu, ponderada pelo
# cosseno, cada ponto vê? Difusa branca, luz directa só; Position/Normal dão o ponto e a normal.
#
# Corra:
#   bash scripts/ph2d-run.sh blender -b -X --python docs/3DModeling/ferramentas/oraculo_oclusao_malha_blender.py -- \
#       <malha.obj> <saida.csv>
# A malha vem no NOSSO referencial (y para cima); o importador do Blender converte (eixo cima = Y).
import os
import sys

import bpy
from mathutils import Vector

LADO = 192
AMOSTRAS = 1024
DE = (1.0, 1.2, -0.9)

argv = sys.argv[sys.argv.index("--") + 1 :]
obj, saida = argv[0], argv[1]


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
bpy.ops.wm.obj_import(filepath=obj, forward_axis="NEGATIVE_Z", up_axis="Y")
o = bpy.context.selected_objects[0]
mat = bpy.data.materials.new("branca")
mat.use_nodes = True
nt = mat.node_tree
nt.nodes.clear()
dif = nt.nodes.new("ShaderNodeBsdfDiffuse")
dif.inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
out = nt.nodes.new("ShaderNodeOutputMaterial")
nt.links.new(dif.outputs["BSDF"], out.inputs["Surface"])
o.data.materials.clear()
o.data.materials.append(mat)
cantos = [o.matrix_world @ Vector(c) for c in o.bound_box]
lo = Vector([min(c[k] for c in cantos) for k in range(3)])
hi = Vector([max(c[k] for c in cantos) for k in range(3)])
alvo_b = (lo + hi) / 2
meia = 0.6 * (hi - lo).length
cam_d = bpy.data.cameras.new("cam")
cam_d.type = "ORTHO"
cam_d.ortho_scale = 2.0 * meia
cam = bpy.data.objects.new("cam", cam_d)
d = Vector(b(DE)).normalized()
cam.location = alvo_b + d * (6.0 + 4.0 * meia)
cam.rotation_euler = (-d).to_track_quat("-Z", "Y").to_euler()
cam_d.clip_end = 100.0
s.collection.objects.link(cam)
s.camera = cam
exr = os.path.join(os.path.dirname(os.path.abspath(saida)), "_oraculo_malha.exr")
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
    c = [n for n in passes if n.endswith(sufixo)]
    return passes[c[0]]


def le(c, i, j):
    (px, q) = c
    return float(px[j][i][q])


comb, alfa = canal("Combined.R"), canal("Combined.A")
pos = [canal(f"Position.{e}") for e in "XYZ"]
nrm = [canal(f"Normal.{e}") for e in "XYZ"]
linhas = []
for j in range(LADO):
    for i in range(LADO):
        if le(alfa, i, j) < 0.999 or (i + j) % 2:
            continue
        p = nosso(tuple(le(c, i, j) for c in pos))
        n = nosso(tuple(le(c, i, j) for c in nrm))
        linhas.append(f"{i},{j},{p[0]:.5f},{p[1]:.5f},{p[2]:.5f},{n[0]:.5f},{n[1]:.5f},{n[2]:.5f},{le(comb, i, j):.5f}")
with open(saida, "w") as f:
    f.write(f"# ORÁCULO: Blender {bpy.app.version_string} Cycles, {AMOSTRAS} amostras, malha {os.path.basename(obj)}\n")
    f.write("i,j,x,y,z,nx,ny,nz,vis\n")
    f.write("\n".join(linhas) + "\n")
print(f"ORACULO: {len(linhas)} linhas em {saida}")
