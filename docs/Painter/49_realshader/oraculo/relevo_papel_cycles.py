"""O ORÁCULO do Relevo do papel (cura 02/11c): a régua do gate `a_transmissao_e_o_relevo_do_papel_mostram_o_grao`
(o dente na tinta grossa ÷ no papel nu, absoluto) medida no Cycles sobre a MESMA cena. Relevo do papel 1 = a
superfície da tinta assenta no dente do papel: a malha é `h_fina + dente_papel` em TODA a grelha (a tinta de
espessura igual sobre o papel rugoso; o papel nu leva o mesmo dente) contra `h_fina` (sem dente). Material por
cobertura: a tinta = o Principled do `cycles.py` (rugosidade da cena, Specular 0,5); o papel = difuso (Specular 0,
Roughness 1). Luz `studio_sol10` (a de fábrica, a dos `materiais_e3`).

    PH2D_OLEO_PNG=<d> bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib \
        medida_relevo_do_papel_oraculo -- --ignored --nocapture
    python3 relevo_papel_cycles.py prepara <d>
    PH2D_CYCLES_SO=nenhuma bash scripts/ph2d-run.sh blender -b --factory-startup -P relevo_papel_cycles.py -- \
        <d>/entrada <d>/saida h_fina 256
    python3 relevo_papel_cycles.py regua <d>
"""

import os
import sys

try:
    import bpy  # noqa: F401
except ImportError:
    bpy = None

if bpy is None:
    import numpy as np
    from PIL import Image

    def pnm(c):
        b = open(c, "rb").read()
        p = b.split(maxsplit=4)
        w, h = int(p[1]), int(p[2])
        return np.frombuffer(p[4], dtype=np.uint8).reshape(h, w, -1).squeeze().astype(np.float64)

    modo, d = sys.argv[1], sys.argv[2]
    e = os.path.join(d, "entrada")
    if modo == "prepara":
        Image.fromarray(pnm(f"{e}/albedo.ppm").astype(np.uint8)).save(f"{e}/albedo.png")
        Image.fromarray(pnm(f"{e}/cobertura.pgm").astype(np.uint8)).save(f"{e}/cobertura.png")
        sys.exit(0)
    import OpenImageIO as oiio

    def srgb8(v):
        v = np.clip(v, 0.0, 1.0)
        s = np.where(v <= 0.0031308, 12.92 * v, 1.055 * v ** (1 / 2.4) - 0.055)
        return np.floor(s * 255.0 + 0.5)

    def lum8(f):
        c = srgb8(oiio.ImageBuf(f).get_pixels(oiio.FLOAT)[..., :3].astype(np.float64))
        return c @ np.array([0.2126, 0.7152, 0.0722])

    # A régua do gate (`dente_na_tela`): o desvio-padrão da diferença de luminância sRGB 8 bits nas amostras.
    def dente(a, b, m):
        x = (a - b)[m]
        return float(np.sqrt(((x - x.mean()) ** 2).mean()))

    miolo, nu = pnm(f"{e}/miolo.pgm") > 0, pnm(f"{e}/nu.pgm") > 0
    sai = os.path.join(d, "saida")
    fs = [f[len("relevo_"):-len(".exr")].split("_") for f in os.listdir(sai) if f.startswith("relevo_")]
    for amostras in sorted({a for _, a in fs}, key=int):
        l0 = lum8(f"{sai}/relevo_0_{amostras}.exr")
        for k in sorted({float(k) for k, a in fs if a == amostras and float(k) > 0}):
            l1 = lum8(f"{sai}/relevo_{k:g}_{amostras}.exr")
            dm, dn = dente(l1, l0, miolo), dente(l1, l0, nu)
            print(f"Cycles {amostras} amostras, dente × {k:g}: miolo {dm:.3f} · nu {dn:.3f} · miolo ÷ nu {dm / dn:.3f}")
    sys.exit(0)

# --- dentro do Blender: a cena do `cycles.py` (sem renders), depois as duas malhas. ---
_aq = os.path.join(os.path.dirname(os.path.abspath(__file__)), "cycles.py")
os.environ["PH2D_CYCLES_SO"] = "nenhuma"
exec(compile(open(_aq).read(), _aq, "exec"))  # noqa: S102 — o mesmo cenário, câmara, luz e normalização

dente_papel = np.fromfile(os.path.join(entrada, "dente_papel.f32"), dtype="<f4").reshape(gh, gw)  # noqa: F821
nt = mat.node_tree  # noqa: F821
tcob = nt.nodes.new("ShaderNodeTexImage")
tcob.image = bpy.data.images.load(os.path.join(entrada, "cobertura.png"))  # noqa: F821
tcob.image.colorspace_settings.name = "Non-Color"
tcob.interpolation = "Closest"
esp = nt.nodes.new("ShaderNodeMath")
esp.operation = "MULTIPLY"
esp.inputs[1].default_value = 0.5
nt.links.new(tcob.outputs["Color"], esp.inputs[0])
nt.links.new(esp.outputs["Value"], bsdf.inputs["Specular IOR Level"])  # noqa: F821
rg = nt.nodes.new("ShaderNodeMapRange")
rg.inputs["To Min"].default_value = 1.0
rg.inputs["To Max"].default_value = rug  # noqa: F821
nt.links.new(tcob.outputs["Color"], rg.inputs["Value"])
nt.links.new(rg.outputs["Result"], bsdf.inputs["Roughness"])  # noqa: F821
for l in list(nt.links):
    if l.to_socket == bsdf.inputs["Base Color"]:  # noqa: F821
        nt.links.remove(l)
nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])  # noqa: F821

ang, com_sol, ceu_v = LUZES["studio_sol10"]  # noqa: F821
sol_d.angle = math.radians(ang)  # noqa: F821
sol_d.energy = forca_sol  # noqa: F821
amb.image, forca, rot = hdri(ceu_v)  # noqa: F821
mapa.inputs["Rotation"].default_value = (0.0, 0.0, rot)  # noqa: F821
fundo.inputs["Color"].default_value = (ceu, ceu, ceu, 1.0)  # noqa: F821
fundo.inputs["Strength"].default_value = forca * ceu  # noqa: F821
ceu_nos.links.new(amb.outputs["Color"], fundo.inputs["Color"])  # noqa: F821
cena.cycles.max_bounces = 12  # noqa: F821
cena.view_settings.view_transform = "Standard"  # noqa: F821
cena.view_settings.look = "None"  # noqa: F821
im = r.image_settings  # noqa: F821
if hasattr(im, "media_type"):
    im.media_type = "IMAGE"
im.file_format = "OPEN_EXR"
im.color_depth = "32"
# `PH2D_RELEVO_K="0,1,4"`: o dente × k (a amplitude do papel nu; 0 = sem dente, a régua).
for k in [float(x) for x in os.environ.get("PH2D_RELEVO_K", "0,1").split(",")]:
    z = (h + k * dente_papel).astype(np.float64).ravel()  # noqa: F821
    co = np.stack([ii.ravel(), -jj.ravel(), z], axis=-1).astype(np.float32)  # noqa: F821
    malha.vertices.foreach_set("co", co.ravel())  # noqa: F821
    malha.update()  # noqa: F821
    r.filepath = os.path.join(saida, f"relevo_{k:g}_{amostras}.exr")  # noqa: F821
    bpy.ops.render.render(write_still=True)
    print(f"[oraculo] {r.filepath}")
print("[oraculo] relevo fim")
