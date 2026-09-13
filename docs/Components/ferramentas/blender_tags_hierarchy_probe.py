# ORÁCULO v2: a HIERARQUIA das coleções do Blender sobre a fixture NOSSA do plano de Tags
# (docs/Components/08_plano_tags.md §5.1). Blender é GPL — só a SAÍDA é usada; nenhum fonte lido
# (CLAUDE.md §0.9: posições e pertenças de uma cena NOSSA não são obra baseada no programa).
#
# Correr (a saída vai, com cabeçalho, para a fixture do gate `the_hierarchy_is_the_one_blender_measures`):
#   blender -b --factory-startup --python docs/Components/ferramentas/blender_tags_hierarchy_probe.py
#
# Versão 2 (2026-09-13). A v1 (`blender_collections_probe.py`) mediu as PROPRIEDADES (pertença
# múltipla · contenção · renomear); esta mede a RESPOSTA sobre a cena exacta que o gate reconstrói
# do nosso lado, em quatro estágios, para o gate comparar conjunto a conjunto:
#   BASE   — Enemy › Flying › Boss, as raízes irmãs Statue e Player, e os sete objectos;
#   RENAME — Enemy passa a Monster: a pertença fica (a coleção é referência);
#   MOVE   — Boss sai de Flying para a raiz: o Dragon deixa de estar no `all_objects` de Monster;
#   CYCLE  — pôr Monster dentro de Boss é permitido (Boss já não descende dele); pôr Flying dentro
#            de Boss depois de Boss voltar para dentro de Flying é o ciclo, e o Blender recusa-o.
# E a linha CASE mede a divergência DECLARADA (D2): o Blender distingue as grafias.
#
# Formato de cada linha de pertença: `ALL <estágio> <caminho> = <nomes ordenados, vírgula>`.
import bpy

print("BLENDER", bpy.app.version_string)

scene_root = bpy.context.scene.collection


def new_collection(name, parent):
    c = bpy.data.collections.new(name)
    parent.children.link(c)
    return c


def new_object(name, collection):
    o = bpy.data.objects.new(name, None)
    collection.objects.link(o)
    return o


enemy = new_collection("Enemy", scene_root)
flying = new_collection("Flying", enemy)
boss = new_collection("Boss", flying)
statue = new_collection("Statue", scene_root)
player = new_collection("Player", scene_root)

for name in ("Goblin A", "Goblin B"):
    new_object(name, enemy)
for name in ("Bat A", "Bat B"):
    new_object(name, flying)
new_object("Dragon", boss)
new_object("Statue", statue)
new_object("Hero", player)

MINE = [enemy, flying, boss, statue, player]


def path_of(c):
    parts = [c.name]
    cur = c
    while True:
        parents = [p for p in bpy.data.collections if cur.name in p.children]
        if not parents:
            break
        # A cena é uma árvore nesta sonda até ao estágio CYCLE; a asserção diz-o em voz alta.
        assert len(parents) == 1, (cur.name, [p.name for p in parents])
        cur = parents[0]
        parts.append(cur.name)
    return "/".join(reversed(parts))


def dump(stage):
    for c in sorted(MINE, key=path_of):
        names = sorted(o.name for o in c.all_objects)
        print("ALL", stage, path_of(c), "=", ",".join(names))


dump("BASE")

enemy.name = "Monster"
dump("RENAME")

flying.children.unlink(boss)
scene_root.children.link(boss)
dump("MOVE")

# Monster dentro de Boss: Boss já não descende de Monster, logo não há ciclo.
try:
    boss.children.link(enemy)
    print("CYCLE monster_into_boss ok")
except RuntimeError as err:
    print("CYCLE monster_into_boss refused", str(err).strip().replace("\n", " "))
boss.children.unlink(enemy)

# Flying dentro de Boss, com Boss de volta DENTRO de Flying: o ciclo.
scene_root.children.unlink(boss)
flying.children.link(boss)
try:
    boss.children.link(flying)
    print("CYCLE flying_into_its_own_child ok")
except RuntimeError as err:
    print("CYCLE flying_into_its_own_child refused", str(err).strip().replace("\n", " "))

a = bpy.data.collections.new("Inimigo")
b = bpy.data.collections.new("inimigo")
c = bpy.data.collections.new("inímigo")
print("CASE", a.name, "|", b.name, "|", c.name)
