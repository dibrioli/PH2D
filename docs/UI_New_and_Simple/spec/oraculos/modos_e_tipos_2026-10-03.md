# Oráculo — lista de TIPOS criáveis e MODOS / editores contextuais por tipo · Blender e Godot

> **Fixture medida, não lida.** Data: **2026-10-03**. Os dois apps foram CORRIDOS por script (nunca
> lido o fonte); HOME/XDG isolados em `/tmp/claude-1000/oraculo-modos/`. Pergunta: como cada app
> organiza (1) a lista de tipos de objecto/nó que o utilizador cria e (2) os modos de edição /
> editores contextuais por tipo? Modo explícito por objecto (Blender) × contexto automático pelo
> tipo seleccionado (Godot)?

## Triagem de licença (por artefacto instalado, `LANG=C pacman -Qi blender godot`)

| Pacote | Versão | Licenças | Uso aqui |
|---|---|---|---|
| `blender` | 17:5.2.2-2 (`Blender 5.2.2 LTS`, hash d13f752e3b9c) | Apache-2.0 BSD-2/3 GPL-2.0+ GPL-3.0+ LGPL-2.1+ MIT MPL-2.0 Zlib | programa GPL ⇒ **só oráculo corrido** |
| `godot` | 4.7.2-1.1 (`4.7.2-stable (arch_linux)`) | MIT | porta permissiva; aqui só corrido |

## Comandos exactos

```bash
D=/tmp/claude-1000/oraculo-modos
# Blender, headless (a, b, c, d, e-parte-API)
HOME=$D/home timeout 60  blender -b --factory-startup --python enums.py
HOME=$D/home timeout 120 blender -b --factory-startup --python matriz.py -- $D/matriz_bg.json
HOME=$D/home timeout 60  blender -b --factory-startup --python menus.py
HOME=$D/home timeout 60  blender -b --factory-startup --python menudraw.py
HOME=$D/home timeout 60  blender -b --factory-startup --python comport.py
# Blender COM janela (e-parte-clique), só dentro de kwin aninhado invisível; o roteiro de sessão
# recusa se WAYLAND_DISPLAY=wayland-0 ou DISPLAY=:0 (dentro: WAYLAND_DISPLAY=ph2d-oraculo-modos DISPLAY=:1)
XDG_CONFIG_HOME=$D/kwincfg timeout 120 kwin_wayland --virtual --width 1600 --height 1000 --xwayland \
  --socket ph2d-oraculo-modos --exit-with-session $D/sessao_blender.sh
#   sessao_blender.sh: TMPDIR=$D/tmp HOME=$D/home XDG_CONFIG_HOME=$D/home/.config \
#                      timeout 90 blender --factory-startup --python $D/janela.py
# Godot
HOME=$D/ghome XDG_CONFIG_HOME=… XDG_DATA_HOME=… timeout 60  godot --headless --script classdb.gd
HOME=$D/ghome XDG_CONFIG_HOME=… XDG_DATA_HOME=… XDG_CACHE_HOME=… \
  timeout 180 godot --headless --editor --path $D/gproj     # EditorPlugin @tool "sonda" activo
```

⚠️ A 1.ª corrida com janela do Blender gravou `/tmp/quit.blend` (sessão de recuperação ao sair) — fora
do HOME isolado; apagado, e as corridas seguintes levam `TMPDIR=$D/tmp`.
⚠️ O editor do Godot apanhou o locale da máquina (pt-BR): textos de botão abaixo saem traduzidos.

## BLENDER

### (a) Enum `Object.type` — 16 tipos
`MESH` Mesh · `CURVE` Curve · `SURFACE` Surface · `META` Metaball · `FONT` Text · `CURVES` Hair Curves ·
`POINTCLOUD` Point Cloud · `VOLUME` Volume · `GREASEPENCIL` Grease Pencil · `ARMATURE` Armature ·
`LATTICE` Lattice · `EMPTY` Empty · `LIGHT` Light · `LIGHT_PROBE` Light Probe · `CAMERA` Camera ·
`SPEAKER` Speaker. (Não há `GPENCIL` legado no enum de 5.2. «Image empty» = `EMPTY` com
`empty_display_type='IMAGE'`.)

### (b) Enum `Object.mode` — 14 modos
`OBJECT` · `EDIT` · `POSE` · `SCULPT` · `VERTEX_PAINT` · `WEIGHT_PAINT` · `TEXTURE_PAINT` ·
`PARTICLE_EDIT` · `EDIT_GPENCIL` · `SCULPT_GREASE_PENCIL` · `PAINT_GREASE_PENCIL` (nome «Draw Mode») ·
`WEIGHT_GREASE_PENCIL` · `VERTEX_GREASE_PENCIL` · `SCULPT_CURVES`.

### (c) Matriz tipo × modo (`bpy.ops.object.mode_set(mode=X)` em headless)

`Y` = entrou (e `obj.mode == X` depois); `-` = recusado. `mode_set.poll()` foi **True em todas as
células** (o poll só pede objecto activo). A recusa NÃO é do poll: o enum `mode` do operador é
**dinâmico, filtrado pelo tipo do objecto activo**, e a excepção cita a lista oferecida
(`enum "POSE" not found in ('OBJECT', 'EDIT', …)`). A lista citada coincide célula a célula com os `Y`.

