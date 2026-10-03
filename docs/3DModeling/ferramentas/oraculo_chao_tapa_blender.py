# ⭐ ORÁCULO DO CHÃO QUE TAPA AS PEÇAS — o Blender (Cycles) CORRIDO sobre uma cena NOSSA.
#
# Pergunta que ele responde (e só esta): sob um céu UNIFORME 1, com peças pousadas num chão branco
# difuso infinito (`y = 0`), que luz do céu chega a cada ponto das peças — a directa do céu que nada
# tapa MAIS a que o chão devolve (o chão iluminado pelo céu, escurecido onde as peças o tapam)?
# As peças são brancas para a câmara e PRETAS para todo o resto (Light Path «Is Camera Ray»): tapam
# o céu e o chão, mas não devolvem luz — a nossa lei não tem inter-reflexo entre peças. Um ricochete
# difuso (câmara → peça → chão → céu): o passe Combined é `E/π` com a mesma unidade do céu.
#
# Três corridas da MESMA câmara:
#   `sem`   — o chão escondido: só o céu e as vizinhas (o oráculo do contacto, nesta cena);
#   `com`   — o chão branco (albedo 1): debaixo das peças ele é o próprio escurecimento do céu do chão;
#   `preto` — o chão de albedo 0: a FORMA FECHADA de controlo — um ponto que nada mais tapa vê
#             exactamente `(1 + n.y)/2` do céu (o semi-espaço de baixo apagado).
# E o REFLEXO: as peças, para a câmara, passam a um metal branco (Glossy GGX) — `espelho` (rugosidade
# 0) e `aspero` (rugosidade 0,5, α = 0,25), cada um sem e com o chão branco: o reflexo do chão escuro
# do contacto. E o VERNIZ: Principled metal branco de rugosidade 0,5 com verniz (Coat) nítido de
# índice 1,5 — duas perguntas de reflexo com rugosidades diferentes no MESMO pixel.
#
# Corra (o arnês põe o Cycles na fatia da linha):
#   cd <worktree> && bash scripts/ph2d-run.sh blender -b -X --python \
#       docs/3DModeling/ferramentas/oraculo_chao_tapa_blender.py -- \
#       crates/ph2d-mesh-forward/fixtures/oraculo_chao_tapa.csv
#
# Convenção: o Blender é Z-para-cima; o produto é Y-para-cima. (x, y, z)_nosso = (x, z, -y)_blender.
import os
import sys

import bpy
from mathutils import Vector

LADO = 256
AMOSTRAS = 4096
AMOSTRAS_PRETO = 1024
# (nome, centro nosso, raio): pousada · a flutuar 6 cm acima do chão.
ESFERAS = [
    ("pousada", (-0.45, 0.3, 0.0), 0.3),
    ("a_flutuar", (0.05, 0.21, 0.5), 0.15),
]
CAIXA = ((0.4, 0.2, -0.05), 0.4)  # (centro nosso, aresta) — pousada
# A câmara ortográfica, BAIXA (`~7°` acima do chão) para ver a parte de baixo das peças.
DE = (0.6, 0.15, 1.0)
ALVO = (0.0, 0.2, 0.15)
MEIA = 0.8

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
s.cycles.use_denoising = False
s.cycles.use_adaptive_sampling = False
s.cycles.max_bounces = 1
s.cycles.diffuse_bounces = 1
s.cycles.glossy_bounces = 0
s.cycles.transmission_bounces = 0
s.cycles.transparent_max_bounces = 0
s.cycles.caustics_reflective = False
s.cycles.caustics_refractive = False
s.cycles.sample_clamp_indirect = 0.0
s.cycles.filter_width = 0.01
s.cycles.seed = 13
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


def difusa(nome, cor):
    m = bpy.data.materials.new(nome)
    m.use_nodes = True
    nt = m.node_tree
    nt.nodes.clear()
    d = nt.nodes.new("ShaderNodeBsdfDiffuse")
    d.inputs["Color"].default_value = (cor, cor, cor, 1.0)
    o = nt.nodes.new("ShaderNodeOutputMaterial")
    nt.links.new(d.outputs["BSDF"], o.inputs["Surface"])
    return m, d


