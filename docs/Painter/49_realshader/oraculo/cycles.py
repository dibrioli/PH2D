# O ORÁCULO Cycles do Óleo Real (W1c, CLAUDE.md §0.9): corre-se o binário, nunca se lê o fonte.
# Entrada = a NOSSA tinta (`medida_oleo_real_oraculo`): a altura como malha (1 vértice por célula,
# unidades = células), a cor sem relevo como albedo, a mesma luz (azimute/altitude) e câmara ortográfica.
# Normalização: tinta plana iluminada = o albedo (como a nossa lei de neutralidade): céu uniforme de
# radiância `ceu` (+ sol de irradiância (1−ceu)·π/sin(alt)).
#
#   PH2D_GPU=1 bash scripts/ph2d-run.sh blender -b --factory-startup -P docs/Painter/49_realshader/oraculo/cycles.py -- \
#       <entrada> <saida> [altura=h_fina] [amostras=256]
#
# Saída: `<variante>.exr` (multicamada: Combined, DiffDir/Ind/Col, GlossDir/Ind/Col, AO, Normal) e
# `<variante>_agx.png` (a vista AgX, para os olhos). Variantes no MESMO processo, sobre a MESMA malha.
import math
import os
import sys

import bpy
import numpy as np
from mathutils import Vector

argv = sys.argv[sys.argv.index("--") + 1 :]
entrada, saida = argv[0], argv[1]
altura = argv[2] if len(argv) > 2 else "h_fina"
amostras = int(argv[3]) if len(argv) > 3 else 256
os.makedirs(saida, exist_ok=True)

meta = {}
for linha in open(os.path.join(entrada, "oraculo.txt")):
    if "=" in linha and not linha.startswith("#"):
        k, v = linha.strip().split("=", 1)
        meta[k] = float(v)
gw, gh = int(meta["gw"]), int(meta["gh"])
az, alt = math.radians(meta["azimute_graus"]), math.radians(meta["altitude_graus"])
ceu = meta["ceu"]
rug = meta["rugosidade"]

h = np.fromfile(os.path.join(entrada, f"{altura}.f32"), dtype="<f4").reshape(gh, gw)
assert h.size == gw * gh

bpy.ops.wm.read_factory_settings(use_empty=True)
cena = bpy.context.scene

