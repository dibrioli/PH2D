#!/usr/bin/env python3
"""Grava a fixture do ORÁCULO do material: o OpenPBR do MaterialX renderizado SEM interface.

    python3 docs/Render3d/ferramentas/fixture_openpbr.py \
        > crates/ph2d-material/fixtures/materialx_openpbr_direct.txt

# O que corre (§0.9 do CLAUDE.md — corre-se, não se lê)

O `GlslRenderer` do MaterialX 1.39.5 (Apache-2.0) desenha a `sphere.obj` dele com o
`open_pbr_surface` gerado pelo `GlslShaderGenerator`, sob UMA luz direcional, e captura em `float`.
Por cada material e cada luz saem TRÊS passadas da mesma geometria:

1. a **normal** que o shader usou (`normal`, mundo, escrita como emissão sem luz);
2. a **posição** do pixel (`position`, mundo) — ⚠️ o shader tira a direcção de vista de
   `u_viewPosition − P` MESMO com câmera ortográfica, logo `V` não é `(0, 0, 1)`: a fixture guarda o
   `V` do próprio pixel;
3. a **cor** (`open_pbr_surface`).

⚠️ **O «sem céu» que compila é `FIS` com a luz INDIRECTA desligada** — `NONE` gera um shader que
usa `u_refractionTwoSided` sem o declarar (1.39.5). O céu tem gate próprio (o *furnace test*), não
este.

# Formato

    M <id> <nome>=<valor> …          os parâmetros do material (os não escritos têm o valor da nodedef)
    L <id> dx dy dz  r g b  intensidade   a luz: a direcção em que a luz VIAJA, como no MaterialX
    S <m> <l>  nx ny nz  vx vy vz  r g b  uma amostra: normal e vista unitárias, e a cor linear
"""

import hashlib
import math
import sys

import MaterialX as mx
from MaterialX import PyMaterialXGenGlsl as gg
from MaterialX import PyMaterialXGenShader as gs
from MaterialX import PyMaterialXRender as mr
from MaterialX import PyMaterialXRenderGlsl as rg

SIZE = 48
STRIDE = 6
EYE_Z = 5.0

# Cada material exercita um termo do subconjunto da primeira fatia (docs/Render3d/05).
MATERIALS = [
    {},
    {"base_color": (0.8, 0.1, 0.05), "specular_roughness": 0.6, "base_diffuse_roughness": 0.5},
    {"base_metalness": 1.0, "base_color": (1.0, 0.766, 0.336), "specular_color": (1.0, 0.9, 0.7),
     "specular_roughness": 0.2},
    {"base_color": (0.05, 0.2, 0.8), "specular_roughness": 0.5, "coat_weight": 1.0,
     "coat_roughness": 0.05},
    {"base_metalness": 0.5, "specular_weight": 0.5, "specular_ior": 2.0, "specular_roughness": 0.45,
     "coat_weight": 0.5, "coat_color": (0.9, 0.8, 0.6), "coat_ior": 1.4, "coat_darkening": 1.0},
    {"base_weight": 0.5, "emission_luminance": 2.0, "emission_color": (0.2, 0.6, 1.0),
     "coat_weight": 0.3},
    {"base_metalness": 1.0, "base_color": (0.95, 0.95, 0.95), "specular_roughness": 0.02},
    # ⭐ A SUBSUPERFÍCIE (docs/Render3d/10) — os DOIS caminhos, que são fenómenos diferentes.
    # Parede fina: a folha. O que a acende é a luz de TRÁS (a `L 3`).
    {"subsurface_weight": 1.0, "geometry_thin_walled": True,
     "subsurface_color": (0.35, 0.75, 0.2), "base_color": (0.2, 0.5, 0.1),
     "specular_roughness": 0.5},
    # Parede fina com a fase deslocada para a FRENTE: mais transmissão, menos reflexão.
    {"subsurface_weight": 0.8, "geometry_thin_walled": True,
     "subsurface_color": (0.9, 0.7, 0.6), "subsurface_scatter_anisotropy": 0.6,
     "base_diffuse_roughness": 0.4},
    # Maciça: o jade. O caminho livre médio curto contra o raio da esfera.
    {"subsurface_weight": 1.0, "subsurface_color": (0.3, 0.8, 0.5),
     "subsurface_radius": 0.5, "specular_roughness": 0.15},
    # Maciça com o vermelho a viajar MUITO mais fundo — é o que dá a orelha acesa contra o sol.
    {"subsurface_weight": 0.7, "subsurface_color": (0.95, 0.85, 0.75),
     "subsurface_radius": 2.0, "subsurface_radius_scale": (1.0, 0.4, 0.15),
     "subsurface_scatter_anisotropy": -0.4},
]


def unit(v):
    n = math.sqrt(sum(c * c for c in v))
    return tuple(c / n for c in v)


