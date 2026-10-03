# ⭐ ORÁCULO DO CÉU HDRI — o Blender (Cycles) CORRIDO sobre os MESMOS EXR que o produto embarca.
#
# Pergunta que ele responde (e só esta): sob um céu equiretangular, que luz devolve
#   * uma esfera DIFUSA branca  → por pixel, E(n)/π       (a `irradiance` da lei do material)
#   * uma esfera ESPELHO branca → por pixel, L(reflexo)  (a ORIENTAÇÃO do mapa: u, v, sinais, eixos)
# Nada de BRDF rugoso aqui: o Cycles e o OpenPBR diferem no lóbulo, e a régua do pré-filtro rugoso é a
# quadratura (matemática), não um app.
#
# Corra (o arnês põe o Cycles na fatia da linha):
#   cd <worktree> && bash scripts/ph2d-run.sh blender -b -X --python \
#       docs/3DModeling/ferramentas/oraculo_ceu_blender.py -- <saida.csv> <hdri.exr>...
#
# Convenção: o Blender é Z-para-cima; o produto é Y-para-cima. (x, y, z)_nosso = (x, z, -y)_blender.
# A câmara do Blender está em -Y a olhar +Y (a nossa em +z a olhar -z). A normal no CSV é a NOSSA.
import math
import os
import sys

import bpy

LADO = 96
ESCALA = 2.2  # largura ortográfica; a esfera tem raio 1
AMOSTRAS = {"difusa": 2048, "espelho": 64}
PASSO = 5  # um pixel em cada PASSO, nas duas direcções
RAIO_MAX = 0.92  # longe da silhueta (a borda mistura fundo)

argv = sys.argv[sys.argv.index("--") + 1 :]
saida, hdris = argv[0], argv[1:]


def cena_limpa():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    s = bpy.context.scene
    s.render.engine = "CYCLES"
    s.cycles.device = "CPU"
    s.cycles.use_denoising = False
    s.cycles.use_adaptive_sampling = False
    s.cycles.max_bounces = 1
    s.cycles.diffuse_bounces = 1
    s.cycles.glossy_bounces = 1
    s.cycles.transmission_bounces = 0
    s.cycles.volume_bounces = 0
    s.cycles.transparent_max_bounces = 0
    s.cycles.sample_clamp_direct = 0.0
    s.cycles.sample_clamp_indirect = 0.0
    s.cycles.filter_width = 0.01  # amostra o CENTRO do pixel
    s.cycles.seed = 7
    s.render.resolution_x = LADO
    s.render.resolution_y = LADO
    s.render.resolution_percentage = 100
    s.render.film_transparent = False
    s.view_settings.view_transform = "Standard"
    s.view_settings.look = "None"
    s.view_settings.exposure = 0.0
    s.view_settings.gamma = 1.0
    s.render.image_settings.file_format = "OPEN_EXR"
    s.render.image_settings.color_depth = "32"
    s.render.image_settings.exr_codec = "ZIP"
    return s


def mundo(s, hdri):
    w = bpy.data.worlds.new("ceu")
    s.world = w
    w.use_nodes = True
    nt = w.node_tree
    nt.nodes.clear()
    env = nt.nodes.new("ShaderNodeTexEnvironment")
    env.image = bpy.data.images.load(hdri)
    env.interpolation = "Linear"
    env.projection = "EQUIRECTANGULAR"
    bg = nt.nodes.new("ShaderNodeBackground")
    bg.inputs["Strength"].default_value = 1.0
    out = nt.nodes.new("ShaderNodeOutputWorld")
    nt.links.new(env.outputs["Color"], bg.inputs["Color"])
    nt.links.new(bg.outputs["Background"], out.inputs["Surface"])


def esfera(s, tipo):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=256, ring_count=128, radius=1.0)
    ob = bpy.context.active_object
    bpy.ops.object.shade_smooth()
    m = bpy.data.materials.new(tipo)
    m.use_nodes = True
    nt = m.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    if tipo == "difusa":
        b = nt.nodes.new("ShaderNodeBsdfDiffuse")
        b.inputs["Roughness"].default_value = 0.0
    else:
        b = nt.nodes.new("ShaderNodeBsdfGlossy")
        b.distribution = "GGX"
        b.inputs["Roughness"].default_value = 0.0
    b.inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
    nt.links.new(b.outputs["BSDF"], out.inputs["Surface"])
    ob.data.materials.append(m)
    cam_d = bpy.data.cameras.new("cam")
    cam_d.type = "ORTHO"
    cam_d.ortho_scale = ESCALA
    cam = bpy.data.objects.new("cam", cam_d)
    s.collection.objects.link(cam)
    cam.location = (0.0, -5.0, 0.0)
    cam.rotation_euler = (math.pi / 2, 0.0, 0.0)
    s.camera = cam


linhas = []
for hdri in hdris:
    nome = os.path.splitext(os.path.basename(hdri))[0]
    for tipo, spp in AMOSTRAS.items():
        s = cena_limpa()
        s.cycles.samples = spp
        mundo(s, hdri)
        esfera(s, tipo)
        exr = os.path.join(os.path.dirname(os.path.abspath(saida)), f"_oraculo_{nome}_{tipo}.exr")
        s.render.filepath = exr
        bpy.ops.render.render(write_still=True)
        img = bpy.data.images.load(exr)
        px = list(img.pixels)
        for py in range(PASSO // 2, LADO, PASSO):
            for pxi in range(PASSO // 2, LADO, PASSO):
                # O Blender guarda a linha 0 em BAIXO.
                xb = ((pxi + 0.5) / LADO * 2.0 - 1.0) * ESCALA / 2.0
                zb = ((py + 0.5) / LADO * 2.0 - 1.0) * ESCALA / 2.0
                r2 = xb * xb + zb * zb
                if r2 > RAIO_MAX * RAIO_MAX:
                    continue
                k = (py * LADO + pxi) * 4
                n = (xb, zb, math.sqrt(1.0 - r2))  # a NOSSA normal: (x, z, -y)_blender
                linhas.append(
                    f"{nome},{tipo},{n[0]:.7f},{n[1]:.7f},{n[2]:.7f},"
                    f"{px[k]:.7g},{px[k + 1]:.7g},{px[k + 2]:.7g}"
                )
        os.remove(exr)

with open(saida, "w") as f:
    f.write(f"# ORÁCULO: Blender {bpy.app.version_string} Cycles CPU, sem denoise, filtro 0,01 px, "
            f"{AMOSTRAS} amostras, ortográfica {LADO}x{LADO} escala {ESCALA}, esfera r=1, branca.\n")
    f.write("# Gerado por docs/3DModeling/ferramentas/oraculo_ceu_blender.py — NÃO editar à mão.\n")
    f.write("# Céus: os EXR do Blender (Poly Haven, CC0). Normal em Y-para-cima: (x, z, -y) do Blender.\n")
    f.write("# difusa: rgb = E(n)/pi · espelho: rgb = L(reflexo de -vista em n), vista = (0, 0, 1).\n")
    f.write("ceu,tipo,nx,ny,nz,r,g,b\n")
    f.write("\n".join(linhas) + "\n")
print(f"ORACULO: {len(linhas)} linhas em {saida}")
