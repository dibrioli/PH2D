# houdini-usd-rive-instances (confidence medium)

## decisive_facts
- [DOC] Nenhuma das três ferramentas permite override esparso por CAMPO de uma peça interna mantendo a propagação nos outros campos: Houdini = unlock total (desliga tudo) ou 'Editable nodes' whitelisted pelo AUTOR do mestre; USD = só a raiz é editável, 'editing scene description via instance proxies... is not allowed'; Rive = só a interface exposta (inputs/stateful props). A hipótese excede as três exatamente aí.
- [DOC] Houdini JÁ É 'instância materializada + sync por change detection' — mas sem nó mestre na cena: 'you're editing the definition of the asset in memory. Other instances of the same asset will get the same changes'; a instância unlocked é a superfície de edição e as locked seguem ao vivo; 'Match Current Definition' descarta o conteúdo e preserva 'parameter values'.
- [DOC] USD: corpo rígido numa instância é possível SÓ na raiz — 'The root prim of a rigid body hierarchy can be instanced'; dentro do prototype é proibido porque 'UsdPhysicsRigidBodyAPI has to be able to modify the xformOp attributes in simulation' (Omniverse RB.005). Peça interna que seja corpo próprio força deinstance — materializar as peças remove este obstáculo ao custo de N entidades por instância.
- [DOC] USD: o detach ('instanceable = false') NÃO corta o vínculo — a reference continua a compor; 'You are just losing a little bit of the instancing performance benefits for the copy you deinstanced'. É o único dos três em que 'detach' preserva a propagação; Houdini (Extract Contents) e Rive Libraries ('not possible to re-attach') são irreversíveis.
- [DOC] Nesting: a editabilidade/interface NÃO sobe de nível em Houdini ('Editability does not bubble up... You would need to list A as an editable node of C') nem em Rive ('Parent components can expose child properties... allowing grandparent-level control'); em USD as opiniões nas peças internas 'are now being ignored' quando o nível de cima vira instância — um override de peça de instância interna exige regra de força explícita (externa > interna > mestre).
- [DOC] USD nesting medido: 44.408 prims sem instancing → 1.711 prims/1.450 instâncias/3 prototypes → aninhado 1.482 prims/25 instâncias/1 prototype; 'The higher you move your instancing up the prim hierarchy, the more performance improvements you gain, but at the cost of authoring flexibility'.
- [CODE/MIT] Rive guarda na instância só pose + interface: NestedArtboard : Drawable : Node (x,y,rotation,scaleX,scaleY,opacity,blendMode) + artboardId, dataBindPathIds, isStateful, speed, isPaused, instanceWidth/Height, mix/speed/isPlaying por animação, nestedValue por input; nest()/clone() recursivos sem guarda de profundidade nem de ciclo.
- [DOC] Custo do undo com instâncias materializadas: NÃO DETERMINADO em nenhuma das três — o 6,89 ms não tem comparável externo; na hipótese materializada o undo incremental é condição de viabilidade, não otimização.

## findings

# Pesquisa — o modelo de EDIÇÃO DE INSTÂNCIA em Houdini HDA, OpenUSD e Rive (só documentação)