# A malha: (i, −j, h) — y da tela para baixo, y do Blender para cima.
jj, ii = np.mgrid[0:gh, 0:gw]
verts = np.stack([ii, -jj, h], axis=-1).reshape(-1, 3).astype(np.float64)
v00 = (jj[:-1, :-1] * gw + ii[:-1, :-1]).ravel()
quads = np.stack([v00, v00 + gw, v00 + gw + 1, v00 + 1], axis=-1)
malha = bpy.data.meshes.new("tinta")
malha.from_pydata(verts.tolist(), [], quads.tolist())
malha.update()
malha.shade_smooth()
uv = malha.uv_layers.new(name="uv")
vi = np.empty(len(malha.loops), dtype=np.int32)
malha.loops.foreach_get("vertex_index", vi)
uvs = np.stack([((vi % gw) + 0.5) / gw, 1.0 - ((vi // gw) + 0.5) / gh], axis=-1)
uv.data.foreach_set("uv", uvs.astype(np.float32).ravel())
obj = bpy.data.objects.new("tinta", malha)
cena.collection.objects.link(obj)

# O material: dieléctrico (F0 0,04 = IOR 1,5), a rugosidade da nossa lei (α = r², a mesma convenção).
mat = bpy.data.materials.new("oleo")
mat.use_nodes = True
nos = mat.node_tree.nodes
bsdf = nos["Principled BSDF"]
tex = nos.new("ShaderNodeTexImage")
tex.image = bpy.data.images.load(os.path.join(entrada, "albedo.png"))
tex.image.colorspace_settings.name = "sRGB"
tex.interpolation = "Closest"
mat.node_tree.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
bsdf.inputs["Roughness"].default_value = rug
bsdf.inputs["IOR"].default_value = 1.5
obj.data.materials.append(mat)

# A câmara: ortográfica, de cima, um pixel por célula.
cam_d = bpy.data.cameras.new("cam")
cam_d.type = "ORTHO"
cam_d.ortho_scale = gw
cam_d.clip_end = 10000
cam = bpy.data.objects.new("cam", cam_d)
cam.location = ((gw - 1) / 2, -(gh - 1) / 2, 1000)
cena.collection.objects.link(cam)
cena.camera = cam
r = cena.render
r.resolution_x, r.resolution_y, r.resolution_percentage = gw, gh, 100
r.engine = "CYCLES"
cena.cycles.samples = amostras
cena.cycles.use_denoising = False
r.film_transparent = False

# A placa, se houver; senão a CPU (a resposta é a mesma).
try:
    pref = bpy.context.preferences.addons["cycles"].preferences
    for tipo in ("OPTIX", "CUDA"):
        try:
            pref.compute_device_type = tipo
            pref.get_devices()
            gpus = [d for d in pref.devices if d.type == tipo]
            if gpus:
                for d in pref.devices:
                    d.use = d.type == tipo
                cena.cycles.device = "GPU"
                print(f"[oraculo] dispositivo {tipo}: {[d.name for d in gpus]}")
                break
        except TypeError:
            continue
except Exception as e:  # noqa: BLE001 — o oráculo corre na CPU se a placa falhar
    print(f"[oraculo] CPU ({e})")

vl = cena.view_layers[0]
for p in (
    "use_pass_diffuse_direct",
    "use_pass_diffuse_indirect",
    "use_pass_diffuse_color",
    "use_pass_glossy_direct",
    "use_pass_glossy_indirect",
    "use_pass_glossy_color",
    "use_pass_ambient_occlusion",
    "use_pass_normal",
):
    setattr(vl, p, True)
cena.world = bpy.data.worlds.new("ceu")
cena.world.use_nodes = True
fundo = cena.world.node_tree.nodes["Background"]

sol_d = bpy.data.lights.new("sol", "SUN")
sol = bpy.data.objects.new("sol", sol_d)
cena.collection.objects.link(sol)
# x direita, y BAIXO na tela ⇒ y do Blender trocado; o sol emite ao longo do seu −Z.
para_luz = Vector((math.cos(alt) * math.cos(az), -math.cos(alt) * math.sin(az), math.sin(alt)))
sol.rotation_euler = para_luz.to_track_quat("Z", "Y").to_euler()
forca_sol = (1.0 - ceu) * math.pi / math.sin(alt)

# Os HDRI que o Blender instala (CC0, Poly Haven — `studiolights/world/license.txt`). Cada um roda-se para
# o ponto mais claro ficar no azimute da nossa luz e escala-se para a irradiância do plano = π (tinta lisa
# = o albedo, a mesma normalização do céu + sol).
DIR_HDRI = os.path.join(
    bpy.utils.system_resource("DATAFILES"), "studiolights", "world"
)
ceu_nos = cena.world.node_tree
amb = ceu_nos.nodes.new("ShaderNodeTexEnvironment")
mapa = ceu_nos.nodes.new("ShaderNodeMapping")
coord = ceu_nos.nodes.new("ShaderNodeTexCoord")
ceu_nos.links.new(coord.outputs["Generated"], mapa.inputs["Vector"])
ceu_nos.links.new(mapa.outputs["Vector"], amb.inputs["Vector"])


def hdri(nome_hdri):
    im = bpy.data.images.load(os.path.join(DIR_HDRI, f"{nome_hdri}.exr"))
    w, hh = im.size
    px = np.empty(w * hh * 4, dtype=np.float32)
    im.pixels.foreach_get(px)
    lum = (px.reshape(hh, w, 4)[..., :3] @ np.array([0.2126, 0.7152, 0.0722])).astype(np.float64)
    u = (np.arange(w) + 0.5) / w
    v = (np.arange(hh) + 0.5) / hh
    fi = (u - 0.5) * 2 * math.pi
    te = (v - 0.5) * math.pi
    z = np.sin(te)[:, None]
    dw = (2 * math.pi / w) * (math.pi / hh) * np.cos(te)[:, None]
    e = float((lum * np.maximum(z, 0) * dw).sum())
    # O azimute do lado mais claro do hemisfério de cima, pesado pelo cosseno: o «sol» do HDRI.
    peso = lum * np.maximum(z, 0) * dw
    # ⛔ W2 E2: a equirectangular do Cycles é `u = ½ − atan2(y, x)/2π` (kernel `direction_to_equirectangular`).
    # Até 27/10 este `claro` lia-se com `x = −cos fi, y = sin fi` (a moldura do `prefiltra_ambiente.py`, rodada
    # de 180°): o HDRI ficava com o lado claro OPOSTO à luz (regressão do difuso na normal: +37° × −135°).
    fi_b = math.pi - 2 * math.pi * u
    dx = (np.cos(fi_b)[None, :] * np.cos(te)[:, None] * peso).sum()
    dy = (np.sin(fi_b)[None, :] * np.cos(te)[:, None] * peso).sum()
    claro = math.atan2(dy, dx)
    alvo = math.atan2(para_luz.y, para_luz.x)
    print(f"[oraculo] {nome_hdri}: E={e:.3f} claro={math.degrees(claro):.0f}° alvo={math.degrees(alvo):.0f}°")
    return im, math.pi / e, claro - alvo


# Os termos separam-se pelo MATERIAL (o Blender 5.2 desta máquina não grava os passes do Cycles no EXR —
# medido 26/10): `cor` = a imagem; `cinza` = albedo 0,5 sem brilho (÷0,5 = o difuso relativo, comparável a
# `t_e`); `cinza_direto` = o mesmo com 0 ricochetes (a diferença é o REBOTE); `brilho` = base preta.
LUZES = {
    "ceu": (0.0, False, "uniforme"),
    "sol05": (0.5, True, "uniforme"),
    "sol10": (10.0, True, "uniforme"),
    "sol25": (25.0, True, "uniforme"),
    "hdri_studio": (0.0, False, "studio"),
    # A receita do PH2D: o ambiente leva `ceu` e a lâmpada o resto.
    "studio_sol10": (10.0, True, "studio"),
    "hdri_interior": (0.0, False, "interior"),
    "hdri_courtyard": (0.0, False, "courtyard"),
    # W2 E2: o difuso por sector também no interior e no pátio, puros e com a lâmpada.
    "interior_sol10": (10.0, True, "interior"),
    "courtyard_sol10": (10.0, True, "courtyard"),
}
MODOS_TODOS = ("cor", "cinza", "cinza_direto", "brilho")
TODOS_OS_MODOS = (
    "ceu", "sol10", "hdri_studio", "studio_sol10", "hdri_interior", "hdri_courtyard", "interior_sol10",
    "courtyard_sol10",
)
# `PH2D_CYCLES_SO="a,b"`: só essas luzes (uma rodada mais curta).
SO = [x for x in os.environ.get("PH2D_CYCLES_SO", "").split(",") if x]
VARIANTES = [
    (l, m)
    for l in LUZES
    if not SO or l in SO
    for m in (MODOS_TODOS if l in TODOS_OS_MODOS else ("cor", "cinza"))
]
for luz, modo in VARIANTES:
    ang, com_sol, ceu_v = LUZES[luz]
    nome = f"{luz}_{modo}"
    sol_d.angle = math.radians(ang)
    sol_d.energy = forca_sol if com_sol else 0.0
    for l in list(ceu_nos.links):
        if l.to_node == fundo and l.to_socket.name == "Color":
            ceu_nos.links.remove(l)
    fundo.inputs["Color"].default_value = (ceu, ceu, ceu, 1.0)
    fundo.inputs["Strength"].default_value = 1.0
    if ceu_v != "uniforme":
        amb.image, forca, rot = hdri(ceu_v)
        mapa.inputs["Rotation"].default_value = (0.0, 0.0, rot)
        fundo.inputs["Strength"].default_value = forca * (ceu if com_sol else 1.0)
        ceu_nos.links.new(amb.outputs["Color"], fundo.inputs["Color"])
    for l in list(mat.node_tree.links):
        if l.to_socket == bsdf.inputs["Base Color"]:
            mat.node_tree.links.remove(l)
    if modo == "cor":
        mat.node_tree.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    cinza = 0.0 if modo == "brilho" else 0.5
    bsdf.inputs["Base Color"].default_value = (cinza, cinza, cinza, 1.0)
    bsdf.inputs["Specular IOR Level"].default_value = 0.0 if modo.startswith("cinza") else 0.5
    cena.cycles.max_bounces = 0 if modo == "cinza_direto" else 12
    # Os números em linear, sem vista; o PNG para os olhos em AgX.
    cena.view_settings.view_transform = "Standard"
    cena.view_settings.look = "None"
    im = r.image_settings
    if hasattr(im, "media_type"):
        im.media_type = "IMAGE"
    im.file_format = "OPEN_EXR"
    im.color_depth = "32"
    r.filepath = os.path.join(saida, f"{nome}.exr")
    bpy.ops.render.render(write_still=True)
    print(f"[oraculo] {r.filepath}")
    if modo == "cor":
        cena.view_settings.view_transform = "AgX"
        im.file_format = "PNG"
        im.color_depth = "8"
        bpy.data.images["Render Result"].save_render(os.path.join(saida, f"{nome}_agx.png"), scene=cena)

# W2 E3-B: os MATERIAIS (`materiais_e3.py`, a mesma lista da sonda) — o Principled com o MESMO valor sobre a
# MESMA tinta. `PH2D_CYCLES_MAT="a,b"` (ou `todos`) liga-os; cada um grava `mat_<nome>_cor` e `_brilho`.
# A estria (`t_estria.f32`) vai à Tangent e a passagem da película (`t_passagem.f32`) pesa a Transmission
# (Thin Wall) sobre um PAPEL branco difuso logo abaixo da tinta.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import materiais_e3  # noqa: E402

MAT_SO = [x for x in os.environ.get("PH2D_CYCLES_MAT", "").split(",") if x]
if MAT_SO == ["todos"]:
    MAT_SO = list(materiais_e3.MATERIAIS)
if MAT_SO:
    nv = gw * gh
    est = np.fromfile(os.path.join(entrada, "t_estria.f32"), dtype="<f4").reshape(nv, 2).astype(np.float64)
    zero = np.hypot(est[:, 0], est[:, 1]) < 1e-6
    est[zero] = (1.0, 0.0)
    tang = np.stack([est[:, 0], -est[:, 1], np.zeros(nv)], axis=-1)  # y da tela para baixo
    at = malha.attributes.new("estria", "FLOAT_VECTOR", "POINT")
    at.data.foreach_set("vector", tang.astype(np.float32).ravel())
    pas = np.fromfile(os.path.join(entrada, "t_passagem.f32"), dtype="<f4")
    ap = malha.attributes.new("passagem", "FLOAT", "POINT")
    ap.data.foreach_set("value", pas.astype(np.float32))
    nt = mat.node_tree
    a_est = nt.nodes.new("ShaderNodeAttribute")
    a_est.attribute_name = "estria"
    a_pas = nt.nodes.new("ShaderNodeAttribute")
    a_pas.attribute_name = "passagem"
    peso_tr = nt.nodes.new("ShaderNodeMath")
    peso_tr.operation = "MULTIPLY"
    nt.links.new(a_pas.outputs["Fac"], peso_tr.inputs[0])
    tinta_esp = nt.nodes.new("ShaderNodeMix")
    tinta_esp.data_type = "RGBA"
    tinta_esp.inputs["A"].default_value = (1.0, 1.0, 1.0, 1.0)
    nt.links.new(tex.outputs["Color"], tinta_esp.inputs["B"])
    # O papel: um plano branco difuso logo abaixo do zero da tinta (só aparece pela película translúcida).
    bpy.ops.mesh.primitive_plane_add(size=1.0)
    papel = bpy.context.active_object
    papel.scale = (gw * 2, gh * 2, 1)
    papel.location = ((gw - 1) / 2, -(gh - 1) / 2, -0.02)
    mp = bpy.data.materials.new("papel")
    mp.use_nodes = True
    mp.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (1.0, 1.0, 1.0, 1.0)
    mp.node_tree.nodes["Principled BSDF"].inputs["Specular IOR Level"].default_value = 0.0
    mp.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 1.0
    papel.data.materials.append(mp)

    # O material é da TINTA: cada lóbulo pesa-se pela cobertura (o papel do `albedo` fica dieléctrico — sem
    # isto o metal1 punha o papel branco a espelho, e o reflexo das encostas via-o).
    tcob = nt.nodes.new("ShaderNodeTexImage")
    tcob.image = bpy.data.images.load(os.path.join(entrada, "cobertura.png"))
    tcob.image.colorspace_settings.name = "Non-Color"
    tcob.interpolation = "Closest"
    peso_tr_cob = nt.nodes.new("ShaderNodeMath")
    peso_tr_cob.operation = "MULTIPLY"
    nt.links.new(peso_tr.outputs["Value"], peso_tr_cob.inputs[0])
    nt.links.new(tcob.outputs["Color"], peso_tr_cob.inputs[1])
    fator_tinta = nt.nodes.new("ShaderNodeMath")
    fator_tinta.operation = "MULTIPLY"
    nt.links.new(tcob.outputs["Color"], fator_tinta.inputs[0])
    nt.links.new(fator_tinta.outputs["Value"], tinta_esp.inputs["Factor"])
    pesos = {}

    def pesa(entrada_bsdf, valor):
        if entrada_bsdf not in pesos:
            no = nt.nodes.new("ShaderNodeMath")
            no.operation = "MULTIPLY"
            nt.links.new(tcob.outputs["Color"], no.inputs[0])
            pesos[entrada_bsdf] = no
        pesos[entrada_bsdf].inputs[1].default_value = valor
        liga(pesos[entrada_bsdf].outputs["Value"] if valor > 0 else None, entrada_bsdf)
        if valor <= 0:
            bsdf.inputs[entrada_bsdf].default_value = 0.0

    def liga(saida_no, entrada_bsdf):
        for l in list(nt.links):
            if l.to_socket == bsdf.inputs[entrada_bsdf]:
                nt.links.remove(l)
        if saida_no is not None:
            nt.links.new(saida_no, bsdf.inputs[entrada_bsdf])

    for nome_m in MAT_SO:
        m = materiais_e3.MATERIAIS[nome_m]
        luz_m = materiais_e3.luz(nome_m)
        ang, com_sol, ceu_v = LUZES[luz_m]
        sol_d.angle = math.radians(ang)
        sol_d.energy = forca_sol if com_sol else 0.0
        for l in list(ceu_nos.links):
            if l.to_node == fundo and l.to_socket.name == "Color":
                ceu_nos.links.remove(l)
        fundo.inputs["Color"].default_value = (ceu, ceu, ceu, 1.0)
        fundo.inputs["Strength"].default_value = 1.0
        if ceu_v != "uniforme":
            amb.image, forca, rot = hdri(ceu_v)
            mapa.inputs["Rotation"].default_value = (0.0, 0.0, rot)
            fundo.inputs["Strength"].default_value = forca * (ceu if com_sol else 1.0)
            ceu_nos.links.new(amb.outputs["Color"], fundo.inputs["Color"])
        pesa("Metallic", m.get("metal", 0.0))
        bsdf.inputs["Roughness"].default_value = min(1.0, rug * m.get("rugosidade", 1.0))
        bsdf.inputs["Specular IOR Level"].default_value = 0.5 * m.get("especular", 1.0)
        fator_tinta.inputs[1].default_value = m.get("tinta_reflexo", 0.0)
        liga(tinta_esp.outputs["Result"] if m.get("tinta_reflexo", 0.0) > 0 else None, "Specular Tint")
        pesa("Coat Weight", m.get("verniz", 0.0))
        bsdf.inputs["Coat Roughness"].default_value = min(1.0, 0.03 * m.get("verniz_rugosidade", 1.0))
        pesa("Sheen Weight", m.get("veludo", 0.0))
        pesa("Anisotropic", m.get("anisotropia", 0.0))
        liga(a_est.outputs["Vector"] if m.get("anisotropia", 0.0) > 0 else None, "Tangent")
        peso_tr.inputs[1].default_value = m.get("translucidez", 0.0)
        liga(peso_tr_cob.outputs["Value"] if m.get("translucidez", 0.0) > 0 else None, "Transmission Weight")
        bsdf.inputs["Thin Wall"].default_value = m.get("translucidez", 0.0) > 0
        papel.hide_render = m.get("translucidez", 0.0) <= 0
        cena.cycles.max_bounces = 12
        for modo in ("cor", "brilho"):
            if modo == "cor":
                liga(tex.outputs["Color"], "Base Color")
            else:
                liga(None, "Base Color")
                bsdf.inputs["Base Color"].default_value = (0.0, 0.0, 0.0, 1.0)
            cena.view_settings.view_transform = "Standard"
            cena.view_settings.look = "None"
            im = r.image_settings
            if hasattr(im, "media_type"):
                im.media_type = "IMAGE"
            im.file_format = "OPEN_EXR"
            im.color_depth = "32"
            r.filepath = os.path.join(saida, f"mat_{nome_m}_{modo}.exr")
            bpy.ops.render.render(write_still=True)
            print(f"[oraculo] {r.filepath}")
            if modo == "cor":
                cena.view_settings.view_transform = "AgX"
                im.file_format = "PNG"
                im.color_depth = "8"
                bpy.data.images["Render Result"].save_render(
                    os.path.join(saida, f"mat_{nome_m}_cor_agx.png"), scene=cena
                )
print("[oraculo] fim")
