# ⭐ ORÁCULO DO CÉU QUE O CHÃO VÊ — o Blender (Cycles) CORRIDO sobre uma cena NOSSA.
#
# Pergunta que ele responde (e só esta): sob um céu UNIFORME (sem sol, sem caixa), um chão que só
# recebe (shadow catcher) com uma esfera e uma caixa POUSADAS — quanto escurece cada ponto do chão?
# (`1 − o passe Shadow Catcher`, luz directa só: é a fracção do céu, ponderada pelo cosseno, que a peça
# tapa.) O gate `tests_chao::o_ceu_do_chao_e_o_do_cycles` lê o CSV e compara o desenhista de jogo
# PASSO A PASSO (as mesmas linhas de pixels do mesmo enquadramento).
#
# Corra (o arnês põe o Cycles na fatia da linha):
#   cd <worktree> && bash scripts/ph2d-run.sh blender -b -X --python \
#       docs/3DModeling/ferramentas/oraculo_ceu_do_chao_blender.py -- \
#       crates/ph2d-mesh-forward/fixtures/oraculo_ceu_do_chao.csv
#
# Convenção: o Blender é Z-para-cima; o produto é Y-para-cima. (x, y, z)_nosso = (x, z, -y)_blender.
# ⚠️ As constantes da CENA repetem-se em `tests_chao.rs` — o cabeçalho do CSV leva-as e o gate recusa
# um CSV de outra cena.
import math
import os
import sys

import bpy

LADO = 384  # pixels do enquadramento quadrado
# (centro x, centro z, meia-aresta) — nosso: a cena inteira, vista de cima.
QUADRO = (0.0, 0.0, 1.6)
ESFERA = ((-0.8, 0.3, 0.0), 0.3)  # (centro nosso, raio) — pousada: toca o chão em (−0,8, 0, 0)
CAIXA = ((0.8, 0.2, 0.0), 0.4)  # (centro nosso, aresta) — pousada
AMOSTRAS = 2048

argv = sys.argv[sys.argv.index("--") + 1 :]
saida = argv[0]


def nosso_para_blender(p):
    return (p[0], -p[2], p[1])


def cena():
    (cx, cz, meia) = QUADRO
    bpy.ops.wm.read_factory_settings(use_empty=True)
    s = bpy.context.scene
    s.render.engine = "CYCLES"
    s.cycles.device = "CPU"
    s.cycles.samples = AMOSTRAS
    s.cycles.use_denoising = False
    s.cycles.use_adaptive_sampling = False
    s.cycles.max_bounces = 0  # luz DIRECTA do céu: a pergunta é quanto céu a peça tapa
    s.cycles.filter_width = 0.01  # o CENTRO do pixel
    s.cycles.seed = 11
    s.render.resolution_x = LADO
    s.render.resolution_y = LADO
    s.render.resolution_percentage = 100
    s.render.film_transparent = True
    s.view_layers[0].cycles.use_pass_shadow_catcher = True
    s.render.image_settings.media_type = "MULTI_LAYER_IMAGE"  # Blender 5: o multicamada é um tipo
    s.render.image_settings.file_format = "OPEN_EXR_MULTILAYER"
    s.render.image_settings.color_depth = "32"
    w = bpy.data.worlds.new("uniforme")
    w.use_nodes = True
    w.node_tree.nodes["Background"].inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
    w.node_tree.nodes["Background"].inputs["Strength"].default_value = 1.0
    s.world = w
    bpy.ops.mesh.primitive_plane_add(size=40.0, location=(0.0, 0.0, 0.0))
    bpy.context.active_object.is_shadow_catcher = True
    (c, r) = ESFERA
    bpy.ops.mesh.primitive_uv_sphere_add(segments=256, ring_count=128, radius=r,
                                         location=nosso_para_blender(c))
    bpy.ops.object.shade_smooth()
    bpy.context.active_object.visible_camera = False
    (c, a) = CAIXA
    bpy.ops.mesh.primitive_cube_add(size=a, location=nosso_para_blender(c))
    bpy.context.active_object.visible_camera = False
    cam_d = bpy.data.cameras.new("cam")
    cam_d.type = "ORTHO"
    cam_d.ortho_scale = 2.0 * meia
    cam = bpy.data.objects.new("cam", cam_d)
    cam.location = (cx, -cz, 10.0)  # olha −Z; o cima da imagem é +Y_b = −z nosso
    s.collection.objects.link(cam)
    s.camera = cam
    return s


s = cena()
exr = os.path.join(os.path.dirname(os.path.abspath(saida)), "_oraculo_ceu_do_chao.exr")
s.render.filepath = exr
bpy.ops.render.render(write_still=True)
import OpenImageIO as oiio

# ⚠️ O Blender 5 escreve um EXR MULTI-PARTE: cada passe na sua sub-imagem.
inp = oiio.ImageInput.open(exr)
sub, k, px = 0, [], None
while inp.seek_subimage(sub, 0):
    spec = inp.spec()
    nomes = list(spec.channelnames)
    k = [i for i, n in enumerate(nomes) if "Shadow Catcher" in n and n.endswith(".R")]
    if k:
        px = inp.read_image(sub, 0, 0, spec.nchannels, oiio.FLOAT)
        break
    sub += 1
inp.close()
if px is None:
    raise SystemExit("sem o passe Shadow Catcher")
os.remove(exr)

(cx, cz, meia) = QUADRO


def mundo(i, j):
    return (cx + ((i + 0.5) / LADO * 2.0 - 1.0) * meia, cz + ((j + 0.5) / LADO * 2.0 - 1.0) * meia)


def pixel(x, z):
    return (int((x - (cx - meia)) / (2.0 * meia) * LADO), int((z - (cz - meia)) / (2.0 * meia) * LADO))


linhas = []


def guarda(corte, i, j):
    x, z = mundo(i, j)
    # (i, j) com a linha j de CIMA; o OIIO lê a linha 0 em cima.
    linhas.append(f"{corte},{i},{j},{x:.6f},{z:.6f},{1.0 - float(px[j][i][k[0]]):.6f}")


# A linha do meio (atravessa as duas peças), a coluna de cada uma e a diagonal da caixa (a quina).
j0 = LADO // 2
for i in range(LADO):
    guarda("linha", i, j0)
for nome, (c, _) in (("esfera", ESFERA), ("caixa", CAIXA)):
    i0, _ = pixel(c[0], 0.0)
    for j in range(LADO):
        guarda(f"coluna_{nome}", i0, j)
i0, j0 = pixel(CAIXA[0][0], 0.0)
for t in range(-LADO // 4, LADO // 4):
    if 0 <= i0 + t < LADO and 0 <= j0 + t < LADO:
        guarda("diagonal_caixa", i0 + t, j0 + t)

with open(saida, "w") as f:
    f.write(f"# ORÁCULO: Blender {bpy.app.version_string} Cycles CPU, {AMOSTRAS} amostras, sem denoise, "
            f"luz directa só (max_bounces 0), filtro 0,01 px.\n")
    f.write("# Gerado por docs/3DModeling/ferramentas/oraculo_ceu_do_chao_blender.py — NÃO editar à mão.\n")
    f.write(f"# CENA lado={LADO} quadro={QUADRO} esfera={ESFERA} caixa={CAIXA} céu=uniforme branco 1\n")
    f.write("# escuro = 1 - passe Shadow Catcher (o chão a y=0, objetos invisíveis à câmara).\n")
    f.write("corte,i,j,x,z,escuro\n")
    f.write("\n".join(linhas) + "\n")
print(f"ORACULO: {len(linhas)} linhas em {saida}")
