# ⭐ ORÁCULO DOS REFLEXOS DE PERTO — o Blender (Cycles) CORRIDO sobre uma cena NOSSA (a da cena 42).
#
# O report do dono de 04/10: de perto, o reflexo das vizinhas no cromo tinha bordas em DEGRAUS,
# MORDIDAS e cunhas escuras. O oráculo de longe (`oraculo_reflexo_vizinhas_blender.py`) não o via: o
# cromo tinha ~100 px e o contorno de cada vizinha refletida 1 px. Aqui o cromo enche o quadro (512 px).
#
# A cena: o CROMO (r 0,3) no meio, pousado; quatro vizinhas foscas de albedos diferentes à volta (a
# arrumação da cena 42, sem o alumínio). Céu uniforme 1, chão branco difuso infinito; as peças pretas aos
# raios difusos (a nossa lei não tem inter-reflexo difuso). Um reflexo e dois ricochetes difusos.
#
# Corridas da MESMA câmara: `viz` (o cromo metal branco de rugosidade 0), `viz2` (`RUG2`: 0,05, o da cena
# 42; `0,3` no modo `par`), `solo` / `solo2` (o cromo sozinho, sem chão: só o céu — a normalização).
#
# Corra (o arnês põe o Cycles na fatia da linha):
#   cd <worktree> && bash scripts/ph2d-run.sh blender -b -X --python \
#       docs/3DModeling/ferramentas/oraculo_reflexo_perto_blender.py -- \
#       crates/ph2d-mesh-forward/fixtures/oraculo_reflexo_perto.csv.gz
#
# Convenção: o Blender é Z-para-cima; o produto é Y-para-cima. (x, y, z)_nosso = (x, z, -y)_blender.
import os
import sys

import bpy
from mathutils import Vector

LADO = 512
AMOSTRAS = 1024
AMOSTRAS_SOLO = 256
# (nome, centro nosso, raio ou meia-aresta, é caixa, albedo): o cromo primeiro.
PECAS = [
    ("cromo", (0.0, 0.3, 0.0), 0.3, False, 1.0),
    ("bola_a", (0.0, 0.2, -0.7), 0.2, False, 0.8),
    ("caixa_b", (0.0, 0.18, 0.72), 0.18, True, 0.15),
    ("bola_c", (0.55, 0.17, -0.25), 0.17, False, 0.5),
    ("caixa_d", (-0.6, 0.15, 0.2), 0.15, True, 0.3),
]
DE = (1.0, 0.45, 0.3)
ALVO = (0.0, 0.3, 0.0)
MEIA = 0.34

argv = sys.argv[sys.argv.index("--") + 1 :]
saida = argv[0]
# A 2.ª rugosidade do cromo (a coluna `viz2`/`solo2`).
RUG2 = 0.05
# `-- <saida> par`: o report do dono de 04/10 (a «junta»): a caixa AZUL atrás da VERDE vista do centro do
# cromo, e o cromo áspero (`0,3`) — a parte da azul que a captura não vê, e o áspero.
if len(argv) > 1 and argv[1] == "par":
    PECAS = [
        ("cromo", (0.0, 0.3, 0.0), 0.3, False, 1.0),
        ("caixa_verde", (0.0, 0.18, 0.72), 0.18, True, 0.15),
        ("caixa_azul", (-0.5, 0.15, 1.0), 0.15, True, 0.5),
    ]
    DE = (1.0, 0.45, 0.6)
    RUG2 = 0.3
