# ⭐ ORÁCULO DAS CAPTURAS DE REFLEXO — o Blender (Cycles) CORRIDO sobre uma cena NOSSA.
#
# Pergunta que ele responde (e só esta): uma peça ESPELHADA pousada ao lado de outras, sob um céu
# UNIFORME 1 e sobre um chão branco difuso infinito (`y = 0`) — que luz chega ao olho pelo reflexo?
# A do céu, a do chão (escurecido onde as peças o tapam) e a das VIZINHAS, foscas e escuras (albedo
# `ALBEDO`), cada uma com a luz que o céu e o chão lhe dão.
#
# A cena e a câmara são as do `oraculo_chao_tapa_blender.py` (as mesmas três peças). As peças são o
# material delas para a câmara e para o raio do REFLEXO (glossy), e PRETAS para os raios difusos: a
# nossa lei não tem inter-reflexo difuso entre peças (o mesmo «Is Camera Ray» do oráculo do chão). Um
# reflexo e dois ricochetes difusos (câmara → espelho → vizinha → chão → céu).
#
# Corridas da MESMA câmara:
#   `viz_espelho` — a esfera POUSADA é metal branco nítido (rugosidade 0); a caixa e a flutuante, foscas;
#   `viz_aspero`  — a mesma, de rugosidade 0,5 (α = 0,25 — o pré-filtro por rugosidade);
#   `viz_caixa`   — a CAIXA é o espelho nítido (faces planas: a paralaxe), as esferas foscas;
#   `solo_espelho` / `solo_aspero` — todas as peças metal, invisíveis a todo raio que não seja da câmara,
#   sem chão: cada uma vê só o céu (a resposta SOZINHA que normaliza a régua).
#
# Corra (o arnês põe o Cycles na fatia da linha):
#   cd <worktree> && bash scripts/ph2d-run.sh blender -b -X --python \
#       docs/3DModeling/ferramentas/oraculo_reflexo_vizinhas_blender.py -- \
#       crates/ph2d-mesh-forward/fixtures/oraculo_reflexo_vizinhas.csv
#
# Convenção: o Blender é Z-para-cima; o produto é Y-para-cima. (x, y, z)_nosso = (x, z, -y)_blender.
import os
import sys

import bpy
from mathutils import Vector

LADO = 256
AMOSTRAS = 2048
AMOSTRAS_SOLO = 512
ALBEDO = 0.3
ESFERAS = [
    ("pousada", (-0.45, 0.3, 0.0), 0.3),
    ("a_flutuar", (0.05, 0.21, 0.5), 0.15),
]
CAIXA = ((0.4, 0.2, -0.05), 0.4)
DE = (0.6, 0.15, 1.0)
ALVO = (0.0, 0.2, 0.15)
MEIA = 0.8

saida = sys.argv[sys.argv.index("--") + 1 :][0]


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
s.cycles.max_bounces = 3
s.cycles.diffuse_bounces = 2
s.cycles.glossy_bounces = 1
s.cycles.transmission_bounces = 0
s.cycles.transparent_max_bounces = 0
s.cycles.caustics_reflective = False
s.cycles.caustics_refractive = False
s.cycles.sample_clamp_indirect = 0.0
s.cycles.filter_width = 0.01
s.cycles.seed = 17
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


def material(nome):
    """Uma peça: `visivel` (metal ou fosca) à câmara e ao reflexo, preta aos raios difusos."""
    m = bpy.data.materials.new(nome)
    m.use_nodes = True
    nt = m.node_tree
    nt.nodes.clear()
    fosca = nt.nodes.new("ShaderNodeBsdfDiffuse")
    fosca.inputs["Color"].default_value = (ALBEDO, ALBEDO, ALBEDO, 1.0)
    metal = nt.nodes.new("ShaderNodeBsdfGlossy")
    metal.distribution = "GGX"
    metal.inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
    preta = nt.nodes.new("ShaderNodeBsdfDiffuse")
    preta.inputs["Color"].default_value = (0.0, 0.0, 0.0, 1.0)
    lp = nt.nodes.new("ShaderNodeLightPath")
    mix = nt.nodes.new("ShaderNodeMixShader")
    # Fac 1 (raio difuso) → a entrada 2 = preta.
    nt.links.new(lp.outputs["Is Diffuse Ray"], mix.inputs["Fac"])
    nt.links.new(preta.outputs["BSDF"], mix.inputs[2])
    nt.links.new(fosca.outputs["BSDF"], mix.inputs[1])
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    nt.links.new(mix.outputs["Shader"], out.inputs["Surface"])
    return m, nt, mix, fosca, metal


pecas = []
(c, a) = CAIXA
bpy.ops.mesh.primitive_cube_add(size=a, location=b(c))
o = bpy.context.active_object
o.pass_index = 1
pecas.append(o)
for k, (_, c, r) in enumerate(ESFERAS):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=256, ring_count=128, radius=r, location=b(c))
    bpy.ops.object.shade_smooth()
    o = bpy.context.active_object
    o.pass_index = 2 + k
    pecas.append(o)
mats = []
for k, o in enumerate(pecas):
    m = material(f"peca{k}")
    o.data.materials.append(m[0])
    mats.append(m)