> **Data de acesso de todas as fontes: 2026-08-21.** Tags: **[DOC]** doc oficial do fornecedor · **[COM]** fórum/comunidade · **[CODE]** lido de fonte permissiva (rive-runtime é **MIT**, confirmado em <https://raw.githubusercontent.com/rive-app/rive-runtime/main/LICENSE>; OpenUSD é Apache-2.0) · **[INF]** inferência minha. Nenhuma linha de código proprietário ou GPL foi lida. Onde a doc não responde, está escrito **não determinado**.
>
> **Contexto:** isto alimenta a hipótese *"instâncias MATERIALIZADAS como subárvores reais + link por peça `{master_root, master_piece}` + overrides esparsos por `(piece-path, type_id, field)` na raiz da instância + sync mestre→instâncias por change detection + nesting recursivo + variant = mestre cujas peças são instâncias (IsA) + undo incremental por change ticks"*. §5 cruza cada fato com ela.

---

## §1 — Houdini Digital Assets (HDA)

### §1.1 Vocabulário exato
- **definition** (a `.hda` em disco ou **`Embedded`** no `.hip`) × **instance** (o nó na cena). Um nó **locked** *"matches the current definition"*; **unlocked** = *"allows editing of contents"* (badge próprio) — [DOC] <https://www.sidefx.com/docs/houdini/assets/edit.html>.
- No HDK o par é **synchronized/unsynchronized**: *"A node is synchronized (locked) when it should exactly match the HDA definition, and it is unsynchronized (unlocked) when it can deviate from the current HDA definition"*; sonda `OP_Node::getMatchesOTLDefinition()` — [DOC] <https://www.sidefx.com/docs/hdk/_h_d_k__h_d_a_intro.html>.
- Verbos de menu (nomes na tela): **Allow Editing of Contents** · **Match Current Definition** · **Save Node Type** (o HOM chama o mesmo gesto de **"Save Operator Type"**: `updateFromNode` *"is equivalent to selecting 'Save Operator Type' on the node's menu"*) · **Extract Contents** (Actions) · **Increase Major/Minor Version** · **Use this definition** · **Type Properties** — [DOC] edit.html; <https://www.sidefx.com/docs/houdini/hom/hou/HDADefinition.html>; <https://www.sidefx.com/docs/houdini/assets/create.html>.
- HOM: `allowEditingOfContents(propagate=False)` · `matchCurrentDefinition()` · `isLockedHDA()` · `matchesCurrentDefinition()` · `isEditableInsideLockedHDA()` · `syncDelayedDefinition()`/`isDelayedDefinition()` (estado **"delay-sync"**) — [DOC] <https://www.sidefx.com/docs/houdini/hom/hou/OpNode.html>.

### §1.2 A regra-mãe, verbatim
> *"New digital asset instances are normally locked, meaning that they are read-only, and they automatically update when the asset's definition changes. An unlocked instance is editable, does not update when the definition changes, and you can save its contents to change the definition."* — [DOC] HDADefinition.html

> *"When you edit the nodes inside an asset, you're editing the definition of the asset **in memory**. Other instances of the same asset will get the same changes, but the original definition of the asset still exists on disk."* — [DOC] edit.html

> `matchCurrentDefinition`: *"If this node is an unlocked digital asset, change its contents to match what is stored in the definition and lock it. **The parameter values are unchanged.** If this node is locked or is not a digital asset, this method has no effect."* — [DOC] OpNode.html

[INF] Leitura conjunta: em Houdini **não existe um "nó mestre" na cena**. A definição é abstrata (disco/memória); a instância **unlocked** vira a *superfície de edição* da definição em memória, e as instâncias **locked** seguem-na **ao vivo** (isto é, literalmente, um sync por mudança da instância editada para as irmãs). `Save Node Type` só persiste. O que a instância unlocked **não** recebe são mudanças vindas de outra fonte (reload do disco, outra instância unlocked salva). Com duas instâncias unlocked divergentes, quem manda é o último `Save Node Type` — **não determinado** na doc se há aviso de conflito.

### §1.3 As quatro perguntas
| Pergunta | Resposta Houdini |
|---|---|
| **Editar in-place mantendo o vínculo?** | **Só na granularidade que a DEFINIÇÃO declarou.** (a) **Valores de parâmetro** são sempre por instância e sobrevivem ao re-sync (*"parameter values are unchanged"*). (b) **Editable nodes** (Type Properties ▸ Node): *"A space-separated list of node paths. These nodes can be edited even if this asset is locked… for complex operations it might be expedient to let the user dive inside and modify nodes such as paint nodes or curves"* — [DOC] <https://www.sidefx.com/docs/houdini/ref/windows/optype.html>. (c) Qualquer outra edição de conteúdo exige **unlock do conteúdo INTEIRO**, e aí o vínculo de propagação **desliga por completo** (binário por instância, nunca por campo). |
| **Granularidade** | Parâmetro (valor) · nó inteiro whitelisted pelo autor do mestre · conteúdo inteiro (unlock). ⚠️ Spare parms adicionados na instância via "Edit Parameter Interface" **não entram na definição** e não propagam: *"do not use Gear icon on DA to mess with parameters they will not become part of definition and thus not get propagated"* — [COM] Tamte, 2011, <https://www.sidefx.com/forum/topic/19019/>. |
| **Verbo de detach** | **Dois, com semânticas distintas.** *Reversível*: **Allow Editing of Contents** (para de receber; **Match Current Definition** religa **descartando** as edições de conteúdo, preservando parâmetros). *Permanente*: **RMB ▸ Actions ▸ Extract Contents** converte em subnet comum sem tipo — [COM, staff SideFX Michael Goldfarb, 2019-09-23] <https://www.sidefx.com/forum/post/294743/>; um resultado de busca acrescenta que é preciso unlock antes do Extract [COM]. *Irreversível por desenho*: **black box** (`save(..., black_box=True)`: *"cannot be unlocked or edited"*) — [DOC] HDADefinition.html. *Órfão*: desinstalar a definição deixa as instâncias a *"warn that they are using an incomplete asset definition. They will, however, retain their parameter values as spare parameters"* — [DOC] <https://www.sidefx.com/docs/houdini/hom/hou/hda.html>. |
| **Profundidade de nesting** | **Sem limite documentado** (não determinado). Regras que valem em cada nível: *"If you are using assets containing other assets… you need to make sure the library files for all assets are installed, not just the 'top' asset"*; instalar depois **religa sem reiniciar** — [DOC] <https://www.sidefx.com/docs/houdini/assets/install.html>. ⚠️ **A editabilidade NÃO sobe de nível:** *"Editability does not 'bubble up' to assets containing other assets. If node A is editable in asset B, that does not automatically make it editable in an asset C which contains an instance of B. You would need to list A as an editable node of C."* — [DOC] optype.html. |

### §1.4 Propagação através do nesting
- [DOC] Uma instância **locked** do asset interno dentro da definição do externo é, por construção, *"read-only… automatically update when the asset's definition changes"* — logo a mudança do interno chega às instâncias do externo **sem re-salvar o externo**, desde que o interno esteja locked lá dentro. [INF] Se o interno estiver **unlocked** dentro do externo, o conteúdo dele fica gravado no *contents section* do externo e **congela**. Um snippet de busca do fórum SideFX resume: *"Locked assets are NOT saved with the parent hda (otl). Unlocked assets ARE saved"* — [COM] <https://www.sidefx.com/forum/topic/41248/> (página exige login; não verificado na fonte).
- [COM] O cgwiki de tokeru afirma o oposto para o caso geral (*"when you update smaller HDAs on disk, they won't automatically refresh in larger containing HDAs. You must manually open the parent HDA, update internal components, and resave"*) — <https://tokeru.com/cgwiki/HoudiniHDA.html>. [INF] As duas afirmações só se conciliam pela regra locked/unlocked acima: o cgwiki descreve o caso do interno **unlocked**. **Não determinado** sem teste.
- `allowEditingOfContents(propagate=False)`: a assinatura existe, **a descrição do `propagate` está ausente** da doc (verificado nas versões atual e 20.5) — [DOC]. [INF] Só o nome sugere *unlock recursivo dos HDAs aninhados*.
- Opções do autor do mestre: **Unlock new nodes on creation** (*"This should always be off for assets you will give to users"*) e **Save Contents as Locked** (*"Never turn this off"*) — [DOC] optype.html. Definições embutidas: *"Definitions embedded in the hip file have priority over definitions stored on disk"* (Asset Manager) vs. por default *"Houdini will load the asset from the library"* salvo **Prefer definitions saved with HIP file** — [DOC] <https://www.sidefx.com/docs/houdini/ref/windows/optypemanager.html>, install.html. Versões: *"instances of previous versions can still exist in HIP files and will still work"* — [DOC] create.html.

---

## §2 — OpenUSD: `instanceable` / prototypes / instance proxies

### §2.1 Vocabulário exato
- **instanceable** (metadatum, *"candidate for instancing"*) · **prototype** (*"special UsdPrim whose sole purpose is to serve as the parent for the scenegraph shared by its associated instance prims… do not exist in scene description"*) · **instance proxy** (*"a UsdPrim that represents a descendant prim beneath an instance, even though no such prim actually exists in the scenegraph"*) · **instancing key** · **nested instancing** · **un-instanceable** / **deinstance** · **inherits** / **specializes** / **class** · **PointInstancer** (`prototypes`, `protoIndices`, `ids`, `inactiveIds`, `invisibleIds`) — [DOC] <https://openusd.org/release/api/_usd__page__scenegraph_instancing.html>; <https://openusd.org/release/glossary.html>; <https://openusd.org/release/api/class_usd_geom_point_instancer.html>.
- NVIDIA Learn OpenUSD define em três linhas: *instanceable prim = "The mutable root of an instance"* · *prototype = "A unique, shared sub-structure"* · *instance proxy = "A read-only addressable stand-in of the prototype prims of an instance"* — [DOC] <https://docs.nvidia.com/learn-openusd/latest/asset-modularity-instancing/authoring-scenegraph-instancing/scenegraph-instancing-intro.html>. As estratégias têm nome: **refinement** = *Deinstancing · Hierarchical · Variant Sets · Ad Hoc Arcs · Broadcasted* — [DOC] <https://docs.nvidia.com/learn-openusd/latest/asset-modularity-instancing/refining-scenegraph-instances/scenegraph-instance-refinement.html>.

### §2.2 A regra-mãe, verbatim
> *"Properties and metadata (e.g., variant selections) on instance prims can be edited and overridden like any other prim. However, properties and metadata on descendant prims beneath instance prims cannot be overridden."* · *"editing scene description via instance proxies and their properties is not allowed."* · *"that instance must be made un-instanceable… so that it will no longer participate in instancing."* · *"A consumer could add inherit or specializes arcs to instances, then make edits to the class targeted by those arcs. Those edits would then affect all of the specified instances."* — [DOC] openusd.org scenegraph instancing.

> *"Only fields of the 'instanceable root' prim, such as composition arcs, transforms, primvars, etc. may vary from its underlying 'prototype'… Opinions directly expressed on descendants of instanceable prims are ignored to preserve the shareability of the prototypes across instances."* · *"it is possible to 'edit' prototypes by modifying the instances' arc targets"* — [DOC] <https://docs.omniverse.nvidia.com/usd/latest/learn-openusd/independent/modularity-guide/instancing.html>.

> Chave de prototype: *"direct composition arcs (strongest to weakest) · variant selections · value clips · stage load rules · population mask"*; *"[UsdStage] groups the instanceable prims by their key and generates a prototype prim for each group."* Caminhos de prototype *"not stable and may vary from run-to-run"* — [DOC] openusd.org.

### §2.3 As quatro perguntas
| Pergunta | Resposta USD |
|---|---|
| **Editar in-place mantendo o vínculo?** | **Sim, mas só na RAIZ da instância** — e aí é arbitrário: transform, visibility, activation, primvars, variant selection, arcos extra. *Hierarchical refinement* explora o que **herda pela hierarquia** (xformOps, visibility, primvars): *"does not introduce any new prototypes… The only new opinions are found on the instanceable prims"* — [DOC] <…/scenegraph-hierarchical-refinement.html>. Abaixo da raiz, **nada** (instance proxy é read-only). |
| **Granularidade** | Raiz = qualquer propriedade. Descendentes = zero. Caminhos intermédios: **Variant set** (muda a chave ⇒ prototype novo partilhado por quem escolhe o mesmo variant); **Ad hoc arcs** (*"New composition arcs can be added to an instanceable prim on the local layer stack"*; *"If you don't foresee a lot of instances leveraging the new ad hoc composition arc, then deinstancing may be a better option"*) — [DOC] <…/scenegraph-ad-hoc-arcs-refinement.html>; **Broadcasted** (inherits/specializes a um class prim: *"author opinions on the class prim namespace. This creates a new prototype that all affected instances share, without altering the original asset"*) — [DOC] <…/scenegraph-broadcasted-refinement.html>. PointInstancer: *"there is no way to do sparse overrides on arrays"*; per-instance só via primvars `vertex`, `invisibleIds` (animável) e `inactiveIds` (*"list-edited, time-invariant"*); **"promotion"** = desativar o ponto e referenciar o asset inteiro no lugar — [DOC] <…/authoring-point-instancing/point-instancing-intro.html>, <…/refining-point-instances.html>. |
| **Verbo de detach** | **`instanceable = false`** ("deinstance"). ⚠️ **Não corta o vínculo:** a reference continua a compor; o prim só deixa de **partilhar** o prototype — *"You will still benefit from all the performance benefits from composition and modular asset reuse. You are just losing a little bit of the instancing performance benefits for the copy you deinstanced"*; custo linear (1.711→1.771 prims ao deinstanciar 2 caixas) — [DOC] <…/scenegraph-deinstance-refinement.html>. [INF] É o único "detach" dos três que **preserva a propagação** (o mestre continua a chegar; só a partilha de memória acaba). `instanceable` é metadata compósita — *"can be authored on different layers and have its value overridden by stronger layers"* [DOC], logo o detach é ele próprio uma opinião desfazível por camada. |
| **Profundidade de nesting** | **Ilimitada:** *"An instanceable prim may have children that are themselves instanceable. This 'nested instancing' allows consumers to build up large aggregate assets from smaller ones and use instancing to share as much of the scenegraph as possible, even between the smaller pieces."* — [DOC] openusd.org. *"You can include PointInstancers within a scenegraph instance and vice versa"* — [DOC] <…/nested-instancing.html>. **Medido no exercício NVIDIA:** sem instancing 44.408 prims; instancing plano 1.711 prims/1.450 instâncias/3 prototypes; **aninhado 1.482 prims/25 instâncias/1 prototype**. ⚠️ **E o preço é exatamente o ponto da hipótese:** *"The higher you move your instancing up the prim hierarchy, the more performance improvements you gain, but at the cost of authoring flexibility"* — as opiniões locais autoradas nas peças internas *"are now being ignored"* quando o nível de cima vira instância — [DOC] <…/exercise-nested-instancing.html>. |

### §2.4 Física sobre instâncias (UsdPhysics)
- [DOC] UsdPhysics oficial: `RigidBodyAPI` *"Applies physics body attributes to any UsdGeomXformable prim"*; *"All prims in the hierarchy below this prim should move rigidly along with the body, except when the descendant prim has its own UsdPhysicsRigidBodyAPI"* — <https://openusd.org/release/api/class_usd_physics_rigid_body_a_p_i.html>. A proposta original diz *"It is not possible to have nested bodies… ignored"* (superada pela regra acima) — <https://openusd.org/dev/wp_rigid_body_physics.html>. O parser oficial *"will traverse instance proxies"* e *"point instancer hierarchies are skipped"* — <https://openusd.org/release/api/usd_physics_page_front.html>. **Nenhuma frase do schema oficial proíbe ou permite RigidBodyAPI em instâncias** (não determinado a nível de schema).
- [DOC NVIDIA] Requisito **RB.005** (*"Rigid bodies cannot be part of a scene graph instance"*): *"UsdPhysicsRigidBodyAPI has to be able to modify the xformOp attributes in simulation, it cannot be part of a scene graph instance (because instanceable prims prohibit changes to their internal prims)."* **Exceção:** *"The root prim of a rigid body hierarchy can be instanced"* — padrão conforme: `RigidBodyAPI` **na própria prim instanceable**, colisores **dentro do prototype** — <https://docs.omniverse.nvidia.com/kit/docs/asset-requirements/1.4.1/capabilities/physics_bodies/physics_rigid_bodies/requirements/rigid-body-no-instancing.html>.
- [DOC NVIDIA] Omni Physics: *"Rigid body scenegraph instancing permits scene hierarchies of collision geometry to be instanced without modification"*; *"Scenegraph instancing allows only collision geometry to be referenced: rigid body parameters may not be referenced with this technique"*; corpos por ponto via `UsdGeom.PointInstancer` (*"Rigid bodies created using point instancing may not have associated joints"*); *"Articulation links may not be instanced using neither scenegraph instancing nor UsdGeom.PointInstancer"* — <https://docs.omniverse.nvidia.com/kit/docs/omni_physics/107.2/dev_guide/rigid_bodies_articulations/rigid_bodies.html>.
- [INF] Tradução: **uma instância PODE ser um corpo — o corpo mora na raiz editável; as peças partilhadas só podem ser colisores solidários.** Uma peça interna que precise de ser corpo **próprio** (ragdoll, porta articulada) força deinstance. Joints que apontam para peças dentro de instâncias são o caso não coberto (a doc só fala de point instancer).

---

## §3 — Rive: Components (ex-Nested Artboards)

### §3.1 Vocabulário exato
- **Component** (artboard marcado; *"purple solid diamonds for Components, purple hollow diamonds for Instances"*) · **Instance** (*"Copies of Components in your file are called Instances"*) · **Component Tool** (`N`, *"formerly known as the Nested Artboard Tool"*) · **expose to main artboard** (inputs) · **Inputs panel** · **Simple / Remap** animation · **Mix** · **Mode: Node / Leaf / Layout** · **Data Bind ▸ Model** · **Stateful component** · **Detach** / **Library Options ▸ Update Component** / **version dropdown** (Libraries) — [DOC] <https://rive.app/docs/editor/fundamentals/components>; <https://rive.app/blog/components-are-here-nested-artboards-done-right> (2025-09-04); <https://rive.app/docs/editor/libraries>; <https://rive.app/docs/editor/data-binding/stateful-components>.
- Regra estrutural: *"Artboards can't be nested directly inside other artboards. To nest an artboard, first convert it to a component."* — [DOC] <https://rive.app/docs/editor/fundamentals/artboards>. *"Only flagged Components (plus your main artboard) get exported to runtime"*; *"When opening older files, Rive auto-converts all artboards into Components"* — [DOC] blog.

### §3.2 A regra-mãe, verbatim
> *"Changes made to the source component are reflected across all of its instances."* — [DOC] components

> *"Stateful components let you expose specific view model properties directly on a nested component, so each instance can have its own values… Because the values are owned by the component instance, you don't need a separate view model instance for each one."* — [DOC] stateful-components

> Libraries: *"Detaching a component will decouple it from the source and copy over its contents into your active file."* · *"It's not possible to re-attach a component after it has been detached."* · *"Upon publishing an updated version of the library, any files that have imported elements from it will display a small badge indicating an available update."* · *"Each republish creates a new version. Library consumers can preview changes and choose when to adopt or stay pinned."* — [DOC] libraries; blog Libraries (2025-09-30) <https://rive.app/blog/libraries-publish-once-reuse-everywhere-in-your-project>.

### §3.3 As quatro perguntas
| Pergunta | Resposta Rive |
|---|---|
| **Editar in-place mantendo o vínculo?** | **O conteúdo, NUNCA** (não há "unlock"; edita-se o source). **A interface, SIM**, e ela é **declarada pelo mestre**: (a) inputs com *"expose to main artboard"* (*"Exposing an Input allows the parent artboard to access and manipulate it"*; o pai liga-os por Listeners, Events ou keys na timeline) — [DOC] <https://rive.app/docs/editor/state-machine/inputs>; (b) propriedades de view model expostas em **stateful components**; (c) escolha de state machine, animações (Simple: *start point* + *playback speed*; Remap: tempo em %), **mix** por animação; (d) **Mode** Node/Leaf/Layout; (e) **Data Bind ▸ Model** (qual VM instance alimenta a instância). |
| **Granularidade** | [CODE] A instância é um nó de cena com transform próprio: `NestedArtboardBase : Drawable` → `Node` (`x`=13, `y`=14) → `TransformComponent` (`rotation`=15, `scaleX`=16, `scaleY`=17) → `WorldTransformComponent` (`opacity`=18); `Drawable` (`blendModeValue`=23, `drawableFlags`=129); próprios: `artboardId`=197, `dataBindPathIds`=582, `isPaused`=895, `speed`=907, `quantize`=908, `isStateful`=1014; layout: `NestedArtboardLayout` (`instanceWidth/Height`=663/664 + unidades 665/666 + scale type 667/668); animações: `NestedLinearAnimation.mix`=200, `NestedSimpleAnimation.speed`=199/`isPlaying`=201, `NestedStateMachine`; inputs: `NestedInput.inputId`=237, `NestedNumber.nestedValue`=239 — <https://raw.githubusercontent.com/rive-app/rive-runtime/main/include/rive/generated/{nested_artboard_base,drawable_base,node_base,transform_component_base,world_transform_component_base,nested_artboard_layout_base}.hpp> e `…/generated/animation/{nested_linear_animation_base,nested_simple_animation_base,nested_input_base,nested_number_base,nested_state_machine_base}.hpp`. [INF] Ou seja: **o ficheiro guarda na instância só o que é interface + pose**; nenhuma propriedade de peça interna é endereçável a partir da instância. |
| **Verbo de detach** | **Só para componentes importados de Library**: *"Detach"* (irreversível, copia o conteúdo para o ficheiro). **Para instâncias locais do mesmo ficheiro, nenhum verbo de detach foi encontrado na doc** (não determinado; [INF] "duplicar" em Rive = duplicar o artboard-fonte). Swap de fonte em runtime: *"swap the source of an artboard at runtime. Load an artboard from a separate file, inject it into your layout, and preserve its constraints, animations, and position"* — [DOC] <https://rive.app/blog/data-binding-supercharged-lists-images-and-artboards>. |
| **Profundidade de nesting** | **Recursiva, sem limite documentado.** [DOC] *"Stateful components can be nested inside other components. Parent components can expose child properties by binding them to their own view models and adding them to the Properties panel as inputs, allowing grandparent-level control"*; blog: *"Nest them, and their logic stays self-contained"*; Libraries: *"support for nested libraries"*. [CODE] `NestedArtboard::nest()` / `clone()` instanciam `m_referencedArtboard->instance()` recursivamente e **não há guarda de profundidade nem de ciclo** no runtime — <https://raw.githubusercontent.com/rive-app/rive-runtime/main/src/nested_artboard.cpp>. [INF] A prevenção de auto-aninhamento tem de estar no editor (não determinado). [COM] Utilizadores relatam que remapear eventos *"quickly gets out of hand when multiple levels of nesting is there"* — <https://community.rive.app/c/support/listen-for-events-from-nested-artboards>. ⚠️ Mesma lei do Houdini: **a interface não sobe de nível sozinha** — o avô só controla o neto se o pai re-expuser. |

---

## §4 — Tabela comparativa (as 4 perguntas × 3 ferramentas)

| | **Houdini HDA** | **OpenUSD** | **Rive Components** |
|---|---|---|---|
| **Onde vive o conteúdo da instância** | **Materializado**: nós reais dentro do nó-instância | Virtual: prototype partilhado + instance proxies read-only | Virtual: `ArtboardInstance` clonado do source em runtime |
| **Editar in-place mantendo o link** | Parâmetros (sempre) · **Editable nodes** (whitelist do AUTOR do mestre) | **Qualquer coisa, só na RAIZ**; descendentes zero | **Só a interface que o mestre expôs** (inputs, stateful props) + pose/animações/mix/sizing |
| **Override por campo de PEÇA interna** | ❌ (ou nó inteiro whitelisted, ou unlock total) | ❌ (deinstance) | ❌ (editar o source) |
| **Propagação após edição local** | Binária por instância: locked=recebe tudo · unlocked=nada | Contínua: a raiz sobrepõe, o resto segue; deinstance **mantém** a reference | Contínua (interface é disjunta do conteúdo) |
| **Broadcast para todas** | Editar qualquer instância unlocked (= definição em memória) → `Save Node Type` | `inherits`/`specializes` a um class prim; ou editar o asset referenciado | Editar o source; Libraries: republish + badge + *adopt or stay pinned* |
| **Detach reversível** | **Allow Editing of Contents** ↔ **Match Current Definition** (descarta conteúdo, guarda parâmetros) | `instanceable=false` (opinião por camada, desfazível; link preservado) | — |
| **Detach permanente** | **Extract Contents** (subnet sem tipo) · black box impede unlock | flatten (fora do escopo pesquisado) | **Detach** (só Library; *"not possible to re-attach"*) |
| **Nesting** | Sem limite doc.; editabilidade **não sobe**; interno locked auto-atualiza, unlocked congela | Sem limite; **prototypes partilhados entre níveis**; opiniões em peças internas **ignoradas** quando o nível de cima instancia | Sem limite doc.; recursão sem guarda no runtime; interface **não sobe** sem re-expor |
| **Física na instância** | n/a (não pesquisado) | Corpo na **raiz** OK; corpo em peça interna ❌ (RB.005); PointInstancer: corpos sim, joints não | n/a |
| **Undo / custo de captura** | **não determinado** | **não determinado** | **não determinado** |

---

## §5 — O que isto diz sobre a hipótese de trabalho

1. **A hipótese é estritamente MAIS rica do que qualquer das três — e o ponto em que ela excede todas é o mesmo: override esparso por campo de PEÇA interna com a propagação a continuar nos campos não-tocados.** Houdini resolve o merge **não fazendo merge** (unlock = desliga tudo; Match = descarta tudo); USD resolve **proibindo** (proxy read-only; deinstance); Rive resolve **por interface declarada**. Nenhuma ferramenta das três mantém, por instância, um diff por campo contra uma subárvore partilhada. [INF] O modelo proposto pertence à família Figma/Unity/Godot já coberta no doc 02 §2.8; estas três não o validam nem o refutam — **mostram três formas de o evitar**, e a razão comum é a *segurança de merge* quando o mestre muda estrutura.

2. **Houdini É "instância materializada + sync por change detection", e a fonte da verdade é a instância unlocked, não um mestre na cena.** *"you're editing the definition of the asset in memory. Other instances of the same asset will get the same changes."* [INF] Isto é uma alternativa de desenho ao "mestre subárvore + sync": *qualquer* instância pode ser promovida a superfície de edição, e as irmãs locked seguem ao vivo. Tem um preço registado: **duas unlocked divergem em silêncio**, e a regra "quem recebe" é binária por instância.

3. **A raiz da instância é, nas três, o único lugar com override arbitrário** (USD explícito; Rive pela hierarquia `Node`; Houdini pelos parâmetros). [INF] A proposta de guardar os overrides **na raiz** em ordem canónica coincide com onde as três ferramentas os guardam — o que diverge é só o **alcance** (peças internas).

4. **Nesting: duas leis documentadas que a hipótese tem de escrever explicitamente.** (a) *A interface/editabilidade NÃO sobe de nível* (Houdini verbatim; Rive verbatim) — um override de peça de instância interna, visto da externa, precisa de regra de força própria (externa > interna > mestre interno), senão cai no caso USD (b): *"The higher you move your instancing up… at the cost of authoring flexibility"* — opiniões internas **ignoradas**. (c) Propagação em cadeia no Houdini depende do estado **locked do interno dentro do externo**; na hipótese, um "sync topológico" tem de definir o que acontece quando a instância interna dentro do **mestre externo** tem overrides próprios (eles viram parte do mestre externo e propagam às instâncias externas — é o comportamento Houdini para interno unlocked, congelado).

5. **Física: a lição USD é precisa.** Corpo rígido na **raiz** da instância é compatível com partilha; corpo em **peça** exige que a peça seja entidade própria e mutável pelo solver. [INF] Materializar as peças (a hipótese) **remove** o obstáculo que USD documenta, ao custo de perder a partilha de memória — que é exatamente o custo que o undo por frame paga (fato §0.1 do doc 04). Ou seja, a hipótese troca o problema "instance proxy read-only" pelo problema "N entidades por instância", e por isso **o undo incremental não é opcional nela: é a condição de viabilidade**. Nenhuma das três ferramentas documenta o custo do undo (não determinado) — o número 6,89 ms não tem comparável externo nesta pesquisa.

6. **"Duplicate" e "instance" nas três:** Houdini — *copy* de um nó locked é outra instância; *independente* = Extract Contents (ou unlock e nunca sincronizar). USD — não há "cópia independente" nativa: deinstance **continua** referenciado; independência real = flatten. Rive — dentro do ficheiro, duplicar o artboard-fonte; instância de Library = Detach (irreversível). [INF] O verbo "Detach" irreversível é o consenso; o USD é o único com "detach que continua a receber", e é o mais próximo de *"instância física sem perder propagação"*.

7. **Variant = mestre cujas peças são instâncias de outro mestre (IsA):** em USD isto é literalmente *specializes/inherits a um class prim* (broadcasted refinement: *"creates a new prototype that all affected instances share, without altering the original asset"*); em Houdini é um HDA cujo conteúdo é um HDA locked + parâmetros; em Rive é um Component que instancia outro. [INF] As três suportam a forma; nenhuma documenta como o override do variant se compõe com o override da instância do variant (é a regra de força de §5.4 outra vez).

---

## §6 — Não determinado
- Semântica exata do `propagate` em `allowEditingOfContents` (assinatura documentada, descrição ausente nas versões atual e 20.5).
- Houdini: comportamento com **duas** instâncias unlocked do mesmo tipo ao salvar (aviso? último vence?); limite de profundidade de nesting; a regra "locked não é salvo com o pai" veio de snippet de fórum sob login.
- USD: posição do **schema oficial** sobre `RigidBodyAPI` em instâncias (só NVIDIA/Omniverse a escreve); joints que apontem para peças dentro de instâncias.
- Rive: verbo de detach para instâncias **locais** (não de Library); guarda contra auto-aninhamento no editor; profundidade máxima.
- Custo/modelo de **undo** nas três ferramentas — fora do alcance das docs lidas.


## sources
- https://www.sidefx.com/docs/houdini/assets/edit.html
- https://www.sidefx.com/docs/houdini/hom/hou/HDADefinition.html
- https://www.sidefx.com/docs/houdini/hom/hou/OpNode.html
- https://www.sidefx.com/docs/houdini20.5/hom/hou/OpNode.html
- https://www.sidefx.com/docs/houdini/hom/hou/hda.html
- https://www.sidefx.com/docs/houdini/ref/windows/optype.html
- https://www.sidefx.com/docs/houdini/ref/windows/optypemanager.html
- https://www.sidefx.com/docs/houdini/assets/install.html
- https://www.sidefx.com/docs/houdini/assets/create.html
- https://www.sidefx.com/docs/hdk/_h_d_k__h_d_a_intro.html
- https://www.sidefx.com/forum/post/294743/
- https://www.sidefx.com/forum/topic/19019/
- https://www.sidefx.com/forum/topic/41248/
- https://tokeru.com/cgwiki/HoudiniHDA.html
- https://openusd.org/release/api/_usd__page__scenegraph_instancing.html
- https://openusd.org/release/glossary.html
- https://openusd.org/release/api/class_usd_geom_point_instancer.html
- https://openusd.org/release/api/class_usd_physics_rigid_body_a_p_i.html
- https://openusd.org/release/api/usd_physics_page_front.html
- https://openusd.org/dev/wp_rigid_body_physics.html
- https://docs.nvidia.com/learn-openusd/latest/asset-modularity-instancing/authoring-scenegraph-instancing/scenegraph-instancing-intro.html
- https://docs.nvidia.com/learn-openusd/latest/asset-modularity-instancing/authoring-scenegraph-instancing/nested-instancing.html
- https://docs.nvidia.com/learn-openusd/latest/asset-modularity-instancing/authoring-scenegraph-instancing/exercise-nested-instancing.html
- https://docs.nvidia.com/learn-openusd/latest/asset-modularity-instancing/refining-scenegraph-instances/scenegraph-instance-refinement.html
- https://docs.nvidia.com/learn-openusd/latest/asset-modularity-instancing/refining-scenegraph-instances/scenegraph-deinstance-refinement.html
- https://docs.nvidia.com/learn-openusd/latest/asset-modularity-instancing/refining-scenegraph-instances/scenegraph-hierarchical-refinement.html
- https://docs.nvidia.com/learn-openusd/latest/asset-modularity-instancing/refining-scenegraph-instances/scenegraph-ad-hoc-arcs-refinement.html
- https://docs.nvidia.com/learn-openusd/latest/asset-modularity-instancing/refining-scenegraph-instances/scenegraph-broadcasted-refinement.html
- https://docs.nvidia.com/learn-openusd/latest/asset-modularity-instancing/authoring-point-instancing/point-instancing-intro.html
- https://docs.nvidia.com/learn-openusd/latest/asset-modularity-instancing/refining-point-instances.html
- https://docs.nvidia.com/learn-openusd/latest/asset-modularity-instancing/instancing-faq.html
- https://docs.omniverse.nvidia.com/usd/latest/learn-openusd/independent/modularity-guide/instancing.html
- https://docs.omniverse.nvidia.com/kit/docs/asset-requirements/1.4.1/capabilities/physics_bodies/physics_rigid_bodies/requirements/rigid-body-no-instancing.html
- https://docs.omniverse.nvidia.com/kit/docs/asset-requirements/1.7.1/capabilities/physics_bodies/physics_rigid_bodies/capability-physics_rigid_bodies.html
- https://docs.omniverse.nvidia.com/kit/docs/omni_physics/107.2/dev_guide/rigid_bodies_articulations/rigid_bodies.html
- https://rive.app/docs/editor/fundamentals/components
- https://rive.app/docs/editor/fundamentals/artboards
- https://rive.app/docs/editor/fundamentals/nested-artboards
- https://rive.app/docs/editor/state-machine/inputs
- https://rive.app/docs/editor/data-binding/stateful-components
- https://rive.app/docs/editor/data-binding/view-models
- https://rive.app/docs/editor/libraries
- https://rive.app/blog/components-are-here-nested-artboards-done-right
- https://rive.app/blog/libraries-publish-once-reuse-everywhere-in-your-project
- https://rive.app/blog/data-binding-supercharged-lists-images-and-artboards
- https://rive.app/changelog/nested-artboard-fixes
- https://community.rive.app/c/support/listen-for-events-from-nested-artboards
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/LICENSE
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/include/rive/nested_artboard.hpp
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/src/nested_artboard.cpp
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/include/rive/generated/nested_artboard_base.hpp
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/include/rive/generated/nested_artboard_layout_base.hpp
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/include/rive/generated/drawable_base.hpp
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/include/rive/generated/node_base.hpp
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/include/rive/generated/transform_component_base.hpp
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/include/rive/generated/world_transform_component_base.hpp
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/include/rive/generated/animation/nested_linear_animation_base.hpp
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/include/rive/generated/animation/nested_simple_animation_base.hpp
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/include/rive/generated/animation/nested_input_base.hpp
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/include/rive/generated/animation/nested_number_base.hpp
- https://raw.githubusercontent.com/rive-app/rive-runtime/main/include/rive/generated/animation/nested_state_machine_base.hpp
- https://deepwiki.com/rive-app/rive-runtime/2.4-nested-artboards
