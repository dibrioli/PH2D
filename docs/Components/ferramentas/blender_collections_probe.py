# ORÁCULO: as coleções do Blender 5.2.1 (GPL — só a SAÍDA é usada; nenhum fonte lido),
# corridas sem interface — docs/Components/08_plano_tags.md §1.
#
# Correr:
#   blender -b --factory-startup --python docs/Components/ferramentas/blender_collections_probe.py
#
# Mede o que o Blender TENTOU e ABANDONOU (os `Groups` saíram na 2.80, trocados pelas coleções),
# e as três propriedades que o plano de Tags copia ou recusa:
#   1. pertença MÚLTIPLA (um objecto em N coleções);
#   2. contenção HIERÁRQUICA (o objecto só da coleção-filha aparece no `all_objects` do pai);
#   3. RENOMEAR mantém a pertença (a coleção é referência, não texto) — e um nome repetido
#      vira `nome.001`, que é o sufixo que um separador hierárquico por PONTO não pode ter.
import bpy

print("HAS_GROUPS", hasattr(bpy.data, "groups"), "HAS_COLLECTIONS", hasattr(bpy.data, "collections"))

parent = bpy.data.collections.new("enemy")
child = bpy.data.collections.new("flying")
parent.children.link(child)
bpy.context.scene.collection.children.link(parent)

both = bpy.data.objects.new("Goblin", None)
parent.objects.link(both)
child.objects.link(both)
print("MULTI users_collection", [c.name for c in both.users_collection])

child_only = bpy.data.objects.new("Bat", None)
child.objects.link(child_only)
print(
    "CHILD_ONLY parent.objects", [x.name for x in parent.objects],
    "parent.all_objects", sorted(x.name for x in parent.all_objects),
)

child.name = "winged"
print("RENAME users_collection", [c.name for c in child_only.users_collection])

dup = bpy.data.collections.new("enemy")
print("DUP_NAME", dup.name)

print("ASSET_TAG_TYPE", hasattr(bpy.types, "AssetTag"))
both.asset_mark()
both.asset_data.tags.new("Enemy")
both.asset_data.tags.new("Enemy")
print("ASSET_TAGS", [t.name for t in both.asset_data.tags])