| tipo | OBJ | EDIT | POSE | SCULPT | VP | WP | TP | PART | EDIT_GP | SC_GP | DRAW_GP | WP_GP | VP_GP | SC_CURVES |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| MESH | Y | Y | - | Y | Y | Y | Y | - | - | - | - | - | - | - |
| MESH + sistema de partículas | Y | Y | - | Y | Y | Y | Y | **Y** | - | - | - | - | - | - |
| CURVE · SURFACE · META · FONT · POINTCLOUD · LATTICE | Y | Y | - | - | - | - | - | - | - | - | - | - | - | - |
| CURVES (hair) | Y | Y | - | - | - | - | - | - | - | - | - | - | - | Y |
| GREASEPENCIL (v3) | Y | Y | - | - | - | - | - | - | **-** | Y | Y | Y | Y | - |
| ARMATURE | Y | Y | Y | - | - | - | - | - | - | - | - | - | - | - |
| VOLUME · EMPTY · EMPTY(image) · LIGHT · LIGHT_PROBE · CAMERA · SPEAKER | Y | - | - | - | - | - | - | - | - | - | - | - | - | - |

Notas medidas: `PARTICLE_EDIT` só aparece quando a malha TEM um sistema de partículas (a lista depende
do DADO, não só do tipo). O Grease Pencil v3 edita por `EDIT` genérico; `EDIT_GPENCIL` é recusado.
**Nenhuma célula foi recusada por falta de janela**: SCULPT e os três PAINT entram em `-b`.

### (d) Menu Add
Classes `VIEW3D_MT_*add*` (bl_label): `VIEW3D_MT_add` Add · `_add_object` Add Primitive ·
`_armature_add` Armature · `_camera_add` Camera · `_curve_add` Curve · `_edit_curves_add` Add ·
`_empty_add` Empty · `_grease_pencil_add` Grease Pencil · `_image_add` Add Image · `_lattice_add`
Lattice · `_light_add` Light · `_lightprobe_add` Light Probe · `_mesh_add` Mesh · `_metaball_add`
Metaball · `_surface_add` Surface · `_volume_add` Volume.

Desenho de `VIEW3D_MT_add` gravado por um `layout` falso que regista as chamadas (`menudraw.py`;
contexto headless, modo OBJECT — ramos condicionais ao contexto podem diferir com janela):
```
Mesh ▸ | Curve ▸ | Surface ▸ | Metaball ▸ | Text | Point Cloud | Volume ▸ | Grease Pencil ▸
---  Armature | Lattice ▸
---  Empty ▸ | Image ▸
---  Light ▸ | Light Probe ▸
---  Camera
---  Speaker
---  Force Field ▸ (operator_menu_enum object.effector_add type)
---  Collection Instance ▸
```
Submenus: Mesh = Plane, Cube, Circle, UV Sphere, Ico Sphere, Cylinder, Cone, Torus, ---, Grid, Monkey ·
Curve = Bézier, Circle, ---, Nurbs Curve, Nurbs Circle, Path, ---, Empty Hair, Fur · Grease Pencil =
Blank, Stroke, Monkey, ---, Scene/Collection/Object Line Art · Empty = Plain Axes, Arrows, Single Arrow,
Circle, Cube, Sphere, Cone · Image = Reference…, Background…, Mesh Plane…, Empty Image.
⇒ o menu é por TIPO de dado, com submenu de PRESETS (primitivas) por tipo; Hair vive sob Curve.

Operadores: `mesh.primitive_*` = circle, cone, cube, cube_add_gizmo, cylinder, grid, ico_sphere, monkey,
plane, torus, uv_sphere · `curve.primitive_*` = bezier_circle, bezier_curve, nurbs_circle, nurbs_curve,
nurbs_path · `surface.primitive_nurbs_surface_*` = circle, curve, cylinder, sphere, surface, torus ·
`object.*_add` = armature, camera, collection(_instance), curves_empty_hair, curves_random, data_instance,
effector, empty, empty_image, grease_pencil, light, lightprobe, metaball, pointcloud_random, speaker,
text, volume (+ não-criadores: constraint, modifier, material_slot, particle_system, shaderfx, shape_key,
vertex_group, grease_pencil_*_modifier_segment). `object.add(type=…)` aceita os 16 tipos do enum.

### (e) Comportamento dos modos
Prefs/defaults (factory): `scene.tool_settings.lock_object_mode = True`; `preferences.edit` NÃO tem
`use_mode_switch` (atributos com «mode»: `auto_keying_mode`, `use_enter_edit_mode`).

Multi-objecto (headless, API):
| caso | `context.objects_in_mode` | modos |
|---|---|---|
| só A seleccionado, EDIT | [A] | A=EDIT |
| A+B (malhas) seleccionados, activo A, EDIT | **[A, B]** | A=B=EDIT |
| A+B+Curva seleccionados, activo A, EDIT | [A, B] | a curva fica OBJECT (só o tipo do activo entra) |
| A+B seleccionados, SCULPT | [A] | só o activo esculpe |