LIGHTS = [
    ((0.0, 0.0, -1.0), (1.0, 1.0, 1.0), 1.0),
    (unit((-0.5, -0.6, -0.62)), (1.0, 0.9, 0.8), 2.5),
    (unit((1.0, 0.05, -0.2)), (1.0, 1.0, 1.0), 1.5),
    # ⭐⭐⭐ A luz de TRÁS — ela VIAJA para +z, logo vem de detrás da esfera, contra a câmera.
    # ⚠️ Sem ela a fixture não contém o fenómeno que a parede fina existe para produzir: nas três
    # luzes acima a transmissão lê `max(−N·L, 0) = 0` em quase todo pixel visível, e um corpus sem o
    # fenómeno não afirma nada sobre a lei que o produz.
    ((0.0, 0.0, 1.0), (1.0, 0.95, 0.85), 2.0),
]

sp = mx.FileSearchPath("/usr/share/materialx")
lib = mx.createDocument()
mx.loadLibraries(mx.getDefaultDataLibraryFolders(), sp, lib)


def value(v):
    # ⚠️ O `bool` PRIMEIRO: em Python ele é subtipo de `int`, e `float(True)` poria um `1.0` num
    # input que a nodedef declara `boolean` (`geometry_thin_walled`).
    if isinstance(v, bool):
        return v
    if isinstance(v, tuple):
        return mx.Color3(*v)
    return float(v)


def literal(v):
    """Como o valor se escreve na linha `M` — o `true`/`false` é o da linha `D` da nodedef."""
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, tuple):
        return ",".join(f"{c:g}" for c in v)
    return f"{v:g}"


def material(kind, params, _alive=[]):  # noqa: B006 — a lista É o ponto, ver abaixo
    doc = mx.createDocument()
    doc.importLibrary(lib)
    if kind == "opbr":
        sh = doc.addNode("open_pbr_surface", "SR", "surfaceshader")
        for k, v in params.items():
            sh.setInputValue(k, value(v))
    else:
        g = doc.addNode(kind, "G", "vector3")
        g.setInputValue("space", "world")
        c = doc.addNode("convert", "C", "color3")
        c.addInput("in", "vector3").setNodeName("G")
        sh = doc.addNode("surface_unlit", "SR", "surfaceshader")
        sh.addInput("emission_color", "color3").setNodeName("C")
    mat = doc.addMaterialNode("M", sh)
    ok, msg = doc.validate()
    assert ok, msg
    # ⚠️ O DOCUMENTO tem de ficar vivo: a ligação Python guarda-o por referência FRACA a partir do
    # elemento, e um material cujo documento o GC já levou falha ao gerar com «Requested root of
    # orphaned element» (medido — foi a 1.ª corrida deste script).
    _alive.append(doc)
    return mat


gen = gg.GlslShaderGenerator.create()
ctx = gs.GenContext(gen)
ctx.registerSourceCodeSearchPath(sp)
opt = ctx.getOptions()
opt.hwSpecularEnvironmentMethod = gs.HwSpecularEnvironmentMethod.SPECULAR_ENVIRONMENT_FIS
opt.hwMaxActiveLightSources = 1
opt.hwShadowMap = False
opt.hwAmbientOcclusion = False
opt.hwSrgbEncodeOutput = False

ldoc = mx.createDocument()
ldoc.importLibrary(lib)
light_nodes = []
for i, (d, c, k) in enumerate(LIGHTS):
    n = ldoc.addNode("directional_light", f"light{i}", "lightshader")
    n.setInputValue("direction", mx.Vector3(*d))
    n.setInputValue("color", mx.Color3(*c))
    n.setInputValue("intensity", float(k))
    light_nodes.append(n)
lh = mr.LightHandler.create()
lh.registerLights(ldoc, light_nodes, ctx)
lh.setDirectLighting(True)
lh.setIndirectLighting(False)

R = rg.GlslRenderer.create(SIZE, SIZE, mr.BaseType.FLOAT)
R.initialize()
R.setImageHandler(rg.GLTextureHandler.create(mr.StbImageLoader.create()))
R.setLightHandler(lh)
geo = mr.GeometryHandler.create()
geo.addLoader(mr.TinyObjLoader.create())
obj = "/usr/share/materialx/resources/Geometry/sphere.obj"
assert geo.loadGeometry(mx.FilePath(obj), False)
R.setGeometryHandler(geo)
cam = R.getCamera()
cam.setWorldMatrix(mx.Matrix44.IDENTITY)
cam.setViewMatrix(mr.Camera.createViewMatrix(mx.Vector3(0, 0, EYE_Z), mx.Vector3(0, 0, 0), mx.Vector3(0, 1, 0)))
cam.setProjectionMatrix(mr.Camera.createOrthographicMatrix(-1.05, 1.05, -1.05, 1.05, 0.1, 10.0))


