# ORÁCULO DA ORDEM DOS EFEITOS (F50, 2026-10-02) — corre o Blender 5.2 (Grease Pencil v3) sobre um
# traço NOSSO de 4 nós e uma corrente NOSSA de 3 ossos, e pergunta: ao prender com "With Automatic
# Weights", onde fica o modificador Armature na pilha em relação a um efeito que JÁ existia, e a
# um efeito acrescentado DEPOIS?
#
#   blender -b --factory-startup -P docs/Skeleton/oraculo/oraculo_ordem_dos_efeitos.py
#
# SAÍDA GRAVADA (2026-10-02, Blender 5.2.2):
#   PILHA depois de prender (efeito ja existia): [('Noise', 'GREASE_PENCIL_NOISE'), ('Armature', 'GREASE_PENCIL_ARMATURE')]
#   PILHA com efeito novo: [('Noise', 'GREASE_PENCIL_NOISE'), ('Armature', 'GREASE_PENCIL_ARMATURE'), ('Build', 'GREASE_PENCIL_BUILD')]
# ⇒ o efeito que existe ao prender corre ANTES da pele (em repouso) e dobra com o traço; o
#   acrescentado depois corre sobre o traço dobrado. A lei da F50 é a primeira, para toda a pilha.
import bpy

bpy.ops.wm.read_factory_settings(use_empty=True)
arm_d = bpy.data.armatures.new("Arm")
arm = bpy.data.objects.new("Arm", arm_d)
bpy.context.scene.collection.objects.link(arm)
bpy.context.view_layer.objects.active = arm
bpy.ops.object.mode_set(mode='EDIT')
prev = None
for k in range(3):
    b = arm_d.edit_bones.new(f"b{k}")
    b.head = (2 * k, 0, 0)
    b.tail = (2 * k + 2, 0, 0)
    if prev:
        b.parent = prev
        b.use_connect = True
    prev = b
bpy.ops.object.mode_set(mode='OBJECT')
gp = bpy.data.grease_pencils.new("GP")
ob = bpy.data.objects.new("GP", gp)
bpy.context.scene.collection.objects.link(ob)
d = gp.layers.new("L").frames.new(1).drawing
d.add_strokes([4])
for i, p in enumerate(d.strokes[0].points):
    p.position = (2 * i, 0, 0)
ob.modifiers.new("Noise", type='GREASE_PENCIL_NOISE')
bpy.ops.object.select_all(action='DESELECT')
ob.select_set(True)
arm.select_set(True)
bpy.context.view_layer.objects.active = arm
bpy.ops.object.parent_set(type='ARMATURE_AUTO')
print("PILHA depois de prender (efeito ja existia):", [(x.name, x.type) for x in ob.modifiers])
ob.modifiers.new("Build", type='GREASE_PENCIL_BUILD')
print("PILHA com efeito novo:", [(x.name, x.type) for x in ob.modifiers])