Trocar o activo **pela API** (`view_layer.objects.active = B`), lock on e off iguais: A fica em SCULPT
(ou EDIT), B fica OBJECT, `context.mode` passa a `OBJECT` — **estado misto que a API não impede**; um
`mode_set(SCULPT)` a seguir deixa A **e** B em SCULPT.

Trocar pelo CLIQUE (`view3d.select(location=px)` com janela real no kwin aninhado; operador corrido com
`temp_override`, sem evento de rato). Controlo: em OBJECT o clique no px de B torna B activo e no px da
curva torna C activo (ambos FINISHED).
| A em | lock_object_mode | clique em | resultado |
|---|---|---|---|
| SCULPT | True | B | CANCELLED; activo continua A; A=SCULPT |
| EDIT | True | B / Curva | CANCELLED; activo A; A=EDIT |
| WEIGHT_PAINT | True | B | CANCELLED; activo A |
| SCULPT | **False** | B | FINISHED; activo = B; **A continua SCULPT, B=OBJECT**, `context.mode=OBJECT` |
| WEIGHT_PAINT | False | B | idem (A=WEIGHT_PAINT, B=OBJECT) |
| EDIT | False | B / Curva | CANCELLED (em EDIT o clique escolhe elementos de A) |
`object.transfer_mode` (props: `use_flash_on_transfer`): `poll=True` com A num modo não-OBJECT e
janela; `False` em headless; não invocado (pede evento de rato).
`mode_set(mode='EDIT', toggle=True)` duas vezes: EDIT → OBJECT.

## GODOT

### (a) ClassDB (`can_instantiate` ∧ `is_class_enabled`, descendentes a qualquer profundidade)
- `Node2D`: **46** · `Node3D`: **107** · `Control`: **67**.
- Filhos directos de `CanvasItem`: `Control`, `Node2D` (só dois).
- Filhos directos de `Node`: AnimationMixer(abstr.), AudioStreamPlayer, CanvasItem(abstr.), CanvasLayer,
  EditorFileSystem(abstr.), EditorPlugin, EditorResourcePreview(abstr.), HTTPRequest,
  InstancePlaceholder(abstr.), MissingNode, MultiplayerSpawner, MultiplayerSynchronizer,
  NavigationAgent2D, NavigationAgent3D, Node3D, ResourcePreloader, ShaderGlobalsOverride,
  StatusIndicator, Timer, Viewport(abstr.), WorldEnvironment.

Árvore de `Node2D` tal como o **diálogo «Create New Node» real** a mostra
(`EditorInterface.popup_create_dialog(cb, "Node2D", …)`, leitura do `Tree` da `CreateDialog`; ordem do
diálogo, que difere da alfabética do ClassDB nos ramos):
```
Node2D
  CollisionObject2D > PhysicsBody2D > {StaticBody2D > AnimatableBody2D, CharacterBody2D,
                                       RigidBody2D > PhysicalBone2D} ; Area2D
  AnimatedSprite2D, AudioListener2D, AudioStreamPlayer2D, BackBufferCopy, Bone2D, CPUParticles2D,
  Camera2D, CanvasGroup, CanvasModulate, CollisionPolygon2D, CollisionShape2D,
  Joint2D > {DampedSpringJoint2D, GrooveJoint2D, PinJoint2D}
  Light2D > {DirectionalLight2D, PointLight2D}
  GPUParticles2D, LightOccluder2D, Line2D, Marker2D, MeshInstance2D, MultiMeshInstance2D,
  NavigationLink2D, NavigationObstacle2D, NavigationRegion2D, Parallax2D, ParallaxLayer, Path2D,
  PathFollow2D, Polygon2D, RayCast2D, RemoteTransform2D, ShapeCast2D, Skeleton2D, Sprite2D, TileMap,
  TileMapLayer, TouchScreenButton, VisibleOnScreenNotifier2D > VisibleOnScreenEnabler2D
```
⇒ a lista é a **árvore de herança** (abstractas como nós de agrupamento: CollisionObject2D,
PhysicsBody2D, Joint2D, Light2D), sem categorias temáticas. `TileMap` e `ParallaxLayer` aparecem.

### (b) Editor contextual por tipo (editor headless + EditorPlugin; para cada tipo:
`selection.clear(); selection.add_node(n); EditorInterface.edit_node(n)`, 6 frames, e diff da árvore
VISÍVEL de `get_base_control()` contra a linha de base = raiz seleccionada)

Ecrã principal (filho visível de `get_editor_main_screen()`; sinal `main_screen_changed`):
| ecrã antes | tipos 2D (Sprite2D … Camera2D, Node2D, Control) | tipos 3D (MeshInstance3D, CSGBox3D, GridMap, Path3D, Node3D) | AnimationPlayer, Timer |
|---|---|---|---|
| 2D | fica 2D | **muda para 3D** (`main_screen_changed=["3D"]`) | fica |
| 3D | **muda para 2D** | fica 3D | fica |
| Script | fica Script | fica Script | fica |

