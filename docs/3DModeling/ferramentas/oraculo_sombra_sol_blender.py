# ⭐ ORÁCULO DA SOMBRA DO SOL — o Blender (Cycles) CORRIDO sobre uma cena NOSSA.
#
# Pergunta que ele responde (e só esta): um sol de DISCO uniforme (direcção e raio angular
# conhecidos) sobre um chão que só recebe (shadow catcher), com uma esfera a flutuar e uma caixa
# pousada — quanto escurece cada ponto do chão? (`1 − o passe Shadow Catcher`, luz directa só.)
# O gate `tests_sol::a_sombra_do_sol_e_a_do_cycles` lê o CSV e compara o desenhista de jogo PASSO A
# PASSO (linhas e colunas de pixels do mesmo enquadramento).
#
# Corra (o arnês põe o Cycles na fatia da linha):
#   cd <worktree> && bash scripts/ph2d-run.sh blender -b -X --python \
#       docs/3DModeling/ferramentas/oraculo_sombra_sol_blender.py -- \
#       crates/ph2d-mesh-forward/fixtures/oraculo_sombra_sol.csv
#
# Convenção: o Blender é Z-para-cima; o produto é Y-para-cima. (x, y, z)_nosso = (x, z, -y)_blender.
# ⚠️ As constantes da CENA repetem-se em `tests_sol.rs` — o cabeçalho do CSV leva-as e o gate recusa
# um CSV de outra cena.
import math
import os
import sys

import bpy
from mathutils import Vector

LADO = 384  # pixels do enquadramento quadrado
# Os enquadramentos (nome, centro x, centro z, meia-aresta — nosso): a cena inteira, e DE PERTO a borda
# da sombra da caixa junto do canto onde ela toca o chão (aí a sombra verdadeira é dura).
QUADROS = [("cena", 0.1, 0.0, 1.5), ("perto", 0.85, -0.2, 0.05)]
ESFERA = ((-0.8, 0.6, 0.0), 0.3)  # (centro nosso, raio) — flutua 0,3 acima do chão
CAIXA = ((0.6, 0.2, 0.0), 0.4)  # (centro nosso, aresta) — pousada
CASOS = [(40.0, 1.0), (40.0, 4.0), (15.0, 1.0), (40.0, 10.0)]  # (altura do sol, raio angular), graus
AMOSTRAS = 512

argv = sys.argv[sys.argv.index("--") + 1 :]
saida = argv[0]


def nosso_para_blender(p):
    return (p[0], -p[2], p[1])


def cena(altura, raio, quadro):
    (_, cx, cz, meia) = quadro
    bpy.ops.wm.read_factory_settings(use_empty=True)
    s = bpy.context.scene
    s.render.engine = "CYCLES"
    s.cycles.device = "CPU"
    s.cycles.samples = AMOSTRAS
    s.cycles.use_denoising = False
    s.cycles.use_adaptive_sampling = False
    s.cycles.max_bounces = 0  # luz DIRECTA: a pergunta é a sombra, não o ricochete
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
    w = bpy.data.worlds.new("preto")
    w.use_nodes = True
    w.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.0
    s.world = w
    # O SOL: disco uniforme de diâmetro angular `2·raio`, a vir de L = (−cos h, sen h, 0) (nosso).
    h = math.radians(altura)
    l_b = Vector(nosso_para_blender((-math.cos(h), math.sin(h), 0.0)))
    luz = bpy.data.lights.new("sol", type="SUN")
    luz.energy = 3.0
    luz.angle = math.radians(2.0 * raio)
    ob = bpy.data.objects.new("sol", luz)
    ob.rotation_euler = l_b.to_track_quat("Z", "Y").to_euler()
    s.collection.objects.link(ob)
    # O chão que só recebe.
    bpy.ops.mesh.primitive_plane_add(size=20.0, location=(0.0, 0.0, 0.0))
    bpy.context.active_object.is_shadow_catcher = True
    # A esfera e a caixa: fazem sombra, a câmara não as vê (o chão é que é medido).
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


linhas = []
for quadro, altura, raio in [(QUADROS[0], h, r) for h, r in CASOS] + [(QUADROS[1], 40.0, 1.0)]:
    (nome, cx, cz, meia) = quadro
    s = cena(altura, raio, quadro)
    exr = os.path.join(os.path.dirname(os.path.abspath(saida)), f"_oraculo_sol_{nome}_{altura}_{raio}.exr")
    s.render.filepath = exr
    bpy.ops.render.render(write_still=True)
    # O passe do Shadow Catcher vem numa camada própria do multilayer; o Blender traz o OIIO.
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

    def escuro(i, j):
        # (i, j) com a linha j de CIMA; o OIIO lê a linha 0 em cima.
        return 1.0 - float(px[j][i][k[0]])

    def mundo(i, j):
        return (cx + ((i + 0.5) / LADO * 2.0 - 1.0) * meia, cz + ((j + 0.5) / LADO * 2.0 - 1.0) * meia)

    def guarda(corte, i, j):
        x, z = mundo(i, j)
        linhas.append(f"{nome},{altura},{raio},{corte},{i},{j},{x:.6f},{z:.6f},{escuro(i, j):.6f}")

    if nome == "cena":
        # A linha do meio (z ≈ 0) e a coluna que atravessa a sombra da esfera.
        j0 = LADO // 2
        sx = ESFERA[0][0] + (ESFERA[0][1]) / math.tan(math.radians(altura))
        i0 = min(LADO - 1, max(0, int((sx - (cx - meia)) / (2.0 * meia) * LADO)))
        for i in range(LADO):
            guarda("linha", i, j0)
        for j in range(LADO):
            guarda("coluna", i0, j)
    else:
        # Três colunas que cruzam a borda da sombra, cada vez mais longe do canto.
        for i0 in (LADO // 4, LADO // 2, 3 * LADO // 4):
            for j in range(LADO):
                guarda("coluna", i0, j)

with open(saida, "w") as f:
    f.write(f"# ORÁCULO: Blender {bpy.app.version_string} Cycles CPU, {AMOSTRAS} amostras, sem denoise, "
            f"luz directa só (max_bounces 0), filtro 0,01 px.\n")
    f.write("# Gerado por docs/3DModeling/ferramentas/oraculo_sombra_sol_blender.py — NÃO editar à mão.\n")
    f.write(f"# CENA lado={LADO} quadros={QUADROS} esfera={ESFERA} caixa={CAIXA} "
            f"sol=disco de luz SUN, L=(-cos h, sen h, 0) nosso\n")
    f.write("# escuro = 1 - passe Shadow Catcher (o chão a y=0, objetos invisíveis à câmara).\n")
    f.write("quadro,altura,raio,corte,i,j,x,z,escuro\n")
    f.write("\n".join(linhas) + "\n")
print(f"ORACULO: {len(linhas)} linhas em {saida}")
