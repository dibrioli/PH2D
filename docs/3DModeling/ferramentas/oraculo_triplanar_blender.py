# ⭐ ORÁCULO DA TRIPLANAR — o Blender (Cycles) CORRIDO sobre entradas NOSSAS.
#
# A triplanar do Blender é o nó Image Texture com projecção «Box» e o «Blend». Pergunta que este
# script responde, POR PASSO:
#   1. «pesos»: com uma imagem-RAMPA float (R = u, G = v, B = u·v, Non-Color), que (u, v) cada vista lê
#      e com que peso as três se misturam — por ponto, em função da normal e do Blend;
#   2. «cor»: com a textura de teste COLORIDA (PNG sRGB 8 bits, a fixtura nossa), a cor que sai,
#      com a interpolação «Linear» (bilinear, sem mipmaps) e repetição.
# Objectos: uma esfera r = 1 (suave) e uma caixa 1,6 × 1,0 × 0,6 (faces planas), os dois GIRADOS — as
# coordenadas são as do OBJECTO, como o «Object» do nó Texture Coordinate. Cada pixel é lido em três
# renders de EMISSÃO com a mesma câmara e 1 amostra no centro do pixel: posição, normal e textura.
#
# Corra (o arnês põe o Cycles na fatia da linha):
#   cd <worktree> && bash scripts/ph2d-run.sh blender -b -X --python \
#       docs/3DModeling/ferramentas/oraculo_triplanar_blender.py -- \
#       crates/ph2d-triplanar/fixtures/oraculo_triplanar.csv \
#       crates/ph2d-triplanar/fixtures/teste_colorida.png
#
# Convenção: o Blender é Z-para-cima; o produto é Y-para-cima. (x, y, z)_nosso = (x, z, -y)_blender.
# A linha 0 de uma imagem no Blender é a de BAIXO: v = 0 é a última linha do PNG.
import math
import os
import sys

import bpy

LADO = 96
ESCALA = 3.0
PASSO = 3
# A projecção da cor: vector = Object · TEXTURA_ESCALA + TEXTURA_DESLOCA (Blender).
TEXTURA_ESCALA = 1.3
TEXTURA_DESLOCA = (0.17, 0.31, 0.05)
N_RAMPA = 256
CASOS = [("pesos", b) for b in (0.2, 0.5, 1.0)] + [("cor", b) for b in (0.0, 0.3, 1.0)]
OBJECTOS = ("esfera", "caixa")
GIRO = (0.31, -0.52, 0.77)  # rad, XYZ do Blender
VISTAS = ((1.0, -1.2, 0.9), (-1.0, 1.2, -0.9))  # de onde a câmara olha para a origem (Blender)

argv = sys.argv[sys.argv.index("--") + 1 :]
saida, png = argv[0], argv[1]


def cena_limpa():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    s = bpy.context.scene
    s.render.engine = "CYCLES"
    s.cycles.device = "CPU"
    s.cycles.samples = 1
    s.cycles.use_denoising = False
    s.cycles.use_adaptive_sampling = False
    s.cycles.filter_width = 0.01  # amostra o CENTRO do pixel
    s.cycles.seed = 7
    s.render.resolution_x = LADO
    s.render.resolution_y = LADO
    s.render.resolution_percentage = 100
    s.render.film_transparent = True
    s.view_settings.view_transform = "Standard"
    s.view_settings.look = "None"
    s.render.image_settings.file_format = "OPEN_EXR"
    s.render.image_settings.color_mode = "RGBA"
    s.render.image_settings.color_depth = "32"
    return s


def rampa():
    im = bpy.data.images.new("rampa", N_RAMPA, N_RAMPA, alpha=True, float_buffer=True)
    im.colorspace_settings.name = "Non-Color"
    px = [0.0] * (N_RAMPA * N_RAMPA * 4)
    for j in range(N_RAMPA):
        for i in range(N_RAMPA):
            k = (j * N_RAMPA + i) * 4
            u = (i + 0.5) / N_RAMPA
            v = (j + 0.5) / N_RAMPA
            px[k], px[k + 1], px[k + 2], px[k + 3] = u, v, u * v, 1.0
    im.pixels = px
    return im


def objecto(s, qual):
    if qual == "esfera":
        bpy.ops.mesh.primitive_uv_sphere_add(segments=512, ring_count=256, radius=1.0)
        ob = bpy.context.active_object
        bpy.ops.object.shade_smooth()
    else:
        bpy.ops.mesh.primitive_cube_add(size=1.0)
        ob = bpy.context.active_object
        # As dimensões ficam na MALHA (coordenadas do objecto), não na escala do objecto.
        for v in ob.data.vertices:
            v.co = (v.co.x * 1.6, v.co.y * 1.0, v.co.z * 0.6)
        bpy.ops.object.shade_flat()
    ob.rotation_euler = GIRO
    return ob