O que aparece por tipo (ecrã 2D/3D; `onde` = antepassado não-genérico medido):
| tipo | editor/ferramentas que ficam visíveis |
|---|---|
| Sprite2D | menu «Sprite2D» na barra do CanvasItemEditor |
| TileMapLayer | `TileMapLayerEditor` no **EditorBottomPanel** (+ botões de camada, grelha, apagar, aleatório) |
| Polygon2D | `Polygon2DEditor` na barra do CanvasItemEditor (criar/editar/apagar pontos) + `EditorDock` novo com UV · Pontos · Polígonos · Ossos · Grade · Atração |
| Line2D | `Line2DEditor` (criar/editar/apagar pontos) na barra do CanvasItemEditor |
| Path2D | `Path2DEditor` («Criar Curva») na barra do CanvasItemEditor |
| CollisionPolygon2D | `CollisionPolygon2DEditor` na barra do CanvasItemEditor |
| Skeleton2D | menu «Skeleton2D» na barra (sem classe *Editor* própria visível) |
| GPUParticles2D · Camera2D | menu com o nome do tipo na barra |
| Control | `ControlEditorToolbar` na barra do CanvasItemEditor |
| AnimationPlayer | `AnimationPlayerEditor` + `AnimationTrackEditor` no **EditorBottomPanel** |
| MeshInstance3D | menu «Malha» na barra do Node3DEditor |
| CSGBox3D | menu «CSG» |
| GridMap | `GridMapEditor` no **EditorBottomPanel** |
| Path3D | «Criar Curva» na barra do Node3DEditor |
| Node2D · Node3D · Timer | nada específico (só Inspector) |

Persistência medida: TileMapLayerEditor, GridMapEditor e os editores de barra **somem** quando a raiz é
seleccionada; o painel de **AnimationPlayer FICA aberto** depois de seleccionar a raiz e outros nós
(visto antes/depois de MeshInstance3D e CSGBox3D) até outro painel inferior (GridMap) o substituir.

## Conclusão
- Blender: **modo explícito por objecto activo**; o conjunto de modos é função do TIPO (e do dado:
  partículas). Multi-edição só junta objectos seleccionados do MESMO tipo do activo, e só em EDIT.
  `lock_object_mode=True` (default) faz o clique noutro objecto ser recusado enquanto não-OBJECT.
- Godot: **sem modos**; o contexto é **automático pelo tipo seleccionado** — ecrã 2D↔3D troca sozinho
  (excepto a partir do Script), e cada tipo com editor injecta barra/painel/dock.

## Não medido (e porquê)
- Clique real com evento de rato no Blender (`transfer_mode`/Alt+Q, clique em modo EDIT com
  lock off a trocar de objecto pelo outliner): só o operador `view3d.select` por `temp_override`; XTest
  não chega na Xwayland aninhada e ydotool é proibido.
- Ramos condicionais do menu Add com janela/outros modos: o desenho foi gravado num layout falso em `-b`.
- Godot: selecção pelo Scene dock com rato (medido só por `edit_node`/`EditorSelection`); se uma
  configuração de editor altera a troca automática de ecrã (não procurada); o tom «abstracta» no
  diálogo Create (o `Tree` não marca as abstractas como não-seleccionáveis; cor não lida); onde
  exactamente vive o `EditorDock` do Polygon2D; o que o painel UV faz.

## Sondas (verbatim; corridas a partir de `/tmp/claude-1000/oraculo-modos/`)