# A peça: branca para a câmara, preta para os ricochetes (tapa, não devolve).
peca = bpy.data.materials.new("peca")
peca.use_nodes = True
nt = peca.node_tree
nt.nodes.clear()
branca = nt.nodes.new("ShaderNodeBsdfDiffuse")
branca.inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
metal = nt.nodes.new("ShaderNodeBsdfGlossy")
metal.distribution = "GGX"
metal.inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
preta = nt.nodes.new("ShaderNodeBsdfDiffuse")
preta.inputs["Color"].default_value = (0.0, 0.0, 0.0, 1.0)
lp = nt.nodes.new("ShaderNodeLightPath")
mix = nt.nodes.new("ShaderNodeMixShader")
nt.links.new(lp.outputs["Is Camera Ray"], mix.inputs["Fac"])
nt.links.new(preta.outputs["BSDF"], mix.inputs[1])
nt.links.new(branca.outputs["BSDF"], mix.inputs[2])
out = nt.nodes.new("ShaderNodeOutputMaterial")
nt.links.new(mix.outputs["Shader"], out.inputs["Surface"])

(c, a) = CAIXA
bpy.ops.mesh.primitive_cube_add(size=a, location=b(c))
bpy.context.active_object.data.materials.append(peca)
bpy.context.active_object.pass_index = 1
for k, (_, c, r) in enumerate(ESFERAS):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=256, ring_count=128, radius=r, location=b(c))
    bpy.ops.object.shade_smooth()
    bpy.context.active_object.data.materials.append(peca)
    bpy.context.active_object.pass_index = 2 + k

chao_mat, chao_dif = difusa("chao", 1.0)
bpy.ops.mesh.primitive_plane_add(size=200.0, location=(0.0, 0.0, 0.0))
chao = bpy.context.active_object
chao.data.materials.append(chao_mat)
chao.visible_camera = False

cam_d = bpy.data.cameras.new("cam")
cam_d.type = "ORTHO"
cam_d.ortho_scale = 2.0 * MEIA
cam = bpy.data.objects.new("cam", cam_d)
d = Vector(DE).normalized()
cam.location = Vector(b(ALVO)) + Vector(b(tuple(d * 6.0)))
cam.rotation_euler = (-Vector(b(tuple(d)))).to_track_quat("-Z", "Y").to_euler()
s.collection.objects.link(cam)
s.camera = cam

import OpenImageIO as oiio


def corre(nome, amostras):
    s.cycles.samples = amostras
    exr = os.path.join(os.path.dirname(os.path.abspath(saida)), f"_oraculo_chao_tapa_{nome}.exr")
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
    return passes


def canal(passes, sufixo):
    k = [n for n in passes if n.endswith(sufixo)]
    if len(k) != 1:
        raise SystemExit(f"canal {sufixo}: {k} de {sorted(passes)}")
    return passes[k[0]]


def le(c, i, j):
    (px, k) = c
    return float(px[j][i][k])


chao.hide_render = True
p_sem = corre("sem", AMOSTRAS)
chao.hide_render = False
p_com = corre("com", AMOSTRAS)
chao_dif.inputs["Color"].default_value = (0.0, 0.0, 0.0, 1.0)
p_preto = corre("preto", AMOSTRAS_PRETO)
chao_dif.inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
nt.links.new(metal.outputs["BSDF"], mix.inputs[2])
s.cycles.glossy_bounces = 1
brilho = {}
for nome, rug in (("espelho", 0.0), ("aspero", 0.5)):
    metal.inputs["Roughness"].default_value = rug
    chao.hide_render = True
    brilho[nome + "_sem"] = canal(corre(nome + "_sem", AMOSTRAS_PRETO), "Combined.R")
    chao.hide_render = False
    brilho[nome + "_com"] = canal(corre(nome + "_com", AMOSTRAS_PRETO), "Combined.R")
