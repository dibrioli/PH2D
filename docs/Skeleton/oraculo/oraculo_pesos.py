# ORÁCULO DE PESOS — corre o Blender sobre a NOSSA malha e o NOSSO esqueleto, e despeja os pesos
# que o "With Automatic Weights" (difusão de calor) atribui. A entrada é nossa; a saída é livre.
import bpy, sys, json

ARCO, MEIA_ALT, N = 4.0, 2.4, 48
OUT = sys.argv[-1]

bpy.ops.wm.read_factory_settings(use_empty=True)

# --- A NOSSA arte: uma grelha regular sobre a caixa da fixtura da régua da dobra.
verts, faces = [], []
x0, x1 = -0.1 * ARCO, 1.1 * ARCO
for j in range(N + 1):
    for i in range(N + 1):
        verts.append((x0 + (x1 - x0) * i / N, -MEIA_ALT + 2 * MEIA_ALT * j / N, 0.0))
for j in range(N):
    for i in range(N):
        a = j * (N + 1) + i
        faces.append((a, a + 1, a + N + 2, a + N + 1))
me = bpy.data.meshes.new("arte")
me.from_pydata(verts, [], faces)
me.update()
ob = bpy.data.objects.new("arte", me)
bpy.context.collection.objects.link(ob)

# --- O NOSSO esqueleto: dois ossos de comprimento ARCO/2 ao longo de +x.
arm = bpy.data.armatures.new("rig")
rig = bpy.data.objects.new("rig", arm)
bpy.context.collection.objects.link(rig)
bpy.context.view_layer.objects.active = rig
bpy.ops.object.mode_set(mode='EDIT')
l = ARCO / 2.0
b0 = arm.edit_bones.new("b0"); b0.head = (0, 0, 0); b0.tail = (l, 0, 0)
b1 = arm.edit_bones.new("b1"); b1.head = (l, 0, 0); b1.tail = (2 * l, 0, 0); b1.parent = b0
bpy.ops.object.mode_set(mode='OBJECT')

# --- Parent with Automatic Weights (a difusão de calor).
bpy.ops.object.select_all(action='DESELECT')
ob.select_set(True)
rig.select_set(True)
bpy.context.view_layer.objects.active = rig
bpy.ops.object.parent_set(type='ARMATURE_AUTO')

nomes = [g.name for g in ob.vertex_groups]
pesos = []
for v in me.vertices:
    linha = {}
    for g in v.groups:
        linha[ob.vertex_groups[g.group].name] = g.weight
    pesos.append([linha.get("b0", 0.0), linha.get("b1", 0.0)])

json.dump({
    "arco": ARCO, "meia_alt": MEIA_ALT, "n": N, "grupos": nomes,
    "verts": [[v[0], v[1]] for v in verts],
    "faces": faces,
    "pesos": pesos,
}, open(OUT, "w"))
print(f"[oraculo] {len(verts)} vertices, grupos={nomes}")