### `matriz.py`
```python
import bpy, sys, json
scn = bpy.context.scene
col = scn.collection
for o in list(bpy.data.objects): bpy.data.objects.remove(o)
def mesh_data(name):
    m = bpy.data.meshes.new(name)
    m.from_pydata([(0,0,0),(1,0,0),(1,1,0),(0,1,0)],[],[(0,1,2,3)]); m.update(); return m
def make(kind):
    if kind=="MESH": d=mesh_data("m")
    elif kind=="MESH+particles": d=mesh_data("mp")
    elif kind=="CURVE":
        d=bpy.data.curves.new("c","CURVE"); s=d.splines.new("BEZIER"); s.bezier_points.add(1)
    elif kind=="SURFACE":
        d=bpy.data.curves.new("s","SURFACE"); s=d.splines.new("NURBS"); s.points.add(3)
    elif kind=="META":
        d=bpy.data.metaballs.new("mb"); d.elements.new()
    elif kind=="FONT": d=bpy.data.curves.new("t","FONT")
    elif kind=="CURVES": d=bpy.data.hair_curves.new("h")
    elif kind=="POINTCLOUD": d=bpy.data.pointclouds.new("p")
    elif kind=="VOLUME": d=bpy.data.volumes.new("v")
    elif kind=="GREASEPENCIL": d=bpy.data.grease_pencils.new("g"); d.layers.new("L")
    elif kind=="ARMATURE": d=bpy.data.armatures.new("a")
    elif kind=="LATTICE": d=bpy.data.lattices.new("l")
    elif kind in ("EMPTY","EMPTY(image)"): d=None
    elif kind=="LIGHT": d=bpy.data.lights.new("li","POINT")
    elif kind=="LIGHT_PROBE": d=bpy.data.lightprobes.new("lp","SPHERE")
    elif kind=="CAMERA": d=bpy.data.cameras.new("ca")
    elif kind=="SPEAKER": d=bpy.data.speakers.new("sp")
    o = bpy.data.objects.new(kind, d)
    col.objects.link(o)
    if kind=="EMPTY(image)": o.empty_display_type="IMAGE"
    if kind=="MESH+particles":
        o.modifiers.new("ps","PARTICLE_SYSTEM")
    if kind=="ARMATURE":
        bpy.context.view_layer.objects.active=o
        bpy.ops.object.mode_set(mode="EDIT"); b=d.edit_bones.new("b"); b.tail=(0,0,1); bpy.ops.object.mode_set(mode="OBJECT")
    return o
KINDS=["MESH","MESH+particles","CURVE","SURFACE","META","FONT","CURVES","POINTCLOUD","VOLUME","GREASEPENCIL","ARMATURE","LATTICE","EMPTY","EMPTY(image)","LIGHT","LIGHT_PROBE","CAMERA","SPEAKER"]
MODES=[e.identifier for e in bpy.types.Object.bl_rna.properties['mode'].enum_items]
res={}
for k in KINDS:
    o=make(k)
    for x in scn.objects:
        if x: x.select_set(False)
    bpy.context.view_layer.objects.active=o; o.select_set(True)
    row={}
    for m in MODES:
        poll=bpy.ops.object.mode_set.poll()
        err=""
        try:
            r=bpy.ops.object.mode_set(mode=m)
        except Exception as e:
            r=None; err=str(e).strip().splitlines()[-1]
        got=o.mode
        row[m]=dict(poll=poll, ret=(sorted(r) if r else None), got=got, err=err)
        try: bpy.ops.object.mode_set(mode="OBJECT")
        except Exception as e: row[m]["back_err"]=str(e)[:60]
    res[k]=row
    bpy.data.objects.remove(o)
json.dump(res, open(sys.argv[-1],"w"), indent=1)
# compact
ab={"OBJECT":"OBJ","EDIT":"ED","POSE":"POSE","SCULPT":"SC","VERTEX_PAINT":"VP","WEIGHT_PAINT":"WP","TEXTURE_PAINT":"TP","PARTICLE_EDIT":"PE","EDIT_GPENCIL":"EGP","SCULPT_GREASE_PENCIL":"SGP","PAINT_GREASE_PENCIL":"DGP","WEIGHT_GREASE_PENCIL":"WGP","VERTEX_GREASE_PENCIL":"VGP","SCULPT_CURVES":"SCC"}
print("MATRIX", "kind".ljust(15), " ".join(ab[m].rjust(4) for m in MODES))
for k in KINDS:
    print("MATRIX", k.ljust(15), " ".join(("Y" if res[k][m]["got"]==m else ("p" if res[k][m]["poll"] else "-")).rjust(4) for m in MODES))
errs=set()
for k in KINDS:
    for m in MODES:
        e=res[k][m]["err"]
        if e: errs.add(e)
print("ERRS"); [print("  ",e) for e in sorted(errs)]
print("ALLOWED-ENUM (from the operator's dynamic enum, quoted in its refusal)")
import re
for k in KINDS:
    tup=None
    for m in MODES:
        e=res[k][m]["err"]
        g=re.search(r"not found in \((.*)\)", e)
        if g: tup=g.group(1); break
    print("ALLOWED", k.ljust(15), tup)
```

### `comport.py`
```python
import bpy
scn=bpy.context.scene; vl=bpy.context.view_layer; ts=scn.tool_settings
print("DEFAULT lock_object_mode =", ts.lock_object_mode)
print("PREFS edit attrs w/ mode:", [a for a in dir(bpy.context.preferences.edit) if "mode" in a.lower()])
print("PREFS input attrs w/ mode:", [a for a in dir(bpy.context.preferences.inputs) if "mode" in a.lower()])
for o in list(bpy.data.objects): bpy.data.objects.remove(o)
def mesh(n):
    m=bpy.data.meshes.new(n); m.from_pydata([(0,0,0),(1,0,0),(0,1,0)],[],[(0,1,2)])
    o=bpy.data.objects.new(n,m); scn.collection.objects.link(o); return o
A=mesh("A"); B=mesh("B")
cu=bpy.data.curves.new("C","CURVE"); cu.splines.new("BEZIER"); C=bpy.data.objects.new("C",cu); scn.collection.objects.link(C)
def sel(objs, act):
    for o in scn.objects: o.select_set(False)
    for o in objs: o.select_set(True)
    vl.objects.active=act
def inmode(): return sorted(o.name for o in bpy.context.objects_in_mode)
def modes(): return {o.name:o.mode for o in scn.objects}
sel([A],A); bpy.ops.object.mode_set(mode="EDIT"); print("E1 only A selected -> objects_in_mode", inmode(), modes()); bpy.ops.object.mode_set(mode="OBJECT")
sel([A,B],A); bpy.ops.object.mode_set(mode="EDIT"); print("E2 A+B selected, active A -> objects_in_mode", inmode(), modes()); bpy.ops.object.mode_set(mode="OBJECT")
sel([A,B,C],A); bpy.ops.object.mode_set(mode="EDIT"); print("E3 A+B+Curve selected, active A -> objects_in_mode", inmode(), modes()); bpy.ops.object.mode_set(mode="OBJECT")
sel([A,B],A); bpy.ops.object.mode_set(mode="SCULPT"); print("S0 A+B selected, SCULPT -> objects_in_mode", inmode(), modes()); bpy.ops.object.mode_set(mode="OBJECT")
for lock in (True, False):
    ts.lock_object_mode=lock
    sel([A],A); bpy.ops.object.mode_set(mode="SCULPT")
    before=modes()
    vl.objects.active=B
    print(f"S1 lock={lock}: A in SCULPT, then view_layer.objects.active=B -> before {before} after {modes()} ctx.mode={bpy.context.mode} active={vl.objects.active.name}")
    print(f"   transfer_mode poll={bpy.ops.object.transfer_mode.poll()}  mode_set poll={bpy.ops.object.mode_set.poll()}")
    try:
        r=bpy.ops.object.mode_set(mode="SCULPT"); print("   mode_set(SCULPT) with B active ->", r, modes())
    except Exception as e: print("   mode_set(SCULPT) with B active -> EXC", str(e).splitlines()[-1])
    vl.objects.active=A
    try: bpy.ops.object.mode_set(mode="OBJECT")
    except Exception as e: print("   back exc", e)
    vl.objects.active=B
    try: bpy.ops.object.mode_set(mode="OBJECT")
    except Exception as e: print("   back exc", e)
    print("   reset ->", modes())
    # EDIT then switch active
    sel([A],A); bpy.ops.object.mode_set(mode="EDIT"); vl.objects.active=B
    print(f"E4 lock={lock}: A in EDIT, active=B -> {modes()} ctx.mode={bpy.context.mode} in_mode={inmode()}")
    vl.objects.active=A; bpy.ops.object.mode_set(mode="OBJECT")
    # active switched to a CURVE while A edit
    sel([A],A); bpy.ops.object.mode_set(mode="EDIT"); vl.objects.active=C
    print(f"E5 lock={lock}: A in EDIT, active=Curve -> {modes()} ctx.mode={bpy.context.mode}")
    vl.objects.active=A; bpy.ops.object.mode_set(mode="OBJECT")
# mode_set toggle behaviour
sel([A],A); bpy.ops.object.mode_set(mode="EDIT", toggle=True); m1=A.mode; bpy.ops.object.mode_set(mode="EDIT", toggle=True); print("TOGGLE EDIT twice ->", m1, A.mode)
print("mode_set props:", [p.identifier for p in bpy.ops.object.mode_set.get_rna_type().properties])
```