chao_mat = bpy.data.materials.new("chao")
chao_mat.use_nodes = True
nt = chao_mat.node_tree
nt.nodes.clear()
d = nt.nodes.new("ShaderNodeBsdfDiffuse")
d.inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
o_ = nt.nodes.new("ShaderNodeOutputMaterial")
nt.links.new(d.outputs["BSDF"], o_.inputs["Surface"])
bpy.ops.mesh.primitive_plane_add(size=200.0, location=(0.0, 0.0, 0.0))
chao = bpy.context.active_object
chao.data.materials.append(chao_mat)
chao.visible_camera = False

cam_d = bpy.data.cameras.new("cam")
cam_d.type = "ORTHO"
cam_d.ortho_scale = 2.0 * MEIA
cam = bpy.data.objects.new("cam", cam_d)
dv = Vector(DE).normalized()
cam.location = Vector(b(ALVO)) + Vector(b(tuple(dv * 6.0)))
cam.rotation_euler = (-Vector(b(tuple(dv)))).to_track_quat("-Z", "Y").to_euler()
s.collection.objects.link(cam)
s.camera = cam

import OpenImageIO as oiio


def corre(nome, amostras):
    s.cycles.samples = amostras
    exr = os.path.join(os.path.dirname(os.path.abspath(saida)), f"_oraculo_reflexo_{nome}.exr")
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


def espelho(k, rug):
    """A peça `k` passa a metal de rugosidade `rug`; as outras, foscas."""
    for j, (_, nt_, mix, fosca, metal) in enumerate(mats):
        metal.inputs["Roughness"].default_value = rug
        fonte = metal if j == k else fosca
        nt_.links.new(fonte.outputs["BSDF"], mix.inputs[1])


res = {}
p_base = None
for nome, k, rug in (("viz_espelho", 1, 0.0), ("viz_aspero", 1, 0.5), ("viz_caixa", 0, 0.0)):
    espelho(k, rug)
    p = corre(nome, AMOSTRAS)
    res[nome] = canal(p, "Combined.R")
    p_base = p_base or p

# O SOLO: todas metal, invisíveis aos raios que não são da câmara, sem chão — só o céu.
for o in pecas:
    o.visible_glossy = o.visible_diffuse = o.visible_shadow = o.visible_transmission = False
chao.hide_render = True
for nome, rug in (("solo_espelho", 0.0), ("solo_aspero", 0.5)):
    for _, nt_, mix, _, metal in mats:
        metal.inputs["Roughness"].default_value = rug
        nt_.links.new(metal.outputs["BSDF"], mix.inputs[1])
    res[nome] = canal(corre(nome, AMOSTRAS_SOLO), "Combined.R")

COLUNAS = ("viz_espelho", "viz_aspero", "viz_caixa", "solo_espelho", "solo_aspero")
alfa = canal(p_base, "Combined.A")
pos = [canal(p_base, f"Position.{e}") for e in "XYZ"]
nrm = [canal(p_base, f"Normal.{e}") for e in "XYZ"]
idx = canal(p_base, "Object Index.X")
linhas = []
for j in range(LADO):
    for i in range(LADO):
        if le(alfa, i, j) < 0.999:
            continue
        p = nosso(tuple(le(c, i, j) for c in pos))
        n = nosso(tuple(le(c, i, j) for c in nrm))
        o = int(round(le(idx, i, j)))
        linhas.append(
            f"{i},{j},{o},{p[0]:.5f},{p[1]:.5f},{p[2]:.5f},{n[0]:.5f},{n[1]:.5f},{n[2]:.5f},"
            + ",".join(f"{le(res[k], i, j):.5f}" for k in COLUNAS)
        )

with open(saida, "w") as f:
    f.write(f"# ORÁCULO: Blender {bpy.app.version_string} Cycles CPU, {AMOSTRAS} amostras ({AMOSTRAS_SOLO} nos "
            f"`solo_*`), sem denoise, 1 reflexo + 2 ricochetes difusos, filtro 0,01 px, céu uniforme 1, chão "
            f"difuso branco infinito em y = 0 invisível à câmara; as peças pretas aos raios difusos.\n")
    f.write("# Gerado por docs/3DModeling/ferramentas/oraculo_reflexo_vizinhas_blender.py — NÃO editar à mão.\n")
    f.write(f"# CENA lado={LADO} caixa={CAIXA} esferas={ESFERAS} de={DE} alvo={ALVO} meia={MEIA} "
            f"albedo_das_foscas={ALBEDO}\n")
    f.write("# obj: 1 = caixa, 2.. = as esferas pela ordem; p e n no MUNDO nosso; viz_espelho/viz_aspero = a "
            "esfera pousada metal branco de rugosidade 0 / 0,5 e as outras foscas; viz_caixa = a caixa metal "
            "nítido e as esferas foscas; solo_* = todas metal a ver só o céu.\n")
    f.write("i,j,obj,x,y,z,nx,ny,nz," + ",".join(COLUNAS) + "\n")
    f.write("\n".join(linhas) + "\n")
print(f"ORACULO: {len(linhas)} linhas em {saida}")