def material(ob, saida_de, caso, blend):
    m = bpy.data.materials.new("m")
    m.use_nodes = True
    nt = m.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    em = nt.nodes.new("ShaderNodeEmission")
    em.inputs["Strength"].default_value = 1.0
    tc = nt.nodes.new("ShaderNodeTexCoord")
    nt.links.new(em.outputs["Emission"], out.inputs["Surface"])
    if saida_de in ("posicao", "normal"):
        # (v · 0,25 + 0,5): a emissão fica positiva; o leitor desfaz.
        mp = nt.nodes.new("ShaderNodeMapping")
        mp.inputs["Scale"].default_value = (0.25, 0.25, 0.25)
        mp.inputs["Location"].default_value = (0.5, 0.5, 0.5)
        fonte = tc.outputs["Object"] if saida_de == "posicao" else tc.outputs["Normal"]
        nt.links.new(fonte, mp.inputs["Vector"])
        nt.links.new(mp.outputs["Vector"], em.inputs["Color"])
    else:
        mp = nt.nodes.new("ShaderNodeMapping")
        tx = nt.nodes.new("ShaderNodeTexImage")
        tx.interpolation = "Linear"
        tx.projection = "BOX"
        tx.projection_blend = blend
        if caso == "pesos":
            # A rampa: o objecto inteiro cabe em [0,1] sem repetir.
            mp.inputs["Scale"].default_value = (0.25, 0.25, 0.25)
            mp.inputs["Location"].default_value = (0.5, 0.5, 0.5)
            tx.image = rampa()
            tx.extension = "EXTEND"
        else:
            mp.inputs["Scale"].default_value = (TEXTURA_ESCALA,) * 3
            mp.inputs["Location"].default_value = TEXTURA_DESLOCA
            img = bpy.data.images.load(png)
            img.colorspace_settings.name = "sRGB"
            tx.image = img
            tx.extension = "REPEAT"
        nt.links.new(tc.outputs["Object"], mp.inputs["Vector"])
        nt.links.new(mp.outputs["Vector"], tx.inputs["Vector"])
        nt.links.new(tx.outputs["Color"], em.inputs["Color"])
    ob.data.materials.clear()
    ob.data.materials.append(m)


def camara(s, de):
    cd = bpy.data.cameras.new("c")
    cd.type = "ORTHO"
    cd.ortho_scale = ESCALA
    cam = bpy.data.objects.new("c", cd)
    s.collection.objects.link(cam)
    d = [x * 5.0 / math.sqrt(sum(y * y for y in de)) for x in de]
    cam.location = d
    alvo = bpy.data.objects.new("alvo", None)
    s.collection.objects.link(alvo)
    tr = cam.constraints.new("TRACK_TO")
    tr.target = alvo
    tr.track_axis = "TRACK_NEGATIVE_Z"
    tr.up_axis = "UP_Y"
    s.camera = cam


def render(s, nome):
    exr = os.path.join(os.path.dirname(os.path.abspath(saida)), f"_oraculo_tri_{nome}.exr")
    s.render.filepath = exr
    bpy.ops.render.render(write_still=True)
    img = bpy.data.images.load(exr)
    px = list(img.pixels)
    bpy.data.images.remove(img)
    os.remove(exr)
    return px


def nosso(v):
    return (v[0], v[2], -v[1])


linhas = []
for qual in OBJECTOS:
    for vi, de in enumerate(VISTAS):
        imgs = {}
        for chave in ("posicao", "normal") + tuple(f"{c}:{b}" for c, b in CASOS):
            s = cena_limpa()
            ob = objecto(s, qual)
            camara(s, de)
            if ":" in chave:
                caso, b = chave.split(":")
                material(ob, "textura", caso, float(b))
            else:
                material(ob, chave, None, 0.0)
            imgs[chave] = render(s, f"{qual}_{vi}_{chave.replace(':', '_')}")
        for py in range(PASSO // 2, LADO, PASSO):
            for pxi in range(PASSO // 2, LADO, PASSO):
                k = (py * LADO + pxi) * 4
                if imgs["posicao"][k + 3] < 0.999:
                    continue
                p = [(imgs["posicao"][k + q] - 0.5) * 4.0 for q in range(3)]
                n = [(imgs["normal"][k + q] - 0.5) * 4.0 for q in range(3)]
                ln = math.sqrt(sum(x * x for x in n))
                n = [x / ln for x in n]
                pn, nn = nosso(p), nosso(n)
                for c, b in CASOS:
                    t = imgs[f"{c}:{b}"]
                    linhas.append(
                        f"{qual},{c},{b},{pn[0]:.6f},{pn[1]:.6f},{pn[2]:.6f},"
                        f"{nn[0]:.6f},{nn[1]:.6f},{nn[2]:.6f},"
                        f"{t[k]:.7g},{t[k + 1]:.7g},{t[k + 2]:.7g}"
                    )

with open(saida, "w") as f:
    f.write(f"# ORÁCULO: Blender {bpy.app.version_string} Cycles CPU, 1 amostra no centro do pixel (filtro "
            f"0,01 px), emissão, ortográfica {LADO}x{LADO} escala {ESCALA}, um pixel em cada {PASSO}.\n")
    f.write("# Gerado por docs/3DModeling/ferramentas/oraculo_triplanar_blender.py — NÃO editar à mão.\n")
    f.write("# Image Texture, projecção BOX, interpolação Linear; vector = coordenadas do OBJECTO (Blender)\n")
    f.write(f"#   pesos: rampa float {N_RAMPA}² (r = u, g = v, b = u·v, Non-Color), vector·0,25 + 0,5, EXTEND\n")
    f.write(f"#   cor:   teste_colorida.png (sRGB 8 bits, 64²), vector·{TEXTURA_ESCALA} + {TEXTURA_DESLOCA}, REPEAT\n")
    f.write(f"# Objectos girados {GIRO} rad (XYZ Blender): esfera r=1 (suave), caixa 1,6x1,0x0,6 (plana).\n")
    f.write("# p e n em Y-para-cima, do OBJECTO: (x, z, -y) do Blender. rgb = linear (cena).\n")
    f.write("objecto,caso,blend,x,y,z,nx,ny,nz,r,g,b\n")
    f.write("\n".join(linhas) + "\n")
print(f"ORACULO: {len(linhas)} linhas em {saida}")