### `janela.py`
```python
import bpy, sys, traceback
from bpy_extras.view3d_utils import location_3d_to_region_2d
OUT="/tmp/claude-1000/oraculo-modos/janela_out.txt"
log=open(OUT,"w")
def P(*a): print(*a, file=log, flush=True)
def ctx3d():
    w=bpy.context.window_manager.windows[0]
    for a in w.screen.areas:
        if a.type=="VIEW_3D":
            r=[x for x in a.regions if x.type=="WINDOW"][0]
            return w,a,r
def run():
    try:
        scn=bpy.context.scene; vl=bpy.context.view_layer; ts=scn.tool_settings
        w,a,r=ctx3d()
        P("window", w.width, w.height, "area", a.type)
        for o in list(bpy.data.objects): bpy.data.objects.remove(o)
        with bpy.context.temp_override(window=w, area=a, region=r):
            bpy.ops.mesh.primitive_cube_add(location=(-2.5,0,0)); A=bpy.context.object; A.name="A"
            bpy.ops.mesh.primitive_cube_add(location=(2.5,0,0)); B=bpy.context.object; B.name="B"
            bpy.ops.curve.primitive_bezier_circle_add(location=(0,3,0)); C=bpy.context.object; C.name="C"
            bpy.ops.object.select_all(action="SELECT"); bpy.ops.view3d.view_all()
        def modes(): return {o.name:o.mode for o in scn.objects}
        def px(o):
            import mathutils; off=mathutils.Vector((1,0,0)) if o.type=='CURVE' else mathutils.Vector((0,0,0)); v=location_3d_to_region_2d(r, a.spaces.active.region_3d, o.location+off); return (int(v.x), int(v.y))
        with bpy.context.temp_override(window=w, area=a, region=r):
            for o in scn.objects: o.select_set(False)
            vl.objects.active=A; A.select_set(True)
            res=bpy.ops.view3d.select(location=px(B))
            P(f"CONTROL OBJECT mode, view3d.select on B at {px(B)}: ret={res} active={vl.objects.active.name} selected={[o.name for o in scn.objects if o.select_get()]}")
            for o in scn.objects: o.select_set(False)
            res=bpy.ops.view3d.select(location=px(C))
            P(f"CONTROL OBJECT mode, view3d.select on C at {px(C)}: ret={res} active={vl.objects.active.name}")
        for lock in (True, False):
          for mode, target in (("SCULPT","B"),("EDIT","B"),("EDIT","C"),("WEIGHT_PAINT","B")):
            ts.lock_object_mode=lock
            with bpy.context.temp_override(window=w, area=a, region=r):
                bpy.ops.object.mode_set(mode="OBJECT") if bpy.context.object and bpy.context.object.mode!="OBJECT" else None
                for o in scn.objects: o.select_set(False)
                vl.objects.active=A; A.select_set(True)
                bpy.ops.object.mode_set(mode=mode)
                T=scn.objects[target]; loc=px(T)
                try:
                    res=bpy.ops.view3d.select(location=loc)
                except Exception as e: res="EXC "+str(e).splitlines()[-1]
                P(f"CLICK lock={lock} A in {mode}, view3d.select on {target} at {loc}: ret={res} active={vl.objects.active.name} modes={modes()} ctx.mode={bpy.context.mode}")
                # transfer_mode (Alt+Q): poll + invoke attempt
                P(f"   transfer_mode poll={bpy.ops.object.transfer_mode.poll()}")
                for o in scn.objects:
                    if o.mode!="OBJECT":
                        vl.objects.active=o
                        try: bpy.ops.object.mode_set(mode="OBJECT")
                        except Exception as e: P("   reset exc", e)
        # transfer mode with location? list props
        P("transfer_mode props", [p.identifier for p in bpy.ops.object.transfer_mode.get_rna_type().properties])
    except Exception:
        P("TRACE", traceback.format_exc())
    log.close()
    bpy.ops.wm.quit_blender()
bpy.app.timers.register(run, first_interval=3.0)
```