# `-- <saida> junta`: o report 3 do dono de 04/10 (a «junta» que o `par` não reproduziu): a caixa AZUL LONGE
# meio atrás da VERDE PERTO vistas do centro do cromo (a borda delas partilhada na captura), e com FUNDO entre
# elas visto de cada ponto do cromo; a vista amplia o par refletido. O cromo a `0,05` (o da cena 42).
if len(argv) > 1 and argv[1] == "junta":
    PECAS = [
        ("cromo", (0.0, 0.3, 0.0), 0.3, False, 1.0),
        ("caixa_verde", (0.0, 0.18, 0.72), 0.18, True, 0.15),
        ("caixa_azul", (0.7, 0.15, 1.7), 0.15, True, 0.5),
    ]
    DE = (1.0, 0.45, 0.8)
    ALVO = (-0.0562, 0.2449, 0.1013)
    MEIA = 0.14


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


def material(nome, albedo):
    """Uma peça: `visivel` (metal ou fosca) à câmara e ao reflexo, preta aos raios difusos."""
    m = bpy.data.materials.new(nome)
    m.use_nodes = True
    nt = m.node_tree
    nt.nodes.clear()
    fosca = nt.nodes.new("ShaderNodeBsdfDiffuse")
    fosca.inputs["Color"].default_value = (albedo, albedo, albedo, 1.0)
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
mats = []
for k, (nome, c, r, caixa, albedo) in enumerate(PECAS):
    if caixa:
        bpy.ops.mesh.primitive_cube_add(size=2.0 * r, location=b(c))
    else:
        bpy.ops.mesh.primitive_uv_sphere_add(segments=256, ring_count=128, radius=r, location=b(c))
        bpy.ops.object.shade_smooth()
    o = bpy.context.active_object
    o.pass_index = 1 + k
    m = material(nome, albedo)
    o.data.materials.append(m[0])
    pecas.append(o)
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
    exr = os.path.join(os.path.dirname(os.path.abspath(saida)), f"_oraculo_perto_{nome}.exr")
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
for nome, rug in (("viz", 0.0), ("viz2", RUG2)):
    espelho(0, rug)
    p = corre(nome, AMOSTRAS)
    res[nome] = canal(p, "Combined.R")
    p_base = p_base or p

# O SOLO: o cromo sozinho (as outras escondidas), sem chão — só o céu.
for o in pecas[1:]:
    o.hide_render = True
chao.hide_render = True
for nome, rug in (("solo", 0.0), ("solo2", RUG2)):
    espelho(0, rug)
    res[nome] = canal(corre(nome, AMOSTRAS_SOLO), "Combined.R")

COLUNAS = ("viz", "viz2", "solo", "solo2")
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
        # Só o CROMO (os pixels das vizinhas medem a difusa, que o oráculo do chão já mede): `~0,7 MB`.
        if o != 1:
            continue
        linhas.append(
            f"{i},{j}," + ",".join(f"{le(res[k], i, j):.4f}" for k in COLUNAS)
        )

import gzip

with gzip.open(saida, "wt", encoding="utf-8") as f:
    f.write(f"# ORÁCULO: Blender {bpy.app.version_string} Cycles CPU, {AMOSTRAS} amostras ({AMOSTRAS_SOLO} nos "
            f"`solo*`), sem denoise, 1 reflexo + 2 ricochetes difusos, filtro 0,01 px, céu uniforme 1, chão "
            f"difuso branco infinito em y = 0 invisível à câmara; as peças pretas aos raios difusos.\n")
    f.write("# Gerado por docs/3DModeling/ferramentas/oraculo_reflexo_perto_blender.py — NÃO editar à mão.\n")
    f.write(f"# CENA lado={LADO} pecas={PECAS} de={DE} alvo={ALVO} meia={MEIA}\n")
    f.write("# Só os pixels do cromo, com gzip; o ponto e a normal de cada um tiram-se da esfera e da câmara; viz/viz2 = o cromo metal "
            f"branco de rugosidade 0 / {RUG2} com as vizinhas e o chão; solo/solo2 = o cromo sozinho a ver o céu.\n")
    f.write("i,j," + ",".join(COLUNAS) + "\n")
    f.write("\n".join(linhas) + "\n")
print(f"ORACULO: {len(linhas)} linhas em {saida}")