verniz = nt.nodes.new("ShaderNodeBsdfPrincipled")
verniz.inputs["Base Color"].default_value = (1.0, 1.0, 1.0, 1.0)
verniz.inputs["Metallic"].default_value = 1.0
verniz.inputs["Roughness"].default_value = 0.5
verniz.inputs["Coat Weight"].default_value = 1.0
verniz.inputs["Coat Roughness"].default_value = 0.0
verniz.inputs["Coat IOR"].default_value = 1.5
nt.links.new(verniz.outputs["BSDF"], mix.inputs[2])
chao.hide_render = True
brilho["verniz_sem"] = canal(corre("verniz_sem", AMOSTRAS_PRETO), "Combined.R")
chao.hide_render = False
brilho["verniz_com"] = canal(corre("verniz_com", AMOSTRAS_PRETO), "Combined.R")
BRILHO = ("espelho_sem", "espelho_com", "aspero_sem", "aspero_com", "verniz_sem", "verniz_com")

sem = canal(p_sem, "Combined.R")
com = canal(p_com, "Combined.R")
preto = canal(p_preto, "Combined.R")
alfa = canal(p_com, "Combined.A")
pos = [canal(p_com, f"Position.{e}") for e in "XYZ"]
nrm = [canal(p_com, f"Normal.{e}") for e in "XYZ"]
idx = canal(p_com, "Object Index.X")
linhas = []
controlo = []
for j in range(LADO):
    for i in range(LADO):
        if le(alfa, i, j) < 0.999:
            continue
        vs, vc, vp = le(sem, i, j), le(com, i, j), le(preto, i, j)
        p = nosso(tuple(le(c, i, j) for c in pos))
        n = nosso(tuple(le(c, i, j) for c in nrm))
        o = int(round(le(idx, i, j)))
        if vs > 0.995:
            controlo.append(abs(vp - (1.0 + n[1]) / 2.0))
        linhas.append(
            f"{i},{j},{o},{p[0]:.5f},{p[1]:.5f},{p[2]:.5f},{n[0]:.5f},{n[1]:.5f},{n[2]:.5f},"
            f"{vs:.5f},{vc:.5f},{vp:.5f}," + ",".join(f"{le(brilho[k], i, j):.5f}" for k in BRILHO)
        )

ctl = sum(controlo) / max(len(controlo), 1)
with open(saida, "w") as f:
    f.write(f"# ORÁCULO: Blender {bpy.app.version_string} Cycles CPU, {AMOSTRAS} amostras ({AMOSTRAS_PRETO} no "
            f"`preto`), sem denoise, 1 ricochete difuso, filtro 0,01 px, céu uniforme 1, chão difuso infinito "
            f"em y = 0 invisível à câmara; peças brancas à câmara e pretas aos ricochetes.\n")
    f.write("# Gerado por docs/3DModeling/ferramentas/oraculo_chao_tapa_blender.py — NÃO editar à mão.\n")
    f.write(f"# CENA lado={LADO} caixa={CAIXA} esferas={ESFERAS} de={DE} alvo={ALVO} meia={MEIA}\n")
    f.write(f"# CONTROLO da forma fechada: nos {len(controlo)} px que nada tapa, |preto − (1 + n.y)/2| médio "
            f"{ctl:.4f}.\n")
    f.write("# obj: 1 = caixa, 2.. = as esferas pela ordem; p e n no MUNDO nosso; sem/com/preto = E/π da "
            "difusa; espelho_*/aspero_* = o metal branco de rugosidade 0 / 0,5, sem e com o chão; verniz_* = o "
            "metal 0,5 com verniz nítido.\n")
    f.write("i,j,obj,x,y,z,nx,ny,nz,sem,com,preto," + ",".join(BRILHO) + "\n")
    f.write("\n".join(linhas) + "\n")
print(f"ORACULO: {len(linhas)} linhas em {saida} · controlo {ctl:.4f} sobre {len(controlo)} px")