### `menudraw.py`
```python
import bpy
class Rec:
    def __init__(s, out, depth): s.out=out; s.d=depth
    def __getattr__(s, name):
        def f(*a, **k):
            if name in ("operator","operator_menu_enum","menu","label","separator","menu_contents","prop","operator_enum","popover"):
                txt=k.get("text"); 
                if name=="separator": s.out.append("  "*s.d+"---")
                else: s.out.append("  "*s.d+f"{name}({', '.join(repr(x) for x in a)}{', text='+repr(txt) if txt is not None else ''})")
            return Rec(s.out, s.d+ (1 if name in ("column","row","box","split") else 0))
        return f
    def __setattr__(s,n,v):
        if n in ("out","d"): object.__setattr__(s,n,v)
    def __bool__(s): return True
class Fake: pass
def dump(menu):
    out=[]; self=Fake(); self.layout=Rec(out,1); self.bl_label=getattr(bpy.types,menu).bl_label
    try: getattr(bpy.types,menu).draw(self, bpy.context)
    except Exception as e: out.append("  !! "+type(e).__name__+": "+str(e)[:100])
    print("DRAW", menu); [print("DRAW", l) for l in out]
for m in ("VIEW3D_MT_add","VIEW3D_MT_mesh_add","VIEW3D_MT_curve_add","VIEW3D_MT_grease_pencil_add","VIEW3D_MT_empty_add","VIEW3D_MT_image_add"):
    dump(m)
```

### `classdb.gd`
```gdscript
extends SceneTree
func _inst_desc(base: String) -> PackedStringArray:
	var out := PackedStringArray()
	for c in ClassDB.get_inheriters_from_class(base):
		if ClassDB.can_instantiate(c) and ClassDB.is_class_enabled(c): out.append(c)
	out.sort(); return out
func _tree(c: String, depth: int, lines: PackedStringArray) -> void:
	var kids := PackedStringArray()
	for k in ClassDB.get_inheriters_from_class(c):
		if ClassDB.get_parent_class(k) == c: kids.append(k)
	kids.sort()
	lines.append("  ".repeat(depth) + c + ("" if ClassDB.can_instantiate(c) else " (abstract)"))
	for k in kids: _tree(k, depth + 1, lines)
func _init() -> void:
	print("VERSION ", Engine.get_version_info().string)
	for b in ["Node2D", "Node3D", "Control"]:
		var l := _inst_desc(b)
		print("COUNT %s instantiable descendants: %d" % [b, l.size()])
		print("LIST %s: %s" % [b, ", ".join(l)])
	var direct := PackedStringArray()
	for k in ClassDB.get_inheriters_from_class("CanvasItem"):
		if ClassDB.get_parent_class(k) == "CanvasItem": direct.append(k + ("" if ClassDB.can_instantiate(k) else "(abstract)"))
	direct.sort(); print("CANVASITEM direct children: ", ", ".join(direct))
	var nd := PackedStringArray()
	for k in ClassDB.get_inheriters_from_class("Node"):
		if ClassDB.get_parent_class(k) == "Node": nd.append(k + ("" if ClassDB.can_instantiate(k) else "(abstract)"))
	nd.sort(); print("NODE direct children: ", ", ".join(nd))
	var lines := PackedStringArray(); _tree("Node2D", 0, lines)
	print("TREE Node2D\n" + "\n".join(lines))
	quit()
```