def capture(mat):
    R.createProgram(gen.generate("s", mat, ctx))
    R.render()
    return R.captureImage(None)


def texel(img, x, y):
    c = img.getTexelColor(x, y)
    return (c[0], c[1], c[2])


def sha256(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


lh.setLightSources([light_nodes[0]])
normals = capture(material("normal", {}))
positions = capture(material("position", {}))

samples = []
for y in range(0, SIZE, STRIDE):
    for x in range(0, SIZE, STRIDE):
        n = texel(normals, x, y)
        length = math.sqrt(sum(c * c for c in n))
        if abs(length - 1.0) > 1e-3:
            continue  # fundo: a cor de limpeza, não uma normal
        p = texel(positions, x, y)
        v = unit((-p[0], -p[1], EYE_Z - p[2]))
        samples.append((x, y, unit(n), v))
assert len(samples) >= 30, f"amostras de esfera a mais poucas: {len(samples)}"

print("# ORÁCULO: o `open_pbr_surface` do MaterialX renderizado sem interface pelo `GlslRenderer`")
print(f"# MaterialX {mx.getVersionString()} (Apache-2.0) · sphere.obj sha256 {sha256(obj)}")
print(f"# {SIZE}x{SIZE} ortográfica, olho em z={EYE_Z:g}, uma amostra a cada {STRIDE} px, "
      f"{len(samples)} pixels de esfera por imagem · luz directa só (FIS, indirecta desligada)")
print("# gerado por docs/Render3d/ferramentas/fixture_openpbr.py · formato no docstring do script")
# ⭐ Os VALORES PADRÃO da nodedef, lidos do padrão instalado — é contra ESTA linha que o `Default` da
# crate se prova, e não contra números copiados à mão para o código.
nodedef = lib.getNodeDef("ND_open_pbr_surface_surfaceshader")
print("D " + " ".join(f"{i.getName()}={i.getValueString().replace(' ', '')}"
                      for i in nodedef.getInputs() if i.getValueString()))
for i, m in enumerate(MATERIALS):
    fields = " ".join(f"{k}={literal(v)}" for k, v in m.items())
    print(f"M {i} {fields}".rstrip())
for i, (d, c, k) in enumerate(LIGHTS):
    print(f"L {i} " + " ".join(f"{t:.9e}" for t in d) + "  " + " ".join(f"{t:g}" for t in c) + f"  {k:g}")
for mi, m in enumerate(MATERIALS):
    mat = material("opbr", m)
    for li in range(len(LIGHTS)):
        lh.setLightSources([light_nodes[li]])
        img = capture(mat)
        for x, y, n, v in samples:
            c = texel(img, x, y)
            print(f"S {mi} {li}  " + " ".join(f"{t:.9e}" for t in n) + "  "
                  + " ".join(f"{t:.9e}" for t in v) + "  " + " ".join(f"{t:.9e}" for t in c))

# ── A LUZ INDIRECTA, contra um céu CONSTANTE ────────────────────────────────────────────────────
#
# ⭐ Com `PREFILTER` e um mapa de ambiente UNIFORME, a consulta pré-filtrada devolve a mesma cor a
# qualquer rugosidade e em qualquer direcção — logo o oráculo prova a LEI indirecta (a radiância na
# direcção espelhada × o albedo direcional, a irradiância × o albedo da difusa) sem nenhuma
# aproximação do lado dele. O céu analítico da casa tem gate próprio.
#
#   E r g b                                   a radiância (e a irradiância normalizada) do céu
#   I <m>  nx ny nz  vx vy vz  r g b          uma amostra: só luz indirecta (e a emissão)
ENV = (0.5, 0.6, 0.7)
env = mr.Image.create(32, 16, 3, mr.BaseType.FLOAT)
env.createResourceBuffer()
env.setUniformColor(mx.Color4(ENV[0], ENV[1], ENV[2], 1.0))
lh.setEnvRadianceMap(env)
lh.setEnvIrradianceMap(env)
lh.setDirectLighting(False)
lh.setIndirectLighting(True)
opt.hwSpecularEnvironmentMethod = gs.HwSpecularEnvironmentMethod.SPECULAR_ENVIRONMENT_PREFILTER
print("# luz indirecta: PREFILTER, luz directa desligada, mapa de ambiente uniforme · E r g b · "
      "I <m> n v cor")
print("E " + " ".join(f"{t:g}" for t in ENV))
for mi, m in enumerate(MATERIALS):
    img = capture(material("opbr", m))
    for x, y, n, v in samples:
        c = texel(img, x, y)
        print(f"I {mi}  " + " ".join(f"{t:.9e}" for t in n) + "  "
              + " ".join(f"{t:.9e}" for t in v) + "  " + " ".join(f"{t:.9e}" for t in c))
sys.stdout.flush()