### `gproj/addons/sonda/sonda.gd`
```gdscript
@tool
extends EditorPlugin
const OUT := "/tmp/claude-1000/oraculo-modos/godot_editor_out.txt"
const TYPES := ["Sprite2D","TileMapLayer","Polygon2D","Line2D","Path2D","Skeleton2D","CollisionPolygon2D","GPUParticles2D","Camera2D","AnimationPlayer","MeshInstance3D","CSGBox3D","GridMap","Path3D","Node2D","Node3D","Control","Timer"]
const GENERIC := ["Control","Container","VBoxContainer","HBoxContainer","MarginContainer","PanelContainer","Panel","Label","Button","CheckBox","CheckButton","MenuButton","OptionButton","TextureRect","ColorRect","HSeparator","VSeparator","ScrollContainer","HSplitContainer","VSplitContainer","SplitContainer","LineEdit","SpinBox","HScrollBar","VScrollBar","Tree","ItemList","TabBar","TabContainer","GridContainer","CenterContainer","FlowContainer","HFlowContainer","VFlowContainer","RichTextLabel","PopupMenu","Window","SubViewport","SubViewportContainer","AspectRatioContainer","LinkButton","TextEdit","CodeEdit","HSlider","VSlider","ProgressBar","ReferenceRect","NinePatchRect","TextureButton","ColorPickerButton","Node","Timer","HTTPRequest","EditorSpinSlider","EditorProperty"]
var f: FileAccess
var screen_log := []
func P(s: String) -> void:
	f.store_line(s); f.flush()
func _enter_tree() -> void:
	main_screen_changed.connect(func(n): screen_log.append(n))
	_run.call_deferred()
func _frames(n: int) -> void:
	for i in n: await get_tree().process_frame
func _main_screen() -> String:
	var out := []
	for c in EditorInterface.get_editor_main_screen().get_children():
		if c is Control and c.visible: out.append(c.get_class())
	return ",".join(out)
func _snap() -> Dictionary:
	var d := {}
	_walk(EditorInterface.get_base_control(), d)
	return d
func _walk(n: Node, d: Dictionary) -> void:
	if n is CanvasItem and not n.is_visible_in_tree(): return
	if n is Window and not n.visible: return
	var cls := n.get_class()
	if not (cls in GENERIC):
		d["C:" + cls] = true
	if n is BaseButton and n is Button and (n.text != "" or n.tooltip_text != ""):
		var t: String = n.text if n.text != "" else ("tip:" + n.tooltip_text.split("\n")[0].left(50))
		d["B:" + t + " <" + n.get_parent().get_class() + ">"] = true
	for c in n.get_children(true): _walk(c, d)
func _diff(a: Dictionary, b: Dictionary) -> Array:
	var r := []
	for k in b: if not a.has(k): r.append("+" + k)
	for k in a: if not b.has(k): r.append("-" + k)
	r.sort(); return r
func _run() -> void:
	f = FileAccess.open(OUT, FileAccess.WRITE)
	P("VERSION " + Engine.get_version_info().string)
	await _frames(10)
	EditorInterface.open_scene_from_path("res://main.tscn")
	await _frames(10)
	var root := EditorInterface.get_edited_scene_root()
	P("edited root: %s" % [root])
	var nodes := {}
	for t in TYPES:
		var n: Node = ClassDB.instantiate(t)
		n.name = t
		root.add_child(n); n.owner = root
		nodes[t] = n
	await _frames(5)
	for start in ["2D", "Script", "3D"]:
		P("=== pass: main screen reset to %s before each selection" % start)
		for t in TYPES:
			EditorInterface.get_selection().clear()
			EditorInterface.edit_node(root)
			EditorInterface.set_main_screen_editor(start)
			await _frames(4)
			var before := _main_screen()
			var s0 := _snap()
			screen_log.clear()
			EditorInterface.get_selection().clear()
			EditorInterface.get_selection().add_node(nodes[t])
			EditorInterface.edit_node(nodes[t])
			await _frames(6)
			var after := _main_screen()
			var s1 := _snap()
			P("TYPE %s | main screen %s -> %s | main_screen_changed=%s" % [t, before, after, screen_log])
			var watch := ["C:TileMapLayerEditor","C:Polygon2DEditor","C:AnimationPlayerEditor","C:GridMapEditor","C:Line2DEditor","C:Path2DEditor","C:CollisionPolygon2DEditor","C:ControlEditorToolbar"]
			var w0 := []; var w1 := []
			for k in watch:
				if s0.has(k): w0.append(k.substr(2))
				if s1.has(k): w1.append(k.substr(2))
			P("    WATCH before=%s after=%s" % [w0, w1])
			if start == "2D":
				for k in w1:
					for nd in EditorInterface.get_base_control().find_children("*", k, true, false):
						if nd.is_visible_in_tree():
							var chain := []
							var a = nd.get_parent()
							while a:
								if not (a.get_class() in GENERIC): chain.append(a.get_class())
								a = a.get_parent()
							P("    WHERE %s <- %s" % [k, " <- ".join(chain)])
			if start == "2D":
				for line in _diff(s0, s1): P("    " + line)
	# create dialog probe
	P("=== create dialog")
	if EditorInterface.has_method("popup_create_dialog"):
		EditorInterface.popup_create_dialog(func(x): pass, "Node2D", "", "probe", [])
		await _frames(6)
		var trees := EditorInterface.get_base_control().find_children("*", "Tree", true, false)
		for tr in trees:
			var w = tr.get_window()
			if w and w.visible and w != EditorInterface.get_base_control().get_window() and tr.is_visible_in_tree() and tr.get_root():
				P("TREE in window " + w.get_class() + " title=" + str(w.title))
				_dump_tree(tr.get_root(), 0)
	else:
		P("no popup_create_dialog")
	P("END")
	f.close()
	get_tree().quit()
func _dump_tree(it: TreeItem, depth: int) -> void:
	if depth > 0 or it.get_text(0) != "":
		P("  ".repeat(depth) + it.get_text(0) + ("" if it.is_selectable(0) else " [not selectable]"))
	var c := it.get_first_child()
	while c:
		_dump_tree(c, depth + 1); c = c.get_next()
```
